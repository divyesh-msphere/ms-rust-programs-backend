use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::db_error::db_error;
use crate::adapters::services::postgres::program::program_repository::record_change;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::notifications::{NotificationTrigger, Notifications, NotificationsRepository};
use async_trait::async_trait;
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use std::sync::Arc;
use uuid::Uuid;

pub struct NotificationsRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl NotificationsRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

#[derive(Debug, FromRow)]
struct TriggerRow {
    key: String,
    label: String,
    email: bool,
    sms: bool,
    push: bool,
    in_app: bool,
}

impl From<TriggerRow> for NotificationTrigger {
    fn from(row: TriggerRow) -> Self {
        Self { key: row.key, label: row.label, email: row.email, sms: row.sms, push: row.push, in_app: row.in_app }
    }
}

fn to_json(triggers: &[NotificationTrigger]) -> Value {
    Value::Array(
        triggers
            .iter()
            .map(|t| json!({ "key": t.key, "label": t.label, "email": t.email, "sms": t.sms, "push": t.push, "in_app": t.in_app }))
            .collect(),
    )
}

/// The page of a live program that has the module. `FOR UPDATE` when a change follows, so two admins
/// cannot interleave; the program row is the lock, as for the setting values.
async fn load(conn: &mut PgConnection, program_id: Uuid, lock: bool) -> Result<Notifications, ApplicationError> {
    let query = format!(
        "SELECT p.config_version FROM programs p WHERE p.id = $1 AND p.state = 'ACTIVE' \
           AND EXISTS (SELECT 1 FROM program_modules pm JOIN module_definitions md ON md.id = pm.module_id \
                       WHERE pm.program_id = p.id AND md.handler_key = 'notifications'){}",
        if lock { " FOR UPDATE OF p" } else { "" }
    );
    let version: i32 = sqlx::query_scalar(&query)
        .bind(program_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(db_error)?
        .ok_or_else(|| ApplicationError::not_found("Notifications of program", program_id))?;
    let rows = sqlx::query_as::<_, TriggerRow>(
        "SELECT key, label, email, sms, push, in_app FROM program_notification_triggers \
         WHERE program_id = $1 ORDER BY position, key",
    )
    .bind(program_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(db_error)?;
    Ok(Notifications { program_id, version, triggers: rows.into_iter().map(Into::into).collect() })
}

#[async_trait]
impl NotificationsRepository for NotificationsRepositoryPostgresImpl {
    async fn get(&self, program_id: Uuid) -> Result<Notifications, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        load(&mut conn, program_id, false).await
    }

    async fn replace(
        &self,
        program_id: Uuid,
        expected_version: Option<i32>,
        triggers: Vec<NotificationTrigger>,
        actor: &str,
    ) -> Result<Notifications, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = load(&mut tx, program_id, true).await?;

        if let Some(expected) = expected_version
            && expected != current.version
        {
            return Err(ApplicationError::conflict(&format!(
                "the program changed since you loaded it (you have version {expected}, it is now {}); reload and try again",
                current.version
            )));
        }

        // Saving the grid as it already is changes nothing, bumps nothing.
        if current.triggers == triggers {
            tx.rollback().await.map_err(db_error)?;
            return Ok(current);
        }

        let keys: Vec<&str> = triggers.iter().map(|t| t.key.as_str()).collect();
        sqlx::query("DELETE FROM program_notification_triggers WHERE program_id = $1 AND NOT (key = ANY($2))")
            .bind(program_id)
            .bind(&keys)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        for (position, trigger) in triggers.iter().enumerate() {
            sqlx::query(
                "INSERT INTO program_notification_triggers (program_id, key, label, email, sms, push, in_app, position) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
                 ON CONFLICT (program_id, key) DO UPDATE SET label = EXCLUDED.label, email = EXCLUDED.email, \
                    sms = EXCLUDED.sms, push = EXCLUDED.push, in_app = EXCLUDED.in_app, \
                    position = EXCLUDED.position, update_time = now()",
            )
            .bind(program_id)
            .bind(&trigger.key)
            .bind(&trigger.label)
            .bind(trigger.email)
            .bind(trigger.sms)
            .bind(trigger.push)
            .bind(trigger.in_app)
            .bind(i32::try_from(position).unwrap_or(i32::MAX))
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        }

        // The version drives the "unpublished changes" check, as for the setting values.
        let version = current.version + 1;
        sqlx::query(
            "UPDATE programs SET config_version = $2, config_hash = md5(id::text || ':' || $2::text), update_time = now() WHERE id = $1",
        )
        .bind(program_id)
        .bind(version)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        record_change(
            &mut tx,
            program_id,
            actor,
            "notifications_updated",
            Some(json!({ "triggers": to_json(&current.triggers) })),
            Some(json!({ "triggers": to_json(&triggers) })),
        )
        .await
        .map_err(db_error)?;

        tx.commit().await.map_err(db_error)?;
        Ok(Notifications { program_id, version, triggers })
    }
}
