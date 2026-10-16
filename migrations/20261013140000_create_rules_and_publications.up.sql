-- Rule engine, minimal: one rule set per RULE LEAF (a leaf whose `program_modules.mode`
-- is `rule`), immutable versions, and publications (what the Overview's "Published rules /
-- drift" card reads). CSV entries, manual products and test cases are not part of this
-- design yet.

-- The rule engine itself is a front-end component; `mode = 'rule'` on the leaf is what tells
-- the frontend to render it, so the backend defines NO rule types. `rule_type` is optional and
-- informational (it defaults from the module's `rule_family` when there is one). The composite
-- foreign key means a rule set can only exist on a module this program actually chose, and
-- UNIQUE makes it one per leaf.
CREATE TABLE IF NOT EXISTS rule_sets (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL,
    module_id UUID NOT NULL,
    rule_type TEXT,
    draft_version INT,
    published_version INT,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (program_id, module_id),
    FOREIGN KEY (program_id, module_id)
        REFERENCES program_modules (program_id, module_id) ON DELETE CASCADE
) WITH (fillfactor = 85);

-- Immutable, append-only versions. `definition` is the admin-facing rule model;
-- `compiled` is the generated decision graph. Both live in Postgres, in the same
-- transaction as the version row, audit entry and outbox event (no object storage,
-- no orphan files). Nothing ever updates a published version.
CREATE TABLE IF NOT EXISTS rule_versions (
    id UUID PRIMARY KEY,
    rule_set_id UUID NOT NULL REFERENCES rule_sets (id) ON DELETE CASCADE,
    version INT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',            -- draft | published | superseded
    definition JSONB NOT NULL,
    compiled JSONB,
    valid BOOLEAN NOT NULL DEFAULT false,
    validation_errors JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- Hash of `definition`: an unchanged save creates no new version.
    content_hash TEXT NOT NULL,
    size_bytes INT NOT NULL DEFAULT 0,
    created_by TEXT,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (rule_set_id, version)
);
-- List and "latest version": UUIDv7 ids are in creation order.
CREATE INDEX IF NOT EXISTS idx_rule_versions_rule_set_id_id ON rule_versions (rule_set_id, id);

-- Published bundles. Append-only.
CREATE TABLE IF NOT EXISTS publications (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    config_version INT NOT NULL,
    config_hash TEXT NOT NULL,
    bundle JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by TEXT,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- "Latest publication of a program" (the drift badge, every pod's bundle load) is
-- one backward index step. INCLUDE makes it index-only: the comparison against
-- programs.config_hash never touches the heap.
CREATE INDEX IF NOT EXISTS idx_publications_program_id_id
    ON publications (program_id, id) INCLUDE (config_version, config_hash);
