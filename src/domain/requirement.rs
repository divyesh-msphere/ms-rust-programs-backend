// domain/requirement.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;

/// The fixed requirement types (one of each per program) and the repeatable one.
pub const FIXED_REQUIREMENT_TYPES: [&str; 4] = ["signup", "card_on_file", "credit_application", "contract"];
pub const CUSTOM_FIELD: &str = "custom_field";

const MAX_CONFIG_BYTES: usize = 16 * 1024;

/// What a store must satisfy before it can be approved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: Uuid,                   // OUTPUT_ONLY
    pub program_id: Uuid,           // OUTPUT_ONLY
    pub requirement_type: String,   // IMMUTABLE
    pub name: String,
    pub config: Value,
    pub position: i32,
    pub create_time: DateTime<Utc>, // OUTPUT_ONLY
    pub update_time: DateTime<Utc>, // OUTPUT_ONLY
}

#[derive(Debug, Clone)]
pub struct NewRequirement {
    /// Optional client-specified ID (AEP-133); server generates a UUID v7 if omitted.
    pub id: Option<Uuid>,
    pub program_id: Uuid,
    pub requirement_type: String,
    pub name: String,
    pub config: Value,
    /// `None` puts it after the last requirement.
    pub position: Option<i32>,
    pub actor: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateRequirement {
    pub name: Option<String>,
    pub config: Option<Value>,
    pub position: Option<i32>,
    pub actor: String,
}

pub fn validate_type(requirement_type: &str) -> Result<(), ApplicationError> {
    if requirement_type == CUSTOM_FIELD || FIXED_REQUIREMENT_TYPES.contains(&requirement_type) {
        Ok(())
    } else {
        Err(ApplicationError::unprocessable_entity(&format!(
            "type {requirement_type:?} must be one of: {}, {CUSTOM_FIELD}",
            FIXED_REQUIREMENT_TYPES.join(", ")
        )))
    }
}

pub fn validate_name(name: &str) -> Result<(), ApplicationError> {
    if name.trim().is_empty() || name.chars().count() > 120 {
        return Err(ApplicationError::unprocessable_entity("name must be 1-120 characters"));
    }
    Ok(())
}

/// `config` is a JSON object, kept small: it is read with every requirement list.
pub fn validate_config(config: &Value) -> Result<(), ApplicationError> {
    if !config.is_object() {
        return Err(ApplicationError::unprocessable_entity("config must be a JSON object"));
    }
    if config.to_string().len() > MAX_CONFIG_BYTES {
        return Err(ApplicationError::unprocessable_entity("config must be at most 16 KB"));
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
pub trait RequirementRepository: Send + Sync {
    /// In display order. The program must exist (the use case checks).
    async fn list(&self, program_id: Uuid) -> Result<Vec<Requirement>, ApplicationError>;

    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<Requirement, ApplicationError>;

    /// ONE transaction: the requirement, the program's version bump, the audit entry, the outbox
    /// event. A fixed type that already exists is `already_exists`.
    async fn create(&self, input: NewRequirement) -> Result<Requirement, ApplicationError>;

    /// Same transaction shape. A request that changes nothing writes nothing.
    async fn update(&self, program_id: Uuid, id: Uuid, input: UpdateRequirement) -> Result<Requirement, ApplicationError>;

    async fn delete(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<(), ApplicationError>;

    /// Sets many positions in one statement and returns the list in its new order.
    async fn reorder(
        &self,
        program_id: Uuid,
        positions: Vec<(Uuid, i32)>,
        actor: &str,
    ) -> Result<Vec<Requirement>, ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_known_types_only() {
        for t in ["signup", "card_on_file", "credit_application", "contract", "custom_field"] {
            assert!(validate_type(t).is_ok());
        }
        assert!(validate_type("passport").is_err());
        assert!(validate_type("").is_err());
    }

    #[test]
    fn name_must_be_present_and_short() {
        assert!(validate_name("Sign up").is_ok());
        assert!(validate_name("  ").is_err());
        assert!(validate_name(&"a".repeat(121)).is_err());
    }

    #[test]
    fn config_must_be_a_small_object() {
        assert!(validate_config(&json!({})).is_ok());
        assert!(validate_config(&json!({"label": "x"})).is_ok());
        assert!(validate_config(&json!([1])).is_err());
        assert!(validate_config(&json!("x")).is_err());
        assert!(validate_config(&json!({"big": "a".repeat(17 * 1024)})).is_err());
    }

    #[test]
    fn position_is_bounded() {
        assert!(validate_position(0).is_ok());
        assert!(validate_position(10_000).is_ok());
        assert!(validate_position(-1).is_err());
        assert!(validate_position(10_001).is_err());
    }
}
