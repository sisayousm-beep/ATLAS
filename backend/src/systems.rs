//! The Phase 1 simulation loop, expressed as ECS systems.
//!
//! Causal chain (design §2): pops → production → consumption → happiness →
//! population change. Each system runs once per simulated day, in order.

use crate::components::*;
use crate::corporation::*;
use crate::diplomacy::*;
use crate::finance::*;
use crate::politics::*;
use crate::production::*;
use crate::resources::*;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Grain a single farmer produces per day on baseline land.
const FARM_YIELD: f64 = 0.02;
/// Grain a single person eats per day.
const FOOD_NEED: f64 = 0.01;
/// Monthly tax rate applied to pop wealth.
const TAX_RATE: f64 = 0.01;
/// Raw good a single laborer extracts per day at richness 1.0 (before infra).
const EXTRACT_RATE: f64 = 0.01;
/// Labour budget a single engineer supplies to factories per day.
const MANUFACTURE_RATE: f64 = 0.01;
/// Cars a pop wants per person per day, scaled by wealth (luxury demand). Sized
/// so consumer demand roughly meets what the car firms can build — the pull that
/// keeps the value chain (and the firms feeding it) profitable.
const CAR_APPETITE: f64 = 4.0e-5;
/// How sharply prices react to a supply/demand imbalance each day.
const PRICE_ELASTICITY: f64 = 0.08;
/// Wage a corporation pays per unit of recipe labour it runs.
const WAGE: f64 = 0.5;
/// Share of a profitable firm's monthly profit taken as corporate tax.
const CORP_TAX: f64 = 0.15;
/// Engineers a profitable firm hires each month when it can afford to.
const HIRE_STEP: f64 = 2_000.0;
/// Capital a firm spends per engineer hired.
const HIRE_COST: f64 = 0.5;
/// Floor a firm shrinks its workforce toward when losing money.
const MIN_EMPLOYEES: f64 = 500.0;
/// Per-day share of a region's surplus the logistics network can ship.
const TRADE_CAP: f64 = 0.5;
/// Fraction of a shipment lost in transit (design §10 logistics friction).
const TRANSPORT_LOSS: f64 = 0.02;
/// Quantities below this are treated as zero in the trade clearing pass.
const TRADE_EPS: f64 = 1.0e-9;
/// Monthly government outlay per head: welfare, services, administration.
const WELFARE_PER_CAPITA: f64 = 2.0e-2;
/// Share of a budget surplus a nation uses to pay down its debt each month.
const DEBT_REPAY_SHARE: f64 = 0.25;
/// How fast realised inflation eases toward this month's monetary growth.
const INFLATION_EASE: f64 = 0.5;
/// Inflation above this trips the financial-crisis flag (hyperinflation).
const CRISIS_INFLATION: f64 = 0.12;
/// Government debt above this share of the money supply also trips the flag.
const CRISIS_DEBT_RATIO: f64 = 0.6;

/// Build the daily schedule: all Phase 1 systems in fixed causal order.
pub fn build_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            advance_clock,
            produce_food,
            extract_resources,
            trade_goods,
            corporate_production,
            consume_food,
            consume_goods,
            pop_growth,
            political_unrest,
            diplomacy,
            collect_taxes,
            corporate_decisions,
            government_finance,
            update_inflation,
            monetary_policy,
            finance_oversight,
            update_market,
            report,
        )
            .chain(),
    );
    schedule
}

/// Advance the calendar by one day.
pub fn advance_clock(mut clock: ResMut<GameClock>) {
    clock.day += 1;
}

/// Farmers turn labour and land into grain, stored in their region.
pub fn produce_food(
    pops: Query<&Pop>,
    mut regions: Query<(&Region, &mut ResourceStock)>,
    mut market: ResMut<Market>,
) {
    for pop in &pops {
        if pop.profession != Profession::Farmer {
            continue;
        }
        if let Ok((region, mut stock)) = regions.get_mut(pop.region) {
            let yield_per_capita = FARM_YIELD
                * terrain_food_mod(region.terrain)
                * climate_food_mod(region.climate)
                * (0.5 + region.infrastructure);
            let produced = pop.size as f64 * yield_per_capita;
            stock.add(Good::Grain, produced);
            market.record_supply(Good::Grain, produced);
        }
    }
}

