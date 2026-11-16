-- `display` belongs on a setting and on a rule, not on the module itself: drop the module-level switch
-- (added in 20261105100000) and give the rule set its own. `icon` stays on the module.
ALTER TABLE program_modules DROP COLUMN IF EXISTS display;
ALTER TABLE parked_program_modules DROP COLUMN IF EXISTS display;
ALTER TABLE rule_sets ADD COLUMN IF NOT EXISTS display BOOLEAN NOT NULL DEFAULT true;
