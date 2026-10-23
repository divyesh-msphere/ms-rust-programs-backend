// domain/program_overview.rs
//
// The default Overview page of a program, and the audit log behind its Recent activity and the
// History / View changes buttons.
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;

/// The workflow statuses of an application, in the order the funnel shows them. A status with no
/// applications is still listed (with 0), so the page always draws the same funnel.
pub const FUNNEL_STATUSES: [&str; 6] = ["draft", "submitted", "in_review", "approved", "rejected", "withdrawn"];

/// The counters on the Overview. One row per program, kept up to date by the changes that move
/// them, so this page never counts anything.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProgramStats {
    pub total_applications: i32,
    pub pending_approvals: i32,
    pub active_enrollments: i32,
    pub enrollments_total: i32,
}

/// One rule leaf's sync state, for the "Published rules" card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSync {
    pub module_key: String,
    pub name: String,
    pub rule_type: Option<String>,
    /// `never` (never published), `drifted` (a newer draft exists) or `in_sync`.
    pub state: String,
    pub draft_version: Option<i32>,
    pub published_version: Option<i32>,
}

pub fn rule_state(draft_version: Option<i32>, published_version: Option<i32>) -> &'static str {
    match (draft_version, published_version) {
        (_, None) => "never",
        (Some(draft), Some(published)) if draft > published => "drifted",
        _ => "in_sync",
    }
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub id: Uuid,
    pub program_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub actor: String,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

impl AuditEntry {
    /// The entry's AEP resource path.
    pub fn path(&self) -> String {
        format!("programs/{}/audit-logs/{}", self.program_id, self.id)
    }

    /// The line the Recent activity list shows.
    pub fn summary(&self) -> String {
        let what = match self.target_type.as_str() {
            "program" => "Program",
            other => other,
        };
        if self.target_type == "application" {
            return format!("Application {}", self.action);
        }
        match self.action.as_str() {
            "created" => format!("{what} created"),
            "updated" => format!("{what} updated"),
            "deleted" => format!("{what} deleted"),
            "undeleted" => format!("{what} restored"),
            "values_updated" => "Settings updated".to_string(),
            other => {
                let mut text = other.replace('_', " ");
                if let Some(first) = text.get_mut(0..1) {
                    first.make_ascii_uppercase();
                }
                text
            }
        }
    }
}

/// Where a page of audit entries resumes: just after this entry, newest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditCursor {
    pub create_time: DateTime<Utc>,
    pub id: Uuid,
}

impl AuditCursor {
    pub fn of(entry: &AuditEntry) -> Self {
        Self { create_time: entry.create_time, id: entry.id }
    }

    /// `<microseconds since the epoch>.<entry id>`: both parts are needed because the log is
    /// ordered, and partitioned, by time.
    pub fn encode(&self) -> String {
        format!("{}.{}", self.create_time.timestamp_micros(), self.id)
    }

    pub fn decode(token: &str) -> Result<Self, ApplicationError> {
        let invalid = || ApplicationError::validation_error("page_token is not valid");
        let (micros, id) = token.trim().split_once('.').ok_or_else(invalid)?;
        let create_time = micros
            .parse::<i64>()
            .ok()
            .and_then(DateTime::from_timestamp_micros)
            .ok_or_else(invalid)?;
        let id = Uuid::parse_str(id).map_err(|_| invalid())?;
        Ok(Self { create_time, id })
    }
}

#[derive(Debug, Clone)]
pub struct Overview {
    pub stats: ProgramStats,
    /// Applications by workflow status, every status present (see `FUNNEL_STATUSES`).
    pub funnel: Vec<(String, i64)>,
    pub rules: Vec<RuleSync>,
    /// The `config_version` of the latest publication, if there is one.
    pub published_config_version: Option<i32>,
    /// The latest few audit entries, newest first.
    pub recent_activity: Vec<AuditEntry>,
}

#[async_trait]
pub trait ProgramOverviewRepository: Send + Sync {
    /// Everything the Overview shows, in ONE database round trip.
    async fn get(&self, program_id: Uuid, recent_activity: i64) -> Result<Overview, ApplicationError>;
}

#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// A program's audit entries, newest first, `page_size` at a time after `cursor`. Returns the
    /// cursor of the next page when there is one. `actions` narrows the list to those actions.
    async fn list(
        &self,
        program_id: Uuid,
        page_size: i64,
        cursor: Option<AuditCursor>,
        actions: &[String],
    ) -> Result<(Vec<AuditEntry>, Option<AuditCursor>), ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn entry(action: &str, target_type: &str) -> AuditEntry {
        AuditEntry {
            id: Uuid::new_v4(),
            program_id: Uuid::new_v4(),
            create_time: Utc.with_ymd_and_hms(2026, 10, 16, 8, 30, 15).unwrap() + chrono::Duration::microseconds(123_456),
            actor: "divyesh".into(),
            action: action.into(),
            target_type: target_type.into(),
            target_id: "x".into(),
            before: None,
            after: None,
        }
    }

    #[test]
    fn rule_states() {
        assert_eq!(rule_state(None, None), "never");
        assert_eq!(rule_state(Some(3), None), "never");
        assert_eq!(rule_state(Some(3), Some(3)), "in_sync");
        assert_eq!(rule_state(Some(4), Some(3)), "drifted");
        assert_eq!(rule_state(None, Some(3)), "in_sync");
        assert_eq!(rule_state(Some(2), Some(3)), "in_sync");
    }

    #[test]
    fn summaries_read_like_the_screen() {
        assert_eq!(entry("created", "program").summary(), "Program created");
        assert_eq!(entry("submitted", "application").summary(), "Application submitted");
        assert_eq!(entry("updated", "program").summary(), "Program updated");
        assert_eq!(entry("deleted", "program").summary(), "Program deleted");
        assert_eq!(entry("undeleted", "program").summary(), "Program restored");
        assert_eq!(entry("values_updated", "program").summary(), "Settings updated");
        assert_eq!(entry("report_generated", "report").summary(), "Report generated");
    }

    #[test]
    fn a_cursor_round_trips_exactly() {
        let e = entry("created", "program");
        let cursor = AuditCursor::of(&e);
        let decoded = AuditCursor::decode(&cursor.encode()).unwrap();
        assert_eq!(decoded, cursor, "microsecond precision must survive");
    }

    #[test]
    fn a_bad_cursor_is_rejected() {
        for bad in ["", "abc", "123", "123.", ".abc", "x.00000000-0000-0000-0000-000000000000", "99999999999999999999.00000000-0000-0000-0000-000000000000"] {
            assert!(AuditCursor::decode(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_funnel_lists_every_status_once() {
        let mut seen = std::collections::HashSet::new();
        assert!(FUNNEL_STATUSES.iter().all(|s| seen.insert(*s)));
        assert_eq!(FUNNEL_STATUSES.len(), 6);
    }
}
