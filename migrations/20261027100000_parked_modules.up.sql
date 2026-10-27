-- Holding tables for module pages that are switched off (`[features]` in config.toml).
--
-- When a feature is off, the service moves that feature's rows out of `module_definitions` and
-- `program_modules` into these two tables at start-up, so the page disappears from the module
-- tree, from every program's manifest and from module selection, with no change to any query.
-- Turning the feature on moves the rows back, exactly as they were. The feature's own tables
-- (requirements, approval_steps, applications, signoffs) are never touched.
--
-- Same columns as the live tables, but no constraints or indexes: nothing reads these tables.
CREATE TABLE IF NOT EXISTS parked_module_definitions (LIKE module_definitions INCLUDING DEFAULTS);
CREATE TABLE IF NOT EXISTS parked_program_modules (LIKE program_modules INCLUDING DEFAULTS);
