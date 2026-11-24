UPDATE module_definitions SET source = 'premade'
WHERE key = 'configuration_oversight.settings.universal.danger_zone';
UPDATE module_definitions SET selectable = true
WHERE key = 'configuration_oversight.settings.universal.program_level';
