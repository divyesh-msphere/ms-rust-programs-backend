-- The icon the frontend draws beside a module in the catalog (an icon name such as `settings`, set per module).
-- Empty until it is set; nothing is seeded. The parked table mirrors the live one column for column.
ALTER TABLE module_definitions ADD COLUMN IF NOT EXISTS icon TEXT;
ALTER TABLE parked_module_definitions ADD COLUMN IF NOT EXISTS icon TEXT;
