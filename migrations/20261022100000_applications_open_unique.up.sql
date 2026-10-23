-- A store can have only ONE open application per program and website: draft, submitted or
-- in_review. A decided or withdrawn one does not count, so a store can apply again. The
-- database enforces it, so two requests at once cannot both win and nothing has to lock the
-- program to check.
CREATE UNIQUE INDEX IF NOT EXISTS uq_applications_open
    ON applications (program_id, store_id, website_id)
    WHERE status IN ('draft', 'submitted', 'in_review');
