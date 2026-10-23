use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ApprovalStepPresenter {
    pub path: String,
    pub id: String,
    pub name: String,
    pub approver_role: String,
    pub position: i32,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ApprovalStepsPresenter {
    pub results: Vec<ApprovalStepPresenter>,
}
