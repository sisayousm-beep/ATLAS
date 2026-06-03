//! Phase 2 economy: production recipes and the price-forming market.
//!
//! Raw goods are dug out of a region's [`Deposits`](crate::resources::Deposits);
//! recipes then refine them up the three tiers (design §6–§7). A single global
//! [`Market`] forms a price for each good from the day's supply vs demand —
//! there is no central planner, prices emerge from the imbalance (design §9).

use crate::resources::Good;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

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
}

impl Default for Market {
    fn default() -> Self {
        let prices = TRADED.iter().map(|&g| (g, base_price(g))).collect();
        Market { prices, supply: HashMap::new(), demand: HashMap::new() }
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
    /// Clear the day's flows once prices have been updated from them.
    pub fn clear_flows(&mut self) {
        self.supply.clear();
        self.demand.clear();
    }
}
