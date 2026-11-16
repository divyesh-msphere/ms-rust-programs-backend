-- A module a program gets (the defaults, the premade pages, the ones an admin adds) starts SWITCHED OFF:
-- an admin turns each one on with `enabled: true`. Rows that already exist are left as they are.
ALTER TABLE program_modules ALTER COLUMN enabled SET DEFAULT false;
ALTER TABLE parked_program_modules ALTER COLUMN enabled SET DEFAULT false;
