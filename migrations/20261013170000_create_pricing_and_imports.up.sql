-- Bulk-written by price import: kept to the primary key, the natural-key
-- unique (the ON CONFLICT target of the bulk upsert) and the list index. That
-- is the minimum that still serves every query, since each extra index
-- multiplies the cost of a 100k-row import.
CREATE TABLE IF NOT EXISTS sku_prices (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    sku TEXT NOT NULL,
    price NUMERIC(18, 4) NOT NULL,
    currency_code TEXT NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (program_id, sku, currency_code)
) WITH (fillfactor = 90);
CREATE INDEX IF NOT EXISTS idx_sku_prices_program_id_id
    ON sku_prices (program_id, id);

CREATE TABLE IF NOT EXISTS core_return_fees (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    sku TEXT NOT NULL,
    fee NUMERIC(18, 4) NOT NULL,
    currency_code TEXT NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (program_id, sku, currency_code)
) WITH (fillfactor = 90);
CREATE INDEX IF NOT EXISTS idx_core_return_fees_program_id_id
    ON core_return_fees (program_id, id);

-- `status` / counters are updated as the job progresses, hence the spare room.
CREATE TABLE IF NOT EXISTS import_jobs (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    file_key TEXT NOT NULL,
    total_rows INT NOT NULL DEFAULT 0,
    failed_rows INT NOT NULL DEFAULT 0,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
) WITH (fillfactor = 80);
CREATE INDEX IF NOT EXISTS idx_import_jobs_program_id_id
    ON import_jobs (program_id, id);
-- Workers poll for runnable jobs; the index holds only unfinished ones.
CREATE INDEX IF NOT EXISTS idx_import_jobs_unfinished
    ON import_jobs (id) WHERE status IN ('pending', 'running');

-- Insert-only, one row per input line, written in batches. UUIDv7 ids arrive in
-- insertion order, so `(import_job_id, id)` is both the only secondary index
-- and the file-order listing. `row_number` is deliberately not indexed.
-- Default fillfactor (100): rows are never updated, so no spare room is wasted.
CREATE TABLE IF NOT EXISTS import_job_rows (
    id UUID PRIMARY KEY,
    import_job_id UUID NOT NULL REFERENCES import_jobs (id) ON DELETE CASCADE,
    row_number INT NOT NULL,
    status TEXT NOT NULL,
    data JSONB NOT NULL DEFAULT '{}'::jsonb,
    error TEXT
);
CREATE INDEX IF NOT EXISTS idx_import_job_rows_job_id_id
    ON import_job_rows (import_job_id, id);
-- Failed-rows view of a job: failures are a small fraction, so index only them.
CREATE INDEX IF NOT EXISTS idx_import_job_rows_failed
    ON import_job_rows (import_job_id, id) WHERE status = 'failed';
