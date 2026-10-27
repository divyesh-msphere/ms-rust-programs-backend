-- Settings > Universal no longer has a "Website & Customer Group" page. IDEMPOTENT.
-- Programs that had selected it lose that selection (its settings and rules go with it, by cascade).
DELETE FROM program_modules
WHERE module_id IN (SELECT id FROM module_definitions
                    WHERE key = 'configuration_oversight.settings.universal.website_customer_group');
DELETE FROM module_definitions
WHERE key = 'configuration_oversight.settings.universal.website_customer_group';