/// Laborers dig raw goods out of their region's deposits, scaled by deposit
/// richness and local infrastructure. Output feeds the region's warehouse and
/// counts as market supply.
pub fn extract_resources(
    pops: Query<&Pop>,
    mut regions: Query<(Entity, &Region, &Deposits, &mut ResourceStock)>,
    mut market: ResMut<Market>,
) {
    let mut labor: HashMap<Entity, f64> = HashMap::new();
    for pop in &pops {
        if pop.profession == Profession::Laborer {
            *labor.entry(pop.region).or_default() += pop.size as f64;
        }
    }
    for (entity, region, deposits, mut stock) in &mut regions {
        let workers = labor.get(&entity).copied().unwrap_or(0.0);
        if workers <= 0.0 || deposits.0.is_empty() {
            continue;
        }
        // Split the region's labour evenly across the goods it can mine.
        let per_good = workers * EXTRACT_RATE / deposits.0.len() as f64;
        for (&good, &richness) in &deposits.0 {
            let amount = per_good * richness * (0.5 + region.infrastructure);
            if amount <= 0.0 {
                continue;
            }
            stock.add(good, amount);
            market.record_supply(good, amount);
        }
    }
}

/// The logistics network (design §10): ship each good from regions holding more
/// than their share toward regions holding less, so factories aren't starved by
/// where the ore happens to sit. Throughput is capped per day and scaled by the
/// infrastructure at both ends, and a slice of every shipment is lost in transit.
/// Cross-border flows land on each nation's trade balance; everything is tallied
/// in the [`TradeLedger`].
pub fn trade_goods(
    mut regions: Query<(Entity, &Region, &mut ResourceStock)>,
    mut nations: Query<&mut Nation>,
    market: Res<Market>,
    mut ledger: ResMut<TradeLedger>,
    mut diplo: ResMut<Diplomacy>,
) {
    /// A region's standing for one good during the clearing pass.
    struct Node {
        entity: Entity,
        owner: Entity,
        infra: f64,
        orig: f64,
        stock: f64,
    }

    let mut trade_value: HashMap<Entity, (f64, f64)> = HashMap::new(); // (exports, imports)

    for &good in TRADED {
        let mut nodes: Vec<Node> = regions
            .iter()
            .map(|(entity, region, stock)| Node {
                entity,
                owner: region.owner,
                infra: region.infrastructure,
                orig: stock.get(good),
                stock: stock.get(good),
            })
            .collect();

        let total: f64 = nodes.iter().map(|n| n.stock).sum();
        if nodes.len() < 2 || total <= 0.0 {
            continue;
        }
        let target = total / nodes.len() as f64;

        // Greedily match the biggest surplus with the biggest deficit. One pass
        // settles at most one pair, so cap passes at the region count.
        for _ in 0..nodes.len() {
            let mut sell = None;
            let mut buy = None;
            let (mut best_sur, mut best_def) = (TRADE_EPS, TRADE_EPS);
            for (i, n) in nodes.iter().enumerate() {
                if n.stock - target > best_sur {
                    best_sur = n.stock - target;
                    sell = Some(i);
                }
                if target - n.stock > best_def {
                    best_def = target - n.stock;
                    buy = Some(i);
                }
            }
            let (Some(s), Some(d)) = (sell, buy) else { break };

            let cap = target * TRADE_CAP * nodes[s].infra.min(nodes[d].infra);
            let qty = (nodes[s].stock - target)
                .min(target - nodes[d].stock)
                .min(cap);
            if qty <= TRADE_EPS {
                break;
            }
            let delivered = qty * (1.0 - TRANSPORT_LOSS);
            nodes[s].stock -= qty;
            nodes[d].stock += delivered;

            let value = qty * market.price(good);
            ledger.record(good, qty, value);
            if nodes[s].owner != nodes[d].owner {
                trade_value.entry(nodes[s].owner).or_default().0 += value;
                trade_value.entry(nodes[d].owner).or_default().1 += value;
                // Commerce across a border warms the two nations' relations.
                diplo.record_trade(nodes[s].owner, nodes[d].owner, value);
            }
        }

        // Commit the net movement back into each region's warehouse.
        for n in &nodes {
            if let Ok((_, _, mut stock)) = regions.get_mut(n.entity) {
                stock.add(good, n.stock - n.orig);
            }
        }
    }

    for (entity, (exports, imports)) in trade_value {
        if let Ok(mut nation) = nations.get_mut(entity) {
            nation.exports += exports;
            nation.imports += imports;
        }
    }
}

