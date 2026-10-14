use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateProgramPayload {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    /// Blueprint to start from; omitted means "from scratch".
    pub blueprint_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateProgramQueryPayload {
    /// Optional client-specified ID (AEP-133); server generates one if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetProgramPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListProgramsPayload {
    pub max_page_size: Option<i64>,
    pub page_token: Option<String>,
    pub filter: Option<String>,
    pub show_deleted: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProgramPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UpdateProgramBodyPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeleteProgramPayload {
    pub program: String,
}
