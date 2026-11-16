-- Renewal term, grace and reminder are NULL until the program sets them (no 12 / 30 / 30 defaults).
ALTER TABLE programs ALTER COLUMN renewal_term_months DROP NOT NULL;
ALTER TABLE programs ALTER COLUMN renewal_grace_days DROP NOT NULL;
ALTER TABLE programs ALTER COLUMN renewal_reminder_days DROP NOT NULL;
ALTER TABLE programs ALTER COLUMN renewal_term_months DROP DEFAULT;
ALTER TABLE programs ALTER COLUMN renewal_grace_days DROP DEFAULT;
ALTER TABLE programs ALTER COLUMN renewal_reminder_days DROP DEFAULT;
