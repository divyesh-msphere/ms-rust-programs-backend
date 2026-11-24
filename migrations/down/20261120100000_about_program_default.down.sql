DELETE FROM program_modules
WHERE module_id IN (SELECT id FROM module_definitions WHERE key = 'configuration_oversight.settings.universal.about_program');
DELETE FROM module_definitions WHERE key = 'configuration_oversight.settings.universal.about_program';
