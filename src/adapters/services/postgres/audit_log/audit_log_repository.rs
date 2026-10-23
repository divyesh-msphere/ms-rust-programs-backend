use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::program_overview::{AuditCursor, AuditEntry, AuditLogRepository};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ms_telemetry_rs::tracing::error;
use serde_json::Value;
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct AuditRow {
    id: Uuid,
    create_time: DateTime<Utc>,
    actor: String,
    action: String,
    target_type: String,
    target_id: String,
    before: Option<Value>,
    after: Option<Value>,
}

pub struct AuditLogRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl AuditLogRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

#[async_trait]
impl AuditLogRepository for AuditLogRepositoryPostgresImpl {
    async fn list(
        &self,
        program_id: Uuid,
        page_size: i64,
        cursor: Option<AuditCursor>,
        actions: &[String],
    ) -> Result<(Vec<AuditEntry>, Option<AuditCursor>), ApplicationError> {
        // Newest first, on the index (program_id, create_time DESC, id DESC). The keyset condition
        // is a row comparison on exactly that order, so a page costs the same however deep it is,
        // and the table is partitioned by month, so only the months touched are read.
        let mut rows = sqlx::query_as::<_, AuditRow>(
            "SELECT id, create_time, actor, action, target_type, target_id, before, after \
             FROM audit_log \
             WHERE program_id = $1 \
               AND ($2::timestamptz IS NULL OR (create_time, id) < ($2, $3)) \
               AND (cardinality($4::text[]) = 0 OR action = ANY($4)) \
             ORDER BY create_time DESC, id DESC \
             LIMIT $5",
        )
        .bind(program_id)
        .bind(cursor.map(|c| c.create_time))
        .bind(cursor.map(|c| c.id))
        .bind(actions)
        .bind(page_size + 1)
        .fetch_all(self.db_connection.get_pool())
        .await
        .map_err(|err| {
            error!("DB error: {:?}", err);
            ApplicationError::internal_error(Some(err))
        })?;

        let next = if rows.len() as i64 > page_size {
            rows.pop();
            rows.last().map(|row| AuditCursor { create_time: row.create_time, id: row.id })
        } else {
            None
        };

        let entries = rows
            .into_iter()
            .map(|row| AuditEntry {
                id: row.id,
                program_id,
                create_time: row.create_time,
                actor: row.actor,
                action: row.action,
                target_type: row.target_type,
                target_id: row.target_id,
                before: row.before,
                after: row.after,
            })
            .collect();
        Ok((entries, next))
    }
}
