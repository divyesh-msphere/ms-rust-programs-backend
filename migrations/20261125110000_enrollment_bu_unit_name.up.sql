-- The business unit of an enrolled store is a {code, name} pair (`bu` in the API); the code was added before.
ALTER TABLE enrollments ADD COLUMN IF NOT EXISTS bu_unit_name text;
