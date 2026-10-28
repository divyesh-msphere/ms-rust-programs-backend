-- The catalog no longer has: Products & Rules > Live Results, the whole Reports group
-- (Enrollment Reports Panel) and the whole Imports group (Import History, Upload Consignment Orders).
-- IDEMPOTENT. A program that had selected any of them loses that selection; what hangs off the page
-- (settings, rules) goes with it, by cascade.
DELETE FROM program_modules
WHERE module_id IN (SELECT id FROM module_definitions
                    WHERE key = 'catalog_pricing.products_rules.live_results'
                       OR key = 'configuration_oversight.reports' OR key LIKE 'configuration\_oversight.reports.%'
                       OR key = 'order_transaction.imports' OR key LIKE 'order\_transaction.imports.%');
-- Pages first, then their groups (parent_id).
DELETE FROM module_definitions
WHERE key IN ('catalog_pricing.products_rules.live_results',
              'configuration_oversight.reports.enrollment_reports_panel',
              'order_transaction.imports.import_history',
              'order_transaction.imports.upload_consignment_orders');
DELETE FROM module_definitions WHERE key IN ('configuration_oversight.reports', 'order_transaction.imports');
