// domain/application.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;

pub const DRAFT: &str = "draft";
pub const SUBMITTED: &str = "submitted";
pub const IN_REVIEW: &str = "in_review";
pub const APPROVED: &str = "approved";
pub const REJECTED: &str = "rejected";
pub const WITHDRAWN: &str = "withdrawn";

pub const STATUSES: [&str; 6] = [DRAFT, SUBMITTED, IN_REVIEW, APPROVED, REJECTED, WITHDRAWN];

const MAX_ANSWERS_BYTES: usize = 64 * 1024;

/// A store's application to join a program. The list screen reads only this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: Uuid,                            // OUTPUT_ONLY
    pub program_id: Uuid,                    // OUTPUT_ONLY
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    pub status: String,                      // OUTPUT_ONLY — moved by submit / withdraw / signoffs
    pub current_step_id: Option<Uuid>,       // OUTPUT_ONLY
    pub current_step_name: Option<String>,   // OUTPUT_ONLY
    /// The store's answers, keyed by requirement id. Only on a single read.
    pub answers: Option<Value>,
    /// What was required when the application was submitted. Only on a single read.
    pub requirements_snapshot: Option<Value>,
    pub submitted_time: Option<DateTime<Utc>>,
    pub decided_time: Option<DateTime<Utc>>,
    pub create_time: DateTime<Utc>,          // OUTPUT_ONLY
    pub update_time: DateTime<Utc>,          // OUTPUT_ONLY
}

#[derive(Debug, Clone)]
pub struct NewApplication {
    /// Optional client-specified ID (AEP-133); server generates a UUID v7 if omitted.
    pub id: Option<Uuid>,
    pub program_id: Uuid,
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    pub answers: Value,
    pub actor: String,
}

/// What can still be done to an application in this status (the Actions column).
pub fn allowed_actions(status: &str) -> Vec<&'static str> {
    match status {
        DRAFT => vec!["submit", "withdraw"],
        SUBMITTED | IN_REVIEW => vec!["withdraw"],
        _ => vec![],
    }
}

pub fn can_submit(status: &str) -> bool {
    status == DRAFT
}

pub fn can_withdraw(status: &str) -> bool {
    matches!(status, DRAFT | SUBMITTED | IN_REVIEW)
}

/// Waiting for a decision: counted by the Overview's "pending approvals".
pub fn is_pending(status: &str) -> bool {
    matches!(status, SUBMITTED | IN_REVIEW)
}

fn validate_text(field: &str, value: &str, max: usize) -> Result<(), ApplicationError> {
    if value.trim().is_empty() || value.chars().count() > max {
        return Err(ApplicationError::unprocessable_entity(&format!("{field} must be 1-{max} characters")));
    }
    Ok(())
}

fn validate_optional_text(field: &str, value: &Option<String>, max: usize) -> Result<(), ApplicationError> {
    match value {
        Some(text) if text.chars().count() > max => {
            Err(ApplicationError::unprocessable_entity(&format!("{field} must be at most {max} characters")))
        }
        _ => Ok(()),
    }
}

pub fn validate_new(input: &NewApplication) -> Result<(), ApplicationError> {
    validate_text("store_id", &input.store_id, 64)?;
    validate_text("website_id", &input.website_id, 64)?;
    validate_optional_text("store_name", &input.store_name, 200)?;
    validate_optional_text("website_name", &input.website_name, 200)?;
    if !input.answers.is_object() {
        return Err(ApplicationError::unprocessable_entity("answers must be a JSON object"));
    }
    if input.answers.to_string().len() > MAX_ANSWERS_BYTES {
        return Err(ApplicationError::unprocessable_entity("answers must be at most 64 KB"));
    }
    Ok(())
}

/// `status=submitted,in_review` -> the list of statuses; an unknown one is a 400.
pub fn parse_status_filter(raw: Option<&str>) -> Result<Vec<String>, ApplicationError> {
    let mut statuses = Vec::new();
    for part in raw.unwrap_or("").split(',').map(str::trim).filter(|p| !p.is_empty()) {
        if !STATUSES.contains(&part) {
            return Err(ApplicationError::validation_error(&format!(
                "status {part:?} must be one of: {}",
                STATUSES.join(", ")
            )));
        }
        if !statuses.iter().any(|s| s == part) {
            statuses.push(part.to_string());
        }
    }
    Ok(statuses)
}

