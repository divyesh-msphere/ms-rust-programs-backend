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
    /// The input the rule reads: a list of `{name, type, required, description?}`. Execute checks the
    /// caller's input against it; Publish refuses a rule that has none.
    pub input_schema: Option<Value>,
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
    pub input_schema: Option<Value>,
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
    if let Some(schema) = &input.input_schema {
        parse_input_schema(schema)?;
    }
    if let Some(rule_type) = &input.rule_type
        && (rule_type.trim().is_empty() || rule_type.chars().count() > 64)
    {
        return Err(ApplicationError::unprocessable_entity("rule_type must be 1-64 characters"));
    }
    Ok(())
}

const MAX_INPUT_FIELDS: usize = 100;
const INPUT_TYPES: [&str; 6] = ["string", "number", "boolean", "object", "array", "any"];

/// One field of the input a rule reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputField {
    /// The field's name; `a.b` is the field `b` inside the object `a`.
    pub name: String,
    /// `string`, `number`, `boolean`, `object`, `array` or `any`.
    pub field_type: String,
    pub required: bool,
}

/// Reads a rule's `input_schema`: a list of `{name, type, required?, description?}`.
pub fn parse_input_schema(schema: &Value) -> Result<Vec<InputField>, ApplicationError> {
    let list = schema
        .as_array()
        .ok_or_else(|| ApplicationError::unprocessable_entity("input_schema must be a list of fields"))?;
    if list.len() > MAX_INPUT_FIELDS {
        return Err(ApplicationError::unprocessable_entity(&format!(
            "input_schema must have at most {MAX_INPUT_FIELDS} fields"
        )));
    }
    let mut fields: Vec<InputField> = Vec::with_capacity(list.len());
    for (i, item) in list.iter().enumerate() {
        let at = |what: &str| ApplicationError::unprocessable_entity(&format!("input_schema[{i}]: {what}"));
        let object = item.as_object().ok_or_else(|| at("must be an object {name, type, required}"))?;
        let name = object.get("name").and_then(Value::as_str).map(str::trim).unwrap_or_default();
        if name.is_empty()
            || name.chars().count() > 100
            || name.split('.').any(str::is_empty)
            || !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
        {
            return Err(at("name must be 1-100 letters, digits, _ - or . (a.b is the field b inside a)"));
        }
        let field_type = object.get("type").and_then(Value::as_str).unwrap_or("any");
        if !INPUT_TYPES.contains(&field_type) {
            return Err(at(&format!("type must be one of {}", INPUT_TYPES.join(", "))));
        }
        let required = match object.get("required") {
            None => false,
            Some(Value::Bool(b)) => *b,
            Some(_) => return Err(at("required must be true or false")),
        };
        if fields.iter().any(|f| f.name == name) {
            return Err(at(&format!("{name:?} appears twice")));
        }
        fields.push(InputField { name: name.to_string(), field_type: field_type.to_string(), required });
    }
    Ok(fields)
}

/// How an input differs from what the rule reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputMismatch {
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    /// Required fields that are absent (or null).
    pub missing: Vec<String>,
    /// Fields present with the wrong type: `(field, expected, got)`.
    pub wrong_type: Vec<(String, String, String)>,
}

impl InputMismatch {
    /// One sentence that names everything the caller needs (gRPC carries only this).
    pub fn message(&self) -> String {
        let mut parts = Vec::new();
        if !self.missing.is_empty() {
            parts.push(format!("missing: {}", self.missing.join(", ")));
        }
        for (field, expected, got) in &self.wrong_type {
            parts.push(format!("{field} must be {expected}, got {got}"));
        }
        format!(
            "input does not match the rule ({}); required fields: {}",
            parts.join("; "),
            self.required_fields.join(", ")
        )
    }
}

fn type_of(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn lookup<'a>(input: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(input, |value, key| value.get(key)).filter(|v| !v.is_null())
}

