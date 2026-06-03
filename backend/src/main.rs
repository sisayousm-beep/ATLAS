//! Project Atlas — Phase 1 simulation entry point.
//!
//! Builds the world, wires the daily systems into an ordered schedule, then
//! runs in real time: one schedule pass per second = one in-game day.

// Many Nation/Pop fields and most `Good` variants are defined now but only
// exercised in later phases (production, finance, trade). Intentional scaffold.
#![allow(dead_code)]

mod components;
mod production;
mod resources;
mod systems;
mod world_gen;

use bevy_ecs::prelude::*;
use production::Market;
use resources::GameClock;
use std::{thread, time::Duration};

fn main() {
    let mut world = World::new();
    world.insert_resource(GameClock::default());
    world.insert_resource(Market::default());
    world_gen::spawn_world(&mut world);

    let mut schedule = systems::build_schedule();

    println!("Project Atlas — Phase 2 simulation (production · market · prices).");
    println!("1 real second = 1 game day. Monthly reports below. Ctrl+C to stop.\n");

    loop {
        schedule.run(&mut world);
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use crate::components::Pop;
    use crate::production::{base_price, Market};
    use crate::resources::{GameClock, Good, ResourceStock};
    use bevy_ecs::prelude::*;

    /// Stand up a fully-resourced world ready to tick.
    fn new_world() -> World {
        let mut world = World::new();
        world.insert_resource(GameClock::default());
        world.insert_resource(Market::default());
        crate::world_gen::spawn_world(&mut world);
        world
    }

    fn total_stock(world: &mut World, good: Good) -> f64 {
        let mut q = world.query::<&ResourceStock>();
        q.iter(world).map(|s| s.get(good)).sum()
    }

    /// Run a simulated quarter (90 days) headlessly and check the world stays
    /// coherent: clock advances, grain is produced, population survives.
    #[test]
    fn world_runs_a_quarter() {
        let mut world = new_world();
        let mut schedule = crate::systems::build_schedule();

        let pop_before: u64 = {
            let mut q = world.query::<&Pop>();
            q.iter(&world).map(|p| p.size as u64).sum()
        };

        for _ in 0..90 {
            schedule.run(&mut world);
        }

        assert_eq!(world.resource::<GameClock>().day, 90, "clock should advance 90 days");
        assert!(total_stock(&mut world, Good::Grain) > 0.0, "fertile regions should hold grain");

        let pop_after: u64 = {
            let mut q = world.query::<&Pop>();
            q.iter(&world).map(|p| p.size as u64).sum()
        };
        assert!(pop_after > 0, "population collapsed: {pop_before} -> {pop_after}");
    }

    /// Phase 2: the production chain actually refines raw goods up the tiers,
    /// and the market reprices cars away from their base under scarce supply.
    #[test]
    fn production_chain_and_prices_move() {
        let mut world = new_world();
        let mut schedule = crate::systems::build_schedule();

        for _ in 0..120 {
            schedule.run(&mut world);
        }

        // Steel accumulates only if the whole chain ran: ore mined → iron
        // smelted → steel refined. (Raw + iron stocks are transient — they are
        // consumed by the next tier within the same tick.)
        assert!(total_stock(&mut world, Good::Steel) > 0.0, "chain should refine ore to steel");

        // Prices move both ways: scarce cars climb above base, oversupplied
        // steel falls below it.
        let market = world.resource::<Market>();
        assert!(
            market.price(Good::Car) > base_price(Good::Car),
            "scarce cars should price above base {}, got {}",
            base_price(Good::Car),
            market.price(Good::Car)
        );
        assert!(
            market.price(Good::Steel) < base_price(Good::Steel),
            "oversupplied steel should price below base {}, got {}",
            base_price(Good::Steel),
            market.price(Good::Steel)
        );
    }
}
