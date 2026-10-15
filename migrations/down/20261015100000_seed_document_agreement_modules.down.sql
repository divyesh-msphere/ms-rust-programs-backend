-- Leaves first, then the groups, matching the foreign key (ON DELETE RESTRICT).
DELETE FROM module_definitions WHERE ancestry <@ 'document_agreement'::ltree AND nlevel(ancestry) = 3;
DELETE FROM module_definitions WHERE ancestry <@ 'document_agreement'::ltree AND nlevel(ancestry) = 2;
