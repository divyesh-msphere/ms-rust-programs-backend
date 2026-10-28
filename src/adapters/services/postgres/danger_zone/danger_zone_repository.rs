use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::db_error::db_error;
use crate::adapters::services::postgres::program::program_repository::record_change;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::danger_zone::{DangerZone, DangerZoneRepository, DeleteBlockers};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{FromRow, PgConnection};
use std::sync::Arc;
use uuid::Uuid;

pub struct DangerZoneRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl DangerZoneRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

#[derive(Debug, FromRow)]
struct PageRow {
    archive_time: Option<DateTime<Utc>>,
    enrollments: i64,
    applications: i64,
    publications: i64,
    documents: i64,
}

/// The page of a live program. `FOR UPDATE` when a change follows, so two admins cannot interleave.
async fn load(conn: &mut PgConnection, program_id: Uuid, lock: bool) -> Result<DangerZone, ApplicationError> {
    let query = format!(
        "SELECT p.archive_time, \
                (SELECT COUNT(*) FROM enrollments WHERE program_id = p.id) AS enrollments, \
                (SELECT COUNT(*) FROM applications WHERE program_id = p.id) AS applications, \
                (SELECT COUNT(*) FROM publications WHERE program_id = p.id) AS publications, \
                (SELECT COUNT(*) FROM documents WHERE program_id = p.id) AS documents \
         FROM programs p WHERE p.id = $1 AND p.state = 'ACTIVE'{}",
        if lock { " FOR UPDATE OF p" } else { "" }
    );
    let row = sqlx::query_as::<_, PageRow>(&query)
        .bind(program_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(db_error)?
        .ok_or_else(|| ApplicationError::not_found("Program", program_id))?;
    Ok(DangerZone {
        program_id,
        archive_time: row.archive_time,
        blockers: DeleteBlockers {
            enrollments: row.enrollments,
            applications: row.applications,
            publications: row.publications,
            documents: row.documents,
        },
    })
}

#[async_trait]
impl DangerZoneRepository for DangerZoneRepositoryPostgresImpl {
    async fn get(&self, program_id: Uuid) -> Result<DangerZone, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        load(&mut conn, program_id, false).await
    }

    async fn archive(&self, program_id: Uuid, actor: &str) -> Result<DangerZone, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        if load(&mut tx, program_id, true).await?.archived() {
            return Err(ApplicationError::validation_error("program is already archived"));
        }
        sqlx::query("UPDATE programs SET archive_time = now(), update_time = now() WHERE id = $1")
            .bind(program_id)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        record_change(&mut tx, program_id, actor, "archived", Some(json!({ "archived": false })), Some(json!({ "archived": true })))
            .await
            .map_err(db_error)?;
        let page = load(&mut tx, program_id, false).await?;
        tx.commit().await.map_err(db_error)?;
        Ok(page)
    }

    async fn unarchive(&self, program_id: Uuid, actor: &str) -> Result<DangerZone, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        if !load(&mut tx, program_id, true).await?.archived() {
            return Err(ApplicationError::validation_error("program is not archived; nothing to unarchive"));
        }
        sqlx::query("UPDATE programs SET archive_time = NULL, update_time = now() WHERE id = $1")
            .bind(program_id)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        record_change(&mut tx, program_id, actor, "unarchived", Some(json!({ "archived": true })), Some(json!({ "archived": false })))
            .await
            .map_err(db_error)?;
        let page = load(&mut tx, program_id, false).await?;
        tx.commit().await.map_err(db_error)?;
        Ok(page)
    }

    async fn delete_permanently(&self, program_id: Uuid, actor: &str) -> Result<(), ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let page = load(&mut tx, program_id, true).await?;
        if let Some(reason) = page.blockers.reason() {
            return Err(ApplicationError::conflict(&format!("this program cannot be deleted: {reason}")));
        }
        // The audit log has no foreign key to the program: this entry outlives it.
        record_change(&mut tx, program_id, actor, "deleted_permanently", Some(json!({ "state": "ACTIVE" })), None)
            .await
            .map_err(db_error)?;
        // Modules, setting values, rules and the rest of the program's rows go with it (ON DELETE CASCADE).
        sqlx::query("DELETE FROM programs WHERE id = $1")
            .bind(program_id)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(())
    }
}
