-- Classifies the seeded modules: the default Overview, the premade modules and
-- (everything else) containers. Runs after every module seed, because it looks
-- modules up BY KEY.
--
-- IDEMPOTENT: the UPDATEs only touch rows that have not been classified yet
-- (`handler_key IS NULL`), so edits made since are never overwritten.

-- ── Overview: the default module, on every program ──────────────────────────
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position,
     ui_component, universal, requires_publish, source, handler_key, api_base, selectable, allow_components)
VALUES
    (md5('module:overview')::uuid, 'overview', 'Overview', 'leaf', NULL, 'overview', 0,
     'custom:overview', true, false, 'default', 'overview', 'overview', false, false)
ON CONFLICT (key) DO NOTHING;

-- ── Premade modules (code-backed; the admin selects them) ───────────────────
UPDATE module_definitions m
SET source = 'premade', handler_key = v.handler_key, api_base = v.api_base, allow_components = true
FROM (VALUES
    ('enrollment.applications_approvals.application_panel',  'applications',     'applications'),
    ('enrollment.stores.enrolled_stores_control_panel',      'enrolled_stores',  'enrollments'),
    ('catalog_pricing.store_discounts.control_panel',        'store_discounts',  'store-discounts'),
    ('enrollment.requirements.enrollment_requirements',      'requirements',     'requirements'),
    ('document_agreement.documents.documents_panel',         'documents',        'documents')
) AS v(key, handler_key, api_base)
WHERE m.key = v.key AND m.handler_key IS NULL;

-- ── Every other leaf is a container: the admin chooses Rule or Settings for it ──
UPDATE module_definitions
SET allow_components = true
WHERE kind = 'leaf' AND source = 'container' AND handler_key IS NULL AND NOT allow_components;
