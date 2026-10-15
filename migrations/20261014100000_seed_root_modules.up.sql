-- Root (main) modules of the program module tree. More levels are seeded by
-- later files in this directory, each looking its parent up BY KEY, so adding
-- a module is one more INSERT and never a code change.
--
-- IDEMPOTENT: replayed on every run, like every file here. Ids are derived
-- from the key (`md5('module:' || key)` cast to a uuid), so the same module has
-- the same id in every environment and a re-run changes nothing. A key that
-- already exists is left exactly as it is, including edits made since.
--
-- `ancestry` is the key itself for a root; a child's is `<parent ancestry>.<key
-- tail>`. ltree labels allow only letters, digits and `_`, which is why keys
-- use `_` and the tree separator is `.`.
INSERT INTO module_definitions
    (id, key, name, kind, parent_id, ancestry, position, universal, requires_publish)
VALUES
    (md5('module:catalog_pricing')::uuid,         'catalog_pricing',         'Catalog & Pricing',         'group', NULL, 'catalog_pricing',         1, false, true),
    (md5('module:enrollment')::uuid,              'enrollment',              'Enrollment',                'group', NULL, 'enrollment',              2, false, false),
    (md5('module:document_agreement')::uuid,      'document_agreement',      'Document & Agreements',     'group', NULL, 'document_agreement',      3, false, false),
    (md5('module:order_transaction')::uuid,       'order_transaction',       'Order & Transaction',       'group', NULL, 'order_transaction',       4, false, false),
    (md5('module:configuration_oversight')::uuid, 'configuration_oversight', 'Configuration & Oversight', 'group', NULL, 'configuration_oversight', 5, true,  false)
ON CONFLICT (key) DO NOTHING;
