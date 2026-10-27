-- Put any parked rows back first, or they are lost with the tables.
INSERT INTO module_definitions SELECT * FROM parked_module_definitions ORDER BY nlevel(ancestry) ON CONFLICT (key) DO NOTHING;
INSERT INTO program_modules SELECT * FROM parked_program_modules ON CONFLICT (program_id, module_id) DO NOTHING;
DROP TABLE IF EXISTS parked_program_modules;
DROP TABLE IF EXISTS parked_module_definitions;
