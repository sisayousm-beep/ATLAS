//! ECS components for the Phase 1 world: Nations, Regions and Pops.
//!
//! These mirror the entities described in the design doc (§4, §3, §5).
//! Pure data — all behaviour lives in `systems`.

use bevy_ecs::prelude::*;

/// A sovereign nation. Owns regions, holds the treasury, sets policy.
#[derive(Component, Debug)]
pub struct Nation {
    pub name: String,
    pub treasury: f64,
    pub debt: f64,
    pub inflation: f64,
    /// Domestic stability, 0.0 (collapse) .. 1.0 (rock solid).
    pub stability: f64,
    pub prestige: f64,
    /// Aggregate technology level index.
    pub technology: f64,
    pub government: Government,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Government {
    Democracy,
    Autocracy,
    Monarchy,
    Junta,
}

/// A geographic region. Belongs to one nation; hosts pops and a resource stock.
#[derive(Component, Debug)]
pub struct Region {
    pub name: String,
    pub terrain: Terrain,
    pub climate: Climate,
    /// Infrastructure quality, 0.0 .. 1.0. Multiplies production & logistics.
    pub infrastructure: f64,
    /// The `Nation` entity that owns this region.
    pub owner: Entity,
}

#[derive(Debug, Clone, Copy)]
pub enum Terrain {
    Plains,
    Hills,
    Mountains,
    Coast,
    Desert,
}

#[derive(Debug, Clone, Copy)]
pub enum Climate {
    Temperate,
    Tropical,
    Arid,
    Continental,
    Polar,
}

/// A population group ("POP") — the core unit of the simulation (design §5).
/// Pops consume, produce, and (in later phases) vote, protest and migrate.
#[derive(Component, Debug)]
pub struct Pop {
    pub size: u32,
    pub profession: Profession,
    pub wealth: f64,
    /// Literacy, 0.0 .. 1.0.
    pub literacy: f64,
    /// Happiness, 0.0 .. 1.0. Driven by food, wealth and stability.
    pub happiness: f64,
    pub ideology: Ideology,
    /// The `Region` entity this pop lives in.
    pub region: Entity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profession {
    Farmer,
    Laborer,
    Merchant,
    Engineer,
    Researcher,
    Soldier,
    Bureaucrat,
    Unemployed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ideology {
    Conservative,
    Progressive,
    Liberal,
    Socialist,
    Militarist,
}
