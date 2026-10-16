use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct ProgramModuleDbModel {
    pub module_id: Uuid,
    pub key: String,
    pub name: String,
    pub kind: String,
    pub parent_id: Option<Uuid>,
    /// `ancestry::text` — sqlx has no `ltree` type.
    pub ancestry: String,
    pub source: String,
    pub handler_key: Option<String>,
    pub api_base: Option<String>,
    pub ui_component: Option<String>,
    pub allow_components: bool,
    pub requires_publish: bool,
    pub enabled: bool,
    pub mode: Option<String>,
    pub position: i32,
    pub update_time: DateTime<Utc>,
}
