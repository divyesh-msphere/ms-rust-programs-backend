-- The module tree (any depth), seeded and almost never written. The service can
-- cache it in memory, so this table is optimised for the cold-start load and for
-- subtree queries, not for write throughput.
--
-- Every module is one of three kinds (`source`):
--   premade    code-backed (a `handler_key` + the `api_base` the frontend calls);
--              the admin SELECTS it when creating a program
--   container  the admin chooses what the leaf is: a Rule, or Custom settings
--   default    Overview: on every program, not selectable
CREATE TABLE IF NOT EXISTS module_definitions (
    id UUID PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,                              -- group | leaf
    parent_id UUID REFERENCES module_definitions (id) ON DELETE RESTRICT,
    ancestry LTREE NOT NULL,
    position INT NOT NULL DEFAULT 0,
    ui_component TEXT,
    rule_family TEXT,
    universal BOOLEAN NOT NULL DEFAULT false,
    requires_publish BOOLEAN NOT NULL DEFAULT false,
    permission_key TEXT,
    source TEXT NOT NULL DEFAULT 'container',
    handler_key TEXT,
    api_base TEXT,
    selectable BOOLEAN NOT NULL DEFAULT true,
    allow_components BOOLEAN NOT NULL DEFAULT false,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Children of a node, in display order.
CREATE INDEX IF NOT EXISTS idx_module_definitions_parent_position_id
    ON module_definitions (parent_id, position, id);
-- Ancestor / descendant (<@, @>) for "everything under Catalog & Pricing" ...
CREATE INDEX IF NOT EXISTS idx_module_definitions_ancestry_gist
    ON module_definitions USING GIST (ancestry);
-- ... and ordered / keyset reads of the whole tree in path order.
CREATE INDEX IF NOT EXISTS idx_module_definitions_ancestry_btree
    ON module_definitions (ancestry);
