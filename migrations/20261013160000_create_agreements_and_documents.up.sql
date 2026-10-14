CREATE TABLE IF NOT EXISTS agreement_templates (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    latest_version INT NOT NULL DEFAULT 0,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_agreement_templates_code_ci
    ON agreement_templates (lower(code));

-- Immutable, append-only: a contract pins one of these forever.
CREATE TABLE IF NOT EXISTS agreement_template_versions (
    id UUID PRIMARY KEY,
    template_id UUID NOT NULL REFERENCES agreement_templates (id) ON DELETE CASCADE,
    version INT NOT NULL,
    body TEXT NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (template_id, version)
);
CREATE INDEX IF NOT EXISTS idx_agreement_template_versions_template_id_id
    ON agreement_template_versions (template_id, id);

CREATE TABLE IF NOT EXISTS agreement_contracts (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    enrollment_id UUID NOT NULL REFERENCES enrollments (id) ON DELETE CASCADE,
    template_id UUID NOT NULL REFERENCES agreement_templates (id) ON DELETE RESTRICT,
    template_version INT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    signed_time TIMESTAMPTZ,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_agreement_contracts_program_id_id
    ON agreement_contracts (program_id, id);
CREATE INDEX IF NOT EXISTS idx_agreement_contracts_enrollment_id
    ON agreement_contracts (enrollment_id);

CREATE TABLE IF NOT EXISTS document_folders (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    parent_id UUID REFERENCES document_folders (id) ON DELETE CASCADE,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_document_folders_program_parent_id
    ON document_folders (program_id, parent_id, id);

-- File bytes live in S3 under `file_key`; the presigned URI is computed at read.
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY,
    program_id UUID NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    folder_id UUID REFERENCES document_folders (id) ON DELETE SET NULL,
    module_id UUID REFERENCES module_definitions (id) ON DELETE SET NULL,
    content_type TEXT NOT NULL,
    size_bytes INT NOT NULL DEFAULT 0,
    file_key TEXT NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_documents_program_folder_id
    ON documents (program_id, folder_id, id);
-- Manuals attached to a module (a sparse column: index only the rows that set it).
CREATE INDEX IF NOT EXISTS idx_documents_program_module
    ON documents (program_id, module_id) WHERE module_id IS NOT NULL;
