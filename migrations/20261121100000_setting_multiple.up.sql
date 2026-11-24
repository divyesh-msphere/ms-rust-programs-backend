-- A choice setting is single or multiple. Checkbox and custom_option_multi were always multiple.
ALTER TABLE program_fields ADD COLUMN IF NOT EXISTS multiple boolean NOT NULL DEFAULT false;
UPDATE program_fields SET multiple = true WHERE field_type IN ('checkbox', 'custom_option_multi') AND multiple = false;