/// Corporations run the factories (design §8). Each firm claims engineer labour
/// in its home region (capped by the engineers actually there), then works its
/// recipes in tier order: it buys the inputs and sells the output on the market,
/// keeping the margin as capital. Inputs consumed are demand, output made is
/// supply — the market mechanics are unchanged, the money is the new layer.
pub fn corporate_production(
    pops: Query<&Pop>,
    mut corps: Query<&mut Corporation>,
    mut regions: Query<&mut ResourceStock>,
    mut market: ResMut<Market>,
) {
    // Engineers per region — the physical cap on how much industry can run.
    let mut engineers: HashMap<Entity, f64> = HashMap::new();
    for pop in &pops {
        if pop.profession == Profession::Engineer {
            *engineers.entry(pop.region).or_default() += pop.size as f64;
        }
    }

    for mut corp in &mut corps {
        let available = engineers.get(&corp.region).copied().unwrap_or(0.0);
        let workers = corp.employees.min(available);
        if workers <= 0.0 {
            continue;
        }
        *engineers.get_mut(&corp.region).unwrap() -= workers;

        let mut budget = workers * MANUFACTURE_RATE;
        let industries = corp.industries.clone();
        let Ok(mut stock) = regions.get_mut(corp.region) else {
            continue;
        };
        for output in industries {
            if budget <= 0.0 {
                break;
            }
            let Some(recipe) = recipe_for(output) else {
                continue;
            };
            // Units: limited by labour budget and every input in stock.
            let mut units = budget / recipe.labor;
            for &(good, qty) in recipe.inputs {
                units = units.min(stock.get(good) / qty);
            }
            if units <= 0.0 {
                continue;
            }
            let mut input_cost = 0.0;
            for &(good, qty) in recipe.inputs {
                let used = qty * units;
                stock.take(good, used);
                market.record_demand(good, used);
                input_cost += used * market.price(good);
            }
            let wages = units * recipe.labor * WAGE;
            let revenue = units * market.price(output);
            stock.add(output, units);
            market.record_supply(output, units);

            let margin = revenue - input_cost - wages;
            corp.capital += margin;
            corp.profit += margin;
            budget -= units * recipe.labor;
        }
    }
}

/// Every pop eats from its region's grain stock. The fraction of demand met
/// nudges happiness up (surplus) or down (shortage).
pub fn consume_food(
    mut pops: Query<&mut Pop>,
    mut regions: Query<&mut ResourceStock>,
    mut market: ResMut<Market>,
) {
    for mut pop in &mut pops {
        let need = pop.size as f64 * FOOD_NEED;
        if need <= 0.0 {
            continue;
        }
        market.record_demand(Good::Grain, need);
        if let Ok(mut stock) = regions.get_mut(pop.region) {
            let eaten = stock.take(Good::Grain, need);
            let satisfaction = (eaten / need).clamp(0.0, 1.0);
            // Ease happiness toward today's food satisfaction.
            pop.happiness += (satisfaction - pop.happiness) * 0.05;
            pop.happiness = pop.happiness.clamp(0.0, 1.0);
        }
    }
}

