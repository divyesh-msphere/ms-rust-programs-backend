ALTER TABLE enrollments DROP COLUMN IF EXISTS website_domain;
UPDATE module_definitions SET api_base = 'enrollments' WHERE handler_key = 'enrolled_stores';
