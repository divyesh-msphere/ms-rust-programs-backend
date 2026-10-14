// src/adapters/inputs/http_apis/shared/parse.rs
//
// Ids arrive as strings — in a path segment, a query parameter or a body field
// — and have to become `Uuid`s before the domain sees them. These are the two
// spellings of that, both of which name the offending field so a 400 says which
// one to fix.

use crate::application::errors::application_error::ApplicationError;
use uuid::Uuid;

/// A required id.
pub fn uuid(field: &str, raw: &str) -> Result<Uuid, ApplicationError> {
    Uuid::parse_str(raw.trim())
        .map_err(|_| ApplicationError::validation_error(&format!("{field} must be a UUID")))
}

/// An optional id. Absent and empty both mean "not given"; anything else parses
/// or fails.
pub fn optional_uuid(field: &str, raw: Option<&str>) -> Result<Option<Uuid>, ApplicationError> {
    match raw.map(str::trim) {
        None | Some("") => Ok(None),
        Some(value) => uuid(field, value).map(Some),
    }
}
