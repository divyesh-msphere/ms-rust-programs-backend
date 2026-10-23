use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ListApprovalStepsPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetApprovalStepPayload {
    pub program: String,
    pub approval_step: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateApprovalStepPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateApprovalStepQueryPayload {
    /// Optional client-specified ID (AEP-133); server generates one if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateApprovalStepBodyPayload {
    pub name: String,
    /// The role allowed to decide this step.
    pub approver_role: String,
    /// Display order. Defaults to after the last step.
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateApprovalStepPathPayload {
    pub program: String,
    pub approval_step: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateApprovalStepBodyPayload {
    pub name: Option<String>,
    pub approver_role: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeleteApprovalStepPayload {
    pub program: String,
    pub approval_step: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateApprovalStepsPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateApprovalStepsBodyPayload {
    /// The new order: each step id with its position.
    pub approval_steps: Vec<ApprovalStepPositionPayload>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApprovalStepPositionPayload {
    pub id: String,
    pub position: i32,
}
