use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct ListRequirementsPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetRequirementPayload {
    pub program: String,
    pub requirement: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateRequirementPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateRequirementQueryPayload {
    /// Optional client-specified ID (AEP-133); server generates one if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateRequirementBodyPayload {
    /// `signup`, `card_on_file`, `credit_application`, `contract` (one of each per program) or
    /// `custom_field` (any number).
    #[serde(rename = "type")]
    pub requirement_type: String,
    pub name: String,
    /// A JSON object with this requirement's settings. Defaults to `{}`.
    pub config: Option<Value>,
    /// Display order. Defaults to after the last requirement.
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateRequirementPathPayload {
    pub program: String,
    pub requirement: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateRequirementBodyPayload {
    pub name: Option<String>,
    pub config: Option<Value>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeleteRequirementPayload {
    pub program: String,
    pub requirement: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateRequirementsPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateRequirementsBodyPayload {
    /// The new order: each requirement id with its position.
    pub requirements: Vec<RequirementPositionPayload>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RequirementPositionPayload {
    pub id: String,
    pub position: i32,
}
