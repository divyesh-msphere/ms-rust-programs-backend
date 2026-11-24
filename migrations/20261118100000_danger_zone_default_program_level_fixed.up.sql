-- Danger Zone is a DEFAULT module (like Overview and Program Level); Program Level is not selectable.
UPDATE module_definitions SET source = 'default'
WHERE key = 'configuration_oversight.settings.universal.danger_zone';
UPDATE module_definitions SET selectable = false
WHERE key = 'configuration_oversight.settings.universal.program_level';

-- Every program gets every default module (and the groups above them), switched on.
INSERT INTO program_modules (program_id, module_id, enabled, position, mode)
SELECT p.id, md.id, md.source = 'default', md.position, md.default_mode
FROM programs p
JOIN module_definitions md ON EXISTS (SELECT 1 FROM module_definitions d
                                      WHERE d.source = 'default' AND d.ancestry <@ md.ancestry)
ON CONFLICT (program_id, module_id) DO NOTHING;

UPDATE program_modules SET enabled = true
WHERE enabled = false
  AND module_id IN (SELECT id FROM module_definitions WHERE source = 'default');
