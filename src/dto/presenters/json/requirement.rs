use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct RequirementPresenter {
    pub path: String,
    pub id: String,
    #[serde(rename = "type")]
    pub requirement_type: String,
    pub name: String,
    pub config: serde_json::Value,
    pub position: i32,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct RequirementsPresenter {
    pub results: Vec<RequirementPresenter>,
}
