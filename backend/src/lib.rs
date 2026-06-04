//! Project Atlas simulation as a library.
//!
//! The same engine `main.rs` runs as a CLI is exposed here as a small [`Simulation`]
//! façade so a host (the Tauri desktop shell) can drive it tick-by-tick and read a
//! serialisable [`WorldSnapshot`](snapshot::WorldSnapshot) of its state. The module
//! tree mirrors the binary's; behaviour and data are unchanged.

// As in the binary, much of the world model is scaffold exercised only in later
// phases or only by the CLI report — silence the unused warnings for the library.
#![allow(dead_code)]

pub mod components;
pub mod corporation;
pub mod diplomacy;
pub mod finance;
pub mod nation_ai;
pub mod politics;
pub mod production;
pub mod resources;
pub mod snapshot;
pub mod systems;
pub mod war;
pub mod world_gen;

use bevy_ecs::prelude::*;
use corporation::TradeLedger;
use diplomacy::{Diplomacy, DiplomacyLedger};
use finance::FinanceLedger;
use nation_ai::AiLedger;
use politics::PoliticsLedger;
use production::Market;
use resources::GameClock;
use war::{Warfront, WarLedger};

pub use snapshot::WorldSnapshot;

/// A running Project Atlas world plus its daily schedule. Each [`step`](Self::step)
/// advances the simulation by one in-game day, exactly as one pass of the CLI loop.
pub struct Simulation {
    world: World,
    schedule: Schedule,
}

impl Simulation {
    /// Build a fresh world — same resources, world generation and schedule the
    /// binary sets up — ready to be stepped.
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(GameClock::default());
        world.insert_resource(Market::default());
        world.insert_resource(TradeLedger::default());
        world.insert_resource(FinanceLedger::default());
        world.insert_resource(PoliticsLedger::default());
        world.insert_resource(Diplomacy::default());
        world.insert_resource(DiplomacyLedger::default());
        world.insert_resource(Warfront::default());
        world.insert_resource(WarLedger::default());
        world.insert_resource(AiLedger::default());
        world_gen::spawn_world(&mut world);

        let schedule = systems::build_schedule();
        Simulation { world, schedule }
    }

    /// Advance the world by one in-game day.
    pub fn step(&mut self) {
        self.schedule.run(&mut self.world);
    }

    /// A serialisable view of the world for the client.
    pub fn snapshot(&mut self) -> WorldSnapshot {
        snapshot::world_snapshot(&mut self.world)
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}
