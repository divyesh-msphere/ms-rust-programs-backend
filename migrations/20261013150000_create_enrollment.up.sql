CREATE TABLE IF NOT EXISTS requirements (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    name TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    position INT NOT NULL DEFAULT 0,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_requirements_program_position_id
    ON requirements (program_id, position, id);

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

-- `store_id` / `website_id` are the storefront's ids: plain TEXT, no FK.
-- `requirements_snapshot` freezes what was required at submission.
CREATE TABLE IF NOT EXISTS applications (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    store_id TEXT NOT NULL,
    website_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    requirements_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb,
    submitted_time TIMESTAMPTZ,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
) WITH (fillfactor = 85);

CREATE INDEX IF NOT EXISTS idx_applications_program_id_id
    ON applications (program_id, id);
-- A store's own applications.
CREATE INDEX IF NOT EXISTS idx_applications_store_program
    ON applications (store_id, program_id);
-- The approver queue: only submitted applications are ever scanned for.
CREATE INDEX IF NOT EXISTS idx_applications_submitted
    ON applications (program_id, id) WHERE status = 'submitted';

-- One decision per approval step per application.
CREATE TABLE IF NOT EXISTS signoffs (
    id UUID PRIMARY KEY,
    application_id UUID NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    approval_step_id UUID NOT NULL REFERENCES approval_steps (id) ON DELETE RESTRICT,
    decision TEXT NOT NULL,
    comment TEXT,
    actor TEXT NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (application_id, approval_step_id)
);
CREATE INDEX IF NOT EXISTS idx_signoffs_application_id_id
    ON signoffs (application_id, id);

CREATE TABLE IF NOT EXISTS enrollments (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    store_id TEXT NOT NULL,
    website_id TEXT NOT NULL,
    application_id UUID REFERENCES applications (id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
) WITH (fillfactor = 85);

CREATE INDEX IF NOT EXISTS idx_enrollments_program_id_id
    ON enrollments (program_id, id);
-- At most one live enrollment per store, program and website.
CREATE UNIQUE INDEX IF NOT EXISTS uq_enrollments_live
    ON enrollments (program_id, store_id, website_id)
    WHERE status IN ('pending', 'active');
-- The pricing pods' store -> programs map, built at startup and refreshed from
-- enrollment events: only ACTIVE rows, and covering, so the full load is an
-- index-only scan over the smallest index in the schema.
CREATE INDEX IF NOT EXISTS idx_enrollments_active_store
    ON enrollments (store_id, program_id) INCLUDE (website_id) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_enrollments_application_id
    ON enrollments (application_id) WHERE application_id IS NOT NULL;
