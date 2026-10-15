-- Children of the `enrollment` root: Requirements, Stores, Applications & Approvals.
--
-- IDEMPOTENT, like every file here. Ids are derived from the key
-- (`md5('module:' || key)`); a key that already exists is left exactly as it
-- is, including edits made since. A child's key is its full dotted path, and
-- `ancestry` is that same string.
--
-- All three leaves are hand-built panels (`custom:<name>`): they list and
-- manage records (requirements, enrolled stores, applications), not a
-- settings form. Nothing here changes a price, so requires_publish stays false.

-- ── Level 2: groups ─────────────────────────────────────────────────────────
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position, universal, requires_publish)
SELECT md5('module:' || v.key)::uuid, v.key, v.name, 'group', p.id, v.key::ltree,
       v.position, false, false
FROM (VALUES
    ('enrollment.requirements',           'Requirements',             'enrollment', 1),
    ('enrollment.stores',                 'Stores',                   'enrollment', 2),
    ('enrollment.applications_approvals', 'Applications & Approvals', 'enrollment', 3)
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
    ('enrollment.requirements.enrollment_requirements',          'Enrollment Requirements',       'enrollment.requirements',           1, 'custom:enrollment_requirements'),
    ('enrollment.stores.enrolled_stores_control_panel',          'Enrolled Stores Control Panel', 'enrollment.stores',                 1, 'custom:enrolled_stores'),
    ('enrollment.applications_approvals.application_panel',      'Application Panel',             'enrollment.applications_approvals', 1, 'custom:applications')
) AS v(key, name, parent_key, position, ui_component)
JOIN module_definitions p ON p.key = v.parent_key
ON CONFLICT (key) DO NOTHING;
