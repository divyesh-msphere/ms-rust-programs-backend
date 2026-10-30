ALTER TABLE parked_module_definitions DROP COLUMN IF EXISTS default_input_schema;
ALTER TABLE module_definitions DROP COLUMN IF EXISTS default_input_schema;
ALTER TABLE rule_versions DROP COLUMN IF EXISTS input_schema;
