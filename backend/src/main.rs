//! Project Atlas — Phase 1 simulation entry point.
//!
//! Builds the world, wires the daily systems into an ordered schedule, then
//! runs in real time: one schedule pass per second = one in-game day.

// Many Nation/Pop fields and most `Good` variants are defined now but only
// exercised in later phases (production, finance, trade). Intentional scaffold.
#![allow(dead_code)]

mod components;
mod corporation;
mod diplomacy;
mod finance;
mod politics;
mod production;
mod resources;
mod systems;
mod world_gen;

use bevy_ecs::prelude::*;
use corporation::TradeLedger;
use diplomacy::{Diplomacy, DiplomacyLedger};
use finance::FinanceLedger;
use politics::PoliticsLedger;
use production::Market;
use resources::GameClock;
use std::{thread, time::Duration};

fn main() {
    let mut world = World::new();
    world.insert_resource(GameClock::default());
    world.insert_resource(Market::default());
    world.insert_resource(TradeLedger::default());
    world.insert_resource(FinanceLedger::default());
    world.insert_resource(PoliticsLedger::default());
    world.insert_resource(Diplomacy::default());
    world.insert_resource(DiplomacyLedger::default());
    world_gen::spawn_world(&mut world);

    let mut schedule = systems::build_schedule();

    println!("Project Atlas — Phase 5 simulation (politics · diplomacy · economy).");
    println!("1 real second = 1 game day. Monthly reports below. Ctrl+C to stop.\n");

    loop {
        schedule.run(&mut world);
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use crate::components::{
        Climate, Government, Ideology, Nation, Pop, Profession, Region, Terrain,
    };
    use crate::corporation::{Corporation, TradeLedger};
    use crate::diplomacy::{
        government_affinity, relation_after, relation_status, Diplomacy, DiplomacyLedger, Relation,
    };
    use crate::finance::{policy_rate_after, CentralBank, FinanceLedger, RATE_CAP};
    use crate::politics::{
        government_aligns, government_for, stability_after, Politics, PoliticsLedger,
    };
    use crate::production::{base_price, Market};
    use crate::resources::{Deposits, GameClock, Good, ResourceStock};
    use bevy_ecs::prelude::*;

    /// Stand up a fully-resourced world ready to tick.
    fn new_world() -> World {
        let mut world = World::new();
        world.insert_resource(GameClock::default());
        world.insert_resource(Market::default());
        world.insert_resource(TradeLedger::default());
        world.insert_resource(FinanceLedger::default());
        world.insert_resource(PoliticsLedger::default());
        world.insert_resource(Diplomacy::default());
        world.insert_resource(DiplomacyLedger::default());
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

    /// Phase 5: the stability rule leans the right way. A content, well-represented
    /// nation firms up; a miserable, unrepresented one slides — and stability never
    /// escapes its [0, 1] band.
    #[test]
    fn stability_tracks_happiness_and_support() {
        let up = stability_after(0.5, 0.9, 0.9);
        assert!(up > 0.5, "content + represented should firm up, got {up}");
        let down = stability_after(0.5, 0.0, 0.0);
        assert!(down < 0.5, "miserable + unrepresented should slide, got {down}");
        assert!(stability_after(1.0, 1.0, 1.0) <= 1.0, "stability stays capped at 1");
        assert!(stability_after(0.0, 0.0, 0.0) >= 0.0, "stability never goes negative");
    }

    /// Phase 5: government legitimacy maps the way the design intends — a democracy
    /// represents liberals, not militarists — and a triumphant bloc installs the
    /// matching regime.
    #[test]
    fn legitimacy_and_succession_map_correctly() {
        assert!(government_aligns(Government::Democracy, Ideology::Liberal));
        assert!(!government_aligns(Government::Democracy, Ideology::Militarist));
        assert!(government_aligns(Government::Junta, Ideology::Militarist));
        assert_eq!(government_for(Ideology::Militarist), Government::Junta);
        assert_eq!(government_for(Ideology::Conservative), Government::Monarchy);
        assert_eq!(government_for(Ideology::Liberal), Government::Democracy);
    }

    /// Phase 5: relations reflect both ideology and commerce. Like governments
    /// start friendly and opposed ones cold; heavy trade thaws even a rivalry; and
    /// the status thresholds read off the score as designed.
    #[test]
    fn relations_reflect_ideology_and_trade() {
        assert!(
            government_affinity(Government::Democracy, Government::Democracy)
                > government_affinity(Government::Democracy, Government::Junta),
            "like systems should trust each other more than opposed ones"
        );
        // From cold, a heavy commercial tie pulls the relation up faster than no trade.
        let cold = government_affinity(Government::Democracy, Government::Junta);
        let with_trade = relation_after(0.0, cold, 100_000.0);
        let without = relation_after(0.0, cold, 0.0);
        assert!(with_trade > without, "commerce should warm relations: {with_trade} vs {without}");
        assert_eq!(relation_status(0.8), Relation::Ally);
        assert_eq!(relation_status(0.2), Relation::Neutral);
        assert_eq!(relation_status(-0.9), Relation::Hostile);
    }

    /// Phase 5: the live world stays politically coherent. After a year every
    /// nation's stability sits in [0, 1], the relation web is populated, and
    /// diplomacy has moved prestige off zero somewhere.
    #[test]
    fn world_stays_politically_coherent() {
        let mut world = new_world();
        let mut schedule = crate::systems::build_schedule();
        for _ in 0..360 {
            schedule.run(&mut world);
        }

        let mut q = world.query::<&Nation>();
        for n in q.iter(&world) {
            assert!(
                (0.0..=1.0).contains(&n.stability),
                "stability out of band: {}",
                n.stability
            );
        }

        let diplo = world.resource::<Diplomacy>();
        // Three nations → three pairs, all scored by the diplomacy system.
        assert_eq!(diplo.relations.len(), 3, "every nation pair should have a relation");

        let pol = world.resource::<PoliticsLedger>();
        assert!(
            pol.avg_stability > 0.0 && pol.avg_stability <= 1.0,
            "politics ledger should report a sane average stability: {}",
            pol.avg_stability
        );

        let moved_prestige = {
            let mut q = world.query::<&Nation>();
            q.iter(&world).any(|n| n.prestige != 0.0)
        };
        assert!(moved_prestige, "diplomacy should move prestige off zero");
    }

    /// Phase 5: sustained misery topples a government. A democracy whose people are
    /// starving (no food) and entirely unrepresented (all conservatives) loses
    /// stability month after month until the largest bloc seizes power — here the
    /// conservatives, who install a monarchy (design §13).
    #[test]
    fn sustained_misery_topples_the_government() {
        let mut world = World::new();
        world.insert_resource(GameClock::default());
        world.insert_resource(Market::default());
        world.insert_resource(TradeLedger::default());
        world.insert_resource(FinanceLedger::default());
        world.insert_resource(PoliticsLedger::default());
        world.insert_resource(Diplomacy::default());
        world.insert_resource(DiplomacyLedger::default());

        let nation = world
            .spawn((
                Nation {
                    name: "Faltering Republic".to_string(),
                    treasury: 1_000.0,
                    debt: 0.0,
                    inflation: 0.02,
                    stability: 0.7,
                    prestige: 0.0,
                    technology: 1.0,
                    government: Government::Democracy,
                    exports: 0.0,
                    imports: 0.0,
                },
                CentralBank::seed(1_000.0),
                Politics::seed(),
            ))
            .id();

        // A single region with no farmland and no deposits: nothing to eat, nothing
        // to mine. Pops are all conservatives, so a democracy never represents them.
        let region = world
            .spawn((
                Region {
                    name: "Hinterland".to_string(),
                    terrain: Terrain::Plains,
                    climate: Climate::Temperate,
                    infrastructure: 0.5,
                    owner: nation,
                },
                ResourceStock::default(),
                Deposits::default(),
            ))
            .id();
        for _ in 0..3 {
            world.spawn(Pop {
                size: 100_000,
                profession: Profession::Laborer,
                wealth: 1.0,
                literacy: 0.6,
                happiness: 0.6,
                ideology: Ideology::Conservative,
                region,
            });
        }

        let mut schedule = crate::systems::build_schedule();
        for _ in 0..180 {
            schedule.run(&mut world);
        }

        let gov = world.get::<Nation>(nation).unwrap().government;
        assert_eq!(
            gov,
            Government::Monarchy,
            "starving, unrepresented people should overthrow the republic for a conservative monarchy, got {gov:?}"
        );
    }
}
