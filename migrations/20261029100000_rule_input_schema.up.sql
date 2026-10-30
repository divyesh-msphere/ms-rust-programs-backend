-- Execute rule: every rule version can declare the INPUT it needs (a list of fields: name, type,
-- required). Execute checks the caller's input against it and, when it does not fit, answers with the
-- required field list. A draft may lack it; Publish refuses a rule that has none. IDEMPOTENT.
ALTER TABLE rule_versions ADD COLUMN IF NOT EXISTS input_schema JSONB;

-- A starter rule (default_rule) comes with the input it reads, so a new program can publish it as it is.
ALTER TABLE module_definitions ADD COLUMN IF NOT EXISTS default_input_schema JSONB;
-- The parked copy keeps the same columns in the same order (the feature switch moves whole rows with SELECT *).
ALTER TABLE parked_module_definitions ADD COLUMN IF NOT EXISTS default_input_schema JSONB;
UPDATE module_definitions
SET default_input_schema = '[{"name":"condition","type":"string","required":false,"description":"The return condition the fee depends on"}]'::jsonb
WHERE key = 'order_transaction.core_returns.fees';

-- Starter rules already seeded get it too.
UPDATE rule_versions rv
SET input_schema = md.default_input_schema,
    content_hash = md5(rv.definition::text || md.default_input_schema::text)
FROM rule_sets rs
JOIN module_definitions md ON md.id = rs.module_id
WHERE rv.rule_set_id = rs.id AND md.default_input_schema IS NOT NULL
  AND rv.input_schema IS NULL AND rv.definition = md.default_rule;
