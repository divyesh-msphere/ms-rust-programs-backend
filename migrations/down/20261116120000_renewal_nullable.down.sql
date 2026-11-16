UPDATE programs SET renewal_term_months = 12 WHERE renewal_term_months IS NULL;
UPDATE programs SET renewal_grace_days = 30 WHERE renewal_grace_days IS NULL;
UPDATE programs SET renewal_reminder_days = 30 WHERE renewal_reminder_days IS NULL;
ALTER TABLE programs ALTER COLUMN renewal_term_months SET DEFAULT 12;
ALTER TABLE programs ALTER COLUMN renewal_grace_days SET DEFAULT 30;
ALTER TABLE programs ALTER COLUMN renewal_reminder_days SET DEFAULT 30;
ALTER TABLE programs ALTER COLUMN renewal_term_months SET NOT NULL;
ALTER TABLE programs ALTER COLUMN renewal_grace_days SET NOT NULL;
ALTER TABLE programs ALTER COLUMN renewal_reminder_days SET NOT NULL;
