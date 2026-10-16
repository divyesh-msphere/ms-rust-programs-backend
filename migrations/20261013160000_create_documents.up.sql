-- Documents Panel. File BYTES live in object storage under `file_key` (uploads and
-- downloads use presigned URLs, so they never pass through the service); Postgres
-- keeps only metadata and the counters the screen shows.

-- One small row per program: the left panel and header are ONE primary-key read,
-- never a COUNT(*). Updated in the same transaction as every upload-complete, delete,
-- move and status change.
CREATE TABLE IF NOT EXISTS program_storage (
    program_id UUID PRIMARY KEY REFERENCES programs (id) ON DELETE CASCADE,
    file_count INT NOT NULL DEFAULT 0,
    total_bytes BIGINT NOT NULL DEFAULT 0,
    needs_review_count INT NOT NULL DEFAULT 0,
    quarantined_count INT NOT NULL DEFAULT 0,
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
) WITH (fillfactor = 70);

CREATE TABLE IF NOT EXISTS document_folders (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    -- general | import_artifacts | program_documents | store_enrollment_documents;
    -- NULL for a custom folder. The four system folders are created with each program.
    key TEXT,
    system BOOLEAN NOT NULL DEFAULT false,           -- cannot be renamed or deleted
    parent_id UUID REFERENCES document_folders (id) ON DELETE CASCADE,
    document_count INT NOT NULL DEFAULT 0,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_document_folders_program_parent_id
    ON document_folders (program_id, parent_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_document_folders_key
    ON document_folders (program_id, key) WHERE key IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_document_folders_name
    ON document_folders (program_id, COALESCE(parent_id, '00000000-0000-0000-0000-000000000000'::uuid), lower(name));

CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    folder_id UUID REFERENCES document_folders (id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    file_key TEXT NOT NULL,
    -- uploading (hidden from every list) -> pending_scan -> ok | needs_review | quarantined
    status TEXT NOT NULL DEFAULT 'uploading',
    -- Who created it, when another module did: import_job | enrollment | application | ...
    source_type TEXT,
    source_id TEXT,
    uploaded_by TEXT,
    checksum TEXT,                                   -- SHA-256, verified at upload-complete
    scan_time TIMESTAMPTZ,
    review_note TEXT,
    reviewed_by TEXT,
    reviewed_time TIMESTAMPTZ,
    state TEXT NOT NULL DEFAULT 'ACTIVE',
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
) WITH (fillfactor = 85);

-- All files, and Recent (UUIDv7 ids are in time order): one backward index range.
CREATE INDEX IF NOT EXISTS idx_documents_program_id_desc
    ON documents (program_id, id DESC) WHERE state = 'ACTIVE' AND status <> 'uploading';
-- A folder's files.
CREATE INDEX IF NOT EXISTS idx_documents_program_folder_id
    ON documents (program_id, folder_id, id) WHERE state = 'ACTIVE' AND status <> 'uploading';
-- Needs review / Quarantined scan almost nothing.
CREATE INDEX IF NOT EXISTS idx_documents_needs_review
    ON documents (program_id, id) WHERE state = 'ACTIVE' AND status = 'needs_review';
CREATE INDEX IF NOT EXISTS idx_documents_quarantined
    ON documents (program_id, id) WHERE state = 'ACTIVE' AND status = 'quarantined';
-- Search: a prefix match on the name.
CREATE INDEX IF NOT EXISTS idx_documents_program_name
    ON documents (program_id, lower(name) text_pattern_ops) WHERE state = 'ACTIVE';
-- The job that removes uploads that never completed.
CREATE INDEX IF NOT EXISTS idx_documents_stuck_uploads
    ON documents (create_time) WHERE status = 'uploading';
