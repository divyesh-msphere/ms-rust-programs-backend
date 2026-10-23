use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::program::program_repository::record_change;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::approval_step::{NewApprovalStep, ApprovalStep, ApprovalStepRepository, UpdateApprovalStep};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ms_telemetry_rs::tracing::error;
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use std::sync::Arc;
use uuid::Uuid;

const COLUMNS: &str = "id, program_id, name, approver_role, position, create_time, update_time";

fn db_error(err: sqlx::Error) -> ApplicationError {
    error!("DB error: {:?}", err);
    ApplicationError::internal_error(Some(err))
}

#[derive(Debug, FromRow)]
struct ApprovalStepDbModel {
    id: Uuid,
    program_id: Uuid,
    name: String,
    approver_role: String,
    position: i32,
    create_time: DateTime<Utc>,
    update_time: DateTime<Utc>,
}

impl From<ApprovalStepDbModel> for ApprovalStep {
    fn from(model: ApprovalStepDbModel) -> Self {
        Self {
            id: model.id,
            program_id: model.program_id,
            name: model.name,
            approver_role: model.approver_role,
            position: model.position,
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }
}

fn snapshot(approval_step: &ApprovalStep) -> Value {
    json!({
        "id": approval_step.id,
        "name": approval_step.name,
        "approver_role": approval_step.approver_role,
        "position": approval_step.position,
    })
}

pub struct ApprovalStepRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl ApprovalStepRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

/// Locks the program so two admins cannot interleave, and returns its version.
async fn lock_program(conn: &mut PgConnection, program_id: Uuid) -> Result<i32, ApplicationError> {
    sqlx::query_scalar("SELECT config_version FROM programs WHERE id = $1 AND state = 'ACTIVE' FOR UPDATE")
        .bind(program_id)
        .fetch_optional(conn)
        .await
        .map_err(db_error)?
        .ok_or_else(|| ApplicationError::not_found("Program", program_id))
}

/// A approval_step is program configuration: the version drives "unpublished changes".
async fn bump_version(conn: &mut PgConnection, program_id: Uuid, current: i32) -> Result<(), ApplicationError> {
    let version = current + 1;
    sqlx::query("UPDATE programs SET config_version = $2, config_hash = md5(id::text || ':' || $2::text), update_time = now() WHERE id = $1")
        .bind(program_id)
        .bind(version)
        .execute(conn)
        .await
        .map_err(db_error)?;
    Ok(())
}

#[async_trait]
impl ApprovalStepRepository for ApprovalStepRepositoryPostgresImpl {
    async fn list(&self, program_id: Uuid) -> Result<Vec<ApprovalStep>, ApplicationError> {
        // idx_approval_steps_program_position_id: one index range, already in order.
        let query = format!("SELECT {COLUMNS} FROM approval_steps WHERE program_id = $1 ORDER BY position, id");
        let rows = sqlx::query_as::<_, ApprovalStepDbModel>(&query)
            .bind(program_id)
            .fetch_all(self.db_connection.get_pool())
            .await
            .map_err(db_error)?;
        Ok(rows.into_iter().map(ApprovalStep::from).collect())
    }

    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<ApprovalStep, ApplicationError> {
        let query = format!("SELECT {COLUMNS} FROM approval_steps WHERE id = $1 AND program_id = $2");
        sqlx::query_as::<_, ApprovalStepDbModel>(&query)
            .bind(id)
            .bind(program_id)
            .fetch_optional(self.db_connection.get_pool())
            .await
            .map_err(db_error)?
            .map(ApprovalStep::from)
            .ok_or_else(|| ApplicationError::not_found("ApprovalStep", id))
    }

    async fn create(&self, input: NewApprovalStep) -> Result<ApprovalStep, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = lock_program(&mut tx, input.program_id).await?;

        let query = format!(
            "INSERT INTO approval_steps (id, program_id, name, approver_role, position) \
             VALUES ($1, $2, $3, $4, COALESCE($5, (SELECT COALESCE(MAX(position) + 1, 0) FROM approval_steps WHERE program_id = $2))) \
             RETURNING {COLUMNS}"
        );
        let id = input.id.unwrap_or_else(Uuid::now_v7);
        let created = sqlx::query_as::<_, ApprovalStepDbModel>(&query)
            .bind(id)
            .bind(input.program_id)
            .bind(input.name.trim())
            .bind(input.approver_role.trim())
            .bind(input.position)
            .fetch_one(&mut *tx)
            .await
            .map_err(|err| match &err {
                sqlx::Error::Database(db) if db.constraint() == Some("approval_steps_pkey") => {
                    ApplicationError::already_exists(&format!("an approval step with id {id} already exists"))
                }
                _ => db_error(err),
            })?;
        let created = ApprovalStep::from(created);

        bump_version(&mut tx, input.program_id, current).await?;
        record_change(&mut tx, input.program_id, &input.actor, "approval_step_created", None, Some(snapshot(&created)))
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(created)
    }

    async fn update(&self, program_id: Uuid, id: Uuid, input: UpdateApprovalStep) -> Result<ApprovalStep, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = lock_program(&mut tx, program_id).await?;

