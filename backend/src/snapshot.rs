//! A serialisable, read-only view of the world for the client (UI roadmap Phase 1).
//!
//! The simulation lives in the ECS; the UI needs a flat, JSON-friendly snapshot of
//! it. `world_snapshot` walks the same state the monthly [`report`](crate::systems::report)
//! prints and packs it into structs that serialise (camelCase) to exactly the shape
//! the React client's `WorldView` expects, plus the game clock for the loop viewer.

use crate::components::{Government, Nation, Pop, Profession, Region};
use crate::corporation::Corporation;
use crate::diplomacy::{relation_status, Diplomacy, Relation};
use crate::finance::{CentralBank, FinanceLedger};
use crate::politics::Politics;
use crate::production::{base_price, GdpLedger, Market, TRADED};
use crate::resources::{Deposits, GameClock, Good};
use crate::war::{military_power, Military, Warfront};
use bevy_ecs::prelude::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClockView {
    pub day: u64,
    pub year: u64,
    pub month: u64,
    pub day_of_month: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct NationView {
    pub id: String,
    pub name: String,
    /// Display colour as a 0xRRGGBB integer (Pixi-friendly).
    pub color: u32,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegionView {
    pub id: u32,
    pub name: String,
    pub nation_id: String,
    pub population: u64,
    /// Terrain type, e.g. "Plains".
    pub terrain: String,
    /// Climate type, e.g. "Temperate".
    pub climate: String,
    /// Infrastructure quality, 0..1.
    pub infrastructure: f64,
    /// Natural endowment: raw goods the region is rich in, with abundance (design §6).
    pub resources: Vec<RegionResource>,
    /// Value added in the region over the last month (design §17 GDP).
    pub gdp: f64,
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct RegionResource {
    pub good: String,
    /// Extraction multiplier from the region's deposits.
    pub abundance: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct GoodPrice {
    pub good: String,
    pub tier: String,
    pub price: f64,
    pub base: f64,
    /// The last day's market supply flow (Phase 7 market visualisation).
    pub supply: f64,
    /// The last day's market demand flow (Phase 7 market visualisation).
    pub demand: f64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CorporationView {
    pub name: String,
    pub nation_id: String,
    pub industries: Vec<String>,
    pub capital: f64,
    pub employees: f64,
    /// Gross sales accumulated this month (resets monthly).
    pub revenue: f64,
    /// Margin accumulated this month (resets monthly).
    pub profit: f64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EconomyView {
    pub nation_id: String,
    /// Cash in the national treasury.
    pub treasury: f64,
    /// Gross domestic product: value added across the nation's regions last month.
    pub gdp: f64,
    /// Cumulative value of goods sold abroad (Phase 3 trade balance).
    pub exports: f64,
    /// Cumulative value of goods bought from abroad.
    pub imports: f64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FinanceView {
    pub nation_id: String,
    pub policy_rate: f64,
    pub inflation: f64,
    pub debt: f64,
    pub money_supply: f64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PoliticsView {
    pub nation_id: String,
    pub government: String,
    pub stability: f64,
    pub unrest: f64,
    /// Population-weighted average happiness, 0..1 (drives stability, design §13).
    pub happiness: f64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TechnologyView {
    pub nation_id: String,
    /// Aggregate technology level index (design §12, §17).
    pub level: f64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LaborView {
    pub nation_id: String,
    /// Working population in the labour force. The simulation models only the
    /// economically active population, so every pop is a member of the force.
    pub labor_force: f64,
    /// Heads holding a job (the labour force less the unemployed).
    pub employed: f64,
    /// Heads without work: explicitly unemployed pops plus engineers no firm in
    /// their region is hiring (idle industrial labour, design §8). 실업자.
    pub unemployed: f64,
    /// Unemployed share of the labour force, 0..1. 실업률.
    pub unemployment_rate: f64,
    /// Employed share of the *working-age* population, 0..1 — the labour-force
    /// employment scaled by the participation rate so it reads like a real
    /// employment rate. 고용률.
    pub employment_rate: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct RelationView {
    pub a: String,
    pub b: String,
    pub status: String,
    pub score: f64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MilitaryView {
    pub nation_id: String,
    pub power: f64,
    pub strength: f64,
    pub exhaustion: f64,
    pub at_war: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct WarView {
    pub aggressor: String,
    pub defender: String,
}

/// One cross-border trade route (UI roadmap Phase 9 — trade network). The pair is
/// undirected (the logistics layer records by normalised pair), so `value` is the
/// total commerce flowing between the two nations this month, regardless of direction.
#[derive(Serialize, Clone, Debug)]
pub struct TradeRouteView {
    pub a: String,
    pub b: String,
    /// Cross-border trade value between the pair, accumulated this month.
    pub value: f64,
}

/// One population class (UI roadmap Phase 10 — POP dashboard). Aggregates every
/// pop of a profession worldwide: total heads, plus size-weighted averages of the
/// indicators the engine already carries on each pop.
#[derive(Serialize, Clone, Debug)]
pub struct PopClassView {
    /// Profession name (English enum, labelled client-side).
    pub profession: String,
    /// Total heads in this class, worldwide.
    pub size: f64,
    /// Size-weighted average wealth — the class's income proxy.
    pub income: f64,
    /// Size-weighted average literacy (education), 0..1.
    pub literacy: f64,
    /// Size-weighted average happiness, 0..1.
    pub happiness: f64,
}

/// The whole world, flattened for the client. Mirrors the React `WorldView`,
/// with the game clock added for the loop viewer.
#[derive(Serialize, Clone, Debug)]
pub struct WorldSnapshot {
    pub clock: ClockView,
    pub nations: Vec<NationView>,
    pub regions: Vec<RegionView>,
    pub prices: Vec<GoodPrice>,
    pub corporations: Vec<CorporationView>,
    pub economy: Vec<EconomyView>,
    pub finance: Vec<FinanceView>,
    pub crisis: bool,
    pub politics: Vec<PoliticsView>,
    pub technology: Vec<TechnologyView>,
    pub relations: Vec<RelationView>,
    pub military: Vec<MilitaryView>,
    pub wars: Vec<WarView>,
    /// Per-nation labour market: force, employment, unemployment (실업률/고용률).
    pub labor: Vec<LaborView>,
    /// Cross-border trade routes between nation pairs (Phase 9 trade network).
    #[serde(rename = "tradeRoutes")]
    pub trade_routes: Vec<TradeRouteView>,
    /// Population by class, worldwide (Phase 10 POP dashboard).
    pub population: Vec<PopClassView>,
}

/// Share of the working-age population in the labour force. The model carries
/// only the active population, so this maps the labour-force figures onto a
/// realistic employment rate (고용률 ≈ employed over the whole working-age pop).
const PARTICIPATION: f64 = 0.63;

/// Walk the ECS world and pack it into a flat, serialisable snapshot.
pub fn world_snapshot(world: &mut World) -> WorldSnapshot {
    // --- clock ---
    let clock = {
        let c = world.resource::<GameClock>();
        ClockView { day: c.day, year: c.year(), month: c.month(), day_of_month: c.day_of_month() }
    };

    // --- nations (raw, keyed by entity for the cross-references below) ---
    struct NatRaw {
        e: Entity,
        name: String,
        gov: Government,
        tech: f64,
        stability: f64,
        inflation: f64,
        treasury: f64,
        debt: f64,
        exports: f64,
        imports: f64,
        rate: f64,
        money: f64,
        strength: f64,
        exhaustion: f64,
        unrest: f64,
    }
    let mut nats: Vec<NatRaw> = Vec::new();
    {
        let mut q = world.query::<(Entity, &Nation, &CentralBank, &Politics, &Military)>();
        for (e, n, b, p, m) in q.iter(world) {
            nats.push(NatRaw {
                e,
                name: n.name.clone(),
                gov: n.government,
                tech: n.technology,
                stability: n.stability,
                inflation: n.inflation,
                treasury: n.treasury,
                debt: n.debt,
                exports: n.exports,
                imports: n.imports,
                rate: b.policy_rate,
                money: b.money_supply,
                strength: m.strength,
                exhaustion: m.exhaustion,
                unrest: p.unrest,
            });
        }
    }
    let nation_id_of: HashMap<Entity, String> =
        nats.iter().map(|n| (n.e, nation_id(&n.name))).collect();

    // --- regions (name + owner + terrain/climate/infra + deposits) ---
    struct RegRaw {
        e: Entity,
        name: String,
        owner: Entity,
        terrain: String,
        climate: String,
        infrastructure: f64,
        resources: Vec<RegionResource>,
    }
    let mut regs: Vec<RegRaw> = Vec::new();
    {
        let mut q = world.query::<(Entity, &Region, &Deposits)>();
        for (e, r, d) in q.iter(world) {
            let mut resources: Vec<RegionResource> = d
                .0
                .iter()
                .map(|(&g, &abundance)| RegionResource { good: good_label(g), abundance })
                .collect();
            resources.sort_by(|a, b| b.abundance.total_cmp(&a.abundance));
            regs.push(RegRaw {
                e,
                name: r.name.clone(),
                owner: r.owner,
                terrain: format!("{:?}", r.terrain),
                climate: format!("{:?}", r.climate),
                infrastructure: r.infrastructure,
                resources,
            });
        }
    }
    let region_owner: HashMap<Entity, Entity> = regs.iter().map(|r| (r.e, r.owner)).collect();

    // --- GDP per region (last month's value added), summed per owning nation ---
    let region_gdp: HashMap<Entity, f64> = {
        let gdp = world.resource::<GdpLedger>();
        regs.iter().map(|r| (r.e, gdp.region(r.e))).collect()
    };
    let mut nation_gdp: HashMap<Entity, f64> = HashMap::new();
    for r in &regs {
        *nation_gdp.entry(r.owner).or_default() += region_gdp.get(&r.e).copied().unwrap_or(0.0);
    }

    // --- pops: population per region, soldier manpower + happiness per nation ---
    let mut pop_by_region: HashMap<Entity, u64> = HashMap::new();
    let mut soldiers_by_nation: HashMap<Entity, f64> = HashMap::new();
    // Per nation: total heads and happiness-weighted heads (mirrors the politics
    // system's stability input, design §13).
    let mut heads_by_nation: HashMap<Entity, f64> = HashMap::new();
    let mut happy_by_nation: HashMap<Entity, f64> = HashMap::new();
    // Labour inputs (실업률, design §8): engineers available per region, and the
    // heads explicitly out of work per nation.
    let mut engineers_by_region: HashMap<Entity, f64> = HashMap::new();
    let mut unemployed_by_nation: HashMap<Entity, f64> = HashMap::new();
    // Phase 10 POP dashboard: per-profession worldwide aggregates, carried as
    // (heads, wealth·heads, literacy·heads, happiness·heads) for size-weighted means.
    let mut pop_class: HashMap<Profession, (f64, f64, f64, f64)> = HashMap::new();
    {
        let mut q = world.query::<&Pop>();
        for pop in q.iter(world) {
            *pop_by_region.entry(pop.region).or_default() += pop.size as u64;
            if pop.profession == Profession::Engineer {
                *engineers_by_region.entry(pop.region).or_default() += pop.size as f64;
            }
            {
                let s = pop.size as f64;
                let acc = pop_class.entry(pop.profession).or_default();
                acc.0 += s;
                acc.1 += pop.wealth * s;
                acc.2 += pop.literacy * s;
                acc.3 += pop.happiness * s;
            }
            if let Some(&owner) = region_owner.get(&pop.region) {
                let size = pop.size as f64;
                *heads_by_nation.entry(owner).or_default() += size;
                *happy_by_nation.entry(owner).or_default() += pop.happiness * size;
                if pop.profession == Profession::Soldier {
                    *soldiers_by_nation.entry(owner).or_default() += size;
                }
                if pop.profession == Profession::Unemployed {
                    *unemployed_by_nation.entry(owner).or_default() += size;
                }
            }
        }
    }

    // --- corporations ---
    struct CorpRaw {
        name: String,
        owner: Entity,
        region: Entity,
        capital: f64,
        employees: f64,
        revenue: f64,
        profit: f64,
        industries: Vec<Good>,
    }
    let mut corps: Vec<CorpRaw> = Vec::new();
    {
        let mut q = world.query::<&Corporation>();
        for c in q.iter(world) {
            corps.push(CorpRaw {
                name: c.name.clone(),
                owner: c.owner,
                region: c.region,
                capital: c.capital,
                employees: c.employees,
                revenue: c.revenue,
                profit: c.profit,
                industries: c.industries.clone(),
            });
        }
    }

    // --- labour market (실업률, design §8): the only slack in the modelled
    // workforce is industrial — engineers a region has but no firm there is
    // hiring sit idle. Sum that idle labour (plus any explicitly unemployed pops)
    // against the labour force to read each nation's unemployment. ---
    let mut corp_emp_by_region: HashMap<Entity, f64> = HashMap::new();
    for c in &corps {
        *corp_emp_by_region.entry(c.region).or_default() += c.employees;
    }
    let mut idle_by_nation: HashMap<Entity, f64> = HashMap::new();
    for (&region, &eng) in &engineers_by_region {
        let used = corp_emp_by_region.get(&region).copied().unwrap_or(0.0);
        let idle = (eng - used).max(0.0);
        if let Some(&owner) = region_owner.get(&region) {
            *idle_by_nation.entry(owner).or_default() += idle;
        }
    }

    // --- prices + crisis flag (immutable resource reads) ---
    let prices: Vec<GoodPrice> = {
        let market = world.resource::<Market>();
        TRADED
            .iter()
            .map(|&g| GoodPrice {
                good: good_label(g),
                tier: good_tier(g).to_string(),
                price: market.price(g),
                base: base_price(g),
                supply: market.last_supply(g),
                demand: market.last_demand(g),
            })
            .collect()
    };
    let crisis = world.resource::<FinanceLedger>().crisis;

    // --- relations + active wars + trade routes (Phase 9) ---
    let (relations, wars, at_war, trade_routes): (
        Vec<RelationView>,
        Vec<WarView>,
        HashSet<Entity>,
        Vec<TradeRouteView>,
    ) = {
        let diplo = world.resource::<Diplomacy>();
        let warfront = world.resource::<Warfront>();
        let relations = diplo
            .relations
            .iter()
            .filter_map(|(&(a, b), &score)| {
                Some(RelationView {
                    a: nation_id_of.get(&a)?.clone(),
                    b: nation_id_of.get(&b)?.clone(),
                    status: relation_label(relation_status(score)).to_string(),
                    score,
                })
            })
            .collect();
        // Cross-border commerce accumulated since the last monthly diplomacy pass.
        let mut trade_routes: Vec<TradeRouteView> = diplo
            .trade_flow
            .iter()
            .filter(|(_, &value)| value > 0.0)
            .filter_map(|(&(a, b), &value)| {
                Some(TradeRouteView {
                    a: nation_id_of.get(&a)?.clone(),
                    b: nation_id_of.get(&b)?.clone(),
                    value,
                })
            })
            .collect();
        trade_routes.sort_by(|x, y| y.value.total_cmp(&x.value));
        let wars = warfront
            .wars
            .iter()
            .filter_map(|w| {
                Some(WarView {
                    aggressor: nation_id_of.get(&w.aggressor)?.clone(),
                    defender: nation_id_of.get(&w.defender)?.clone(),
                })
            })
            .collect();
        let mut set = HashSet::new();
        for w in &warfront.wars {
            set.insert(w.aggressor);
            set.insert(w.defender);
        }
        (relations, wars, set, trade_routes)
    };

    // --- assemble the per-nation views ---
    let nations: Vec<NationView> = nats
        .iter()
        .map(|n| NationView { id: nation_id(&n.name), name: n.name.clone(), color: nation_color(&n.name) })
        .collect();
    let economy: Vec<EconomyView> = nats
        .iter()
        .map(|n| EconomyView {
            nation_id: nation_id(&n.name),
            treasury: n.treasury,
            gdp: nation_gdp.get(&n.e).copied().unwrap_or(0.0),
            exports: n.exports,
            imports: n.imports,
        })
        .collect();
    let finance: Vec<FinanceView> = nats
        .iter()
        .map(|n| FinanceView {
            nation_id: nation_id(&n.name),
            policy_rate: n.rate,
            inflation: n.inflation,
            debt: n.debt,
            money_supply: n.money,
        })
        .collect();
    let politics: Vec<PoliticsView> = nats
        .iter()
        .map(|n| {
            let heads = heads_by_nation.get(&n.e).copied().unwrap_or(0.0);
            let happiness =
                if heads > 0.0 { happy_by_nation.get(&n.e).copied().unwrap_or(0.0) / heads } else { 0.0 };
            PoliticsView {
                nation_id: nation_id(&n.name),
                government: format!("{:?}", n.gov),
                stability: n.stability,
                unrest: n.unrest,
                happiness,
            }
        })
        .collect();
    let technology: Vec<TechnologyView> = nats
        .iter()
        .map(|n| TechnologyView { nation_id: nation_id(&n.name), level: n.tech })
        .collect();
    let labor: Vec<LaborView> = nats
        .iter()
        .map(|n| {
            let labor_force = heads_by_nation.get(&n.e).copied().unwrap_or(0.0);
            let unemployed = (unemployed_by_nation.get(&n.e).copied().unwrap_or(0.0)
                + idle_by_nation.get(&n.e).copied().unwrap_or(0.0))
            .min(labor_force);
            let employed = (labor_force - unemployed).max(0.0);
            let (unemployment_rate, employment_rate) = if labor_force > 0.0 {
                (unemployed / labor_force, (employed / labor_force) * PARTICIPATION)
            } else {
                (0.0, 0.0)
            };
            LaborView {
                nation_id: nation_id(&n.name),
                labor_force,
                employed,
                unemployed,
                unemployment_rate,
                employment_rate,
            }
        })
        .collect();
    let military: Vec<MilitaryView> = nats
        .iter()
        .map(|n| {
            let soldiers = soldiers_by_nation.get(&n.e).copied().unwrap_or(0.0);
            MilitaryView {
                nation_id: nation_id(&n.name),
                power: military_power(n.strength, soldiers, n.tech, n.exhaustion),
                strength: n.strength,
                exhaustion: n.exhaustion,
                at_war: at_war.contains(&n.e),
            }
        })
        .collect();

    let regions: Vec<RegionView> = regs
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let (x, y) = region_xy(&r.name, i);
            RegionView {
                id: r.e.index(),
                name: r.name.clone(),
                nation_id: nation_id_of.get(&r.owner).cloned().unwrap_or_default(),
                population: pop_by_region.get(&r.e).copied().unwrap_or(0),
                terrain: r.terrain.clone(),
                climate: r.climate.clone(),
                infrastructure: r.infrastructure,
                resources: r.resources.clone(),
                gdp: region_gdp.get(&r.e).copied().unwrap_or(0.0),
                x,
                y,
            }
        })
        .collect();

    let corporations: Vec<CorporationView> = corps
        .iter()
        .map(|c| CorporationView {
            name: c.name.clone(),
            nation_id: nation_id_of.get(&c.owner).cloned().unwrap_or_default(),
            industries: c.industries.iter().map(|&g| good_label(g)).collect(),
            capital: c.capital,
            employees: c.employees,
            revenue: c.revenue,
            profit: c.profit,
        })
        .collect();

    // --- population by class (Phase 10): size-weighted means from the pop accumulators ---
    let mut population: Vec<PopClassView> = pop_class
        .iter()
        .map(|(&prof, &(size, wealth, lit, hap))| PopClassView {
            profession: format!("{:?}", prof),
            size,
            income: if size > 0.0 { wealth / size } else { 0.0 },
            literacy: if size > 0.0 { lit / size } else { 0.0 },
            happiness: if size > 0.0 { hap / size } else { 0.0 },
        })
        .collect();
    population.sort_by(|a, b| b.size.total_cmp(&a.size));

    WorldSnapshot {
        clock,
        nations,
        regions,
        prices,
        corporations,
        economy,
        finance,
        crisis,
        politics,
        technology,
        relations,
        military,
        wars,
        labor,
        trade_routes,
        population,
    }
}

fn nation_id(name: &str) -> String {
    name.to_lowercase()
}

/// Display colour per nation. The hand-authored world's three nations get the
/// client's existing palette; any other nation gets a stable colour from its name.
fn nation_color(name: &str) -> u32 {
    match name {
        "Aurelia" => 0x4f_9d69,
        "Khoresan" => 0xd9_a441,
        "Nordheim" => 0x5b_7fb5,
        other => {
            // FNV-1a over the name, kept in the mid-brightness band so it reads on dark.
            let mut h: u32 = 2166136261;
            for b in other.bytes() {
                h = (h ^ b as u32).wrapping_mul(16777619);
            }
            0x40_4040 | (h & 0x7f_7f7f)
        }
    }
}

/// Normalised map position (0..1) per region. Known regions are laid out by hand
/// to match the seeded world; unknown ones fall on a deterministic spiral.
fn region_xy(name: &str, index: usize) -> (f64, f64) {
    match name {
        "Goldfields" => (0.22, 0.35),
        "Port Vesper" => (0.32, 0.60),
        "Sandreach" => (0.62, 0.40),
        "Oasis Hold" => (0.70, 0.62),
        "Frostmark" => (0.50, 0.18),
        _ => {
            // Golden-angle spiral around the centre — spreads any extra regions out.
            let golden = 2.399963;
            let a = index as f64 * golden;
            let r = 0.12 + 0.07 * (index as f64).sqrt();
            (0.5 + r * a.cos(), 0.5 + r * a.sin())
        }
    }
}

fn good_label(g: Good) -> String {
    format!("{:?}", g)
}

fn good_tier(g: Good) -> &'static str {
    use Good::*;
    match g {
        Grain | Wood | IronOre | Coal | Oil | Uranium | RareEarth => "raw",
        Iron | Steel | Plastic | Semiconductor | Battery => "intermediate",
        Car | Electronics | Weapon | Computer | Robot => "finished",
    }
}

fn relation_label(r: Relation) -> &'static str {
    match r {
        Relation::Ally => "ally",
        Relation::Neutral => "neutral",
        Relation::Rival => "rival",
        Relation::Hostile => "hostile",
    }
}
