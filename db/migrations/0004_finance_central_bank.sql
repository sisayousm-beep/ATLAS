-- Project Atlas — Phase 4 schema: finance + central bank.
-- Adds a per-nation monetary authority. The fiscal state it drives (debt,
-- inflation) already lives on `nations` from 0001; this stores the bank itself.
-- Mirrors backend/src/finance.rs (CentralBank).

BEGIN;

-- ---------- central banks ----------
-- One monetary authority per nation (design §11). Sets the policy rate to steer
-- inflation toward target and tracks the broad money it has issued; deficits are
-- financed by expanding `money_supply`. `last_money_supply` is the base the next
-- inflation update measures monetary growth against.
CREATE TABLE central_banks (
    nation_id          BIGINT  PRIMARY KEY REFERENCES nations(id) ON DELETE CASCADE,
    policy_rate        DOUBLE PRECISION NOT NULL DEFAULT 0.03 CHECK (policy_rate >= 0),
    target_inflation   DOUBLE PRECISION NOT NULL DEFAULT 0.02,
    money_supply       DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (money_supply >= 0),
    last_money_supply  DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (last_money_supply >= 0)
);

COMMIT;
