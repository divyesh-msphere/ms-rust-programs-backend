-- A program's own icon and Display switch for each of its modules. `icon` NULL falls back to the catalog
-- module's icon. Added to the parked mirror too: they move rows between the two tables with SELECT *.
ALTER TABLE program_modules ADD COLUMN IF NOT EXISTS icon TEXT;
ALTER TABLE program_modules ADD COLUMN IF NOT EXISTS display BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE parked_program_modules ADD COLUMN IF NOT EXISTS icon TEXT;
ALTER TABLE parked_program_modules ADD COLUMN IF NOT EXISTS display BOOLEAN NOT NULL DEFAULT true;
