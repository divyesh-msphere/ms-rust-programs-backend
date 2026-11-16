-- Where a module lives in the product: `configuration` (set a program up) or `operate` (run it day to day).
-- NULL for every other module, including the ones beneath those below (it is set on the named modules only,
-- not inherited). The parked table mirrors the live one column for column.
ALTER TABLE module_definitions ADD COLUMN IF NOT EXISTS module_in TEXT;
ALTER TABLE module_definitions DROP CONSTRAINT IF EXISTS module_definitions_module_in_check;
ALTER TABLE module_definitions ADD CONSTRAINT module_definitions_module_in_check
    CHECK (module_in IS NULL OR module_in IN ('configuration', 'operate'));
ALTER TABLE parked_module_definitions ADD COLUMN IF NOT EXISTS module_in TEXT;

-- Re-running starts clean, so the result is always exactly the modules named below.
UPDATE module_definitions SET module_in = NULL;

-- Products & Rules, Pricing, Documents, Agreements, Settings.
UPDATE module_definitions SET module_in = 'configuration'
WHERE ancestry = ANY (ARRAY[
    'catalog_pricing.products_rules', 'catalog_pricing.pricing',
    'document_agreement.documents', 'document_agreement.agreements',
    'configuration_oversight.settings']::ltree[]);

-- Store Discounts, Stores, Core Returns.
UPDATE module_definitions SET module_in = 'operate'
WHERE ancestry = ANY (ARRAY[
    'catalog_pricing.store_discounts', 'enrollment.stores', 'order_transaction.core_returns']::ltree[]);
