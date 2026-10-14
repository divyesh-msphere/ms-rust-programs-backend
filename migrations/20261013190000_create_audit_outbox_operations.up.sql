-- AUDIT LOG: append-only, grows forever, written on every change. Declarative
-- monthly RANGE partitions on create_time give three things at once:
--   * inserts only ever touch the current month's small, hot partition and its
--     indexes (which stay in cache);
--   * "history of this program" prunes to the months asked for;
--   * retention is `DROP TABLE`/`DETACH PARTITION` — instant, no DELETE, no
--     vacuum debt.
-- No foreign keys: an audit row must survive its subject and must not pay a
-- lookup on every insert. The primary key must contain the partition key.
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    program_id UUID,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    before JSONB,
    after JSONB,
    PRIMARY KEY (create_time, id)
) PARTITION BY RANGE (create_time);

-- Created on the parent, so every partition (present and future) gets it.
-- Newest-first history of one program is a single backward index range.
CREATE INDEX IF NOT EXISTS idx_audit_log_program_time
    ON audit_log (program_id, create_time DESC, id DESC) WHERE program_id IS NOT NULL;

-- Rows outside every monthly partition land here instead of failing the write.
CREATE TABLE IF NOT EXISTS audit_log_default PARTITION OF audit_log DEFAULT;

-- Creates the partitions for the previous month through `months_ahead` months
-- from now. Idempotent. Run it from the migration (below) and then monthly
-- from the service or a scheduler, so the current month always has a home.
CREATE OR REPLACE FUNCTION ensure_audit_log_partitions(months_ahead INT DEFAULT 3)
RETURNS VOID AS $$
DECLARE
    month_start DATE;
    part_name TEXT;
BEGIN
    FOR i IN -1..months_ahead LOOP
        month_start := (date_trunc('month', now()) + make_interval(months => i))::date;
        part_name := 'audit_log_' || to_char(month_start, 'YYYYMM');
        IF to_regclass(part_name) IS NULL THEN
            EXECUTE format(
                'CREATE TABLE %I PARTITION OF audit_log FOR VALUES FROM (%L) TO (%L)',
                part_name, month_start, (month_start + interval '1 month')::date);
        END IF;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

SELECT ensure_audit_log_partitions(3);

-- OUTBOX: written in the same transaction as the change it announces, drained
-- by a relay. Insert-heavy with constant churn, so it is kept tiny:
--   * the only secondary index is PARTIAL on pending rows, so it holds just
--     the unsent backlog (normally a handful of entries) however many sent
--     rows have not been pruned yet;
--   * low fillfactor + aggressive autovacuum keep dead tuples from piling up
--     as rows flip from pending to published.
-- The relay claims work with
--   SELECT ... WHERE status = 'pending' ORDER BY id LIMIT n FOR UPDATE SKIP LOCKED
-- so several relay instances never block each other.
CREATE TABLE IF NOT EXISTS outbox (
    id UUID PRIMARY KEY,
    topic TEXT NOT NULL,
    event_key TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INT NOT NULL DEFAULT 0,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_time TIMESTAMPTZ
) WITH (
    fillfactor = 70,
    autovacuum_vacuum_scale_factor = 0.01,
    autovacuum_vacuum_threshold = 100
);
CREATE INDEX IF NOT EXISTS idx_outbox_pending
    ON outbox (id) WHERE status = 'pending';
-- Pruning of already-published rows by age.
CREATE INDEX IF NOT EXISTS idx_outbox_published_time
    ON outbox (published_time) WHERE status = 'published';

-- AEP-151 long-running operations (publish, imports, report runs).
CREATE TABLE IF NOT EXISTS operations (
    id UUID PRIMARY KEY,
    kind TEXT NOT NULL,
    done BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    response JSONB,
    error JSONB,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
) WITH (fillfactor = 80);
-- Workers and the reaper only look for unfinished operations.
CREATE INDEX IF NOT EXISTS idx_operations_unfinished
    ON operations (id) WHERE done = false;
