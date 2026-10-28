// domain/program.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;
use crate::domain::aep::{ResourceIdentifier, ResourceState};
use crate::domain::program_module::ProgramModule;
use crate::domain::program_setting::ModuleChange;

/// The customer-hierarchy grain a program is anchored at.
pub const PROGRAM_LEVELS: [&str; 3] = ["user", "organization", "location"];
pub const DEFAULT_PROGRAM_LEVEL: &str = "location";

/// A program (e.g. KBB, CPO, Credit Line). `Serialize`/`Deserialize` are for a
/// future cache, not an API response shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub id: Uuid,                   // OUTPUT_ONLY
    pub name: String,
    pub slug: String,               // IMMUTABLE
    pub description: Option<String>,
    pub level: String,
    pub active: bool,
    pub config_version: i32,        // OUTPUT_ONLY — bumped on every configuration change
    pub config_hash: String,        // OUTPUT_ONLY
    pub state: ResourceState,       // OUTPUT_ONLY (AEP-216)
    pub create_time: DateTime<Utc>, // OUTPUT_ONLY
    pub update_time: DateTime<Utc>, // OUTPUT_ONLY
}

#[derive(Debug, Clone)]
pub struct NewProgram {
    /// Optional client-specified ID (AEP-133); server generates a UUID v7 if omitted.
    pub id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub level: String,
    /// The modules the admin picked, each optionally with its mode and settings. Ancestors, the
    /// descendants of a picked group, and the default modules (Overview) are added by the
    /// repository.
    pub modules: Vec<ModuleChange>,
    pub actor: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateProgram {
    pub name: Option<String>,
    /// `Some(None)` clears the description.
    pub description: Option<Option<String>>,
    pub level: Option<String>,
    pub active: Option<bool>,
    /// Module changes: a new key adds the module, `mode` sets what a leaf is, `settings`
    /// replaces its settings, `enabled: false` switches it off. `None` touches no module.
    pub modules: Option<Vec<ModuleChange>>,
    pub actor: String,
}

/// `slug` rule: lowercase letters and digits separated by single hyphens.
pub fn validate_slug(slug: &str) -> Result<(), ApplicationError> {
    let ok = !slug.is_empty()
        && slug.len() <= 64
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--")
        && slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

    if !ok {
        return Err(ApplicationError::unprocessable_entity(&format!(
            "slug {slug:?} must be 1-64 lowercase letters, digits and single hyphens"
        )));
    }

    // `GET /programs/{program}` takes an id OR a slug: a slug that parses as a UUID
    // would be read as an id and never found by slug.
    if Uuid::parse_str(slug).is_ok() {
        return Err(ApplicationError::unprocessable_entity(
            "slug must not look like a UUID (it would be read as an id)",
        ));
    }

    Ok(())
}

pub fn validate_level(level: &str) -> Result<(), ApplicationError> {
    if PROGRAM_LEVELS.contains(&level) {
        Ok(())
    } else {
        Err(ApplicationError::unprocessable_entity(&format!(
            "level {level:?} must be one of: {}",
            PROGRAM_LEVELS.join(", ")
        )))
    }
}

#[async_trait]
pub trait ProgramRepository: Send + Sync {
    /// Accepts the id or the slug. Deleted programs are only found with
    /// `include_deleted` (Undelete needs them).
    async fn get(&self, identifier: &ResourceIdentifier, include_deleted: bool) -> Result<Program, ApplicationError>;

    /// `show_deleted` toggles between active-only (false) and deleted-only
    /// (true); `show_archived` between the working catalog (false: archived programs
    /// left out) and archived-only (true). The returned `i64` is `total_size` for the
    /// same filter.
    async fn list(
        &self,
        page_size: i64,
        page_token: Option<Uuid>,
        show_deleted: bool,
        show_archived: bool,
    ) -> Result<(Vec<Program>, Option<Uuid>, i64), ApplicationError>;

    /// ONE transaction: the program, its modules, the Overview counters row,
    /// the audit entry and the outbox event.
    async fn create(&self, input: NewProgram) -> Result<(Program, Vec<ProgramModule>), ApplicationError>;

    /// One transaction with the audit entry and outbox event. When `input.modules` is given,
    /// the module manifest after the change comes back too.
    async fn update(
        &self,
        id: Uuid,
        input: UpdateProgram,
    ) -> Result<(Program, Option<Vec<ProgramModule>>), ApplicationError>;

    /// Soft delete (state -> DELETED).
    async fn delete(&self, id: Uuid, actor: &str) -> Result<(), ApplicationError>;

    /// Reverses a soft delete.
    async fn undelete(&self, id: Uuid, actor: &str) -> Result<Program, ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_slugs() {
        for slug in ["kbb", "cl1-isp", "a", "pre-owned-devices", "x9", &"a".repeat(64)] {
            assert!(validate_slug(slug).is_ok(), "{slug} should be valid");
        }
    }

    #[test]
    fn rejects_invalid_slugs() {
        for slug in ["", "01a14355-dfba-7716-8068-d62f02de7088", "KBB", "Bad Slug", "-kbb", "kbb-", "kb--b", "kb_b", "kbb!", "ünï", &"a".repeat(65)] {
            assert!(validate_slug(slug).is_err(), "{slug:?} should be invalid");
        }
    }

    #[test]
    fn accepts_only_known_levels() {
        for level in PROGRAM_LEVELS {
            assert!(validate_level(level).is_ok());
        }
        assert!(validate_level("galaxy").is_err());
        assert!(validate_level("").is_err());
        assert!(validate_level("Location").is_err());
    }

    #[test]
    fn default_level_is_valid() {
        assert!(validate_level(DEFAULT_PROGRAM_LEVEL).is_ok());
    }
}
