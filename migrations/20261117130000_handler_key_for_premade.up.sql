-- Every premade / default module carries a handler_key. These two had none (no API of their own,
-- so api_base stays NULL).
UPDATE module_definitions SET handler_key = 'program_level'
WHERE key = 'configuration_oversight.settings.universal.program_level' AND handler_key IS NULL;
UPDATE module_definitions SET handler_key = 'renewal_policy'
WHERE key = 'configuration_oversight.settings.universal.renewal_policy' AND handler_key IS NULL;
