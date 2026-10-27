use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SignoffPresenter {
    pub path: String,
    pub id: String,
    pub application_id: String,
    pub approval_step_id: String,
    pub approval_step_name: String,
    pub decision: String,
    pub comment: Option<String>,
    pub actor: String,
    pub create_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SignoffsPresenter {
    pub results: Vec<SignoffPresenter>,
}
