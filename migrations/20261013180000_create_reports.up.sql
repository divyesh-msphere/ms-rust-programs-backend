CREATE TABLE IF NOT EXISTS report_definitions (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_report_definitions_code_ci
    ON report_definitions (lower(code));

CREATE TABLE IF NOT EXISTS report_runs (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    report_id UUID NOT NULL REFERENCES report_definitions (id) ON DELETE RESTRICT,
    status TEXT NOT NULL DEFAULT 'pending',
    result_key TEXT,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
) WITH (fillfactor = 85);
CREATE INDEX IF NOT EXISTS idx_report_runs_program_id_id
    ON report_runs (program_id, id);
CREATE INDEX IF NOT EXISTS idx_report_runs_report_id
    ON report_runs (report_id);
