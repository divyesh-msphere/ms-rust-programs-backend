// domain/enrollment.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;
use crate::domain::store_query::{StoreFilter, StoreTerm};

pub const PENDING: &str = "pending";
pub const ACTIVE: &str = "active";
pub const SUSPENDED: &str = "suspended";
pub const ENDED: &str = "ended";

pub const STATUSES: [&str; 4] = [PENDING, ACTIVE, SUSPENDED, ENDED];

/// How many stores one bulk enroll may carry.
pub const MAX_BATCH: usize = 500;

/// A store enrolled in a program on one website. The Enrolled Stores list reads only this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrollment {
    pub id: Uuid,                              // OUTPUT_ONLY
    pub program_id: Uuid,                      // OUTPUT_ONLY
    /// The application that produced it, when there was one.
    pub application_id: Option<Uuid>,          // OUTPUT_ONLY
    pub store_id: String,
    /// `None` when the name was not known at enroll time ("Unresolved store").
    pub store_name: Option<String>,
    pub store_resolved: bool,                  // OUTPUT_ONLY
    pub website_id: String,
    pub website_name: Option<String>,
    pub website_domain: Option<String>,
    /// The business unit this website belongs to (its parent in the store-discount levels).
    /// The business unit of the store (`bu` in the API): its code and name. Also the parent of the
    /// website in the store discounts.
    pub bu_unit_code: Option<String>,
    pub bu_unit_name: Option<String>,
    pub customer_group: Option<String>,
    pub status: String,
    pub enrolled_time: Option<DateTime<Utc>>,  // OUTPUT_ONLY
    pub contracts_count: i32,                  // OUTPUT_ONLY
    pub last_contract_time: Option<DateTime<Utc>>, // OUTPUT_ONLY
    pub create_time: DateTime<Utc>,            // OUTPUT_ONLY
    pub update_time: DateTime<Utc>,            // OUTPUT_ONLY
}

#[derive(Debug, Clone)]
pub struct NewEnrollment {
    /// Optional client-specified ID (AEP-133); server generates a UUID v7 if omitted.
    pub id: Option<Uuid>,
    pub program_id: Uuid,
    pub application_id: Option<Uuid>,
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    pub website_domain: Option<String>,
    pub bu_unit_code: Option<String>,
    pub bu_unit_name: Option<String>,
    pub customer_group: Option<String>,
    /// `pending` or `active`.
    pub status: String,
    pub actor: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateEnrollment {
    pub status: Option<String>,
    /// `Some(Some(bu))` replaces the business unit, `Some(None)` clears it.
    pub bu: Option<Option<BuUnit>>,
    /// The name of the location; `Some("")` clears it.
    pub store_name: Option<String>,
    /// `Some("")` clears the group.
    pub customer_group: Option<String>,
    pub actor: String,
}

/// The business unit of a store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuUnit {
    pub code: String,
    pub name: Option<String>,
}

/// The filters of the list (all optional): the search box and the `filter` terms.
#[derive(Debug, Clone, Default)]
pub struct EnrollmentFilter {
    /// ANDed `search` conditions.
    pub search: Vec<StoreTerm>,
    /// The `filter` terms: text conditions and the status / website lists.
    pub list: StoreFilter,
}

/// One item of a bulk enroll and what happened to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchOutcome {
    Created(Uuid),
    AlreadyEnrolled,
    Failed(String),
}

/// Where a status may go: to any other status, `ended` and `pending` included, so a store that
/// was ended or sent back to pending can be moved again. Going live again (pending, active or
/// suspended) is still refused when the store already has another live enrollment on the website.
pub fn allowed_transitions(status: &str) -> Vec<&'static str> {
    if !STATUSES.contains(&status) {
        return vec![];
    }
    STATUSES.iter().copied().filter(|s| *s != status).collect()
}

pub fn can_change(from: &str, to: &str) -> bool {
    allowed_transitions(from).contains(&to)
}

