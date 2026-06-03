//! Phase 6: war. The military each nation fields, and the wars they fight
//! (design §15).
//!
//! War is the *extension* of the economy, not its purpose (design §2): a nation's
//! strength is bought with treasury (산업력·경제력) and manned by soldier pops
//! (병력), then multiplied by technology (기술력) and degraded by exhaustion
//! (보급·사기). Fighting grinds that strength down, burns money the state must
//! often borrow, and wears out the home front — so a war a nation cannot afford
//! ends at the negotiating table. Only the militant reach for the sword, and only
//! when they hold a decisive edge; commerce-bound powers stay at peace.
//!
//! War feeds back into every earlier phase: it embargoes cross-border trade
//! (Phase 3), borrows through the central bank (Phase 4), and its exhaustion drags
//! on regime stability (Phase 5). As elsewhere the data and pure helpers live
//! here; the monthly behaviour lives in [`systems`](crate::systems).

use crate::components::Government;
use bevy_ecs::prelude::*;

/// A nation's armed forces (design §15). Sits on the `Nation` entity beside its
/// central bank and political state.
#[derive(Component, Debug)]
pub struct Military {
    /// Standing strength — bought with military spending (산업력→무기), worn down
    /// by wartime attrition and peacetime upkeep.
    pub strength: f64,
    /// War exhaustion, 0.0 (fresh) .. 1.0 (spent). Climbs while fighting, eases in
    /// peace; high exhaustion drags stability and forces a nation to sue for peace.
    pub exhaustion: f64,
}

impl Military {
    pub fn seed() -> Self {
        Military { strength: 0.0, exhaustion: 0.0 }
    }
}

/// Standing strength bought per unit of treasury spent.
pub const STRENGTH_PER_SPEND: f64 = 0.02;
/// Monthly peacetime upkeep: the fraction of its strength a standing army sheds
/// when it isn't being topped up.
pub const STRENGTH_DECAY: f64 = 0.05;
/// Combat power a single soldier pop head contributes (병력), before tech scaling.
pub const SOLDIER_POWER: f64 = 0.01;
/// Fraction of its own strength a side can grind away in a month, scaled by the
/// enemy's share of the battlefield (attrition).
pub const WAR_INTENSITY: f64 = 0.3;
/// Treasury a belligerent burns each month per unit of its own combat power.
pub const WAR_COST_PER_POWER: f64 = 4.0;
/// War exhaustion gained per month at war (보급 고갈·전쟁 피로).
pub const EXHAUSTION_GAIN: f64 = 0.12;
/// War exhaustion shed per month at peace.
pub const EXHAUSTION_EASE: f64 = 0.1;
/// Exhaustion at or above which a nation sues for peace.
pub const PEACE_EXHAUSTION: f64 = 0.6;
/// How hard war exhaustion drags on regime stability (war-weariness → unrest).
pub const WAR_STABILITY_DRAG: f64 = 0.2;
/// Relation at or below which two rivals can come to blows.
pub const WAR_RELATION: f64 = -0.2;
/// The relation a declaration of war forces the pair down to.
pub const WAR_DECLARE_RELATION: f64 = -0.9;
/// Power edge an aggressor wants over its target before it strikes.
pub const WAR_POWER_EDGE: f64 = 1.3;
/// War appetite an aggressor needs to initiate — only the militant clear this.
pub const WAR_APPETITE_MIN: f64 = 0.5;
/// Prestige the victor gains and the vanquished loses when a war is settled.
pub const WAR_PRESTIGE: f64 = 8.0;
/// Share of the loser's treasury paid to the victor as reparations.
pub const REPARATION_SHARE: f64 = 0.3;
/// Stability the loser's regime is shaken by on defeat.
pub const DEFEAT_STABILITY: f64 = 0.15;

/// Effective combat power (design §15 cost factors): standing strength plus the
/// manpower of soldier pops (병력), all scaled by technology (기술력) and degraded
/// by exhaustion (보급·사기). Pure, so it can be unit-tested on its own.
pub fn military_power(strength: f64, soldiers: f64, technology: f64, exhaustion: f64) -> f64 {
    let raw = (strength + soldiers * SOLDIER_POWER) * (0.5 + technology);
    (raw * (1.0 - exhaustion)).max(0.0)
}

/// How readily a government reaches for the sword (design §15): juntas live by it,
/// autocracies use it freely, monarchies less so, democracies least.
pub fn war_appetite(gov: Government) -> f64 {
    use Government::*;
    match gov {
        Junta => 1.0,
        Autocracy => 0.7,
        Monarchy => 0.4,
        Democracy => 0.2,
    }
}

/// Next standing strength after a month: add what this month's spending buys, then
/// take peacetime upkeep off the lot. Pure, so the rule can be unit-tested.
pub fn strength_after(current: f64, spend: f64) -> f64 {
    ((current + spend * STRENGTH_PER_SPEND) * (1.0 - STRENGTH_DECAY)).max(0.0)
}

/// Strength a side loses to attrition this month: a slice of its own strength,
/// scaled by the enemy's share of the combined power on the field. Pure.
pub fn attrition(own_strength: f64, own_power: f64, enemy_power: f64) -> f64 {
    let total = own_power + enemy_power;
    if total <= 0.0 {
        return 0.0;
    }
    own_strength * WAR_INTENSITY * (enemy_power / total)
}

/// Whether a belligerent sues for peace: when it is worn out, or so outmatched
/// that fighting on is hopeless. Pure.
pub fn wants_peace(exhaustion: f64, own_power: f64, enemy_power: f64) -> bool {
    exhaustion >= PEACE_EXHAUSTION || own_power <= enemy_power * 0.25
}

/// One active war between two nations (design §15). The aggressor declared it; the
/// running `score` tilts to whoever is winning (positive favours the aggressor).
#[derive(Debug, Clone)]
pub struct War {
    pub aggressor: Entity,
    pub defender: Entity,
    /// Months the war has run.
    pub months: u32,
    /// Cumulative war score: the aggressor's lead in combat power summed over the
    /// months. Its sign decides the victor when peace is made.
    pub score: f64,
}

impl War {
    pub fn involves(&self, n: Entity) -> bool {
        self.aggressor == n || self.defender == n
    }
}

/// The world's active wars (design §15). The diplomacy and trade layers consult it
/// to embargo belligerents and freeze their relations.
#[derive(Resource, Debug, Default)]
pub struct Warfront {
    pub wars: Vec<War>,
}

impl Warfront {
    /// Are these two nations currently at war with each other?
    pub fn at_war(&self, a: Entity, b: Entity) -> bool {
        self.wars
            .iter()
            .any(|w| (w.aggressor == a && w.defender == b) || (w.aggressor == b && w.defender == a))
    }

    /// Is this nation fighting anyone at all?
    pub fn belligerent(&self, n: Entity) -> bool {
        self.wars.iter().any(|w| w.involves(n))
    }
}

/// World war dashboard, refreshed monthly for the report — mirrors the finance,
/// politics and diplomacy ledgers.
#[derive(Resource, Debug, Default)]
pub struct WarLedger {
    /// Wars currently being fought.
    pub active_wars: u32,
    /// Wars that broke out this month.
    pub wars_started: u32,
    /// Wars settled this month.
    pub wars_ended: u32,
}
