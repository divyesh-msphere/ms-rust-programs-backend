// domain/aep.rs
//
// Cross-cutting AEP primitives shared by every resource, rather than
// duplicated per resource.

use crate::application::errors::application_error::ApplicationError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AEP-216 `state` field, restricted to the two values this API uses
/// (AEP-164's lightweight soft-delete option: a `DELETED` state value
/// instead of `expire_time` + an Undelete custom method).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceState {
    Active,
    Deleted,
}

impl Default for ResourceState {
    /// Lets `#[serde(default)]` fill this in when a client omits `state`
    /// entirely from a request body (it's OUTPUT_ONLY — never actually
    /// written from a client-supplied value).
    fn default() -> Self {
        ResourceState::Active
    }
}

impl ResourceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceState::Active => "ACTIVE",
            ResourceState::Deleted => "DELETED",
        }
    }

    /// Only the app itself ever writes this column, so an unrecognized value
    /// signals a bug, not a validation failure — same trust boundary as
    /// `AttributeDataType::parse` in `domain/library.rs`.
    pub fn parse(value: &str) -> Self {
        match value {
            "DELETED" => ResourceState::Deleted,
            _ => ResourceState::Active,
        }
    }
}

/// AEP-122 `code` format: the human-facing resource id every resource's
/// `code` field documents itself as, but never actually enforced. Rejects
/// spaces and any character outside `^[A-Za-z0-9._-]+$`.
pub fn validate_resource_code(code: &str) -> Result<(), ApplicationError> {
    let ok = !code.is_empty()
        && code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));

    if ok {
        Ok(())
    } else {
        Err(ApplicationError::unprocessable_entity(&format!(
            "code {code:?} must match ^[A-Za-z0-9._-]+$"
        )))
    }
}

/// A Get path segment that accepts either a resource's id or its (unique)
/// name — "whatever the frontend has on hand" — rather than forcing every
/// caller to resolve a name to an id first. Parsing can't fail: anything that
/// isn't a UUID is simply treated as a name.
#[derive(Debug, Clone)]
pub enum ResourceIdentifier {
    Id(Uuid),
    Name(String),
}

impl ResourceIdentifier {
    pub fn parse(raw: &str) -> Self {
        let raw = raw.trim();
        match Uuid::parse_str(raw) {
            Ok(id) => ResourceIdentifier::Id(id),
            Err(_) => ResourceIdentifier::Name(raw.to_string()),
        }
    }
}
