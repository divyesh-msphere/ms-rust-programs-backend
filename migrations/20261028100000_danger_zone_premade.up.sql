-- Danger zone (Settings > Universal): a PREMADE page with its own API (`danger-zone`: archive,
-- unarchive, delete permanently), present on EVERY program without anyone selecting it.
-- IDEMPOTENT, like every file here.

-- Archive keeps the program and all its data but takes it out of the working catalog (the
-- program list). NULL = not archived. `state` is untouched: ACTIVE / DELETED is the soft delete.
ALTER TABLE programs ADD COLUMN IF NOT EXISTS archive_time TIMESTAMPTZ;

-- The page: code-backed (handler + api_base), nothing to configure, always there.
UPDATE module_definitions
SET source = 'premade', handler_key = 'danger_zone', api_base = 'danger-zone',
    ui_component = 'custom:danger_zone', universal = true, selectable = false, allow_components = false
WHERE key = 'configuration_oversight.settings.universal.danger_zone';

-- Programs that already exist get it too, with the groups above it (Settings, Universal, ...).
-- New programs get it from `ensure_modules` (every premade page marked `universal`).
INSERT INTO program_modules (program_id, module_id, enabled, position, mode)
SELECT p.id, md.id, true, md.position, md.default_mode
FROM programs p
JOIN module_definitions page ON page.key = 'configuration_oversight.settings.universal.danger_zone'
JOIN module_definitions md ON page.ancestry <@ md.ancestry
WHERE p.state = 'ACTIVE'
ON CONFLICT (program_id, module_id) DO NOTHING;
