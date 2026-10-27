use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct RuleVersionPresenter {
    pub path: String,
    pub id: String,
    pub version: i32,
    /// `draft`, `published` or `superseded`.
    pub status: String,
    /// Only on a single read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<serde_json::Value>,
    /// Only on a single read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiled: Option<serde_json::Value>,
    pub valid: bool,
    pub validation_errors: serde_json::Value,
    pub content_hash: String,
    pub size_bytes: i32,
    pub created_by: Option<String>,
    pub create_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SaveRuleVersionPresenter {
    #[serde(flatten)]
    pub version: RuleVersionPresenter,
    /// `false` when the save changed nothing, so no new version was made (this is the newest one).
    pub created: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct RuleSetPresenter {
    pub path: String,
    /// The rule set row, created when the leaf is made a rule leaf; `null` only if it does not exist yet.
    pub id: Option<String>,
    pub module_key: String,
    pub module_name: String,
    pub rule_type: Option<String>,
    pub draft_version: Option<i32>,
    pub published_version: Option<i32>,
    /// `empty` (no rules saved yet), `never` published, `drifted` (a newer draft than what is live) or `in_sync`.
    pub state: &'static str,
    /// The newest version with its definition. Only on a single read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<RuleVersionPresenter>,
    pub update_time: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RuleSetsPresenter {
    pub results: Vec<RuleSetPresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RuleVersionsPresenter {
    pub results: Vec<RuleVersionPresenter>,
    pub next_page_token: Option<String>,
}
