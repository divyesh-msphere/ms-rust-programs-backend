-- "About this Program" is back, as a DEFAULT module (always on, not selectable), like Program Level.
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position, ui_component, universal, requires_publish,
     source, handler_key, selectable, allow_components)
SELECT md5('module:configuration_oversight.settings.universal.about_program')::uuid,
       'configuration_oversight.settings.universal.about_program', 'About this Program', 'leaf', p.id,
       'configuration_oversight.settings.universal.about_program'::ltree, 9, 'field_form', true, false,
       'default', 'about_program', false, false
FROM module_definitions p
WHERE p.key = 'configuration_oversight.settings.universal'
ON CONFLICT (key) DO UPDATE
SET source = 'default', handler_key = 'about_program', selectable = false, allow_components = false;

-- Every program gets it, switched on.
INSERT INTO program_modules (program_id, module_id, enabled, position, mode)
SELECT p.id, md.id, true, md.position, md.default_mode
FROM programs p
JOIN module_definitions md ON md.key = 'configuration_oversight.settings.universal.about_program'
ON CONFLICT (program_id, module_id) DO UPDATE SET enabled = true;
