-- The module tree (any depth), seeded and almost never written. The service
-- caches it in memory, so these tables are optimised for the cold-start load
-- and for subtree queries, not for write throughput.
CREATE TABLE IF NOT EXISTS module_definitions (
    id UUID PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    parent_id UUID REFERENCES module_definitions (id) ON DELETE RESTRICT,
    ancestry LTREE NOT NULL,
    position INT NOT NULL DEFAULT 0,
    ui_component TEXT,
    rule_family TEXT,
    universal BOOLEAN NOT NULL DEFAULT false,
    requires_publish BOOLEAN NOT NULL DEFAULT false,
    permission_key TEXT,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Children of a node, in display order.
CREATE INDEX IF NOT EXISTS idx_module_definitions_parent_position_id
    ON module_definitions (parent_id, position, id);
-- Ancestor/descendant (<@, @>) for "everything under Catalog & Pricing" ...
CREATE INDEX IF NOT EXISTS idx_module_definitions_ancestry_gist
    ON module_definitions USING GIST (ancestry);
-- ... and ordered / keyset reads of the whole tree in path order.
CREATE INDEX IF NOT EXISTS idx_module_definitions_ancestry_btree
    ON module_definitions (ancestry);

-- Fields of a field_form leaf.
CREATE TABLE IF NOT EXISTS module_fields (
    id UUID PRIMARY KEY,
    module_definition_id UUID NOT NULL REFERENCES module_definitions (id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    label TEXT NOT NULL,
    field_type TEXT NOT NULL,
    -- Global-settings library that supplies this field's options. An opaque
    -- id owned by another service, so deliberately NOT a foreign key.
    library_id UUID,
    default_value JSONB,
    required BOOLEAN NOT NULL DEFAULT false,
    position INT NOT NULL DEFAULT 0,
    help_text TEXT,
    visible_when_field_id UUID REFERENCES module_fields (id) ON DELETE SET NULL,
    visible_when_value JSONB,
    derived_from TEXT,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (module_definition_id, key)
);

-- Form load: a leaf's fields in display order, one index range.
CREATE INDEX IF NOT EXISTS idx_module_fields_module_position_id
    ON module_fields (module_definition_id, position, id);
-- Webhook path: "which fields draw from the library that just changed?"
CREATE INDEX IF NOT EXISTS idx_module_fields_library_id
    ON module_fields (library_id) WHERE library_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_module_fields_visible_when_field_id
    ON module_fields (visible_when_field_id) WHERE visible_when_field_id IS NOT NULL;

-- Modules a blueprint starts with. The composite key IS the clustering the
-- read wants (all modules of one blueprint), so there is no surrogate id and
-- no second index; the module id doubles as the resource id.
CREATE TABLE IF NOT EXISTS blueprint_modules (
    blueprint_id UUID NOT NULL REFERENCES blueprints (id) ON DELETE CASCADE,
    module_id UUID NOT NULL REFERENCES module_definitions (id) ON DELETE RESTRICT,
    default_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (blueprint_id, module_id)
);
