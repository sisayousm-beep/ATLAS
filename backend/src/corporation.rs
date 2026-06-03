//! Phase 3 actors: profit-seeking corporations and the trade ledger.
//!
//! Corporations are the economy's independent agents (design §8). They run the
//! production recipes, buy their inputs and sell their output on the [`Market`],
//! and keep whatever margin is left as capital. They do not obey the state —
//! they chase profit, hiring when it flows and firing when it dries up.
//!
//! Goods rarely sit where the factory that needs them is, so a logistics layer
//! (design §10) ships them between regions; the [`TradeLedger`] tallies what
//! moves, and cross-border shipments land on each nation's trade balance.
//!
//! As with the rest of the codebase the data lives here; the behaviour lives in
//! [`systems`](crate::systems).

use crate::production::{Recipe, RECIPES};
use crate::resources::Good;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// An independent firm operating one factory in its home region (design §8).
#[derive(Component, Debug)]
pub struct Corporation {
    pub name: String,
    /// The `Nation` entity the firm is domiciled in (it pays tax there).
    pub owner: Entity,
    /// The `Region` entity where its factory and workforce sit.
    pub region: Entity,
    /// Cash on hand. Grows with profit, spent hiring and on inputs.
    pub capital: f64,
    /// Workforce the firm claims, capped by the engineers actually in `region`.
    pub employees: f64,
    /// Margin accumulated this month; read and reset by the monthly AI.
    pub profit: f64,
    /// Goods the firm produces, listed low→high tier so a single region pass can
    /// chain them (e.g. `[Iron, Steel, Car]`).
    pub industries: Vec<Good>,
}

/// The recipe that yields `output`, if one exists.
pub fn recipe_for(output: Good) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.output == output)
}

/// Running tally of goods moved by the logistics layer (design §10): how much of
/// each good shipped between regions, and the total value of all trade.
#[derive(Resource, Debug, Default)]
pub struct TradeLedger {
    pub volume: HashMap<Good, f64>,
    pub value: f64,
}

impl TradeLedger {
    pub fn record(&mut self, g: Good, qty: f64, value: f64) {
        *self.volume.entry(g).or_insert(0.0) += qty;
        self.value += value;
    }
    pub fn volume(&self, g: Good) -> f64 {
        *self.volume.get(&g).unwrap_or(&0.0)
    }
}
