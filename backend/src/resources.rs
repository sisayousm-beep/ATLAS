//! Tradeable goods, per-region stock, and the global game clock.

use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Every good in the economy across the three production tiers (design §6).
/// Phase 1 only exercises `Grain`; the rest are defined for later phases.
/// (Named `Good`, not `Resource`, to avoid clashing with the `bevy_ecs`
/// `Resource` trait used for ECS singletons like [`GameClock`].)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Good {
    // --- Raw materials ---
    Grain,
    Wood,
    IronOre,
    Coal,
    Oil,
    Uranium,
    RareEarth,
    // --- Intermediate goods ---
    Iron,
    Steel,
    Plastic,
    Semiconductor,
    Battery,
    // --- Finished goods ---
    Car,
    Electronics,
    Weapon,
    Computer,
    Robot,
}

/// Natural endowment of a region: how rich it is in each raw good, as a
/// multiplier on extraction (design §6). Empty = nothing to mine here.
#[derive(Component, Debug, Default)]
pub struct Deposits(pub HashMap<Good, f64>);

/// Per-region warehouse. Maps a good to the amount held.
#[derive(Component, Debug, Default)]
pub struct ResourceStock(pub HashMap<Good, f64>);

impl ResourceStock {
    pub fn get(&self, r: Good) -> f64 {
        *self.0.get(&r).unwrap_or(&0.0)
    }

    pub fn add(&mut self, r: Good, amount: f64) {
        *self.0.entry(r).or_insert(0.0) += amount;
    }

    /// Remove up to `amount`; returns how much was actually taken.
    pub fn take(&mut self, r: Good, amount: f64) -> f64 {
        let have = self.get(r);
        let taken = have.min(amount);
        self.0.insert(r, have - taken);
        taken
    }
}

/// Global clock. One tick = one in-game day; one real second = one day.
/// Uses a simplified 360-day year of twelve 30-day months.
#[derive(Resource, Debug, Default)]
pub struct GameClock {
    /// Days since epoch. Day 0 = Year 1, Month 1, Day 1.
    pub day: u64,
}

impl GameClock {
    pub fn year(&self) -> u64 {
        1 + self.day / 360
    }
    pub fn month(&self) -> u64 {
        1 + (self.day % 360) / 30
    }
    pub fn day_of_month(&self) -> u64 {
        1 + self.day % 30
    }
    /// True on the first day of each month (but not on day 0).
    pub fn is_month_start(&self) -> bool {
        self.day > 0 && self.day % 30 == 0
    }
}
