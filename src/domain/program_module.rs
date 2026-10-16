// domain/program_module.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;
use crate::domain::program_setting::{ProgramRule, ProgramSetting};

/// A module a program has chosen, joined with its registry row, so one read
/// gives the frontend everything it needs to draw the page (the "manifest"):
/// `source` says how to render it, `api_base` which API to call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramModule {
    pub module_id: Uuid,
    pub key: String,
    pub name: String,
    pub kind: String,                 // "group" | "leaf"
    pub parent_id: Option<Uuid>,
    pub ancestry: String,
    pub source: String,               // "premade" | "container" | "default"
    pub handler_key: Option<String>,
    pub api_base: Option<String>,
    pub ui_component: Option<String>,
    pub allow_components: bool,
    pub requires_publish: bool,
    pub enabled: bool,
    /// What the admin made this leaf: `rule`, `settings`, or `None` until chosen
    /// (and always `None` on a module that is not configurable).
    pub mode: Option<String>,
    /// The leaf's settings, when its mode is `settings` (empty otherwise), in display order.
    pub settings: Vec<ProgramSetting>,
    /// The leaf's rule-set summary, when its mode is `rule`.
    pub rule: Option<ProgramRule>,
    pub position: i32,
    pub update_time: DateTime<Utc>,
}

#[async_trait]
pub trait ProgramModuleRepository: Send + Sync {
    /// Every module of the program, enabled or not, parents before children
    /// (shallowest first), then by the program's own `position`.
    async fn list(&self, program_id: Uuid, enabled_only: bool) -> Result<Vec<ProgramModule>, ApplicationError>;

    /// Every global-settings library some live setting reads its options from (used at startup
    /// to warm the library cache, so the first page load is as fast as the rest).
    async fn library_ids_in_use(&self) -> Result<Vec<Uuid>, ApplicationError>;
}
