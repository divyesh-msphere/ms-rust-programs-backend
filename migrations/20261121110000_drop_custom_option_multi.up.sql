-- custom_option_multi is gone: it is a dropdown with multiple = true.
UPDATE program_fields SET field_type = 'dropdown', multiple = true WHERE field_type = 'custom_option_multi';
