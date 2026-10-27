// domain/rule_set.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program_overview::rule_state;

/// One version's definition and compiled graph are each kept under this size.
pub const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;
const MAX_VALIDATION_ERRORS: usize = 100;

pub const DRAFT: &str = "draft";
pub const PUBLISHED: &str = "published";
pub const SUPERSEDED: &str = "superseded";

/// The rules of one Rule leaf (a module whose mode is `rule`). The rule engine itself is a frontend
/// component: the backend keeps what it saves, versioned, and publishes it. A leaf has a rule set
/// only after its first save.
#[derive(Debug, Clone)]
pub struct RuleSet {
    pub program_id: Uuid,
    pub module_id: Uuid,
    /// The leaf's module key, for example `catalog_pricing.products_rules.warranty`. It is how a
    /// rule set is addressed.
    pub module_key: String,
    pub module_name: String,
    pub rule_set_id: Option<Uuid>,
    /// Informational; defaults from the module's rule family.
    pub rule_type: Option<String>,
    pub draft_version: Option<i32>,
    pub published_version: Option<i32>,
    /// The newest saved version with its definition (only on a single read).
    pub draft: Option<RuleVersion>,
    pub update_time: Option<DateTime<Utc>>,
}

impl RuleSet {
    /// `empty` (no rules saved yet), `never` published, `drifted` (a newer draft than what is live) or `in_sync`.
    pub fn state(&self) -> &'static str {
        rule_state(self.draft_version, self.published_version)
    }
}

/// An immutable, numbered save of a rule set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersion {
    pub id: Uuid,
    pub program_id: Uuid,
    pub module_key: String,
    pub version: i32,
    /// `draft` (the newest save), `published` (live) or `superseded`.
    pub status: String,
    /// The rule model the builder draws. Only on a single read.
    pub definition: Option<Value>,
    /// The generated decision graph. Only on a single read.
    pub compiled: Option<Value>,
    /// Whether the rule engine found the definition complete. An invalid draft blocks publishing.
    pub valid: bool,
    pub validation_errors: Value,
    pub content_hash: String,
    pub size_bytes: i32,
    pub created_by: Option<String>,
    pub create_time: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewRuleVersion {
    pub program_id: Uuid,
    /// The leaf: its module key (or the id of its rule set).
    pub rule_set: ResourceIdentifier,
    pub rule_type: Option<String>,
    pub definition: Value,
    pub compiled: Option<Value>,
    pub valid: bool,
    pub validation_errors: Value,
    pub actor: String,
}

/// What a save did: a new version, or none because nothing changed.
#[derive(Debug, Clone)]
pub struct SaveOutcome {
    pub version: RuleVersion,
    pub created: bool,
}

fn validate_document(name: &str, value: &Value) -> Result<(), ApplicationError> {
    if !value.is_object() {
        return Err(ApplicationError::unprocessable_entity(&format!("{name} must be a JSON object")));
    }
    if value.to_string().len() > MAX_DOCUMENT_BYTES {
        return Err(ApplicationError::unprocessable_entity(&format!("{name} must be at most 1 MB")));
    }
    Ok(())
}

pub fn validate_new(input: &NewRuleVersion) -> Result<(), ApplicationError> {
    validate_document("definition", &input.definition)?;
    if let Some(compiled) = &input.compiled {
        validate_document("compiled", compiled)?;
    }
    let errors = input
        .validation_errors
        .as_array()
        .ok_or_else(|| ApplicationError::unprocessable_entity("validation_errors must be a list"))?;
    if errors.len() > MAX_VALIDATION_ERRORS {
        return Err(ApplicationError::unprocessable_entity(&format!(
            "validation_errors must have at most {MAX_VALIDATION_ERRORS} entries"
        )));
    }
    if input.valid && !errors.is_empty() {
        return Err(ApplicationError::unprocessable_entity("a version with validation_errors cannot be valid"));
    }
    if let Some(rule_type) = &input.rule_type
        && (rule_type.trim().is_empty() || rule_type.chars().count() > 64)
    {
        return Err(ApplicationError::unprocessable_entity("rule_type must be 1-64 characters"));
    }
    Ok(())
}

