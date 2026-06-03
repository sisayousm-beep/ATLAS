//! Phase 5: politics. Popular ideology, government legitimacy, stability and
//! unrest (design §13).
//!
//! Pops already carry an [`Ideology`](crate::components::Ideology). A government
//! represents some of those ideologies and not others; the share it represents
//! is its *support*. Stability eases each month toward how content the people are
//! (happiness) and how well they are represented (support). When it collapses and
//! stays collapsed, the largest bloc seizes power — a revolution or coup that
//! installs its own government (design §13 events).
//!
//! Politics feeds the economy back: unrest erodes tax compliance, so an unstable
//! state collects less, borrows more, and (via Phase 4) inflates — which depresses
//! happiness and feeds the unrest. As elsewhere the data and pure helpers live
//! here; the monthly behaviour lives in [`systems`](crate::systems).

use crate::components::{Government, Ideology};
use bevy_ecs::prelude::*;

/// Per-nation political state (design §13). Sits on the `Nation` entity beside
/// its [`CentralBank`](crate::finance::CentralBank).
#[derive(Component, Debug)]
pub struct Politics {
    /// Unrest, 0.0 (calm) .. 1.0 (boiling) — the mirror of `Nation::stability`,
    /// kept here so the report and the tax hook can read it directly.
    pub unrest: f64,
    /// Consecutive months spent in open unrest. Pressure builds here; once it has
    /// run deep for long enough it tips into revolution.
    pub turmoil_months: u32,
}

impl Politics {
    pub fn seed() -> Self {
        Politics { unrest: 0.0, turmoil_months: 0 }
    }
}

/// How fast stability eases toward its target each month.
pub const STABILITY_EASE: f64 = 0.3;
/// Below this stability a nation counts as in open unrest (strikes, riots).
pub const UNREST_THRESHOLD: f64 = 0.4;
/// Below this, and sustained, the regime is at risk of being overthrown.
pub const REVOLT_THRESHOLD: f64 = 0.2;
/// Months of deep unrest before it tips into revolution/coup.
pub const REVOLT_MONTHS: u32 = 3;
/// Stability the new regime starts on — a honeymoon that buys it time.
pub const REVOLT_RESET: f64 = 0.5;
/// Prestige a nation loses in the upheaval of a regime change.
pub const REVOLT_PRESTIGE: f64 = 5.0;

/// Whether `gov`'s ruling bloc politically represents pops of `ideology`
/// (design §13). A represented pop is a supporter; everyone else is latent
/// opposition that erodes stability.
pub fn government_aligns(gov: Government, ideology: Ideology) -> bool {
    use Government::*;
    use Ideology::*;
    match gov {
        Democracy => matches!(ideology, Liberal | Progressive),
        Autocracy => matches!(ideology, Conservative | Militarist),
        Monarchy => matches!(ideology, Conservative | Liberal),
        Junta => matches!(ideology, Militarist | Conservative),
    }
}

/// The government a triumphant `ideology` installs after a revolution (design §13):
/// liberals/progressives open up, conservatives crown a monarch, socialists build
/// a one-party autocracy, militarists seize power as a junta.
pub fn government_for(ideology: Ideology) -> Government {
    use Government::*;
    use Ideology::*;
    match ideology {
        Liberal | Progressive => Democracy,
        Conservative => Monarchy,
        Socialist => Autocracy,
        Militarist => Junta,
    }
}

/// Next stability: ease toward a target set half by how content the people are
/// (happiness) and half by how much of them the government represents (support),
/// clamped to `[0, 1]`. Pure, so the rule can be unit-tested on its own.
pub fn stability_after(current: f64, avg_happiness: f64, support: f64) -> f64 {
    let target = 0.5 * avg_happiness + 0.5 * support;
    (current + STABILITY_EASE * (target - current)).clamp(0.0, 1.0)
}

/// Share of due tax a state actually collects given its stability: a calm nation
/// collects in full, an unstable one loses up to half to evasion and disorder.
pub fn tax_compliance(stability: f64) -> f64 {
    (0.5 + 0.5 * stability).clamp(0.0, 1.0)
}

/// World political dashboard, refreshed monthly for the report — mirrors the
/// finance and trade ledgers.
#[derive(Resource, Debug, Default)]
pub struct PoliticsLedger {
    /// Mean stability across nations.
    pub avg_stability: f64,
    /// The least stable nation's stability.
    pub min_stability: f64,
    /// Nations currently in open unrest.
    pub unrest_count: u32,
    /// Revolutions/coups that fired this month (design §13 events).
    pub revolutions: u32,
}
