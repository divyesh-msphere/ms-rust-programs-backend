-- Leaves first, then the groups, matching the foreign key (ON DELETE RESTRICT).
DELETE FROM module_definitions WHERE ancestry <@ 'catalog_pricing'::ltree AND kind = 'leaf';
DELETE FROM module_definitions WHERE ancestry <@ 'catalog_pricing'::ltree AND key <> 'catalog_pricing';
