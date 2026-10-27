-- Fees goes back to the Core Return Fees Panel, and pages lose their default mode and starter rule.
-- Rule sets already saved on Fees stay (they belong to the programs).
UPDATE module_definitions
SET key = 'order_transaction.core_returns.core_return_fees_panel', name = 'Core Return Fees Panel',
    ancestry = 'order_transaction.core_returns.core_return_fees_panel'::ltree,
    ui_component = 'custom:core_return_fees', rule_family = NULL
WHERE key = 'order_transaction.core_returns.fees';
ALTER TABLE parked_module_definitions DROP COLUMN IF EXISTS default_rule, DROP COLUMN IF EXISTS default_mode;
ALTER TABLE module_definitions DROP COLUMN IF EXISTS default_rule, DROP COLUMN IF EXISTS default_mode;
