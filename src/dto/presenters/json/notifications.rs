use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct NotificationTriggerPresenter {
    pub key: String,
    pub label: String,
    pub email: bool,
    pub sms: bool,
    pub push: bool,
    pub in_app: bool,
}

/// The Notifications page of a program.
#[derive(Debug, Serialize, Clone)]
pub struct NotificationsPresenter {
    pub path: String,
    /// The program's `config_version`: send it back with a save.
    pub version: i32,
    /// The columns of the grid, in order.
    pub channels: Vec<&'static str>,
    pub triggers: Vec<NotificationTriggerPresenter>,
}
