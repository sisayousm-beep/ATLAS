//! Phase 7: the nation AI (design §16) — the strategic "brain" that runs each
//! nation.
//!
//! Every nation carries a fixed *personality* (aggressive, defensive, commercial,
//! diplomatic, scientific). Once a month that personality decides the nation's
//! policy for the month: how hard it arms, how readily it will reach for war, how
//! much it courts its neighbours, and how much treasury it pours into research —
//! the levers the earlier phases already obey (military build-up Phase 6, war
//! Phase 6, diplomacy Phase 5). A scientific power turns that research budget into
//! technology, which currently nothing else grows.
//!
//! The personality is fixed, but the AI is not blind: a nation whose regime is
//! crumbling drops everything and plays for survival (design §16 goals
//! 성장·생존·패권) — it stops picking fights and lets its labs go dark. As
//! elsewhere the data and pure helpers live here; the monthly behaviour lives in
//! [`systems`](crate::systems).

use bevy_ecs::prelude::*;

/// The strategic temperament of a nation's AI (design §16). Fixed at founding;
/// drives every policy lever the AI sets each month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiPersonality {
    /// Arms heavily and goes to war readily — the would-be conqueror.
    Aggressive,
    /// Arms heavily but strikes only under threat — deterrence, not conquest.
    Defensive,
    /// Spends little on arms, never starts a war — grows the treasury through trade.
    Commercial,
    /// Courts its neighbours into alliances; keeps a token army.
    Diplomatic,
    /// Pours treasury into research, climbing the technology ladder (design §12, §17).
    Scientific,
}

impl AiPersonality {
    /// Share of treasury the AI devotes to its standing army each month — its take
    /// on the guns-vs-butter trade-off (consumed by `military_buildup`, Phase 6).
    pub fn military_share(self) -> f64 {
        use AiPersonality::*;
        match self {
            Aggressive => 0.10,
            Defensive => 0.08,
            Scientific => 0.03,
            Commercial => 0.02,
            Diplomatic => 0.02,
        }
    }

    /// Multiplier on the government's raw war appetite (Phase 6 ignition): above 1
    /// the nation reaches for the sword more readily than its regime alone would,
    /// at 0 it never starts a war.
    pub fn aggression(self) -> f64 {
        use AiPersonality::*;
        match self {
            Aggressive => 1.6,
            Scientific => 0.5,
            Defensive => 0.3,
            Commercial => 0.0,
            Diplomatic => 0.0,
        }
    }

    /// How hard the AI courts its neighbours: a bonus added to the target of every
    /// relation it holds (consumed by `diplomacy`, Phase 5), so a diplomatic power
    /// warms even cool ties toward alliance.
    pub fn diplo_drive(self) -> f64 {
        match self {
            AiPersonality::Diplomatic => 0.4,
            _ => 0.0,
        }
    }

    /// Share of treasury the AI sinks into research each month, lifting its
    /// technology. Only a scientific power funds labs in earnest.
    pub fn research_share(self) -> f64 {
        match self {
            AiPersonality::Scientific => 0.06,
            _ => 0.0,
        }
    }
}

/// Stability at or below which a nation's regime is in crisis and the AI plays for
/// survival — it stops picking fights and halts discretionary spending.
pub const SURVIVAL_STABILITY: f64 = 0.35;
/// Fraction of its normal military budget a nation in survival still spends —
/// it cannot fully disarm under threat, but it stops over-arming.
pub const SURVIVAL_MILITARY_TRIM: f64 = 0.5;
/// Technology gained per unit of treasury spent on research, with diminishing
/// returns as the level rises (catching up is cheaper than pushing the frontier).
pub const TECH_PER_SPEND: f64 = 1.0e-4;

/// Whether a nation's regime is in crisis and the AI should play for survival.
/// Pure, so the rule can be unit-tested on its own.
pub fn in_survival(stability: f64) -> bool {
    stability < SURVIVAL_STABILITY
}

/// Next technology level after spending `research_spend` on research: a gain that
/// slows as the level climbs (diminishing returns). Pure.
pub fn technology_after(technology: f64, research_spend: f64) -> f64 {
    technology + TECH_PER_SPEND * research_spend / (1.0 + technology)
}

/// A nation's strategic brain (design §16). Sits on the `Nation` entity beside its
/// central bank, political state and military. The `personality` is fixed; the
/// other fields are the levers the [`nation_ai`](crate::systems::nation_ai) system
/// resolves from it each month for the rest of the schedule to read.
#[derive(Component, Debug)]
pub struct NationAi {
    pub personality: AiPersonality,
    /// Resolved each month: share of treasury for the army (read by Phase 6).
    pub military_share: f64,
    /// Resolved each month: war-appetite multiplier (read by Phase 6); 0 in survival.
    pub aggression: f64,
    /// Resolved each month: relation-target bonus the AI confers (read by Phase 5).
    pub diplo_drive: f64,
}

impl NationAi {
    /// A fresh brain of the given personality. The monthly levers start neutral and
    /// are resolved on the first month-start before any system reads them.
    pub fn new(personality: AiPersonality) -> Self {
        NationAi { personality, military_share: 0.0, aggression: 0.0, diplo_drive: 0.0 }
    }
}

/// World AI dashboard, refreshed monthly for the report — mirrors the finance,
/// politics, diplomacy and war ledgers.
#[derive(Resource, Debug, Default)]
pub struct AiLedger {
    /// Mean technology level across all nations.
    pub avg_technology: f64,
    /// The highest technology level any nation has reached (the tech leader).
    pub max_technology: f64,
    /// Nations whose regimes are in crisis and playing for survival this month.
    pub in_survival: u32,
}