/// The change in the Overview's `active_enrollments` when the status moves.
pub fn active_delta(from: &str, to: &str) -> i32 {
    i32::from(to == ACTIVE) - i32::from(from == ACTIVE)
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

pub fn validate_new(input: &NewEnrollment) -> Result<(), ApplicationError> {
    validate_text("bu_location.code", &input.store_id, 64)?;
    validate_text("website.code", &input.website_id, 64)?;
    validate_optional_text("bu_location.name", &input.store_name, 200)?;
    validate_optional_text("website.name", &input.website_name, 200)?;
    validate_optional_text("website.domain", &input.website_domain, 253)?;
    validate_optional_text("bu.code", &input.bu_unit_code, 64)?;
    validate_optional_text("bu.name", &input.bu_unit_name, 200)?;
    if input.bu_unit_name.as_deref().is_some_and(|n| !n.trim().is_empty())
        && input.bu_unit_code.as_deref().is_none_or(|c| c.trim().is_empty())
    {
        return Err(ApplicationError::unprocessable_entity("bu.name needs bu.code"));
    }
    validate_optional_text("customer_name", &input.customer_group, 64)?;
    if input.status != PENDING && input.status != ACTIVE {
        return Err(ApplicationError::unprocessable_entity("status must be \"pending\" or \"active\" when enrolling"));
    }
    Ok(())
}

pub fn validate_update(input: &UpdateEnrollment) -> Result<(), ApplicationError> {
    if let Some(status) = &input.status
        && !STATUSES.contains(&status.as_str())
    {
        return Err(ApplicationError::unprocessable_entity(&format!(
            "status {status:?} must be one of: {}",
            STATUSES.join(", ")
        )));
    }
    validate_optional_text("bu_location.name", &input.store_name, 200)?;
    if let Some(Some(bu)) = &input.bu {
        validate_text("bu.code", &bu.code, 64)?;
        validate_optional_text("bu.name", &bu.name, 200)?;
    }
    validate_optional_text("customer_name", &input.customer_group, 64)
}

/// `status=active,suspended` -> the statuses; an unknown one is a 400.
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
pub trait EnrollmentRepository: Send + Sync {
    /// Newest first, keyset on the id. Also the number of enrollments matching the filter
    /// (the "N STORES" count).
    async fn list(
        &self,
        program_id: Uuid,
        filter: &EnrollmentFilter,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<Enrollment>, Option<Uuid>, i64), ApplicationError>;

    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<Enrollment, ApplicationError>;

    /// ONE transaction: the enrollment, the Overview counters, the audit entry and the outbox
    /// event. A store with a live (pending, active or suspended) enrollment for the website is
    /// `already_exists`.
    async fn create(&self, input: NewEnrollment) -> Result<Enrollment, ApplicationError>;

    /// Enrolls many stores with one statement per chunk. Every item gets its own outcome, so one
    /// bad or duplicate row cannot fail the rest. One audit entry and one outbox event for the batch.
    async fn batch_create(
        &self,
        program_id: Uuid,
        items: Vec<NewEnrollment>,
        actor: &str,
    ) -> Result<Vec<BatchOutcome>, ApplicationError>;

    /// A status change must follow `allowed_transitions` (`failed_precondition` otherwise) and
    /// keeps the counters, the audit entry and the outbox event in one transaction.
    async fn update(&self, program_id: Uuid, id: Uuid, input: UpdateEnrollment) -> Result<Enrollment, ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_enrollment() -> NewEnrollment {
        NewEnrollment {
            id: None,
            program_id: Uuid::nil(),
            application_id: None,
            store_id: "store-1".into(),
            store_name: Some("Store One".into()),
            website_id: "web-1".into(),
            website_name: None,
            website_domain: None,
            bu_unit_code: None,
            bu_unit_name: None,
            customer_group: Some("tier-1".into()),
            status: ACTIVE.into(),
            actor: "t".into(),
        }
    }

    #[test]
    fn any_status_can_move_to_any_other() {
        for from in STATUSES {
            for to in STATUSES {
                assert_eq!(can_change(from, to), from != to, "{from} -> {to}");
            }
        }
        assert!(can_change(ACTIVE, PENDING), "back to pending");
        assert!(can_change(ENDED, ACTIVE), "an ended store can be brought back");
        assert!(!can_change("gone", ACTIVE));
    }

    #[test]
    fn actions_are_the_other_statuses() {
        assert_eq!(allowed_transitions(ACTIVE), vec![PENDING, SUSPENDED, ENDED]);
        assert_eq!(allowed_transitions(ENDED), vec![PENDING, ACTIVE, SUSPENDED]);
        assert!(allowed_transitions("gone").is_empty());
    }

    #[test]
    fn active_counter_follows_the_status() {
        assert_eq!(active_delta(PENDING, ACTIVE), 1);
        assert_eq!(active_delta(SUSPENDED, ACTIVE), 1);
        assert_eq!(active_delta(ACTIVE, SUSPENDED), -1);
        assert_eq!(active_delta(ACTIVE, ENDED), -1);
        assert_eq!(active_delta(PENDING, ENDED), 0);
        assert_eq!(active_delta(SUSPENDED, ENDED), 0);
    }

    #[test]
    fn new_enrollment_is_validated() {
        assert!(validate_new(&new_enrollment()).is_ok());
        let mut bad = new_enrollment();
        bad.store_id = " ".into();
        assert!(validate_new(&bad).is_err());
        let mut bad = new_enrollment();
        bad.website_id = "w".repeat(65);
        assert!(validate_new(&bad).is_err());
        let mut bad = new_enrollment();
        bad.status = SUSPENDED.into();
        assert!(validate_new(&bad).is_err());
        let mut bad = new_enrollment();
        bad.customer_group = Some("g".repeat(65));
        assert!(validate_new(&bad).is_err());
        let mut ok = new_enrollment();
        ok.status = PENDING.into();
        assert!(validate_new(&ok).is_ok());
    }

    #[test]
    fn update_is_validated() {
        assert!(validate_update(&UpdateEnrollment::default()).is_ok());
        let bad = UpdateEnrollment { status: Some("gone".into()), ..Default::default() };
        assert!(validate_update(&bad).is_err());
        let ok = UpdateEnrollment { status: Some(SUSPENDED.into()), customer_group: Some(String::new()), ..Default::default() };
        assert!(validate_update(&ok).is_ok());
    }

    #[test]
    fn status_filter_parses_and_rejects_unknown() {
        assert!(parse_status_filter(None).unwrap().is_empty());
        assert_eq!(parse_status_filter(Some("active, suspended,active")).unwrap(), vec!["active", "suspended"]);
        assert!(parse_status_filter(Some("approved")).is_err());
    }
}