        let query = format!("SELECT {COLUMNS} FROM approval_steps WHERE id = $1 AND program_id = $2 FOR UPDATE");
        let existing: ApprovalStep = sqlx::query_as::<_, ApprovalStepDbModel>(&query)
            .bind(id)
            .bind(program_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_error)?
            .map(ApprovalStep::from)
            .ok_or_else(|| ApplicationError::not_found("ApprovalStep", id))?;

        let name = input.name.as_deref().map(str::trim).unwrap_or(&existing.name);
        let approver_role = input.approver_role.as_deref().map(str::trim).unwrap_or(&existing.approver_role);
        let position = input.position.unwrap_or(existing.position);
        // Nothing differs: write nothing, bump nothing.
        if name == existing.name && approver_role == existing.approver_role && position == existing.position {
            tx.rollback().await.map_err(db_error)?;
            return Ok(existing);
        }

        let query = format!(
            "UPDATE approval_steps SET name = $3, approver_role = $4, position = $5, update_time = now() \
             WHERE id = $1 AND program_id = $2 RETURNING {COLUMNS}"
        );
        let updated = ApprovalStep::from(
            sqlx::query_as::<_, ApprovalStepDbModel>(&query)
                .bind(id)
                .bind(program_id)
                .bind(name)
                .bind(approver_role)
                .bind(position)
                .fetch_one(&mut *tx)
                .await
                .map_err(db_error)?,
        );

        bump_version(&mut tx, program_id, current).await?;
        record_change(&mut tx, program_id, &input.actor, "approval_step_updated", Some(snapshot(&existing)), Some(snapshot(&updated)))
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(updated)
    }

    async fn delete(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<(), ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = lock_program(&mut tx, program_id).await?;

        let query = format!("DELETE FROM approval_steps WHERE id = $1 AND program_id = $2 RETURNING {COLUMNS}");
        let removed = sqlx::query_as::<_, ApprovalStepDbModel>(&query)
            .bind(id)
            .bind(program_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|err| match &err {
                // signoffs.approval_step_id is ON DELETE RESTRICT.
                sqlx::Error::Database(db) if db.is_foreign_key_violation() => ApplicationError::conflict(
                    "this approval step has recorded signoffs and cannot be removed; rename it or reorder instead",
                ),
                _ => db_error(err),
            })?
            .map(ApprovalStep::from)
            .ok_or_else(|| ApplicationError::not_found("ApprovalStep", id))?;

        bump_version(&mut tx, program_id, current).await?;
        record_change(&mut tx, program_id, actor, "approval_step_deleted", Some(snapshot(&removed)), None)
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(())
    }

    async fn reorder(&self, program_id: Uuid, positions: Vec<(Uuid, i32)>, actor: &str) -> Result<Vec<ApprovalStep>, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = lock_program(&mut tx, program_id).await?;

        let ids: Vec<Uuid> = positions.iter().map(|(id, _)| *id).collect();
        let owned: i64 = sqlx::query_scalar("SELECT count(*) FROM approval_steps WHERE program_id = $1 AND id = ANY($2)")
            .bind(program_id)
            .bind(&ids)
            .fetch_one(&mut *tx)
            .await
            .map_err(db_error)?;
        if owned as usize != ids.len() {
            return Err(ApplicationError::not_found_with_message("a approval_step in the list does not belong to this program"));
        }

        // One statement for every move; only rows whose position really changes are written.
        let payload: Vec<Value> = positions.iter().map(|(id, position)| json!({ "id": id, "position": position })).collect();
        let moved: Vec<(Uuid, i32, i32)> = sqlx::query_as(
            "UPDATE approval_steps r SET position = x.position, update_time = now() \
             FROM (SELECT (v->>'id')::uuid AS id, (v->>'position')::int AS position \
                   FROM jsonb_array_elements($2::jsonb) AS v) x, \
                  (SELECT id, position FROM approval_steps WHERE program_id = $1) old \
             WHERE r.id = x.id AND r.program_id = $1 AND old.id = r.id AND old.position IS DISTINCT FROM x.position \
             RETURNING r.id, old.position, r.position",
        )
        .bind(program_id)
        .bind(Value::Array(payload))
        .fetch_all(&mut *tx)
        .await
        .map_err(db_error)?;

        if !moved.is_empty() {
            let before: Vec<Value> = moved.iter().map(|(id, old, _)| json!({ "id": id, "position": old })).collect();
            let after: Vec<Value> = moved.iter().map(|(id, _, new)| json!({ "id": id, "position": new })).collect();
            bump_version(&mut tx, program_id, current).await?;
            record_change(&mut tx, program_id, actor, "approval_steps_reordered", Some(Value::Array(before)), Some(Value::Array(after)))
                .await
                .map_err(db_error)?;
        }

        let query = format!("SELECT {COLUMNS} FROM approval_steps WHERE program_id = $1 ORDER BY position, id");
        let rows = sqlx::query_as::<_, ApprovalStepDbModel>(&query)
            .bind(program_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(rows.into_iter().map(ApprovalStep::from).collect())
    }
}
