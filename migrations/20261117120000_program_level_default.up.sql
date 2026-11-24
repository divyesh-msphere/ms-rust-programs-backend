-- Program Level: a DEFAULT module (source premade -> default), like Overview.
UPDATE module_definitions
SET source = 'default'
WHERE key = 'configuration_oversight.settings.universal.program_level';
