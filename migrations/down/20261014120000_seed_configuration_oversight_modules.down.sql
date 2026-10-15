-- Everything below the root, deepest first so the foreign key
-- (ON DELETE RESTRICT) is never violated.
DELETE FROM module_definitions WHERE ancestry <@ 'configuration_oversight'::ltree AND nlevel(ancestry) = 4;
DELETE FROM module_definitions WHERE ancestry <@ 'configuration_oversight'::ltree AND nlevel(ancestry) = 3;
DELETE FROM module_definitions WHERE ancestry <@ 'configuration_oversight'::ltree AND nlevel(ancestry) = 2;
