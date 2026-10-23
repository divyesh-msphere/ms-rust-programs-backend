use crate::dto::presenters::json::program::ProgramPresenter;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct StatsPresenter {
    pub total_applications: i32,
    pub pending_approvals: i32,
    pub active_enrollments: i32,
    pub enrollments_total: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct FunnelEntryPresenter {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct RuleFamilyPresenter {
    pub module_key: String,
    pub name: String,
    pub rule_type: Option<String>,
    /// `never`, `drifted` or `in_sync`.
    pub state: String,
    pub draft_version: Option<i32>,
    pub published_version: Option<i32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PublishedRulesPresenter {
    pub total: usize,
    pub in_sync: usize,
    /// At least one rule is not in sync with what is live.
    pub drift_detected: bool,
    pub families: Vec<RuleFamilyPresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AuditEntryPresenter {
    pub path: String,
    pub id: String,
    pub create_time: String,
    pub actor: String,
    pub action: String,
    /// The line a history list shows, e.g. "Settings updated".
    pub summary: String,
    pub target_type: String,
    pub target_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<serde_json::Value>,
}

/// The default Overview page every program has.
#[derive(Debug, Serialize, Clone)]
pub struct OverviewPresenter {
    pub program: ProgramPresenter,
    pub config_version: i32,
    /// The `config_version` of the latest publication; null when never published.
    pub published_config_version: Option<i32>,
    pub unpublished_changes: bool,
    pub stats: StatsPresenter,
    /// Applications by workflow status; every status is listed.
    pub funnel: Vec<FunnelEntryPresenter>,
    pub published_rules: PublishedRulesPresenter,
    pub recent_activity: Vec<AuditEntryPresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ListAuditLogsPresenter {
    pub results: Vec<AuditEntryPresenter>,
    pub next_page_token: Option<String>,
}
