-- Children of the `document_agreement` root: Documents and Agreements.
--
-- IDEMPOTENT, like every file here. Ids are derived from the key
-- (`md5('module:' || key)`); a key that already exists is left exactly as it
-- is, including edits made since. A child's key is its full dotted path, and
-- `ancestry` is that same string.
--
-- Note: "Agreement Manager (GAPP)" is NOT here — the diagram files it under
-- Settings > Program-Specific, where it is already seeded.
--
-- ui_component:
--   field_form     form built from this module's module_fields
--   custom:<name>  hand-built panel
-- Nothing here changes a price, so requires_publish stays false.

-- ── Level 2: groups ─────────────────────────────────────────────────────────
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position, universal, requires_publish)
SELECT md5('module:' || v.key)::uuid, v.key, v.name, 'group', p.id, v.key::ltree,
       v.position, false, false
FROM (VALUES
    ('document_agreement.documents',  'Documents',  'document_agreement', 1),
    ('document_agreement.agreements', 'Agreements', 'document_agreement', 2)
) AS v(key, name, parent_key, position)
JOIN module_definitions p ON p.key = v.parent_key
ON CONFLICT (key) DO NOTHING;

-- ── Level 3: leaves ─────────────────────────────────────────────────────────
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position,
     ui_component, rule_family, universal, requires_publish)
SELECT md5('module:' || v.key)::uuid, v.key, v.name, 'leaf', p.id, v.key::ltree,
       v.position, v.ui_component, NULL, false, false
FROM (VALUES
    -- Documents
    ('document_agreement.documents.documents_panel',         'Documents Panel',      'document_agreement.documents',  1, 'custom:documents'),

    -- Agreements
    ('document_agreement.agreements.enrollment_agreements',  'Enrollment Agreements','document_agreement.agreements', 1, 'custom:enrollment_agreements'),
    ('document_agreement.agreements.agreement_settings',     'Agreement Settings',   'document_agreement.agreements', 2, 'field_form'),
    ('document_agreement.agreements.completion_email',       'Completion E-mail',    'document_agreement.agreements', 3, 'field_form')
) AS v(key, name, parent_key, position, ui_component)
JOIN module_definitions p ON p.key = v.parent_key
ON CONFLICT (key) DO NOTHING;
