//! Builds the initial Phase 1 world: a handful of nations, each with a few
//! regions populated by pops. Kept small and hand-authored for the MVP;
//! procedural generation of 200–500 regions comes later.

use crate::components::*;
use crate::corporation::Corporation;
use crate::finance::CentralBank;
use crate::nation_ai::{AiPersonality, NationAi};
use crate::politics::Politics;
use crate::resources::{Deposits, Good, ResourceStock};
use crate::war::Military;
use bevy_ecs::prelude::*;

struct PopSeed {
    size: u32,
    profession: Profession,
    wealth: f64,
    literacy: f64,
    ideology: Ideology,
}

/// Spawn a region owned by `nation`, plus its starting pops. Returns the region
/// entity so corporations can be sited in it.
fn spawn_region(
    world: &mut World,
    nation: Entity,
    name: &str,
    terrain: Terrain,
    climate: Climate,
    infrastructure: f64,
    deposits: &[(Good, f64)],
    pops: &[PopSeed],
) -> Entity {
    let region = world
        .spawn((
            Region {
                name: name.to_string(),
                terrain,
                climate,
                infrastructure,
                owner: nation,
            },
            ResourceStock::default(),
            Deposits(deposits.iter().copied().collect()),
        ))
        .id();

    for seed in pops {
        world.spawn(Pop {
            size: seed.size,
            profession: seed.profession,
            wealth: seed.wealth,
            literacy: seed.literacy,
            happiness: 0.6,
            ideology: seed.ideology,
            region,
        });
    }

    region
}

fn nation(
    world: &mut World,
    name: &str,
    treasury: f64,
    gov: Government,
    tech: f64,
    personality: AiPersonality,
) -> Entity {
    world
        .spawn((
            Nation {
                name: name.to_string(),
                treasury,
                debt: 0.0,
                inflation: 0.02,
                stability: 0.7,
                prestige: 0.0,
                technology: tech,
                government: gov,
                exports: 0.0,
                imports: 0.0,
            },
            // Phase 4: each nation runs its own central bank (design §11).
            CentralBank::seed(treasury),
            // Phase 5: each nation carries its political state (design §13).
            Politics::seed(),
            // Phase 6: each nation fields a military, armed from the economy (§15).
            Military::seed(),
            // Phase 7: each nation is run by a strategic AI brain (design §16).
            NationAi::new(personality),
        ))
        .id()
}

/// Found a corporation: a firm in `region`, domiciled in `owner`, that produces
/// the listed `industries` (design §8).
fn corp(
    world: &mut World,
    name: &str,
    owner: Entity,
    region: Entity,
    capital: f64,
    employees: f64,
    industries: &[Good],
) {
    world.spawn(Corporation {
        name: name.to_string(),
        owner,
        region,
        capital,
        employees,
        profit: 0.0,
        industries: industries.to_vec(),
    });
}