#[async_trait]
pub trait ApplicationRepository: Send + Sync {
    /// Newest first, keyset on the id. An empty `statuses` means every status. Rows have no
    /// answers or snapshot.
    async fn list(
        &self,
        program_id: Uuid,
        statuses: &[String],
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<Application>, Option<Uuid>), ApplicationError>;

    /// One application with its answers and requirements snapshot.
    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<Application, ApplicationError>;

    /// ONE transaction: the draft, the program's `total_applications`, the audit entry and the
    /// outbox event. A store with an open application for the website is `already_exists`.
    async fn create(&self, input: NewApplication) -> Result<Application, ApplicationError>;

    /// draft -> submitted: freezes the program's requirements into the application, points it
    /// at the first approval step and counts it as pending. Any other status is a
    /// `failed_precondition`.
    async fn submit(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<Application, ApplicationError>;

    /// draft / submitted / in_review -> withdrawn. Any other status is a `failed_precondition`.
    async fn withdraw(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<Application, ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn new_application() -> NewApplication {
        NewApplication {
            id: None,
            program_id: Uuid::nil(),
            store_id: "store-1".into(),
            store_name: Some("Store One".into()),
            website_id: "web-1".into(),
            website_name: None,
            answers: json!({}),
            actor: "t".into(),
        }
    }

    #[test]
    fn actions_follow_the_status() {
        assert_eq!(allowed_actions(DRAFT), vec!["submit", "withdraw"]);
        assert_eq!(allowed_actions(SUBMITTED), vec!["withdraw"]);
        assert_eq!(allowed_actions(IN_REVIEW), vec!["withdraw"]);
        for done in [APPROVED, REJECTED, WITHDRAWN] {
            assert!(allowed_actions(done).is_empty());
        }
    }

    #[test]
    fn only_a_draft_can_be_submitted() {
        assert!(can_submit(DRAFT));
        for s in [SUBMITTED, IN_REVIEW, APPROVED, REJECTED, WITHDRAWN] {
            assert!(!can_submit(s));
        }
    }

    #[test]
    fn only_an_open_application_can_be_withdrawn() {
        for s in [DRAFT, SUBMITTED, IN_REVIEW] {
            assert!(can_withdraw(s));
        }
        for s in [APPROVED, REJECTED, WITHDRAWN] {
            assert!(!can_withdraw(s));
        }
    }

    #[test]
    fn pending_means_waiting_for_a_decision() {
        assert!(is_pending(SUBMITTED) && is_pending(IN_REVIEW));
        assert!(!is_pending(DRAFT) && !is_pending(APPROVED));
    }

    #[test]
    fn status_filter_parses_and_rejects_unknown() {
        assert!(parse_status_filter(None).unwrap().is_empty());
        assert!(parse_status_filter(Some("")).unwrap().is_empty());
        assert_eq!(parse_status_filter(Some("submitted, in_review,submitted")).unwrap(), vec!["submitted", "in_review"]);
        assert!(parse_status_filter(Some("done")).is_err());
    }

    #[test]
    fn new_application_is_validated() {
        assert!(validate_new(&new_application()).is_ok());
        let mut bad = new_application();
        bad.store_id = " ".into();
        assert!(validate_new(&bad).is_err());
        let mut bad = new_application();
        bad.website_id = "w".repeat(65);
        assert!(validate_new(&bad).is_err());
        let mut bad = new_application();
        bad.answers = json!([1]);
        assert!(validate_new(&bad).is_err());
        let mut bad = new_application();
        bad.answers = json!({"big": "a".repeat(65 * 1024)});
        assert!(validate_new(&bad).is_err());
        let mut bad = new_application();
        bad.store_name = Some("n".repeat(201));
        assert!(validate_new(&bad).is_err());
    }
}
