// domain/signoff.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;
use crate::domain::application::{APPROVED, IN_REVIEW, REJECTED};

pub const APPROVE: &str = "approved";
pub const REJECT: &str = "rejected";

const MAX_COMMENT_CHARS: usize = 1000;

/// One decision on one approval step of one application. Never changed or removed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signoff {
    pub id: Uuid,                    // OUTPUT_ONLY
    pub application_id: Uuid,        // OUTPUT_ONLY
    pub program_id: Uuid,            // OUTPUT_ONLY
    pub approval_step_id: Uuid,      // OUTPUT_ONLY
    pub approval_step_name: String,  // OUTPUT_ONLY
    pub decision: String,            // `approved` or `rejected`
    pub comment: Option<String>,
    pub actor: String,               // OUTPUT_ONLY
    pub create_time: DateTime<Utc>,  // OUTPUT_ONLY
}

#[derive(Debug, Clone)]
pub struct NewSignoff {
    pub program_id: Uuid,
    pub application_id: Uuid,
    /// The step the caller was looking at. When given and the application has moved on, the
    /// decision is refused (`Conflict`), so a double click cannot decide two steps.
    pub approval_step_id: Option<Uuid>,
    pub decision: String,
    pub comment: Option<String>,
    pub actor: String,
}

/// What a decision does to the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    /// The application's new status.
    pub status: &'static str,
    /// It moves on to the next approval step (otherwise it leaves its step).
    pub moves_to_next_step: bool,
    /// The audit action (and outbox topic suffix).
    pub action: &'static str,
}

/// Approving a step with another one after it keeps the application in review at that next step;
/// approving the last step approves the application; rejecting any step rejects it.
pub fn outcome(decision: &str, has_next_step: bool) -> Result<Outcome, ApplicationError> {
    match (decision, has_next_step) {
        (APPROVE, true) => Ok(Outcome { status: IN_REVIEW, moves_to_next_step: true, action: "step_approved" }),
        (APPROVE, false) => Ok(Outcome { status: APPROVED, moves_to_next_step: false, action: "approved" }),
        (REJECT, _) => Ok(Outcome { status: REJECTED, moves_to_next_step: false, action: "rejected" }),
        _ => Err(validate_decision(decision).err().unwrap_or_else(|| ApplicationError::validation_error("unknown decision"))),
    }
}

pub fn validate_decision(decision: &str) -> Result<(), ApplicationError> {
    if decision == APPROVE || decision == REJECT {
        Ok(())
    } else {
        Err(ApplicationError::unprocessable_entity(&format!(
            "decision {decision:?} must be {APPROVE:?} or {REJECT:?}"
        )))
    }
}

pub fn validate_comment(comment: &Option<String>) -> Result<(), ApplicationError> {
    match comment {
        Some(text) if text.chars().count() > MAX_COMMENT_CHARS => Err(ApplicationError::unprocessable_entity(
            &format!("comment must be at most {MAX_COMMENT_CHARS} characters"),
        )),
        _ => Ok(()),
    }
}

#[async_trait]
pub trait SignoffRepository: Send + Sync {
    /// The application's decisions in the order they were made.
    async fn list(&self, program_id: Uuid, application_id: Uuid) -> Result<Vec<Signoff>, ApplicationError>;

    async fn get(&self, program_id: Uuid, application_id: Uuid, id: Uuid) -> Result<Signoff, ApplicationError>;

    /// Decides the step the application is waiting at. ONE transaction: the signoff, the
    /// application's new status and step, the pending counter when it is decided for good, the
    /// audit entry and the outbox event. An application that is not waiting at a step is a
    /// `failed_precondition`.
    async fn create(&self, input: NewSignoff) -> Result<Signoff, ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approving_with_a_next_step_keeps_it_in_review() {
        let o = outcome(APPROVE, true).unwrap();
        assert_eq!((o.status, o.moves_to_next_step, o.action), (IN_REVIEW, true, "step_approved"));
    }

    #[test]
    fn approving_the_last_step_approves_the_application() {
        let o = outcome(APPROVE, false).unwrap();
        assert_eq!((o.status, o.moves_to_next_step, o.action), (APPROVED, false, "approved"));
    }

    #[test]
    fn rejecting_rejects_at_any_step() {
        for has_next in [true, false] {
            let o = outcome(REJECT, has_next).unwrap();
            assert_eq!((o.status, o.moves_to_next_step, o.action), (REJECTED, false, "rejected"));
        }
    }

    #[test]
    fn unknown_decisions_are_refused() {
        assert!(outcome("maybe", true).is_err());
        assert!(validate_decision("approve").is_err());
        assert!(validate_decision("").is_err());
        assert!(validate_decision(APPROVE).is_ok() && validate_decision(REJECT).is_ok());
    }

    #[test]
    fn comment_is_bounded() {
        assert!(validate_comment(&None).is_ok());
        assert!(validate_comment(&Some("ok".into())).is_ok());
        assert!(validate_comment(&Some("c".repeat(1001))).is_err());
    }
}