/// Checks the caller's input against the fields a rule reads: every required field is there, and
/// every field that is there (required or not) has its type. Fields the rule does not list are fine.
pub fn check_input(fields: &[InputField], input: &Value) -> Result<(), InputMismatch> {
    let mut mismatch = InputMismatch {
        required_fields: fields.iter().filter(|f| f.required).map(|f| f.name.clone()).collect(),
        optional_fields: fields.iter().filter(|f| !f.required).map(|f| f.name.clone()).collect(),
        missing: Vec::new(),
        wrong_type: Vec::new(),
    };
    for field in fields {
        match lookup(input, &field.name) {
            None if field.required => mismatch.missing.push(field.name.clone()),
            None => {}
            Some(value) if field.field_type != "any" && type_of(value) != field.field_type => {
                mismatch.wrong_type.push((field.name.clone(), field.field_type.clone(), type_of(value).to_string()));
            }
            Some(_) => {}
        }
    }
    if mismatch.missing.is_empty() && mismatch.wrong_type.is_empty() { Ok(()) } else { Err(mismatch) }
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
            input_schema: None,
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

    fn schema() -> Vec<InputField> {
        parse_input_schema(&json!([
            {"name": "store_id", "type": "string", "required": true},
            {"name": "credit_score", "type": "number", "required": true},
            {"name": "store.region", "type": "string"},
            {"name": "notes"}
        ]))
        .unwrap()
    }

    #[test]
    fn a_schema_is_a_list_of_named_typed_fields() {
        assert_eq!(schema().len(), 4);
        assert_eq!(schema()[3], InputField { name: "notes".into(), field_type: "any".into(), required: false });
        for bad in [
            json!({"a": 1}),
            json!([1]),
            json!([{"type": "string"}]),
            json!([{"name": "a b", "type": "string"}]),
            json!([{"name": "a..b", "type": "string"}]),
            json!([{"name": "a", "type": "text"}]),
            json!([{"name": "a", "required": "yes"}]),
            json!([{"name": "a"}, {"name": "a"}]),
        ] {
            assert!(parse_input_schema(&bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_new_version_with_a_bad_schema_is_rejected() {
        let mut v = new_version();
        v.input_schema = Some(json!("nope"));
        assert!(validate_new(&v).is_err());
        v.input_schema = Some(json!([{"name": "a", "type": "string", "required": true}]));
        assert!(validate_new(&v).is_ok());
    }

    #[test]
    fn a_fitting_input_passes_and_extra_fields_are_fine() {
        let input = json!({"store_id": "S1", "credit_score": 710, "store": {"region": "west"}, "extra": true});
        assert_eq!(check_input(&schema(), &input), Ok(()));
        assert_eq!(check_input(&schema(), &json!({"store_id": "S1", "credit_score": 1})), Ok(()), "optional fields may be absent");
    }

    #[test]
    fn missing_and_null_required_fields_are_listed() {
        let err = check_input(&schema(), &json!({"store_id": null})).unwrap_err();
        assert_eq!(err.missing, vec!["store_id", "credit_score"]);
        assert_eq!(err.required_fields, vec!["store_id", "credit_score"]);
        assert_eq!(err.optional_fields, vec!["store.region", "notes"]);
        assert!(err.message().contains("missing: store_id, credit_score"));
        assert!(err.message().contains("required fields: store_id, credit_score"));
    }

    #[test]
    fn wrong_types_are_named_with_what_was_expected() {
        let err = check_input(&schema(), &json!({"store_id": 5, "credit_score": "high", "store": {"region": 1}})).unwrap_err();
        assert_eq!(
            err.wrong_type,
            vec![
                ("store_id".to_string(), "string".to_string(), "number".to_string()),
                ("credit_score".to_string(), "number".to_string(), "string".to_string()),
                ("store.region".to_string(), "string".to_string(), "number".to_string()),
            ]
        );
        assert!(err.missing.is_empty());
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
