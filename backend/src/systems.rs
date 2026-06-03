//! The Phase 1 simulation loop, expressed as ECS systems.
//!
//! Causal chain (design §2): pops → production → consumption → happiness →
//! population change. Each system runs once per simulated day, in order.

use crate::components::*;
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
/// Cars a pop wants per person per day, scaled by wealth (luxury demand).
const CAR_APPETITE: f64 = 1.0e-6;
/// How sharply prices react to a supply/demand imbalance each day.
const PRICE_ELASTICITY: f64 = 0.08;

/// Build the daily schedule: all Phase 1 systems in fixed causal order.
pub fn build_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            advance_clock,
            produce_food,
            extract_resources,
            manufacture,
            consume_food,
            consume_goods,
            pop_growth,
            collect_taxes,
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

/// Engineers run factories: each region spends an engineer-labour budget on
/// recipes in tier order, refining stocked inputs into higher goods. Inputs
/// consumed are demand; outputs produced are supply.
pub fn manufacture(
    pops: Query<&Pop>,
    mut regions: Query<(Entity, &mut ResourceStock)>,
    mut market: ResMut<Market>,
) {
    let mut labor: HashMap<Entity, f64> = HashMap::new();
    for pop in &pops {
        if pop.profession == Profession::Engineer {
            *labor.entry(pop.region).or_default() += pop.size as f64;
        }
    }
    for (entity, mut stock) in &mut regions {
        let mut budget = labor.get(&entity).copied().unwrap_or(0.0) * MANUFACTURE_RATE;
        if budget <= 0.0 {
            continue;
        }
        for recipe in RECIPES {
            if budget <= 0.0 {
                break;
            }
            // Units we can make: limited by labour budget and every input stock.
            let mut units = budget / recipe.labor;
            for &(good, qty) in recipe.inputs {
                units = units.min(stock.get(good) / qty);
            }
            if units <= 0.0 {
                continue;
            }
            for &(good, qty) in recipe.inputs {
                let used = qty * units;
                stock.take(good, used);
                market.record_demand(good, used);
            }
            stock.add(recipe.output, units);
            market.record_supply(recipe.output, units);
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
            nation.treasury += amount;
        }
    }
}

/// Print a monthly state-of-the-world report.
pub fn report(
    clock: Res<GameClock>,
    market: Res<Market>,
    nations: Query<(Entity, &Nation)>,
    regions: Query<(&Region, &ResourceStock)>,
    pops: Query<&Pop>,
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
    for (entity, nation) in &nations {
        let pop = population.get(&entity).copied().unwrap_or(0);
        let g = grain.get(&entity).copied().unwrap_or(0.0);
        let happy = match happiness_weight.get(&entity).copied().unwrap_or(0.0) {
            w if w > 0.0 => happiness_sum.get(&entity).copied().unwrap_or(0.0) / w,
            _ => 0.0,
        };
        println!(
            "  {:<12} pop {:>10}  treasury {:>12.0}  grain {:>10.0}  happiness {:>4.2}",
            nation.name, pop, nation.treasury, g, happy
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
    println!("\n");
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
