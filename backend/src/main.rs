//! Project Atlas — Phase 1 simulation entry point.
//!
//! Builds the world, wires the daily systems into an ordered schedule, then
//! runs in real time: one schedule pass per second = one in-game day.

// Many Nation/Pop fields and most `Good` variants are defined now but only
// exercised in later phases (production, finance, trade). Intentional scaffold.
#![allow(dead_code)]

mod components;
mod corporation;
mod finance;
mod production;
mod resources;
mod systems;
mod world_gen;

use bevy_ecs::prelude::*;
use corporation::TradeLedger;
use finance::FinanceLedger;
use production::Market;
use resources::GameClock;
use std::{thread, time::Duration};

fn main() {
    let mut world = World::new();
    world.insert_resource(GameClock::default());
    world.insert_resource(Market::default());
    world.insert_resource(TradeLedger::default());
    world.insert_resource(FinanceLedger::default());
    world_gen::spawn_world(&mut world);

    let mut schedule = systems::build_schedule();

    println!("Project Atlas — Phase 4 simulation (finance · central bank · economy).");
    println!("1 real second = 1 game day. Monthly reports below. Ctrl+C to stop.\n");

    loop {
        schedule.run(&mut world);
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use crate::components::{Nation, Pop};
    use crate::corporation::{Corporation, TradeLedger};
    use crate::finance::{policy_rate_after, CentralBank, FinanceLedger, RATE_CAP};
    use crate::production::{base_price, Market};
    use crate::resources::{GameClock, Good, ResourceStock};
    use bevy_ecs::prelude::*;

    /// Stand up a fully-resourced world ready to tick.
    fn new_world() -> World {
        let mut world = World::new();
        world.insert_resource(GameClock::default());
        world.insert_resource(Market::default());
        world.insert_resource(TradeLedger::default());
        world.insert_resource(FinanceLedger::default());
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

    /// Phase 3: corporations run the chain as profit-seeking firms, and the
    /// logistics layer moves goods between regions. After a quarter at least one
    /// firm has built capital, and inter-region trade has actually happened.
    #[test]
    fn corporations_trade_and_profit() {
        let mut world = new_world();
        let mut schedule = crate::systems::build_schedule();

        let start_capital: f64 = {
            let mut q = world.query::<&Corporation>();
            q.iter(&world).map(|c| c.capital).sum()
        };

        for _ in 0..120 {
            schedule.run(&mut world);
        }

        // Some firm is making money: total corporate capital has grown.
        let end_capital: f64 = {
            let mut q = world.query::<&Corporation>();
            q.iter(&world).map(|c| c.capital).sum()
        };
        assert!(
            end_capital > start_capital,
            "corporations should accumulate capital: {start_capital} -> {end_capital}"
        );

        // The logistics network moved goods between regions.
        assert!(
            world.resource::<TradeLedger>().value > 0.0,
            "trade should move goods between regions"
        );
    }

    /// Phase 4: the monetary policy rule leans the right way. Above-target
    /// inflation lifts the rate, below-target cuts it, and the rate stays inside
    /// its band in both directions.
    #[test]
    fn central_bank_leans_against_inflation() {
        let up = policy_rate_after(0.03, 0.10, 0.02);
        assert!(up > 0.03, "high inflation should raise the rate, got {up}");
        let down = policy_rate_after(0.03, 0.00, 0.02);
        assert!(down < 0.03, "low inflation should cut the rate, got {down}");
        assert!(policy_rate_after(0.24, 0.50, 0.02) <= RATE_CAP, "rate stays capped");
        assert!(policy_rate_after(0.0, -1.0, 0.02) >= 0.0, "rate never goes negative");
    }

    /// Phase 4: with the government spending on its people every month, at least
    /// one nation is pushed into deficit and borrows to cover it — sovereign debt
    /// and the money supply both grow, and the ledger tracks them.
    #[test]
    fn governments_run_deficits_and_borrow() {
        let mut world = new_world();
        let mut schedule = crate::systems::build_schedule();

        for _ in 0..360 {
            schedule.run(&mut world);
        }

        let total_debt: f64 = {
            let mut q = world.query::<&Nation>();
            q.iter(&world).map(|n| n.debt).sum()
        };
        assert!(total_debt > 0.0, "deficit spending should accumulate sovereign debt");

        let ledger = world.resource::<FinanceLedger>();
        assert!(
            ledger.money_supply > 0.0 && ledger.gov_debt > 0.0,
            "finance ledger should track money and debt"
        );
    }

    /// Phase 4: the monetary transmission bites. Run two identical worlds — in
    /// one the central banks are pinned at the rate cap — and tight money should
    /// leave fewer engineers employed by year's end (dear credit cools hiring).
    #[test]
    fn tight_money_cools_corporate_hiring() {
        fn employment_after(pin_high_rate: bool) -> f64 {
            let mut world = new_world();
            if pin_high_rate {
                let mut q = world.query::<&mut CentralBank>();
                for mut bank in q.iter_mut(&mut world) {
                    bank.policy_rate = RATE_CAP;
                    // Negative target keeps monetary_policy from ever cutting it.
                    bank.target_inflation = -10.0;
                }
            }
            let mut schedule = crate::systems::build_schedule();
            for _ in 0..180 {
                schedule.run(&mut world);
            }
            let mut q = world.query::<&Corporation>();
            q.iter(&world).map(|c| c.employees).sum()
        }

        let cheap = employment_after(false);
        let dear = employment_after(true);
        assert!(
            dear < cheap,
            "tight money should cool hiring: dear {dear} vs cheap {cheap}"
        );
    }
}
