-- Project Atlas — Phase 2 schema: production + market.
-- Adds regional raw deposits and a global price per good.
-- Mirrors backend/src/resources.rs (Deposits) and production.rs (Market).

BEGIN;

-- ---------- region deposits ----------
-- Natural endowment: richness multiplier per raw good a region can mine.
-- One row per (region, resource). Mirrors the ECS `Deposits` component.
CREATE TABLE region_deposits (
    region_id   BIGINT   NOT NULL REFERENCES regions(id) ON DELETE CASCADE,
    resource    resource NOT NULL,
    richness    DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (richness >= 0),
    PRIMARY KEY (region_id, resource)
);

-- ---------- market prices ----------
-- Single global commodity market: one live price per good (design §9).
-- `base_price` is the reference the live price is bounded around.
CREATE TABLE market_prices (
    resource    resource PRIMARY KEY,
    price       DOUBLE PRECISION NOT NULL CHECK (price > 0),
    base_price  DOUBLE PRECISION NOT NULL CHECK (base_price > 0)
);

COMMIT;
