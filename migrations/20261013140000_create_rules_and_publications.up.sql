-- One row per (program, rule family). Tiny and read on every rules screen.
CREATE TABLE IF NOT EXISTS rule_sets (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    family TEXT NOT NULL,
    draft_version INT,
    published_version INT,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (program_id, family)
) WITH (fillfactor = 85);

-- Immutable versions. The JDM document itself lives in S3 under its sha256
-- (`rules/<content_hash>.json`); Postgres holds only the pointer, so these
-- rows stay a few dozen bytes and the table never TOASTs large graphs.
-- Append-only: never updated, so default fillfactor.
CREATE TABLE IF NOT EXISTS rule_versions (
    id UUID PRIMARY KEY,
    rule_set_id UUID NOT NULL REFERENCES rule_sets (id) ON DELETE CASCADE,
    version INT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    content_hash TEXT NOT NULL,
    created_by TEXT,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (rule_set_id, version)
);

-- List and "latest version" both walk (rule_set_id, id); UUIDv7 ids are in
-- creation order, so the newest version is the last entry of the range.
CREATE INDEX IF NOT EXISTS idx_rule_versions_rule_set_id_id
    ON rule_versions (rule_set_id, id);

-- Published bundles (S3 `bundles/<program>/<config_version>.json`). Append-only.
CREATE TABLE IF NOT EXISTS publications (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    config_version INT NOT NULL,
    config_hash TEXT NOT NULL,
    bundle_key TEXT NOT NULL,
    created_by TEXT,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- "Latest publication of a program" (the drift badge, every pod's bundle
-- load) is one backward index step. INCLUDE makes it index-only: the hash
-- comparison against programs.config_hash never touches the heap.
CREATE INDEX IF NOT EXISTS idx_publications_program_id_id
    ON publications (program_id, id) INCLUDE (config_version, config_hash);
