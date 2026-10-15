DELETE FROM module_definitions
WHERE key IN ('catalog_pricing', 'enrollment', 'document_agreement',
              'order_transaction', 'configuration_oversight')
  AND parent_id IS NULL;
