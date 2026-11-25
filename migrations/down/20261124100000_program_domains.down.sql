DELETE FROM program_modules
WHERE module_id IN (SELECT id FROM module_definitions WHERE key = 'configuration_oversight.settings.universal.domains');
DELETE FROM module_definitions WHERE key = 'configuration_oversight.settings.universal.domains';
ALTER TABLE programs DROP COLUMN IF EXISTS domain;
