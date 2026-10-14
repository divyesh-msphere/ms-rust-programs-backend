-- Blueprints: starting points for the create-program wizard. Copied into a
-- program at creation; a program is never live-linked to its blueprint.
CREATE TABLE IF NOT EXISTS blueprints (
    id UUID PRIMARY KEY,
    key TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    active BOOLEAN NOT NULL DEFAULT true,
    state TEXT NOT NULL DEFAULT 'ACTIVE',
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_blueprints_key_active_ci
    ON blueprints (lower(key)) WHERE state = 'ACTIVE';

-- Live-row keyset list; the deleted view (`show_deleted=true`) is rare enough
-- to scan.
CREATE INDEX IF NOT EXISTS idx_blueprints_active_id
    ON blueprints (id) WHERE state = 'ACTIVE';

-- Programs: the root resource. Written on every configuration change
-- (config_version / config_hash bump), so it is created with spare room on
-- each page (fillfactor 85): those updates touch no indexed column and stay
-- HOT — no index entries rewritten, no bloat.
CREATE TABLE IF NOT EXISTS programs (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    description TEXT,
    active BOOLEAN NOT NULL DEFAULT true,
    blueprint_id UUID REFERENCES blueprints (id) ON DELETE SET NULL,
    cloned_from_id UUID REFERENCES programs (id) ON DELETE SET NULL,
    config_version INT NOT NULL DEFAULT 1,
    config_hash TEXT NOT NULL DEFAULT '',
    state TEXT NOT NULL DEFAULT 'ACTIVE',
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
) WITH (fillfactor = 85);

CREATE UNIQUE INDEX IF NOT EXISTS uq_programs_slug_active_ci
    ON programs (lower(slug)) WHERE state = 'ACTIVE';
CREATE INDEX IF NOT EXISTS idx_programs_active_id
    ON programs (id) WHERE state = 'ACTIVE';
-- FK columns that are NULL for most rows: index only the rows that have one.
CREATE INDEX IF NOT EXISTS idx_programs_blueprint_id
    ON programs (blueprint_id) WHERE blueprint_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_programs_cloned_from_id
    ON programs (cloned_from_id) WHERE cloned_from_id IS NOT NULL;
