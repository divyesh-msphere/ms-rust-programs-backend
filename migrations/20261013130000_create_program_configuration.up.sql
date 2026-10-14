-- Modules enabled on a program. THE hottest read in the admin API (every
-- screen open): `WHERE program_id = $1` is a prefix scan of the primary key,
-- so no other index is needed. Settings edits update `config`/`version`,
-- never the key, so fillfactor 80 keeps those updates HOT.
CREATE TABLE IF NOT EXISTS program_modules (
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    module_id UUID NOT NULL REFERENCES module_definitions (id) ON DELETE RESTRICT,
    enabled BOOLEAN NOT NULL DEFAULT true,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    version INT NOT NULL DEFAULT 1,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (program_id, module_id)
) WITH (fillfactor = 80);

-- Values for field-driven modules: ONE ROW PER FIELD PER SCOPE. A multi-select
-- keeps its picks in `option_ids UUID[]` on that one row, instead of one row
-- per picked option: a form save is one upsert per changed field, a form load
-- reads one row per field, and a library with 40 options costs one tuple, not
-- 40.
--
-- `scope_type`/`scope_id` carry program / domain / store precedence. The
-- program-level row uses ('program', '') so the scope columns can sit in the
-- primary key without NULL handling.
--
-- `library_id` is copied from the field at write time so the reverse lookup
-- needs no join. `module_id` is copied for the same reason: loading one
-- leaf's form is a single index range with no join to module_fields.
CREATE TABLE IF NOT EXISTS program_field_values (
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    module_id UUID NOT NULL REFERENCES module_definitions (id) ON DELETE RESTRICT,
    field_id UUID NOT NULL REFERENCES module_fields (id) ON DELETE CASCADE,
    scope_type TEXT NOT NULL DEFAULT 'program',
    scope_id TEXT NOT NULL DEFAULT '',
    value JSONB,
    option_ids UUID[] NOT NULL DEFAULT '{}',
    library_id UUID,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (program_id, field_id, scope_type, scope_id)
) WITH (fillfactor = 80);

-- Form load for one leaf: (program, module) -> its fields' values.
CREATE INDEX IF NOT EXISTS idx_program_field_values_program_module
    ON program_field_values (program_id, module_id);
-- Webhook reverse lookup: "which programs use library X?"
CREATE INDEX IF NOT EXISTS idx_program_field_values_library_program
    ON program_field_values (library_id, program_id) WHERE library_id IS NOT NULL;
-- "Which programs picked option Y?" — used to flag drift when an option is
-- deactivated. GIN pays its cost only on writes that change `option_ids`.
CREATE INDEX IF NOT EXISTS idx_program_field_values_option_ids
    ON program_field_values USING GIN (option_ids);

-- Small per-module lists (3PL fees, grade mapping, upsell products).
CREATE TABLE IF NOT EXISTS module_records (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    module_id UUID NOT NULL REFERENCES module_definitions (id) ON DELETE RESTRICT,
    data JSONB NOT NULL DEFAULT '{}'::jsonb,
    position INT NOT NULL DEFAULT 0,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_module_records_program_module_position_id
    ON module_records (program_id, module_id, position, id);

-- Domain- and store-level config. Precedence is program < domain < store and
-- is resolved once, at publish time, never on the request path.
CREATE TABLE IF NOT EXISTS program_overrides (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    scope_type TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (program_id, scope_type, scope_id)
);

CREATE INDEX IF NOT EXISTS idx_program_overrides_program_id_id
    ON program_overrides (program_id, id);

-- Websites a program applies to, with customer group. `website_id` and
-- `customer_group_id` belong to the storefront, so they are plain TEXT, not
-- foreign keys.
CREATE TABLE IF NOT EXISTS program_domains (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    website_id TEXT NOT NULL,
    domain TEXT NOT NULL,
    customer_group_id TEXT,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (program_id, website_id, domain)
);

CREATE INDEX IF NOT EXISTS idx_program_domains_program_id_id
    ON program_domains (program_id, id);
