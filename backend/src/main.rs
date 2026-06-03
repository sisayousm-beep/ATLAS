//! Project Atlas — Phase 1 simulation entry point.
//!
//! Builds the world, wires the daily systems into an ordered schedule, then
//! runs in real time: one schedule pass per second = one in-game day.

// Many Nation/Pop fields and most `Good` variants are defined now but only
// exercised in later phases (production, finance, trade). Intentional scaffold.
#![allow(dead_code)]

mod components;
mod resources;
mod systems;
mod world_gen;

use bevy_ecs::prelude::*;
use resources::GameClock;
use std::{thread, time::Duration};

fn main() {
    let mut world = World::new();
    world.insert_resource(GameClock::default());
    world_gen::spawn_world(&mut world);

    let mut schedule = systems::build_schedule();

    println!("Project Atlas — Phase 1 simulation.");
    println!("1 real second = 1 game day. Monthly reports below. Ctrl+C to stop.\n");

    loop {
        schedule.run(&mut world);
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use crate::components::Pop;
    use crate::resources::{GameClock, Good, ResourceStock};
    use bevy_ecs::prelude::*;

    /// Run a simulated quarter (90 days) headlessly and check the world stays
    /// coherent: clock advances, grain is produced, population survives.
    #[test]
    fn world_runs_a_quarter() {
        let mut world = World::new();
        world.insert_resource(GameClock::default());
        crate::world_gen::spawn_world(&mut world);
        let mut schedule = crate::systems::build_schedule();

        let pop_before: u64 = {
            let mut q = world.query::<&Pop>();
            q.iter(&world).map(|p| p.size as u64).sum()
        };

        for _ in 0..90 {
            schedule.run(&mut world);
        }

        assert_eq!(world.resource::<GameClock>().day, 90, "clock should advance 90 days");

        let total_grain: f64 = {
            let mut q = world.query::<&ResourceStock>();
            q.iter(&world).map(|s| s.get(Good::Grain)).sum()
        };
        assert!(total_grain > 0.0, "fertile regions should hold grain, got {total_grain}");

        let pop_after: u64 = {
            let mut q = world.query::<&Pop>();
            q.iter(&world).map(|p| p.size as u64).sum()
        };
        assert!(pop_after > 0, "population collapsed: {pop_before} -> {pop_after}");
    }
}
