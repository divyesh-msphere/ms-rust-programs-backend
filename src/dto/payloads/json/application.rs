use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct ListApplicationsPathPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListApplicationsQueryPayload {
    /// Only these statuses, comma-separated (`submitted,in_review`). Omit for all.
    pub status: Option<String>,
    pub max_page_size: Option<i64>,
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetApplicationPayload {
    pub program: String,
    pub application: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateApplicationPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateApplicationQueryPayload {
    /// Optional client-specified ID (AEP-133); server generates one if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateApplicationBodyPayload {
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    /// The store's answers, keyed by requirement id. Defaults to `{}`.
    pub answers: Option<Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SubmitApplicationPayload {
    pub program: String,
    pub application: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WithdrawApplicationPayload {
    pub program: String,
    pub application: String,
}