/// Pops buy finished goods — for now cars — creating market demand that scales
/// with population size and wealth. Met demand is drawn from regional stock;
/// unmet demand still pushes the price up next tick.
pub fn consume_goods(
    pops: Query<&Pop>,
    mut regions: Query<&mut ResourceStock>,
    mut market: ResMut<Market>,
) {
    for pop in &pops {
        let want = pop.size as f64 * pop.wealth * CAR_APPETITE;
        if want <= 0.0 {
            continue;
        }
        market.record_demand(Good::Car, want);
        if let Ok(mut stock) = regions.get_mut(pop.region) {
            stock.take(Good::Car, want);
        }
    }
}

/// Form prices from the day's flows (design §9): goods in net demand rise,
/// goods in surplus fall, each bounded to a band around its base price. The
/// pressure term is normalised to (-1, 1) so a single missing good can't
/// detonate the price. Flows are cleared for the next day.
pub fn update_market(mut market: ResMut<Market>) {
    for &g in TRADED {
        let s = market.supply(g);
        let d = market.demand(g);
        if s + d <= 0.0 {
            continue;
        }
        let pressure = (d - s) / (d + s);
        let base = base_price(g);
        let next = (market.price(g) * (1.0 + PRICE_ELASTICITY * pressure))
            .clamp(base * 0.1, base * 10.0);
        market.prices.insert(g, next);
    }
    market.clear_flows();
}

/// Once a month, populations grow or shrink based on happiness.
/// Happiness 0.5 is break-even; extremes give roughly ±1%/month.
pub fn pop_growth(clock: Res<GameClock>, mut pops: Query<&mut Pop>) {
    if !clock.is_month_start() {
        return;
    }
    for mut pop in &mut pops {
        let rate = (pop.happiness - 0.5) * 0.02;
        let delta = (pop.size as f64 * rate).round() as i64;
        pop.size = (pop.size as i64 + delta).max(0) as u32;
    }
}

/// Once a month, each nation collects tax from the pops in the regions it owns.
pub fn collect_taxes(
    clock: Res<GameClock>,
    mut nations: Query<(Entity, &mut Nation)>,
    regions: Query<&Region>,
    pops: Query<&Pop>,
) {
    if !clock.is_month_start() {
        return;
    }
    let mut income: HashMap<Entity, f64> = HashMap::new();
    for pop in &pops {
        if let Ok(region) = regions.get(pop.region) {
            let tax = pop.size as f64 * pop.wealth * TAX_RATE;
            *income.entry(region.owner).or_default() += tax;
        }
    }
    for (entity, mut nation) in &mut nations {
        if let Some(&amount) = income.get(&entity) {
            // Unrest erodes compliance: an unstable state collects less of its due.
            nation.treasury += amount * tax_compliance(nation.stability);
        }
    }
}

