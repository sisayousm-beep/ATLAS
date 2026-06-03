-- Project Atlas — Phase 1 schema
-- Entities: nations, regions, pops, per-region resource stocks.
-- Mirrors the ECS components in backend/src/components.rs.

BEGIN;

-- ---------- enums ----------
CREATE TYPE government AS ENUM ('democracy', 'autocracy', 'monarchy', 'junta');

CREATE TYPE terrain AS ENUM ('plains', 'hills', 'mountains', 'coast', 'desert');

CREATE TYPE climate AS ENUM ('temperate', 'tropical', 'arid', 'continental', 'polar');

CREATE TYPE profession AS ENUM (
    'farmer', 'laborer', 'merchant', 'engineer',
    'researcher', 'soldier', 'bureaucrat', 'unemployed'
);

CREATE TYPE ideology AS ENUM (
    'conservative', 'progressive', 'liberal', 'socialist', 'militarist'
);

CREATE TYPE resource AS ENUM (
    -- raw
    'grain', 'wood', 'iron_ore', 'coal', 'oil', 'uranium', 'rare_earth',
    -- intermediate
    'iron', 'steel', 'plastic', 'semiconductor', 'battery',
    -- finished
    'car', 'electronics', 'weapon', 'computer', 'robot'
);

-- ---------- nations ----------
CREATE TABLE nations (
    id          BIGSERIAL PRIMARY KEY,
    name        TEXT        NOT NULL,
    treasury    DOUBLE PRECISION NOT NULL DEFAULT 0,
    debt        DOUBLE PRECISION NOT NULL DEFAULT 0,
    inflation   DOUBLE PRECISION NOT NULL DEFAULT 0.02,
    stability   DOUBLE PRECISION NOT NULL DEFAULT 0.7,
    prestige    DOUBLE PRECISION NOT NULL DEFAULT 0,
    technology  DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    government  government   NOT NULL
);

-- ---------- regions ----------
CREATE TABLE regions (
    id             BIGSERIAL PRIMARY KEY,
    name           TEXT      NOT NULL,
    terrain        terrain   NOT NULL,
    climate        climate   NOT NULL,
    infrastructure DOUBLE PRECISION NOT NULL DEFAULT 0.5
                   CHECK (infrastructure BETWEEN 0 AND 1),
    owner_nation   BIGINT    NOT NULL REFERENCES nations(id) ON DELETE CASCADE
);

CREATE INDEX idx_regions_owner ON regions(owner_nation);

-- ---------- pops ----------
CREATE TABLE pops (
    id          BIGSERIAL PRIMARY KEY,
    region_id   BIGINT    NOT NULL REFERENCES regions(id) ON DELETE CASCADE,
    size        INTEGER   NOT NULL CHECK (size >= 0),
    profession  profession NOT NULL,
    wealth      DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    literacy    DOUBLE PRECISION NOT NULL DEFAULT 0.5 CHECK (literacy BETWEEN 0 AND 1),
    happiness   DOUBLE PRECISION NOT NULL DEFAULT 0.6 CHECK (happiness BETWEEN 0 AND 1),
    ideology    ideology  NOT NULL
);

CREATE INDEX idx_pops_region ON pops(region_id);

-- ---------- resource stocks ----------
-- One row per (region, resource). Amount can grow/shrink each tick.
CREATE TABLE resource_stocks (
    region_id   BIGINT  NOT NULL REFERENCES regions(id) ON DELETE CASCADE,
    resource    resource NOT NULL,
    amount      DOUBLE PRECISION NOT NULL DEFAULT 0,
    PRIMARY KEY (region_id, resource)
);

COMMIT;
