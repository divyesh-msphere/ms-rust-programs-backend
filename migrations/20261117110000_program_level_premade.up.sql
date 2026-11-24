-- Program Level: a PREMADE page (source flips from container to premade), nothing to configure.
-- Same shape as Renewal Policy.
UPDATE module_definitions
SET source = 'premade', allow_components = false
WHERE key = 'configuration_oversight.settings.universal.program_level';