/// Politics (design §13), run monthly. Each nation's stability eases toward how
/// content its people are (happiness) and how much of them the government
/// represents (support from aligned ideologies). Deep unrest that persists tips
/// into a revolution or coup: the largest ideological bloc seizes power and
/// installs its own government, resetting stability at a honeymoon level.
pub fn political_unrest(
    clock: Res<GameClock>,
    regions: Query<&Region>,
    pops: Query<&Pop>,
    mut nations: Query<(Entity, &mut Nation, &mut Politics)>,
    mut ledger: ResMut<PoliticsLedger>,
) {
    if !clock.is_month_start() {
        return;
    }

    // Per nation: total heads, happiness-weighted heads, and the size of each
    // ideological bloc (so we can measure support and find the revolution's heir).
    #[derive(Default)]
    struct Tally {
        total: f64,
        happy: f64,
        ideology: HashMap<Ideology, f64>,
    }
    let mut tally: HashMap<Entity, Tally> = HashMap::new();
    for pop in &pops {
        if let Ok(region) = regions.get(pop.region) {
            let t = tally.entry(region.owner).or_default();
            let size = pop.size as f64;
            t.total += size;
            t.happy += pop.happiness * size;
            *t.ideology.entry(pop.ideology).or_default() += size;
        }
    }

    let (mut sum_stab, mut min_stab, mut unrest_count, mut revolutions) = (0.0, 1.0_f64, 0, 0);
    let mut n = 0.0;
    for (entity, mut nation, mut politics) in &mut nations {
        let Some(t) = tally.get(&entity) else { continue };
        if t.total <= 0.0 {
            continue;
        }
        let avg_happy = t.happy / t.total;
        let support: f64 = t
            .ideology
            .iter()
            .filter(|(&id, _)| government_aligns(nation.government, id))
            .map(|(_, &v)| v)
            .sum::<f64>()
            / t.total;

        nation.stability = stability_after(nation.stability, avg_happy, support);

        if nation.stability < UNREST_THRESHOLD {
            politics.turmoil_months += 1;
            unrest_count += 1;
        } else {
            politics.turmoil_months = 0;
        }

        // Sustained, deep unrest tips into revolution/coup (design §13).
        if nation.stability < REVOLT_THRESHOLD && politics.turmoil_months >= REVOLT_MONTHS {
            if let Some((&winner, _)) = t
                .ideology
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                nation.government = government_for(winner);
            }
            nation.stability = REVOLT_RESET; // new-regime honeymoon
            nation.prestige -= REVOLT_PRESTIGE; // upheaval costs standing
            politics.turmoil_months = 0;
            revolutions += 1;
        }

        politics.unrest = 1.0 - nation.stability;
        sum_stab += nation.stability;
        min_stab = min_stab.min(nation.stability);
        n += 1.0;
    }

    ledger.avg_stability = if n > 0.0 { sum_stab / n } else { 0.0 };
    ledger.min_stability = if n > 0.0 { min_stab } else { 0.0 };
    ledger.unrest_count = unrest_count;
    ledger.revolutions = revolutions;
}

/// Diplomacy (design §14), run monthly. Every relation eases toward a target of
/// government compatibility plus a commercial-peace bonus from the month's
/// cross-border trade, then the standing of a nation's neighbours feeds back into
/// its prestige and the stability of its regime (design §14 → §13).
pub fn diplomacy(
    clock: Res<GameClock>,
    mut nations: Query<(Entity, &mut Nation)>,
    mut diplo: ResMut<Diplomacy>,
    mut ledger: ResMut<DiplomacyLedger>,
) {
    if !clock.is_month_start() {
        return;
    }

    // Snapshot governments before we mutate prestige/stability below.
    let govs: Vec<(Entity, Government)> = nations.iter().map(|(e, n)| (e, n.government)).collect();
    // Drain this month's commerce so only recent trade counts toward relations.
    let trade = std::mem::take(&mut diplo.trade_flow);

    let (mut allies, mut rivalries) = (0, 0);
    for i in 0..govs.len() {
        for j in (i + 1)..govs.len() {
            let (a, ga) = govs[i];
            let (b, gb) = govs[j];
            let key = Diplomacy::pair(a, b);
            let trade_value = trade.get(&key).copied().unwrap_or(0.0);
            let current = *diplo.relations.get(&key).unwrap_or(&0.0);
            let next = relation_after(current, government_affinity(ga, gb), trade_value);
            diplo.relations.insert(key, next);
            match relation_status(next) {
                Relation::Ally => allies += 1,
                Relation::Rival | Relation::Hostile => rivalries += 1,
                Relation::Neutral => {}
            }
        }
    }
    ledger.allies = allies;
    ledger.rivalries = rivalries;

    // Average each nation's relations, then let a friendly neighbourhood lift its
    // prestige and steady its regime — and hostility erode both.
    let mut net: HashMap<Entity, (f64, u32)> = HashMap::new();
    for (&(a, b), &score) in &diplo.relations {
        let ea = net.entry(a).or_default();
        ea.0 += score;
        ea.1 += 1;
        let eb = net.entry(b).or_default();
        eb.0 += score;
        eb.1 += 1;
    }
    for (entity, mut nation) in &mut nations {
        if let Some(&(sum, count)) = net.get(&entity) {
            if count > 0 {
                let avg = sum / count as f64;
                nation.prestige += avg * PRESTIGE_GAIN;
                nation.stability = (nation.stability + avg * STABILITY_DIPLO).clamp(0.0, 1.0);
            }
        }
    }
}

