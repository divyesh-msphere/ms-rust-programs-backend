-- A program's own list of domains, stored on the program (the Domains default module edits it).
ALTER TABLE programs ADD COLUMN IF NOT EXISTS domain text[] NOT NULL DEFAULT '{}';

-- "Domains" is a DEFAULT module under Universal Settings (always on, not selectable), like About.
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position, ui_component, universal, requires_publish,
     source, handler_key, selectable, allow_components)
SELECT md5('module:configuration_oversight.settings.universal.domains')::uuid,
       'configuration_oversight.settings.universal.domains', 'Domains', 'leaf', p.id,
       'configuration_oversight.settings.universal.domains'::ltree, 10, 'field_form', true, false,
       'default', 'domains', false, false
FROM module_definitions p
WHERE p.key = 'configuration_oversight.settings.universal'
ON CONFLICT (key) DO UPDATE
SET source = 'default', handler_key = 'domains', selectable = false, allow_components = false;

-- Every program gets it, switched on.
INSERT INTO program_modules (program_id, module_id, enabled, position, mode)
SELECT p.id, md.id, true, md.position, md.default_mode
FROM programs p
JOIN module_definitions md ON md.key = 'configuration_oversight.settings.universal.domains'
ON CONFLICT (program_id, module_id) DO UPDATE SET enabled = true;
