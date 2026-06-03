//! Phase 4: money. Central banks, government debt and inflation (design §11).
//!
//! Every nation carries a [`CentralBank`] on its entity. The bank sets a policy
//! interest rate to steer inflation toward target and tracks the broad money it
//! has issued — government deficits are financed by expanding it. Inflation is
//! no longer a fixed constant: it emerges monthly from money growth and the
//! goods price level (design §9 prices feeding §11 money).
//!
//! As elsewhere the data and pure helpers live here; the monthly behaviour lives
//! in [`systems`](crate::systems).

use bevy_ecs::prelude::*;

/// A nation's monetary authority (design §11). Sits on the `Nation` entity.
#[derive(Component, Debug)]
pub struct CentralBank {
    /// Policy interest rate, e.g. 0.03. Drives debt-service cost and (via the
    /// hiring drag) the pace of corporate expansion.
    pub policy_rate: f64,
    /// The inflation the bank steers toward (design §11 price stability).
    pub target_inflation: f64,
    /// Broad money in circulation. Grows when the bank finances a deficit.
    pub money_supply: f64,
    /// Money supply at the last inflation update — the base for money growth.
    pub last_money_supply: f64,
}

impl CentralBank {
    /// A central bank seeded for a nation holding `treasury`. Money supply is
    /// scaled to the economy so a month's deficit is a visible slice of it —
    /// enough that monetising deficits shows up as inflation.
    pub fn seed(treasury: f64) -> Self {
        let money = treasury * 3.0 + 20_000.0;
        CentralBank {
            policy_rate: 0.03,
            target_inflation: 0.02,
            money_supply: money,
            last_money_supply: money,
        }
    }
}

/// How hard the policy rate leans against the inflation gap (Taylor-style).
pub const POLICY_RESPONSE: f64 = 0.5;
/// The policy rate is held within `[0, RATE_CAP]`.
pub const RATE_CAP: f64 = 0.25;

/// Next policy rate: lean toward where the inflation gap points, then clamp to
/// the allowed band. Pure, so the rule can be unit-tested on its own.
pub fn policy_rate_after(current: f64, inflation: f64, target: f64) -> f64 {
    (current + POLICY_RESPONSE * (inflation - target)).clamp(0.0, RATE_CAP)
}

/// Aggregate finance dashboard + crisis flag (design §11 events), refreshed
/// monthly for the report — mirrors how [`TradeLedger`](crate::corporation::TradeLedger)
/// summarises Phase 3.
#[derive(Resource, Debug, Default)]
pub struct FinanceLedger {
    /// Total government debt across all nations.
    pub gov_debt: f64,
    /// Total broad money across all central banks.
    pub money_supply: f64,
    /// Mean inflation across nations.
    pub avg_inflation: f64,
    /// Highest policy rate any central bank is holding.
    pub max_policy_rate: f64,
    /// True in a financial crisis (design §11): runaway inflation or a
    /// sovereign-debt spiral somewhere in the world.
    pub crisis: bool,
}