/// The corporate "AI" (design §8), run monthly. A firm that turned a profit pays
/// corporate tax to its nation and reinvests by hiring — up to the engineers its
/// region can supply. A firm that lost money sheds workers. Profit is then reset
/// for the new month. Firms answer to their books, not the state.
pub fn corporate_decisions(
    clock: Res<GameClock>,
    pops: Query<&Pop>,
    mut corps: Query<&mut Corporation>,
    mut nations: Query<&mut Nation>,
    banks: Query<&CentralBank>,
) {
    if !clock.is_month_start() {
        return;
    }
    let mut engineers: HashMap<Entity, f64> = HashMap::new();
    for pop in &pops {
        if pop.profession == Profession::Engineer {
            *engineers.entry(pop.region).or_default() += pop.size as f64;
        }
    }

    for mut corp in &mut corps {
        if corp.profit > 0.0 {
            let tax = corp.profit * CORP_TAX;
            if let Ok(mut nation) = nations.get_mut(corp.owner) {
                nation.treasury += tax;
            }
            corp.capital -= tax;

            // Monetary transmission (design §11): dear credit slows expansion.
            // The home central bank's policy rate scales how many engineers a
            // firm takes on this month — at the rate cap hiring nearly stops.
            let rate = banks.get(corp.owner).map(|b| b.policy_rate).unwrap_or(0.03);
            let drag = (1.0 - rate / RATE_CAP).clamp(0.2, 1.0);
            let hire = HIRE_STEP * drag;
            let cost = HIRE_COST * hire;

            // Reinvest in headcount if cash allows and engineers are free.
            let room = engineers.get(&corp.region).copied().unwrap_or(0.0);
            if corp.capital > cost && corp.employees + hire <= room {
                corp.employees += hire;
                corp.capital -= cost;
            }
        } else if corp.profit < 0.0 {
            corp.employees = (corp.employees * 0.9).max(MIN_EMPLOYEES);
        }
        corp.profit = 0.0;
    }
}

/// The fiscal side (design §11): once a month a government spends on its people
/// (welfare, services, administration) and services its existing debt. Tax
/// revenue (collected just before) and corporate tax fund it; any shortfall is
/// borrowed — the central bank prints the money, so debt and the money supply
/// both grow. A government in surplus pays part of it down.
pub fn government_finance(
    clock: Res<GameClock>,
    mut nations: Query<(Entity, &mut Nation, &mut CentralBank)>,
    regions: Query<&Region>,
    pops: Query<&Pop>,
) {
    if !clock.is_month_start() {
        return;
    }
    let mut people: HashMap<Entity, f64> = HashMap::new();
    for pop in &pops {
        if let Ok(region) = regions.get(pop.region) {
            *people.entry(region.owner).or_default() += pop.size as f64;
        }
    }

    for (entity, mut nation, mut bank) in &mut nations {
        let spending = people.get(&entity).copied().unwrap_or(0.0) * WELFARE_PER_CAPITA;
        let interest = nation.debt * bank.policy_rate / 12.0;
        let outlay = spending + interest;

        if nation.treasury >= outlay {
            nation.treasury -= outlay;
            // Run a surplus down against the debt.
            if nation.debt > 0.0 && nation.treasury > 0.0 {
                let repay = (nation.treasury * DEBT_REPAY_SHARE).min(nation.debt);
                nation.treasury -= repay;
                nation.debt -= repay;
            }
        } else {
            // Finance the gap by borrowing — the bank creates the money.
            let shortfall = outlay - nation.treasury;
            nation.treasury = 0.0;
            nation.debt += shortfall;
            bank.money_supply += shortfall;
        }
    }
}

