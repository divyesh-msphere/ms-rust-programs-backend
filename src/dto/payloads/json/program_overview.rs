use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GetOverviewPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListAuditLogsPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListAuditLogsQueryPayload {
    pub max_page_size: Option<i64>,
    pub page_token: Option<String>,
    /// Only these actions, comma-separated (for example `values_updated,updated`). Omit for all.
    pub action: Option<String>,
}
