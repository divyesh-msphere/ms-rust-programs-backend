ALTER TABLE programs DROP COLUMN IF EXISTS renewal_enabled;
UPDATE module_definitions SET source = 'container', allow_components = true
WHERE key = 'configuration_oversight.settings.universal.renewal_policy';
-- The About module is restored by re-running the configuration_oversight seed.