/// Inflation is no longer fixed (design §11): each month it eases toward this
/// month's monetary growth — the new money the bank printed to cover deficits.
/// Stable money settles toward zero inflation; monetising a deficit shows up as
/// inflation a month later. (Inflation is the *change* in the money/price level,
/// not the level itself, so the absolute price-vs-base gap deliberately plays no
/// part here.)
pub fn update_inflation(clock: Res<GameClock>, mut nations: Query<(&mut Nation, &mut CentralBank)>) {
    if !clock.is_month_start() {
        return;
    }
    for (mut nation, mut bank) in &mut nations {
        let growth = (bank.money_supply - bank.last_money_supply) / bank.last_money_supply.max(1.0);
        nation.inflation += (growth - nation.inflation) * INFLATION_EASE;
        bank.last_money_supply = bank.money_supply;
    }
}

/// Monetary policy (design §11): each central bank moves its policy rate to lean
/// against the inflation gap — raising when inflation runs above target, cutting
/// when it falls below — bounded to a sane band.
pub fn monetary_policy(clock: Res<GameClock>, mut banks: Query<(&Nation, &mut CentralBank)>) {
    if !clock.is_month_start() {
        return;
    }
    for (nation, mut bank) in &mut banks {
        bank.policy_rate = policy_rate_after(bank.policy_rate, nation.inflation, bank.target_inflation);
    }
}

/// Roll the per-nation finances up into the [`FinanceLedger`] for the report,
/// and raise the crisis flag (design §11 events) on runaway inflation or a
/// sovereign-debt spiral anywhere in the world.
pub fn finance_oversight(
    clock: Res<GameClock>,
    nations: Query<(&Nation, &CentralBank)>,
    mut ledger: ResMut<FinanceLedger>,
) {
    if !clock.is_month_start() {
        return;
    }
    let (mut gov_debt, mut money, mut infl, mut n) = (0.0, 0.0, 0.0, 0.0);
    let mut max_rate: f64 = 0.0;
    let mut crisis = false;
    for (nation, bank) in &nations {
        gov_debt += nation.debt;
        money += bank.money_supply;
        infl += nation.inflation;
        max_rate = max_rate.max(bank.policy_rate);
        n += 1.0;
        if nation.inflation > CRISIS_INFLATION || nation.debt > CRISIS_DEBT_RATIO * bank.money_supply {
            crisis = true;
        }
    }
    ledger.gov_debt = gov_debt;
    ledger.money_supply = money;
    ledger.avg_inflation = if n > 0.0 { infl / n } else { 0.0 };
    ledger.max_policy_rate = max_rate;
    ledger.crisis = crisis;
}

