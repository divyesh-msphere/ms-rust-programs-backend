-- Back to an ordinary container page that a program selects; archived programs become unarchived.
DELETE FROM program_modules
WHERE module_id IN (SELECT id FROM module_definitions WHERE key = 'configuration_oversight.settings.universal.danger_zone');
UPDATE module_definitions
SET source = 'container', handler_key = NULL, api_base = NULL, selectable = true, allow_components = true
WHERE key = 'configuration_oversight.settings.universal.danger_zone';
ALTER TABLE programs DROP COLUMN IF EXISTS archive_time;
