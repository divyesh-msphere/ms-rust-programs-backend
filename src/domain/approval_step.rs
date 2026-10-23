// domain/approval_step.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;

/// One sign-off an application goes through, in `position` order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStep {
    pub id: Uuid,                   // OUTPUT_ONLY
    pub program_id: Uuid,           // OUTPUT_ONLY
    pub name: String,
    /// The role allowed to decide this step (e.g. `credit_manager`).
    pub approver_role: String,
    pub position: i32,
    pub create_time: DateTime<Utc>, // OUTPUT_ONLY
    pub update_time: DateTime<Utc>, // OUTPUT_ONLY
}

#[derive(Debug, Clone)]
pub struct NewApprovalStep {
    /// Optional client-specified ID (AEP-133); server generates a UUID v7 if omitted.
    pub id: Option<Uuid>,
    pub program_id: Uuid,
    pub name: String,
    pub approver_role: String,
    /// `None` puts it after the last step.
    pub position: Option<i32>,
    pub actor: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateApprovalStep {
    pub name: Option<String>,
    pub approver_role: Option<String>,
    pub position: Option<i32>,
    pub actor: String,
}

pub fn validate_name(name: &str) -> Result<(), ApplicationError> {
    if name.trim().is_empty() || name.chars().count() > 120 {
        return Err(ApplicationError::unprocessable_entity("name must be 1-120 characters"));
    }
    Ok(())
}

pub fn validate_approver_role(role: &str) -> Result<(), ApplicationError> {
    if role.trim().is_empty() || role.chars().count() > 64 {
        return Err(ApplicationError::unprocessable_entity("approver_role must be 1-64 characters"));
    }
    Ok(())
}

pub fn validate_position(position: i32) -> Result<(), ApplicationError> {
    if !(0..=10_000).contains(&position) {
        return Err(ApplicationError::unprocessable_entity("position must be between 0 and 10000"));
    }
    Ok(())
}

#[async_trait]
pub trait ApprovalStepRepository: Send + Sync {
    /// In display order. The program must exist (the use case checks).
    async fn list(&self, program_id: Uuid) -> Result<Vec<ApprovalStep>, ApplicationError>;

    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<ApprovalStep, ApplicationError>;

    /// ONE transaction: the step, the program's version bump, the audit entry, the outbox event.
    async fn create(&self, input: NewApprovalStep) -> Result<ApprovalStep, ApplicationError>;

    /// Same transaction shape. A request that changes nothing writes nothing.
    async fn update(&self, program_id: Uuid, id: Uuid, input: UpdateApprovalStep) -> Result<ApprovalStep, ApplicationError>;

    /// A step that has recorded signoffs cannot be removed (`Conflict`): the history needs it.
    async fn delete(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<(), ApplicationError>;

    /// Sets many positions in one statement and returns the list in its new order.
    async fn reorder(
        &self,
        program_id: Uuid,
        positions: Vec<(Uuid, i32)>,
        actor: &str,
    ) -> Result<Vec<ApprovalStep>, ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_must_be_present_and_short() {
        assert!(validate_name("Credit review").is_ok());
        assert!(validate_name("  ").is_err());
        assert!(validate_name(&"a".repeat(121)).is_err());
    }

    #[test]
    fn approver_role_must_be_present_and_short() {
        assert!(validate_approver_role("credit_manager").is_ok());
        assert!(validate_approver_role("").is_err());
        assert!(validate_approver_role(&"r".repeat(65)).is_err());
    }

    #[test]
    fn position_is_bounded() {
        assert!(validate_position(0).is_ok());
        assert!(validate_position(10_000).is_ok());
        assert!(validate_position(-1).is_err());
        assert!(validate_position(10_001).is_err());
    }
}
