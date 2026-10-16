DELETE FROM module_definitions WHERE key = 'overview';
UPDATE module_definitions SET source = 'container', handler_key = NULL, api_base = NULL, allow_components = false;
