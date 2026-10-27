use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct PublicationPresenter {
    pub path: String,
    pub id: String,
    /// The program's `config_version` this publication covers.
    pub config_version: i32,
    pub config_hash: String,
    /// Counts of what the bundle holds, and the rule versions it carries.
    pub summary: serde_json::Value,
    /// The whole bundle. Only on a single read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<serde_json::Value>,
    pub created_by: Option<String>,
    pub create_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ListPublicationsPresenter {
    pub results: Vec<PublicationPresenter>,
    pub next_page_token: Option<String>,
}
