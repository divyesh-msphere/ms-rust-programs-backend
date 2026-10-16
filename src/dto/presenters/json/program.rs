use crate::dto::presenters::json::program_module::ProgramModulePresenter;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ProgramPresenter {
    pub path: String,
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub level: String,
    pub active: bool,
    pub config_version: i32,
    pub config_hash: String,
    pub create_time: String,
    pub update_time: String,
    pub state: String,
}

/// Create's (and, when modules changed, update's) response: the program plus the module manifest.
#[derive(Debug, Serialize, Clone)]
pub struct ProgramWithModulesPresenter {
    #[serde(flatten)]
    pub program: ProgramPresenter,
    /// Present on create, and on update when the request touched modules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<ProgramModulePresenter>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ListProgramsPresenter {
    pub results: Vec<ProgramPresenter>,
    pub next_page_token: Option<String>,
    pub total_size: i64,
}
