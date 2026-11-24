UPDATE module_definitions SET handler_key = NULL
WHERE key IN ('configuration_oversight.settings.universal.program_level',
              'configuration_oversight.settings.universal.renewal_policy');
