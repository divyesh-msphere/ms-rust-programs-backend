-- Children of the `catalog_pricing` root: three groups, each with its leaves.
--
-- IDEMPOTENT, like every file here. Ids are derived from the key
-- (`md5('module:' || key)`), so every environment agrees on them; a key that
-- already exists is left exactly as it is, including edits made since.
--
-- A child's key is its full dotted path, and `ancestry` is that same string
-- (ltree labels allow only letters, digits and `_`, hence the `_` inside each
-- segment and `.` between levels).
--
-- Left out on purpose: "3PL Fees", "Rules Engine" and "Product Pricing
-- Visibility" are still open product decisions (marked `?` in the diagram).
-- Each is one more row here once decided.
--
-- ui_component:
--   rule_playground  GoRules editor (rule_family says which rule set)
--   field_form       form built from this module's module_fields
--   custom:<name>    hand-built panel
-- requires_publish: the change only reaches pricing after a Publication.
--   False for read-only / computed screens (nothing to publish).

-- ── Level 2: groups ─────────────────────────────────────────────────────────
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position, universal, requires_publish)
SELECT md5('module:' || v.key)::uuid, v.key, v.name, 'group', p.id, v.key::ltree,
       v.position, false, true
FROM (VALUES
    ('catalog_pricing.store_discounts', 'Store Discounts',  'catalog_pricing', 1),
    ('catalog_pricing.products_rules',  'Products & Rules', 'catalog_pricing', 2),
    ('catalog_pricing.pricing',         'Pricing',          'catalog_pricing', 3)
) AS v(key, name, parent_key, position)
JOIN module_definitions p ON p.key = v.parent_key
ON CONFLICT (key) DO NOTHING;

-- ── Level 3: leaves ─────────────────────────────────────────────────────────
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position,
     ui_component, rule_family, universal, requires_publish)
SELECT md5('module:' || v.key)::uuid, v.key, v.name, 'leaf', p.id, v.key::ltree,
       v.position, v.ui_component, v.rule_family, false, v.requires_publish
FROM (VALUES
    -- Store Discounts
    ('catalog_pricing.store_discounts.control_panel',          'Store Discounts Control Panel', 'catalog_pricing.store_discounts', 1, 'custom:store_discounts',          NULL,             true),

    -- Products & Rules
    ('catalog_pricing.products_rules.pre_enrollment',          'Pre-Enrollment',                'catalog_pricing.products_rules',  1, 'rule_playground',                 'pre_enrollment',  true),
    ('catalog_pricing.products_rules.post_enrollment',         'Post-Enrollment',               'catalog_pricing.products_rules',  2, 'rule_playground',                 'post_enrollment', true),
    ('catalog_pricing.products_rules.warranty',                'Warranty',                      'catalog_pricing.products_rules',  3, 'field_form',                      NULL,             true),

    -- Pricing
    ('catalog_pricing.pricing.discount_rules',                 'Discount Rules',                'catalog_pricing.pricing',         1, 'rule_playground',                 'discount',        true),
    ('catalog_pricing.pricing.markup',                         'Markup',                        'catalog_pricing.pricing',         2, 'rule_playground',                 'markup',          true),
    ('catalog_pricing.pricing.price_check',                    'Price Check',                   'catalog_pricing.pricing',         3, 'custom:price_check',              NULL,             false),
    ('catalog_pricing.pricing.price_import',                   'Price Import',                  'catalog_pricing.pricing',         4, 'custom:price_import',             NULL,             true),
    ('catalog_pricing.pricing.recent_reports',                 'Recent Reports',                'catalog_pricing.pricing',         5, 'custom:recent_reports',           NULL,             false),
    ('catalog_pricing.pricing.price_impact',                   'Price Impact (read only)',      'catalog_pricing.pricing',         6, 'custom:price_impact',             NULL,             false),
    ('catalog_pricing.pricing.frontend_pricing_visibility',    'Frontend Pricing Visibility',   'catalog_pricing.pricing',         7, 'field_form',                      NULL,             true)
) AS v(key, name, parent_key, position, ui_component, rule_family, requires_publish)
JOIN module_definitions p ON p.key = v.parent_key
ON CONFLICT (key) DO NOTHING;
