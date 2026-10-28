-- Children of the `configuration_oversight` root: Reports and Settings. Settings
-- splits into Universal (every program has them) and Program-Specific
-- (one-off extras that only some programs use).
--
-- IDEMPOTENT, like every file here. Ids are derived from the key
-- (`md5('module:' || key)`); a key that already exists is left exactly as it
-- is, including edits made since. A child's key is its full dotted path, and
-- `ancestry` is that same string.
--
-- The "(KBB)", "(CPO)", "(Credit Line)" ... labels in the diagram say WHICH
-- programs use a Program-Specific module. That belongs in the blueprints
-- (blueprint_modules), not in this registry, so they are not part of a name.
--
-- ui_component:
--   field_form     form built from this module's module_fields (values come
--                  from global-settings libraries or plain config)
--   custom:<name>  hand-built panel
-- Nothing here changes a price, so requires_publish stays false.

-- ── Level 2: Reports, Settings ──────────────────────────────────────────────
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position, universal, requires_publish)
SELECT md5('module:' || v.key)::uuid, v.key, v.name, 'group', p.id, v.key::ltree,
       v.position, v.universal, false
FROM (VALUES
    ('configuration_oversight.settings', 'Settings', 'configuration_oversight', 2, true)
) AS v(key, name, parent_key, position, universal)
JOIN module_definitions p ON p.key = v.parent_key
ON CONFLICT (key) DO NOTHING;

-- ── Level 3: the two kinds of settings ──────────────────────────────────────
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position, universal, requires_publish)
SELECT md5('module:' || v.key)::uuid, v.key, v.name, 'group', p.id, v.key::ltree,
       v.position, v.universal, false
FROM (VALUES
    ('configuration_oversight.settings.universal',        'Universal',                'configuration_oversight.settings', 1, true),
    ('configuration_oversight.settings.program_specific', 'Program-Specific',         'configuration_oversight.settings', 2, false)
) AS v(key, name, parent_key, position, universal)
JOIN module_definitions p ON p.key = v.parent_key
ON CONFLICT (key) DO NOTHING;

-- ── Leaves ──────────────────────────────────────────────────────────────────
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position,
     ui_component, rule_family, universal, requires_publish)
SELECT md5('module:' || v.key)::uuid, v.key, v.name, 'leaf', p.id, v.key::ltree,
       v.position, v.ui_component, NULL, v.universal, false
FROM (VALUES

    -- Settings > Universal (every program)
    ('configuration_oversight.settings.universal.program_level',              'Program Level',                   'configuration_oversight.settings.universal',       1,  'field_form',                     true),
    ('configuration_oversight.settings.universal.shopping_cart',              'Shopping Cart',                   'configuration_oversight.settings.universal',       2,  'field_form',                     true),
    ('configuration_oversight.settings.universal.renewal_policy',             'Renewal Policy',                  'configuration_oversight.settings.universal',       3,  'field_form',                     true),
    ('configuration_oversight.settings.universal.notifications',              'Notifications',                   'configuration_oversight.settings.universal',       4,  'field_form',                     true),
    ('configuration_oversight.settings.universal.order_settings',             'Order Settings',                  'configuration_oversight.settings.universal',       5,  'field_form',                     true),
    ('configuration_oversight.settings.universal.danger_zone',                'Danger Zone',                     'configuration_oversight.settings.universal',       6,  'custom:danger_zone',             true),
    ('configuration_oversight.settings.universal.program_settings',           'Program Settings',                'configuration_oversight.settings.universal',       7,  'field_form',                     true),
    ('configuration_oversight.settings.universal.vault_payment_methods',      'Vault Payment Methods',           'configuration_oversight.settings.universal',       8,  'field_form',                     true),
    ('configuration_oversight.settings.universal.about_program',              'About this Program',              'configuration_oversight.settings.universal',       9,  'field_form',                     true),

    -- Settings > Program-Specific (one-off extras)
    ('configuration_oversight.settings.program_specific.billing',                'Billing',                         'configuration_oversight.settings.program_specific', 1,  'custom:billing',                  false),
    ('configuration_oversight.settings.program_specific.shipping_label_config',  'Shipping Label Config',           'configuration_oversight.settings.program_specific', 2,  'field_form',                     false),
    ('configuration_oversight.settings.program_specific.enrollment_settings',    'Enrollment Settings',             'configuration_oversight.settings.program_specific', 3,  'field_form',                     false),
    ('configuration_oversight.settings.program_specific.agreement_manager',      'Agreement Manager',               'configuration_oversight.settings.program_specific', 4,  'custom:agreement_manager',        false),
    ('configuration_oversight.settings.program_specific.device_grade_mapping',   'Device Grade Mapping',            'configuration_oversight.settings.program_specific', 5,  'custom:device_grade_mapping',     false),
    ('configuration_oversight.settings.program_specific.insurance',              'Insurance',                       'configuration_oversight.settings.program_specific', 6,  'field_form',                     false),
    ('configuration_oversight.settings.program_specific.enrollment_form',        'Enrollment Form',                 'configuration_oversight.settings.program_specific', 7,  'custom:enrollment_form',         false),
    ('configuration_oversight.settings.program_specific.stock_picking',          'Stock Picking',                   'configuration_oversight.settings.program_specific', 8,  'field_form',                     false),
    ('configuration_oversight.settings.program_specific.manuals',                'Manuals',                         'configuration_oversight.settings.program_specific', 9,  'custom:manuals',                 false),
    ('configuration_oversight.settings.program_specific.device_upsell',          'Device Upsell',                   'configuration_oversight.settings.program_specific', 10, 'field_form',                     false),
    ('configuration_oversight.settings.program_specific.upsell_products',        'Upsell Products',                 'configuration_oversight.settings.program_specific', 11, 'custom:upsell_products',         false),
    ('configuration_oversight.settings.program_specific.payment_configuration',  'Payment Configuration',           'configuration_oversight.settings.program_specific', 12, 'field_form',                     false),
    ('configuration_oversight.settings.program_specific.credit_configuration',   'Credit Configuration',            'configuration_oversight.settings.program_specific', 13, 'field_form',                     false),
    ('configuration_oversight.settings.program_specific.automated_payments',     'Automated Payments',              'configuration_oversight.settings.program_specific', 14, 'field_form',                     false),
    ('configuration_oversight.settings.program_specific.net_term_configuration',  'Net Term Configuration',          'configuration_oversight.settings.program_specific', 15, 'field_form',                     false),
    ('configuration_oversight.settings.program_specific.uploaded_manuals',       'Uploaded Manuals',                'configuration_oversight.settings.program_specific', 16, 'custom:uploaded_manuals',        false)
) AS v(key, name, parent_key, position, ui_component, universal)
JOIN module_definitions p ON p.key = v.parent_key
ON CONFLICT (key) DO NOTHING;
