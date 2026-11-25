DROP INDEX IF EXISTS idx_enrollments_program_bu;
ALTER TABLE enrollments DROP COLUMN IF EXISTS bu_unit_code;
