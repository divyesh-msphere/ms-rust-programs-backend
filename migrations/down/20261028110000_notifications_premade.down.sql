-- Back to an ordinary container page; the trigger grids are dropped.
UPDATE module_definitions
SET source = 'container', handler_key = NULL, api_base = NULL, ui_component = 'field_form', allow_components = true
WHERE key = 'configuration_oversight.settings.universal.notifications';
DROP TABLE IF EXISTS program_notification_triggers;
