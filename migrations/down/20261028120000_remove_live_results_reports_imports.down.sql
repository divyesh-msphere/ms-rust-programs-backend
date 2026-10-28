-- Puts the groups and pages back in the catalog; programs that had selected them do not get them back.
-- Groups first, then their pages (a page joins to its parent, which must already exist).
INSERT INTO module_definitions (id, key, name, kind, parent_id, ancestry, position, ui_component, selectable)
SELECT md5('module:' || v.key)::uuid, v.key, v.name, v.kind, p.id, v.key::ltree, v.position, v.ui, v.kind = 'leaf'
FROM (VALUES
    ('configuration_oversight.reports', 'Reports', 'group', 'configuration_oversight', 1, NULL),
    ('order_transaction.imports', 'Imports', 'group', 'order_transaction', 1, NULL),
    ('configuration_oversight.reports.enrollment_reports_panel', 'Enrollment Reports Panel', 'leaf', 'configuration_oversight.reports', 1, 'custom:enrollment_reports'),
    ('order_transaction.imports.import_history', 'Import History', 'leaf', 'order_transaction.imports', 1, 'custom:import_history'),
    ('order_transaction.imports.upload_consignment_orders', 'Upload Consignment Orders', 'leaf', 'order_transaction.imports', 2, 'custom:upload_consignment_orders'),
    ('catalog_pricing.products_rules.live_results', 'Live Results', 'leaf', 'catalog_pricing.products_rules', 4, 'custom:live_results')
) AS v(key, name, kind, parent_key, position, ui)
JOIN module_definitions p ON p.key = v.parent_key
WHERE v.kind = 'group'
ON CONFLICT (key) DO NOTHING;
INSERT INTO module_definitions (id, key, name, kind, parent_id, ancestry, position, ui_component, selectable)
SELECT md5('module:' || v.key)::uuid, v.key, v.name, v.kind, p.id, v.key::ltree, v.position, v.ui, v.kind = 'leaf'
FROM (VALUES
    ('configuration_oversight.reports', 'Reports', 'group', 'configuration_oversight', 1, NULL),
    ('order_transaction.imports', 'Imports', 'group', 'order_transaction', 1, NULL),
    ('configuration_oversight.reports.enrollment_reports_panel', 'Enrollment Reports Panel', 'leaf', 'configuration_oversight.reports', 1, 'custom:enrollment_reports'),
    ('order_transaction.imports.import_history', 'Import History', 'leaf', 'order_transaction.imports', 1, 'custom:import_history'),
    ('order_transaction.imports.upload_consignment_orders', 'Upload Consignment Orders', 'leaf', 'order_transaction.imports', 2, 'custom:upload_consignment_orders'),
    ('catalog_pricing.products_rules.live_results', 'Live Results', 'leaf', 'catalog_pricing.products_rules', 4, 'custom:live_results')
) AS v(key, name, kind, parent_key, position, ui)
JOIN module_definitions p ON p.key = v.parent_key
WHERE v.kind = 'leaf'
ON CONFLICT (key) DO NOTHING;
