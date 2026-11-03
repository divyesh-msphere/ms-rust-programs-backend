-- Search on the programs list (`?q=`): a substring match on name or slug, kept fast by trigram indexes.
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX IF NOT EXISTS idx_programs_name_trgm ON programs USING gin (lower(name) gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_programs_slug_trgm ON programs USING gin (slug gin_trgm_ops);
