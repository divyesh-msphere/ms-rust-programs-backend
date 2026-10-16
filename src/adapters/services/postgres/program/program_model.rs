use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct ProgramDbModel {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub level: String,
    pub active: bool,
    pub config_version: i32,
    pub config_hash: String,
    pub state: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
