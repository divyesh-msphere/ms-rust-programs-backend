use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ProgramPresenter {
    pub path: String,
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blueprint_id: Option<String>,
    pub create_time: String,
    pub update_time: String,
    pub state: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ListProgramsPresenter {
    pub results: Vec<ProgramPresenter>,
    pub next_page_token: Option<String>,
    pub total_size: i64,
}
