-- The Details step of Create Program: the merchant-facing text, the badge branding and the renewal
-- defaults of a program, plus the per-setting Display switch of the Programs & Module step.
--
-- Every column has a default, so a program (or a setting) created without them reads back the same
-- as before. Re-runnable like every migration here.
ALTER TABLE programs ADD COLUMN IF NOT EXISTS about TEXT;
-- Up to 5 short selling points, in order.
ALTER TABLE programs ADD COLUMN IF NOT EXISTS highlights TEXT[] NOT NULL DEFAULT '{}';
-- The badge: `#RRGGBB` colours and an icon name.
ALTER TABLE programs ADD COLUMN IF NOT EXISTS brand_color TEXT;
ALTER TABLE programs ADD COLUMN IF NOT EXISTS text_color TEXT;
ALTER TABLE programs ADD COLUMN IF NOT EXISTS logo_icon TEXT;
-- Renewal defaults (the Figma defaults: 12 months, 30 days grace, 30 days reminder, no auto-renew).
ALTER TABLE programs ADD COLUMN IF NOT EXISTS renewal_term_months INT NOT NULL DEFAULT 12;
ALTER TABLE programs ADD COLUMN IF NOT EXISTS renewal_grace_days INT NOT NULL DEFAULT 30;
ALTER TABLE programs ADD COLUMN IF NOT EXISTS renewal_reminder_days INT NOT NULL DEFAULT 30;
ALTER TABLE programs ADD COLUMN IF NOT EXISTS auto_renew BOOLEAN NOT NULL DEFAULT false;

-- The Display switch of a setting: false hides it from the form, the setting itself is kept.
ALTER TABLE program_fields ADD COLUMN IF NOT EXISTS display BOOLEAN NOT NULL DEFAULT true;
