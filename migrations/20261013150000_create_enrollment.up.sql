-- Premade modules: Requirements, Applications & Approvals, Enrolled Stores and
-- Store Discounts. `store_id` / `website_id` belong to the storefront, so they are
-- plain TEXT (no foreign keys); the NAMES are copied onto the row at write time so a
-- list never calls the storefront.

-- Enrollment Requirements: the ordered list that IS the program's application form.
CREATE TABLE IF NOT EXISTS requirements (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    type TEXT NOT NULL,                              -- signup | card_on_file | credit_application | contract | custom_field
    name TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    position INT NOT NULL DEFAULT 0,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_requirements_program_position_id
    ON requirements (program_id, position, id);
-- The database itself enforces "one of each fixed type" (the greyed-out buttons);
-- custom fields repeat. `position` is NOT unique: a reorder rewrites many rows at once.
CREATE UNIQUE INDEX IF NOT EXISTS uq_requirements_fixed_type
    ON requirements (program_id, type) WHERE type <> 'custom_field';

CREATE TABLE IF NOT EXISTS approval_steps (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    position INT NOT NULL DEFAULT 0,
    approver_role TEXT NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_approval_steps_program_position_id
    ON approval_steps (program_id, position, id);

-- Applications & Approvals. The list screen reads ONLY this table. `requirements_snapshot`
-- freezes what was required at submission; `answers` holds the store's answers, keyed by
-- requirement id.
CREATE TABLE IF NOT EXISTS applications (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    store_id TEXT NOT NULL,
    store_name TEXT,
    website_id TEXT NOT NULL,
    website_name TEXT,
    status TEXT NOT NULL DEFAULT 'draft',            -- draft | submitted | in_review | approved | rejected | withdrawn
    current_step_id UUID REFERENCES approval_steps (id) ON DELETE SET NULL,
    requirements_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb,
    answers JSONB NOT NULL DEFAULT '{}'::jsonb,
    submitted_time TIMESTAMPTZ,
    decided_time TIMESTAMPTZ,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
) WITH (fillfactor = 85);

-- "All" tab, and each status chip, as one index range with no sort step.
CREATE INDEX IF NOT EXISTS idx_applications_program_id_id
    ON applications (program_id, id);
CREATE INDEX IF NOT EXISTS idx_applications_program_status_id
    ON applications (program_id, status, id);
-- A store's own applications.
CREATE INDEX IF NOT EXISTS idx_applications_store_program
    ON applications (store_id, program_id);
CREATE INDEX IF NOT EXISTS idx_applications_current_step_id
    ON applications (current_step_id) WHERE current_step_id IS NOT NULL;

-- One decision per approval step per application.
CREATE TABLE IF NOT EXISTS signoffs (
    id UUID PRIMARY KEY,
    application_id UUID NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    approval_step_id UUID NOT NULL REFERENCES approval_steps (id) ON DELETE RESTRICT,
    decision TEXT NOT NULL,                          -- approved | rejected
    comment TEXT,
    actor TEXT NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (application_id, approval_step_id)
);
CREATE INDEX IF NOT EXISTS idx_signoffs_application_id_id
    ON signoffs (application_id, id);

-- Enrolled Stores. The list screen reads ONLY this table. Contract columns are plain
-- counters for now (agreements are not part of this design yet).
CREATE TABLE IF NOT EXISTS enrollments (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    application_id UUID REFERENCES applications (id) ON DELETE SET NULL,
    store_id TEXT NOT NULL,
    store_name TEXT,
    store_resolved BOOLEAN NOT NULL DEFAULT true,    -- false = show "Unresolved store"
    website_id TEXT NOT NULL,
    website_name TEXT,
    customer_group TEXT,
    status TEXT NOT NULL DEFAULT 'pending',          -- pending | active | suspended
    enrolled_time TIMESTAMPTZ,
    contracts_count INT NOT NULL DEFAULT 0,
    last_contract_time TIMESTAMPTZ,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
) WITH (fillfactor = 85);

CREATE INDEX IF NOT EXISTS idx_enrollments_program_id_id
    ON enrollments (program_id, id);
-- The "All websites" filter.
CREATE INDEX IF NOT EXISTS idx_enrollments_program_website_id
    ON enrollments (program_id, website_id, id);
-- At most one live enrollment per store, program and website. A SUSPENDED store counts:
-- otherwise it could be enrolled a second time.
CREATE UNIQUE INDEX IF NOT EXISTS uq_enrollments_live
    ON enrollments (program_id, store_id, website_id)
    WHERE status IN ('pending', 'active', 'suspended');
-- The pricing pods' store -> programs map, built at startup and refreshed from events:
-- ACTIVE rows only, and covering, so the full load is an index-only scan.
CREATE INDEX IF NOT EXISTS idx_enrollments_active_store
    ON enrollments (store_id, program_id) INCLUDE (website_id) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_enrollments_application_id
    ON enrollments (application_id) WHERE application_id IS NOT NULL;
-- Store search (Enrolled Stores and Store Discounts): a prefix match on the name.
CREATE INDEX IF NOT EXISTS idx_enrollments_program_store_name
    ON enrollments (program_id, lower(store_name) text_pattern_ops);

-- Store Discounts: the DOMAIN and STORE level values (the program level lives in the
-- module's own config). Precedence is program < domain < store, resolved for the
-- page being shown, and once at publish time. The unique key makes every lookup one
-- index probe.
CREATE TABLE IF NOT EXISTS program_overrides (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    scope_type TEXT NOT NULL,                        -- domain | store
    scope_id TEXT NOT NULL,
    -- {enabled, type: percentage|fixed, amount, currency_code}
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (program_id, scope_type, scope_id)
);
CREATE INDEX IF NOT EXISTS idx_program_overrides_program_id_id
    ON program_overrides (program_id, id);
