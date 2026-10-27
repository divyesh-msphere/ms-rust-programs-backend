-- Every program has the four system folders and one storage-counters row. New programs get them when they are
-- created; this gives the same to the programs that already existed. Idempotent.
INSERT INTO program_storage (program_id)
SELECT p.id FROM programs p
ON CONFLICT (program_id) DO NOTHING;

INSERT INTO document_folders (id, program_id, name, key, system)
SELECT gen_random_uuid(), p.id, f.name, f.key, true
FROM programs p
CROSS JOIN (VALUES
    ('general',                    'General'),
    ('import_artifacts',           'Import artifacts'),
    ('program_documents',          'Program Documents'),
    ('store_enrollment_documents', 'Store Enrollment Documents')
) AS f (key, name)
ON CONFLICT (program_id, key) WHERE key IS NOT NULL DO NOTHING;
