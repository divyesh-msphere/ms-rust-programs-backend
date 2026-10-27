-- Puts the page back in the catalog; programs that had selected it do not get it back.
INSERT INTO module_definitions (id, key, name, kind, parent_id, ancestry, position, ui_component, selectable)
SELECT md5('module:' || k.key)::uuid, k.key, 'Website & Customer Group', 'leaf', p.id, k.key::ltree, 10,
       'custom:program_domains', true
FROM (SELECT 'configuration_oversight.settings.universal.website_customer_group' AS key) k
JOIN module_definitions p ON p.key = 'configuration_oversight.settings.universal'
ON CONFLICT (key) DO NOTHING;
