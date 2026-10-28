-- Notifications (Settings > Universal): a PREMADE page with its own API (`notifications`): the grid
-- of trigger points x channels (Email, SMS, Push, In-App). A program gets it when it is picked in
-- the Create program module list; it comes back in the manifest like every other module.
-- IDEMPOTENT, like every file here.

-- One row per trigger point of a program. The channels are fixed columns: the grid is the same
-- on every program.
CREATE TABLE IF NOT EXISTS program_notification_triggers (
    program_id  UUID        NOT NULL REFERENCES programs (id) ON DELETE CASCADE,
    key         TEXT        NOT NULL,
    label       TEXT        NOT NULL,
    email       BOOLEAN     NOT NULL DEFAULT false,
    sms         BOOLEAN     NOT NULL DEFAULT false,
    push        BOOLEAN     NOT NULL DEFAULT false,
    in_app      BOOLEAN     NOT NULL DEFAULT false,
    position    INT         NOT NULL DEFAULT 0,
    create_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (program_id, key)
);

-- The page: code-backed (handler + api_base), picked by the admin, nothing to configure.
UPDATE module_definitions
SET source = 'premade', handler_key = 'notifications', api_base = 'notifications',
    ui_component = 'custom:notifications', universal = false, selectable = true, allow_components = false
WHERE key = 'configuration_oversight.settings.universal.notifications';
