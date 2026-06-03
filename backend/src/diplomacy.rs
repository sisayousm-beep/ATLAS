//! Phase 5: diplomacy. The web of relations between nations, their status, and
//! the prestige and stability those ties confer (design §14).
//!
//! Each ordered nation pair holds a relation score in `[-1, 1]`. Every month it
//! eases toward a target set by how compatible the two governments are (like
//! systems trust each other) plus a commercial-peace bonus from how much they
//! trade — heavy commerce can thaw even an ideological rivalry. The score maps to
//! a [`Relation`] status (ally → hostile), and the standing of its neighbours
//! feeds back into a nation's prestige and the stability of its regime
//! (design §14 → §13).
//!
//! As elsewhere the data and pure helpers live here; the monthly behaviour lives
//! in [`systems`](crate::systems).

use crate::components::Government;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Relation status between two nations (design §14), derived from the score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    Hostile,
    Rival,
    Neutral,
    Ally,
}

/// Read the relation status off a raw score.
pub fn relation_status(score: f64) -> Relation {
    match score {
        s if s >= 0.5 => Relation::Ally,
        s if s >= 0.0 => Relation::Neutral,
        s if s >= -0.5 => Relation::Rival,
        _ => Relation::Hostile,
    }
}

/// How fast relations ease toward their target each month.
pub const RELATION_EASE: f64 = 0.25;
/// Trade value that counts as full commercial interdependence (warms relations).
pub const TRADE_NORM: f64 = 40_000.0;
/// The most a commercial tie can lift a relation's target.
pub const COMMERCE_CAP: f64 = 0.6;
/// How strongly the average relation moves a nation's prestige each month.
pub const PRESTIGE_GAIN: f64 = 2.0;
/// How strongly a friendly/hostile neighbourhood nudges regime stability.
pub const STABILITY_DIPLO: f64 = 0.03;

/// The baseline relation two governments drift toward on ideology alone
/// (design §14): the same system trusts itself, the authoritarian family
/// (autocracy/monarchy/junta) hangs together, and across the democratic divide
/// it sours.
pub fn government_affinity(a: Government, b: Government) -> f64 {
    use Government::*;
    let authoritarian = |g| matches!(g, Autocracy | Monarchy | Junta);
    if a == b {
        0.5
    } else if authoritarian(a) == authoritarian(b) {
        0.2
    } else {
        -0.3
    }
}

/// Next relation score: ease toward a target of government affinity plus a
/// commercial-peace bonus scaled by trade, clamped to `[-1, 1]`. Pure, so the
/// rule can be unit-tested on its own.
pub fn relation_after(current: f64, affinity: f64, trade_value: f64) -> f64 {
    let commerce = (trade_value / TRADE_NORM).min(COMMERCE_CAP);
    let target = (affinity + commerce).clamp(-1.0, 1.0);
    (current + RELATION_EASE * (target - current)).clamp(-1.0, 1.0)
}

/// The world's relation web (design §14): a score per unordered nation pair, plus
/// the cross-border trade flowing between them this month. The logistics layer
/// records the trade as it ships goods across borders; the diplomacy system
/// drains it monthly so only recent commerce counts toward the relation.
#[derive(Resource, Debug, Default)]
pub struct Diplomacy {
    /// Relation score in `[-1, 1]`, keyed by the normalised nation pair.
    pub relations: HashMap<(Entity, Entity), f64>,
    /// Cross-border trade value accumulated this month, per nation pair.
    pub trade_flow: HashMap<(Entity, Entity), f64>,
}

impl Diplomacy {
    /// Normalise a pair so `(a, b)` and `(b, a)` map to one entry.
    pub fn pair(a: Entity, b: Entity) -> (Entity, Entity) {
        if a.to_bits() <= b.to_bits() {
            (a, b)
        } else {
            (b, a)
        }
    }

    pub fn relation(&self, a: Entity, b: Entity) -> f64 {
        *self.relations.get(&Self::pair(a, b)).unwrap_or(&0.0)
    }

    /// Tally cross-border trade between two nations toward this month's commerce.
    pub fn record_trade(&mut self, a: Entity, b: Entity, value: f64) {
        if a != b {
            *self.trade_flow.entry(Self::pair(a, b)).or_insert(0.0) += value;
        }
    }
}

/// World diplomacy dashboard, refreshed monthly for the report.
#[derive(Resource, Debug, Default)]
pub struct DiplomacyLedger {
    /// Pairs of nations that are allies.
    pub allies: u32,
    /// Pairs that are rivals or openly hostile.
    pub rivalries: u32,
}
