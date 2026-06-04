//! Phase 2 economy: production recipes and the price-forming market.
//!
//! Raw goods are dug out of a region's [`Deposits`](crate::resources::Deposits);
//! recipes then refine them up the three tiers (design §6–§7). A single global
//! [`Market`] forms a price for each good from the day's supply vs demand —
//! there is no central planner, prices emerge from the imbalance (design §9).

use crate::resources::Good;
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::mem;

/// A production recipe: `inputs` are consumed to yield one unit of `output`.
/// `labor` is the engineer-labour budget spent per unit produced.
pub struct Recipe {
    pub output: Good,
    pub inputs: &'static [(Good, f64)],
    pub labor: f64,
}

/// All recipes, ordered raw→intermediate→finished so a single daily pass can
/// feed each tier from the one below it within the same region.
pub const RECIPES: &[Recipe] = &[
    Recipe { output: Good::Iron,    inputs: &[(Good::IronOre, 2.0), (Good::Coal, 1.0)], labor: 1.0 },
    Recipe { output: Good::Steel,   inputs: &[(Good::Iron, 2.0), (Good::Coal, 1.0)],    labor: 1.0 },
    Recipe { output: Good::Plastic, inputs: &[(Good::Oil, 2.0)],                         labor: 1.0 },
    Recipe { output: Good::Car,     inputs: &[(Good::Steel, 3.0), (Good::Plastic, 2.0)], labor: 4.0 },
];

/// Goods the market tracks a price for in Phase 2 (the active production chain).
pub const TRADED: &[Good] = &[
    Good::Grain, Good::Wood, Good::IronOre, Good::Coal, Good::Oil,
    Good::Iron, Good::Steel, Good::Plastic, Good::Car,
];

/// Starting / reference price for a good. Rises up the value chain.
pub fn base_price(g: Good) -> f64 {
    use Good::*;
    match g {
        Grain => 1.0,
        Wood => 1.5,
        IronOre => 2.0,
        Coal => 2.0,
        Oil => 4.0,
        Iron => 6.0,
        Steel => 12.0,
        Plastic => 10.0,
        Car => 120.0,
        _ => 1.0,
    }
}

/// Global commodity market. Holds a live price per good and accumulates the
/// current day's supply and demand flows, which drive the next price update.
#[derive(Resource, Debug)]
pub struct Market {
    pub prices: HashMap<Good, f64>,
    supply: HashMap<Good, f64>,
    demand: HashMap<Good, f64>,
    /// The previous day's completed supply/demand flows, retained when the live
    /// tallies are cleared so the client can read each good's last-day market
    /// activity (Phase 7 market visualisation).
    last_supply: HashMap<Good, f64>,
    last_demand: HashMap<Good, f64>,
}

impl Default for Market {
    fn default() -> Self {
        let prices = TRADED.iter().map(|&g| (g, base_price(g))).collect();
        Market {
            prices,
            supply: HashMap::new(),
            demand: HashMap::new(),
            last_supply: HashMap::new(),
            last_demand: HashMap::new(),
        }
    }
}

impl Market {
    pub fn price(&self, g: Good) -> f64 {
        *self.prices.get(&g).unwrap_or(&base_price(g))
    }
    pub fn record_supply(&mut self, g: Good, amount: f64) {
        *self.supply.entry(g).or_insert(0.0) += amount;
    }
    pub fn record_demand(&mut self, g: Good, amount: f64) {
        *self.demand.entry(g).or_insert(0.0) += amount;
    }
    pub fn supply(&self, g: Good) -> f64 {
        *self.supply.get(&g).unwrap_or(&0.0)
    }
    pub fn demand(&self, g: Good) -> f64 {
        *self.demand.get(&g).unwrap_or(&0.0)
    }
    /// The previous day's completed supply flow for a good (Phase 7).
    pub fn last_supply(&self, g: Good) -> f64 {
        *self.last_supply.get(&g).unwrap_or(&0.0)
    }
    /// The previous day's completed demand flow for a good (Phase 7).
    pub fn last_demand(&self, g: Good) -> f64 {
        *self.last_demand.get(&g).unwrap_or(&0.0)
    }
    /// Retire the day's flows once prices have been formed from them: keep a copy
    /// as the "last day" figures for the client (Phase 7), then clear the live
    /// tallies for the next day.
    pub fn clear_flows(&mut self) {
        self.last_supply = mem::take(&mut self.supply);
        self.last_demand = mem::take(&mut self.demand);
    }
}

/// Per-region GDP (design §17 economic victory). GDP is the *value added* in
/// production: each good's output valued at the market price, net of the
/// intermediate inputs consumed (raw extraction and food add their full value;
/// a factory adds revenue minus the cost of its inputs), so the production chain
/// is never double-counted. The month in progress accumulates in `month_output`;
/// at each month start it is rolled into `region_gdp`, the stable figure the
/// report and the client snapshot read.
#[derive(Resource, Debug, Default)]
pub struct GdpLedger {
    month_output: HashMap<Entity, f64>,
    pub region_gdp: HashMap<Entity, f64>,
}

impl GdpLedger {
    /// Record `value` of value-added produced in `region` today.
    pub fn add(&mut self, region: Entity, value: f64) {
        if value > 0.0 {
            *self.month_output.entry(region).or_insert(0.0) += value;
        }
    }

    /// Finalise the month: the value produced over it becomes the reported GDP.
    pub fn roll(&mut self) {
        self.region_gdp = mem::take(&mut self.month_output);
    }

    /// A region's GDP — the last completed month, or the month in progress
    /// before the first roll (so GDP reads nonzero from the first days).
    pub fn region(&self, region: Entity) -> f64 {
        self.region_gdp
            .get(&region)
            .copied()
            .unwrap_or_else(|| self.month_output.get(&region).copied().unwrap_or(0.0))
    }
}
