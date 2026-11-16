use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct ProgramDbModel {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub about: Option<String>,
    pub highlights: Vec<String>,
    pub brand_color: Option<String>,
    pub text_color: Option<String>,
    pub logo_icon: Option<String>,
    pub renewal_term_months: i32,
    pub renewal_grace_days: i32,
    pub renewal_reminder_days: i32,
    pub auto_renew: bool,
    pub level: String,
    pub active: bool,
    pub config_version: i32,
    pub config_hash: String,
    pub state: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
