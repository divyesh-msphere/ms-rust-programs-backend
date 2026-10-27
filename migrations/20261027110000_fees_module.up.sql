-- Fees: a page in Core Returns that is a RULE page from the moment a program selects it, with a
-- simple starter rule already saved as draft 1. It takes the place of the Core Return Fees Panel.
--
-- IDEMPOTENT, like every file here: edits made since are never overwritten.

-- A page may now say how it starts. `default_mode` is `rule` or `settings`: a program that selects
-- the page gets that mode without the admin choosing it. `default_rule` is the starter rule (the
-- decision graph the rule editor draws); a rule page that has `default_rule` and nothing saved gets
-- it as draft 1. Both are NULL for every other page. The holding table for parked pages
-- (20261027100000) is moved with `SELECT *`, so it must have the same columns in the same order.
ALTER TABLE module_definitions
    ADD COLUMN IF NOT EXISTS default_mode TEXT CHECK (default_mode IN ('rule', 'settings')),
    ADD COLUMN IF NOT EXISTS default_rule JSONB;
ALTER TABLE parked_module_definitions
    ADD COLUMN IF NOT EXISTS default_mode TEXT,
    ADD COLUMN IF NOT EXISTS default_rule JSONB;

-- A database seeded before this change still has the old page: it becomes Fees (same id, same place).
UPDATE module_definitions
SET key = 'order_transaction.core_returns.fees', name = 'Fees',
    ancestry = 'order_transaction.core_returns.fees'::ltree, ui_component = 'rule_playground'
WHERE key = 'order_transaction.core_returns.core_return_fees_panel'
  AND NOT EXISTS (SELECT 1 FROM module_definitions WHERE key = 'order_transaction.core_returns.fees');

-- The starter rule: a GoRules decision graph (request -> decision table -> response) with ONE row that
-- matches every return and charges nothing. The admin edits the table in the rule editor.
UPDATE module_definitions
SET rule_family = 'fees', default_mode = 'rule', default_rule = $rule${
  "contentType": "application/vnd.gorules.decision",
  "nodes": [
    {"id": "fees-request", "type": "inputNode", "name": "Request", "position": {"x": 120, "y": 200}},
    {"id": "fees-table", "type": "decisionTableNode", "name": "Fees", "position": {"x": 440, "y": 200},
     "content": {
       "hitPolicy": "first",
       "inputs": [{"id": "fees-in-condition", "name": "Return condition", "field": "condition"}],
       "outputs": [{"id": "fees-out-fee", "name": "Fee", "field": "fee"}],
       "rules": [{"_id": "fees-rule-1", "fees-in-condition": "", "fees-out-fee": "0"}]
     }},
    {"id": "fees-response", "type": "outputNode", "name": "Response", "position": {"x": 760, "y": 200}}
  ],
  "edges": [
    {"id": "fees-edge-1", "type": "edge", "sourceId": "fees-request", "targetId": "fees-table"},
    {"id": "fees-edge-2", "type": "edge", "sourceId": "fees-table", "targetId": "fees-response"}
  ]
}$rule$::jsonb
WHERE key = 'order_transaction.core_returns.fees' AND default_mode IS NULL;