pub fn spawn_world(world: &mut World) {
    use Ideology::*;
    use Profession::*;

    // --- Aurelia: temperate breadbasket democracy, mines iron + coal ---
    // A diplomatic power: it courts its neighbours into alliances over arms.
    let aurelia = nation(world, "Aurelia", 10_000.0, Government::Democracy, 1.0, AiPersonality::Diplomatic);
    let goldfields = spawn_region(
        world,
        aurelia,
        "Goldfields",
        Terrain::Plains,
        Climate::Temperate,
        0.7,
        &[(Good::IronOre, 0.8), (Good::Coal, 0.7)],
        &[
            PopSeed { size: 120_000, profession: Farmer, wealth: 1.5, literacy: 0.8, ideology: Liberal },
            PopSeed { size: 60_000, profession: Laborer, wealth: 1.2, literacy: 0.7, ideology: Progressive },
            PopSeed { size: 25_000, profession: Engineer, wealth: 3.5, literacy: 0.9, ideology: Liberal },
            PopSeed { size: 15_000, profession: Merchant, wealth: 3.0, literacy: 0.9, ideology: Conservative },
            PopSeed { size: 12_000, profession: Soldier, wealth: 1.8, literacy: 0.8, ideology: Militarist },
        ],
    );
    let port_vesper = spawn_region(
        world,
        aurelia,
        "Port Vesper",
        Terrain::Coast,
        Climate::Temperate,
        0.85,
        &[(Good::Oil, 0.9)],
        &[
            PopSeed { size: 40_000, profession: Farmer, wealth: 1.3, literacy: 0.85, ideology: Liberal },
            PopSeed { size: 90_000, profession: Laborer, wealth: 1.4, literacy: 0.8, ideology: Progressive },
            PopSeed { size: 20_000, profession: Engineer, wealth: 4.0, literacy: 0.95, ideology: Liberal },
        ],
    );

    // --- Khoresan: arid autocracy, oil-rich but thin industry ---
    // An aggressive autocracy — but a poor one, so its hunger for war keeps
    // running into a treasury that can't fund it (and unrest tips it into survival).
    let khoresan = nation(world, "Khoresan", 6_000.0, Government::Autocracy, 0.8, AiPersonality::Aggressive);
    let sandreach = spawn_region(
        world,
        khoresan,
        "Sandreach",
        Terrain::Desert,
        Climate::Arid,
        0.4,
        &[(Good::Oil, 1.2)],
        &[
            PopSeed { size: 70_000, profession: Farmer, wealth: 0.8, literacy: 0.4, ideology: Conservative },
            PopSeed { size: 50_000, profession: Laborer, wealth: 0.9, literacy: 0.5, ideology: Conservative },
            PopSeed { size: 8_000, profession: Engineer, wealth: 1.8, literacy: 0.55, ideology: Conservative },
            PopSeed { size: 10_000, profession: Soldier, wealth: 1.5, literacy: 0.6, ideology: Militarist },
        ],
    );
    let oasis_hold = spawn_region(
        world,
        khoresan,
        "Oasis Hold",
        Terrain::Hills,
        Climate::Arid,
        0.5,
        &[(Good::IronOre, 0.7), (Good::Coal, 0.5)],
        &[
            PopSeed { size: 55_000, profession: Farmer, wealth: 1.0, literacy: 0.5, ideology: Conservative },
            PopSeed { size: 20_000, profession: Laborer, wealth: 0.9, literacy: 0.5, ideology: Conservative },
            PopSeed { size: 6_000, profession: Engineer, wealth: 2.0, literacy: 0.6, ideology: Liberal },
            PopSeed { size: 8_000, profession: Merchant, wealth: 2.5, literacy: 0.7, ideology: Liberal },
        ],
    );

    // --- Nordheim: cold continental monarchy, the industrial powerhouse ---
    // A scientific power: it has the researchers, and pours treasury into climbing
    // the technology ladder (design §12, §17).
    let nordheim = nation(world, "Nordheim", 8_000.0, Government::Monarchy, 1.1, AiPersonality::Scientific);
    let frostmark = spawn_region(
        world,
        nordheim,
        "Frostmark",
        Terrain::Plains,
        Climate::Continental,
        0.75,
        &[(Good::IronOre, 0.9), (Good::Coal, 0.8), (Good::Oil, 0.4)],
        &[
            PopSeed { size: 65_000, profession: Farmer, wealth: 1.4, literacy: 0.85, ideology: Conservative },
            PopSeed { size: 85_000, profession: Laborer, wealth: 1.6, literacy: 0.82, ideology: Socialist },
            PopSeed { size: 30_000, profession: Engineer, wealth: 4.2, literacy: 0.95, ideology: Progressive },
            PopSeed { size: 25_000, profession: Researcher, wealth: 3.5, literacy: 0.97, ideology: Progressive },
            PopSeed { size: 15_000, profession: Soldier, wealth: 2.0, literacy: 0.85, ideology: Militarist },
        ],
    );

    // --- Corporations (design §8) ---
    // Firms specialise in one good so their whole labour budget goes to it. The
    // chain spans regions — foundries smelt iron, steelworks need that iron, car
    // plants need steel and plastic — so the goods only meet because trade hauls
    // them between regions (Phase 3's logistics at work).
    use Good::*;
    corp(world, "Goldfield Foundry", aurelia, goldfields, 4_000.0, 9_000.0, &[Iron]);
    corp(world, "Aurelia Steel", aurelia, goldfields, 5_000.0, 9_000.0, &[Steel]);
    corp(world, "Vesper Plastics", aurelia, port_vesper, 4_000.0, 8_000.0, &[Plastic]);
    corp(world, "Aurelia Motors", aurelia, port_vesper, 6_000.0, 9_000.0, &[Car]);
    corp(world, "Sandreach Petrochem", khoresan, sandreach, 3_000.0, 6_000.0, &[Plastic]);
    corp(world, "Oasis Forge", khoresan, oasis_hold, 3_000.0, 5_000.0, &[Iron]);
    corp(world, "Nordheim Foundry", nordheim, frostmark, 5_000.0, 10_000.0, &[Iron]);
    corp(world, "Nordheim Steel", nordheim, frostmark, 6_000.0, 10_000.0, &[Steel]);
    corp(world, "Nordheim Motors", nordheim, frostmark, 7_000.0, 9_000.0, &[Car]);
}