#[async_trait]
pub trait RuleSetRepository: Send + Sync {
    /// The program's rule leaves (enabled, mode `rule`), each with its rule set if it has one.
    async fn list(&self, program_id: Uuid) -> Result<Vec<RuleSet>, ApplicationError>;

    /// One leaf with its newest version in full. A leaf that was never saved is returned with no
    /// rule set (`not_found` only when the module is not a rule leaf of the program).
    async fn get(&self, program_id: Uuid, rule_set: &ResourceIdentifier) -> Result<RuleSet, ApplicationError>;

    /// Saves a new version. ONE transaction: the version, the rule set pointer, the program's
    /// version bump (rules are program configuration), the audit entry and the outbox event.
    /// Saving what is already the newest version (same definition and validity) creates nothing.
    async fn save_version(&self, input: NewRuleVersion) -> Result<SaveOutcome, ApplicationError>;

    /// Newest first, keyset on the id. Rows carry neither definition nor compiled graph.
    async fn list_versions(
        &self,
        program_id: Uuid,
        rule_set: &ResourceIdentifier,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<RuleVersion>, Option<Uuid>), ApplicationError>;

    async fn get_version(&self, program_id: Uuid, rule_set: &ResourceIdentifier, version: i32) -> Result<RuleVersion, ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn new_version() -> NewRuleVersion {
        NewRuleVersion {
            program_id: Uuid::nil(),
            rule_set: ResourceIdentifier::Name("catalog_pricing.products_rules.warranty".into()),
            rule_type: None,
            definition: json!({"rules": []}),
            compiled: None,
            valid: true,
            validation_errors: json!([]),
            actor: "t".into(),
        }
    }

    fn rule_set(draft: Option<i32>, published: Option<i32>) -> RuleSet {
        RuleSet {
            program_id: Uuid::nil(),
            module_id: Uuid::nil(),
            module_key: "k".into(),
            module_name: "K".into(),
            rule_set_id: None,
            rule_type: None,
            draft_version: draft,
            published_version: published,
            draft: None,
            update_time: None,
        }
    }

    #[test]
    fn state_follows_draft_and_published() {
        assert_eq!(rule_set(None, None).state(), "empty");
        assert_eq!(rule_set(Some(1), None).state(), "never");
        assert_eq!(rule_set(Some(2), Some(1)).state(), "drifted");
        assert_eq!(rule_set(Some(2), Some(2)).state(), "in_sync");
    }

    #[test]
    fn a_normal_version_is_accepted() {
        assert!(validate_new(&new_version()).is_ok());
        let mut v = new_version();
        v.compiled = Some(json!({"nodes": []}));
        v.rule_type = Some("warranty".into());
        assert!(validate_new(&v).is_ok());
    }

    #[test]
    fn definition_and_compiled_must_be_small_objects() {
        let mut v = new_version();
        v.definition = json!([1]);
        assert!(validate_new(&v).is_err());
        let mut v = new_version();
        v.definition = json!({"big": "a".repeat(MAX_DOCUMENT_BYTES)});
        assert!(validate_new(&v).is_err());
        let mut v = new_version();
        v.compiled = Some(json!("text"));
        assert!(validate_new(&v).is_err());
    }

    #[test]
    fn validation_errors_must_agree_with_valid() {
        let mut v = new_version();
        v.validation_errors = json!("nope");
        assert!(validate_new(&v).is_err());
        let mut v = new_version();
        v.validation_errors = json!([{"rule": "w1", "message": "no value"}]);
        assert!(validate_new(&v).is_err(), "cannot be valid with errors");
        v.valid = false;
        assert!(validate_new(&v).is_ok(), "an incomplete draft can be saved");
        let mut v = new_version();
        v.valid = false;
        v.validation_errors = json!((0..101).map(|i| json!(i)).collect::<Vec<_>>());
        assert!(validate_new(&v).is_err());
    }

    #[test]
    fn rule_type_is_bounded() {
        let mut v = new_version();
        v.rule_type = Some(" ".into());
        assert!(validate_new(&v).is_err());
        v.rule_type = Some("t".repeat(65));
        assert!(validate_new(&v).is_err());
    }
}
