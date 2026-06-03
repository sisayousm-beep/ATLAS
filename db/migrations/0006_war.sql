-- Project Atlas — Phase 6 schema: war.
-- Per-nation armed forces and the active wars between nations. Mirrors
-- backend/src/war.rs (Military, and War / Warfront).

BEGIN;

-- ---------- military ----------
-- One row per nation (design §15). `strength` is the standing army bought from
-- the economy; `exhaustion` climbs while at war and, once high enough, forces a
-- nation to sue for peace.
CREATE TABLE military (
    nation_id   BIGINT PRIMARY KEY REFERENCES nations(id) ON DELETE CASCADE,
    strength    DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (strength >= 0),
    exhaustion  DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (exhaustion BETWEEN 0 AND 1)
);

-- ---------- wars ----------
-- One row per active war (design §15). The aggressor declared it; `score` tilts
-- to whoever is winning (positive favours the aggressor) and names the victor when
-- peace is made. A nation cannot be at war with itself, and a pair fights at most
-- one war at a time.
CREATE TABLE wars (
    id          BIGSERIAL PRIMARY KEY,
    aggressor   BIGINT NOT NULL REFERENCES nations(id) ON DELETE CASCADE,
    defender    BIGINT NOT NULL REFERENCES nations(id) ON DELETE CASCADE,
    months      INTEGER NOT NULL DEFAULT 0 CHECK (months >= 0),
    score       DOUBLE PRECISION NOT NULL DEFAULT 0,
    CHECK (aggressor <> defender),
    UNIQUE (aggressor, defender)
);

COMMIT;
