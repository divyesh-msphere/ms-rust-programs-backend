use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct ModuleDefinitionDbModel {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub kind: String,
    pub parent_id: Option<Uuid>,
    /// `ancestry::text` — sqlx has no `ltree` type, so it is cast in the SELECT.
    pub ancestry: String,
    pub position: i32,
    pub ui_component: Option<String>,
    pub rule_family: Option<String>,
    pub universal: bool,
    pub requires_publish: bool,
    pub permission_key: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
