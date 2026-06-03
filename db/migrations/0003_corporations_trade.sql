-- Project Atlas — Phase 3 schema: corporations + trade.
-- Adds profit-seeking firms and a per-nation trade balance.
-- Mirrors backend/src/corporation.rs (Corporation) and the trade fields added
-- to Nation in backend/src/components.rs.

BEGIN;

-- ---------- corporations ----------
-- Independent firms (design §8). Each runs one factory in a home region, is
-- domiciled in a nation, and produces one or more goods (its `industries`).
CREATE TABLE corporations (
    id            BIGSERIAL PRIMARY KEY,
    name          TEXT     NOT NULL,
    owner_nation  BIGINT   NOT NULL REFERENCES nations(id) ON DELETE CASCADE,
    home_region   BIGINT   NOT NULL REFERENCES regions(id) ON DELETE CASCADE,
    capital       DOUBLE PRECISION NOT NULL DEFAULT 0,
    employees     DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (employees >= 0),
    profit        DOUBLE PRECISION NOT NULL DEFAULT 0,
    industries    resource[] NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_corps_owner ON corporations(owner_nation);
CREATE INDEX idx_corps_region ON corporations(home_region);

-- ---------- nation trade balance ----------
-- Cumulative value of goods sold to / bought from other nations (design §14),
-- accumulated by the logistics layer's cross-border shipments.
ALTER TABLE nations
    ADD COLUMN exports DOUBLE PRECISION NOT NULL DEFAULT 0,
    ADD COLUMN imports DOUBLE PRECISION NOT NULL DEFAULT 0;

COMMIT;
