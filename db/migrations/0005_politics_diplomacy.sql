-- Project Atlas — Phase 5 schema: politics + diplomacy.
-- Per-nation political state (stability/prestige already live on `nations` from
-- 0001; this stores the unrest dynamics) and the pairwise relation web between
-- nations. Mirrors backend/src/politics.rs (Politics) and diplomacy.rs (Diplomacy).

BEGIN;

-- ---------- politics ----------
-- One row per nation (design §13). `unrest` mirrors stability; `turmoil_months`
-- counts consecutive months of open unrest — the pressure that tips into a
-- revolution or coup once it has run deep for long enough.
CREATE TABLE politics (
    nation_id       BIGINT  PRIMARY KEY REFERENCES nations(id) ON DELETE CASCADE,
    unrest          DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (unrest BETWEEN 0 AND 1),
    turmoil_months  INTEGER NOT NULL DEFAULT 0 CHECK (turmoil_months >= 0)
);

-- ---------- diplomacy ----------
-- One row per unordered nation pair, `nation_a < nation_b`, holding the relation
-- score in [-1, 1] (design §14). The status (ally/neutral/rival/hostile) is
-- derived from the score, so it is not stored.
CREATE TABLE relations (
    nation_a   BIGINT NOT NULL REFERENCES nations(id) ON DELETE CASCADE,
    nation_b   BIGINT NOT NULL REFERENCES nations(id) ON DELETE CASCADE,
    score      DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (score BETWEEN -1 AND 1),
    PRIMARY KEY (nation_a, nation_b),
    CHECK (nation_a < nation_b)
);

COMMIT;
