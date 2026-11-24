-- Every default module (source = 'default') is on for every program, like Danger Zone.
UPDATE program_modules SET enabled = true
WHERE enabled = false
  AND module_id IN (SELECT id FROM module_definitions WHERE source = 'default');
