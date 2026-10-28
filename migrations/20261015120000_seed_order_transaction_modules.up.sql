-- Children of the `order_transaction` root: Imports and Core Returns.
--
-- IDEMPOTENT, like every file here. Ids are derived from the key
-- (`md5('module:' || key)`); a key that already exists is left exactly as it
-- is, including edits made since. A child's key is its full dotted path, and
-- `ancestry` is that same string.
--
-- The Operate > Imports page shows "Upload Consignment Orders" and "Import
-- history" together; they are two modules (two leaves) that one page renders.
-- Both are backed by import_jobs / import_job_rows (Price Import uses the same).
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
    ('order_transaction.core_returns', 'Core Returns', 'order_transaction', 2)
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

    -- Core Returns
    ('order_transaction.core_returns.terms',                   'Terms',                       'order_transaction.core_returns', 1, 'field_form'),
    ('order_transaction.core_returns.attention',               'Attention',                   'order_transaction.core_returns', 2, 'custom:attention'),
    -- Fees: a rule leaf from the start (see 20261027110000_fees_module: rule_family, default mode and starter rule)
    ('order_transaction.core_returns.fees',                    'Fees',                        'order_transaction.core_returns', 3, 'rule_playground')
) AS v(key, name, parent_key, position, ui_component)
JOIN module_definitions p ON p.key = v.parent_key
ON CONFLICT (key) DO NOTHING;