/// Print a monthly state-of-the-world report.
pub fn report(
    clock: Res<GameClock>,
    market: Res<Market>,
    ledger: Res<TradeLedger>,
    finance: Res<FinanceLedger>,
    politics_led: Res<PoliticsLedger>,
    diplo_led: Res<DiplomacyLedger>,
    nations: Query<(Entity, &Nation, &CentralBank, &Politics)>,
    regions: Query<(&Region, &ResourceStock)>,
    pops: Query<&Pop>,
    corps: Query<&Corporation>,
) {
    if !clock.is_month_start() {
        return;
    }

    // Aggregate population and grain per nation via region ownership.
    let mut population: HashMap<Entity, u64> = HashMap::new();
    let mut grain: HashMap<Entity, f64> = HashMap::new();
    let mut happiness_sum: HashMap<Entity, f64> = HashMap::new();
    let mut happiness_weight: HashMap<Entity, f64> = HashMap::new();

    for (region, stock) in &regions {
        *grain.entry(region.owner).or_default() += stock.get(Good::Grain);
    }
    for pop in &pops {
        if let Ok((region, _)) = regions.get(pop.region) {
            let owner = region.owner;
            *population.entry(owner).or_default() += pop.size as u64;
            *happiness_sum.entry(owner).or_default() += pop.happiness * pop.size as f64;
            *happiness_weight.entry(owner).or_default() += pop.size as f64;
        }
    }

    println!(
        "── Year {} Month {} (day {}) ──",
        clock.year(),
        clock.month(),
        clock.day
    );
    for (entity, nation, bank, politics) in &nations {
        let pop = population.get(&entity).copied().unwrap_or(0);
        let g = grain.get(&entity).copied().unwrap_or(0.0);
        let happy = match happiness_weight.get(&entity).copied().unwrap_or(0.0) {
            w if w > 0.0 => happiness_sum.get(&entity).copied().unwrap_or(0.0) / w,
            _ => 0.0,
        };
        println!(
            "  {:<12} pop {:>10}  treasury {:>12.0}  grain {:>10.0}  happiness {:>4.2}  trade +{:>9.0}/-{:>9.0}",
            nation.name, pop, nation.treasury, g, happy, nation.exports, nation.imports
        );
        println!(
            "  {:<12} rate {:>5.2}%  inflation {:>6.2}%  debt {:>12.0}  money {:>12.0}",
            "", bank.policy_rate * 100.0, nation.inflation * 100.0, nation.debt, bank.money_supply
        );
        println!(
            "  {:<12} {:<10?}  stability {:>4.2}  unrest {:>4.2}  prestige {:>7.1}{}",
            "", nation.government, nation.stability, politics.unrest, nation.prestige,
            if politics.unrest > 1.0 - UNREST_THRESHOLD { "  ⚑ UNREST" } else { "" }
        );
    }

    // Market snapshot across the active production chain.
    print!("  market ");
    for &good in &[
        Good::Grain, Good::IronOre, Good::Coal, Good::Oil,
        Good::Iron, Good::Steel, Good::Plastic, Good::Car,
    ] {
        print!("{:?} {:>6.1}  ", good, market.price(good));
    }
    println!();

    // Corporations: who is making money, and at what scale (design §8).
    for corp in &corps {
        let nation = nations
            .get(corp.owner)
            .map(|(_, n, _, _)| n.name.as_str())
            .unwrap_or("?");
        println!(
            "  firm {:<14} [{:<8}] capital {:>12.0}  staff {:>8.0}  profit {:>10.0}",
            corp.name, nation, corp.capital, corp.employees, corp.profit
        );
    }
    println!("  trade volume: steel {:>8.0}  car {:>8.0}  total value {:>12.0}",
        ledger.volume(Good::Steel), ledger.volume(Good::Car), ledger.value);

    // Finance dashboard (design §11): world money, debt, top rate, mean inflation.
    println!(
        "  finance: money {:>12.0}  gov debt {:>12.0}  top rate {:>5.2}%  avg inflation {:>6.2}%{}",
        finance.money_supply,
        finance.gov_debt,
        finance.max_policy_rate * 100.0,
        finance.avg_inflation * 100.0,
        if finance.crisis { "  ⚠ FINANCIAL CRISIS" } else { "" }
    );

    // Politics (design §13) and diplomacy (design §14) dashboards.
    println!(
        "  politics: avg stability {:>4.2}  min {:>4.2}  in unrest {}  revolutions {}{}",
        politics_led.avg_stability,
        politics_led.min_stability,
        politics_led.unrest_count,
        politics_led.revolutions,
        if politics_led.revolutions > 0 { "  ⚑ REGIME CHANGE" } else { "" }
    );
    println!(
        "  diplomacy: allies {}  rivalries {}",
        diplo_led.allies, diplo_led.rivalries
    );
    println!();
}

fn terrain_food_mod(t: Terrain) -> f64 {
    match t {
        Terrain::Plains => 1.3,
        Terrain::Hills => 0.9,
        Terrain::Mountains => 0.4,
        Terrain::Coast => 1.0,
        Terrain::Desert => 0.2,
    }
}

fn climate_food_mod(c: Climate) -> f64 {
    match c {
        Climate::Temperate => 1.2,
        Climate::Tropical => 1.1,
        Climate::Continental => 0.9,
        Climate::Arid => 0.5,
        Climate::Polar => 0.3,
    }
}
