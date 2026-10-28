use serde::Deserialize;

/// The path of every Notifications call.
#[derive(Debug, Deserialize, Clone)]
pub struct NotificationsPathPayload {
    /// The program's ID or its slug.
    pub program: String,
}

/// One row of the grid.
#[derive(Debug, Deserialize, Clone)]
pub struct NotificationTriggerPayload {
    /// Lower-case machine name, e.g. `balance_threshold`.
    pub key: String,
    /// What the admin sees, e.g. `Balance Threshold`.
    pub label: String,
    #[serde(default)]
    pub email: bool,
    #[serde(default)]
    pub sms: bool,
    #[serde(default)]
    pub push: bool,
    #[serde(default)]
    pub in_app: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReplaceNotificationsBodyPayload {
    /// The program's `config_version` as the caller last read it. A stale one is rejected with 409;
    /// omit it to skip the check.
    pub version: Option<i32>,
    /// The whole grid as it should be after the save.
    pub triggers: Vec<NotificationTriggerPayload>,
}
