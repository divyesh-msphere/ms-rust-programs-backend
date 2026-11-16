ALTER TABLE parked_module_definitions DROP COLUMN IF EXISTS module_in;
ALTER TABLE module_definitions DROP CONSTRAINT IF EXISTS module_definitions_module_in_check;
ALTER TABLE module_definitions DROP COLUMN IF EXISTS module_in;
