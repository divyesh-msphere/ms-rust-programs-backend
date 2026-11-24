-- Danger Zone is always on (it is not selectable): switch it on for every program that has it off.
UPDATE program_modules SET enabled = true
WHERE enabled = false
  AND module_id IN (SELECT id FROM module_definitions WHERE handler_key = 'danger_zone');
