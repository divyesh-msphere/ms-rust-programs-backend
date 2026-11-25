-- The business unit a website belongs to, sent with the website when a store is enrolled. It is the PARENT
-- of the website in the store-discount levels: Program -> Business unit -> Website -> Store.
ALTER TABLE enrollments ADD COLUMN IF NOT EXISTS bu_unit_code text;
CREATE INDEX IF NOT EXISTS idx_enrollments_program_bu ON enrollments (program_id, bu_unit_code) WHERE bu_unit_code IS NOT NULL;
