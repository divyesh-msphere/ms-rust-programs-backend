-- The "About this Program" module is gone: the frontend sets the about text straight from the program.
-- program_modules.module_id is ON DELETE RESTRICT, so the program rows go first. Rules/fields hang off
-- program_modules and cascade.
DELETE FROM program_modules
WHERE module_id IN (SELECT id FROM module_definitions WHERE key = 'configuration_oversight.settings.universal.about_program');
DELETE FROM module_definitions WHERE key = 'configuration_oversight.settings.universal.about_program';
DELETE FROM parked_module_definitions WHERE key = 'configuration_oversight.settings.universal.about_program';

-- Renewal Policy: a PREMADE page (source flips from container to premade), nothing to configure.
UPDATE module_definitions
SET source = 'premade', allow_components = false
WHERE key = 'configuration_oversight.settings.universal.renewal_policy';

-- The renewal flag on a program: false until the program turns it on.
ALTER TABLE programs ADD COLUMN IF NOT EXISTS renewal_enabled BOOLEAN NOT NULL DEFAULT false;

-- Make sure no program carries the old 12 / 30 / 30 defaults it never chose (null = unset).
-- Only rows still holding exactly the old defaults with auto-renew off are cleared.
UPDATE programs SET renewal_term_months = NULL, renewal_grace_days = NULL, renewal_reminder_days = NULL
WHERE renewal_term_months = 12 AND renewal_grace_days = 30 AND renewal_reminder_days = 30 AND NOT auto_renew;
