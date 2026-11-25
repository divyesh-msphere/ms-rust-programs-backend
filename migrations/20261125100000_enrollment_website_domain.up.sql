-- A store's website is now {code, name, domain}; the code is website_id, the name website_name.
ALTER TABLE enrollments ADD COLUMN IF NOT EXISTS website_domain text;

-- The Enrolled Stores page now lives under /stores; the manifest tells the frontend where to call.
UPDATE module_definitions SET api_base = 'stores' WHERE handler_key = 'enrolled_stores';
