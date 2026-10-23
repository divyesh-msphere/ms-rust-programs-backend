use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ApplicationPresenter {
    pub path: String,
    pub id: String,
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    pub status: String,
    pub current_step_id: Option<String>,
    pub current_step_name: Option<String>,
    /// What can still be done in this status (the Actions column).
    pub actions: Vec<&'static str>,
    /// Only on a single read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answers: Option<serde_json::Value>,
    /// Only on a single read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements_snapshot: Option<serde_json::Value>,
    pub submitted_time: Option<String>,
    pub decided_time: Option<String>,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ListApplicationsPresenter {
    pub results: Vec<ApplicationPresenter>,
    pub next_page_token: Option<String>,
}
