use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::program::program_repository::record_change;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::requirement::{NewRequirement, Requirement, RequirementRepository, UpdateRequirement};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ms_telemetry_rs::tracing::error;
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use std::sync::Arc;
use uuid::Uuid;

const COLUMNS: &str = "id, program_id, type AS requirement_type, name, config, position, create_time, update_time";

fn db_error(err: sqlx::Error) -> ApplicationError {
    error!("DB error: {:?}", err);
    ApplicationError::internal_error(Some(err))
}

#[derive(Debug, FromRow)]
struct RequirementDbModel {
    id: Uuid,
    program_id: Uuid,
    requirement_type: String,
    name: String,
    config: Value,
    position: i32,
    create_time: DateTime<Utc>,
    update_time: DateTime<Utc>,
}

impl From<RequirementDbModel> for Requirement {
    fn from(model: RequirementDbModel) -> Self {
        Self {
            id: model.id,
            program_id: model.program_id,
            requirement_type: model.requirement_type,
            name: model.name,
            config: model.config,
            position: model.position,
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }
}

fn snapshot(requirement: &Requirement) -> Value {
    json!({
        "id": requirement.id,
        "type": requirement.requirement_type,
        "name": requirement.name,
        "config": requirement.config,
        "position": requirement.position,
    })
}

pub struct RequirementRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl RequirementRepositoryPostgresImpl {
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

/// A requirement is program configuration: the version drives "unpublished changes".
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
impl RequirementRepository for RequirementRepositoryPostgresImpl {
    async fn list(&self, program_id: Uuid) -> Result<Vec<Requirement>, ApplicationError> {
        // idx_requirements_program_position_id: one index range, already in order.
        let query = format!("SELECT {COLUMNS} FROM requirements WHERE program_id = $1 ORDER BY position, id");
        let rows = sqlx::query_as::<_, RequirementDbModel>(&query)
            .bind(program_id)
            .fetch_all(self.db_connection.get_pool())
            .await
            .map_err(db_error)?;
        Ok(rows.into_iter().map(Requirement::from).collect())
    }

    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<Requirement, ApplicationError> {
        let query = format!("SELECT {COLUMNS} FROM requirements WHERE id = $1 AND program_id = $2");
        sqlx::query_as::<_, RequirementDbModel>(&query)
            .bind(id)
            .bind(program_id)
            .fetch_optional(self.db_connection.get_pool())
            .await
            .map_err(db_error)?
            .map(Requirement::from)
            .ok_or_else(|| ApplicationError::not_found("Requirement", id))
    }

    async fn create(&self, input: NewRequirement) -> Result<Requirement, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = lock_program(&mut tx, input.program_id).await?;

        let query = format!(
            "INSERT INTO requirements (id, program_id, type, name, config, position) \
             VALUES ($1, $2, $3, $4, $5, COALESCE($6, (SELECT COALESCE(MAX(position) + 1, 0) FROM requirements WHERE program_id = $2))) \
             RETURNING {COLUMNS}"
        );
        let id = input.id.unwrap_or_else(Uuid::now_v7);
        let created = sqlx::query_as::<_, RequirementDbModel>(&query)
            .bind(id)
            .bind(input.program_id)
            .bind(&input.requirement_type)
            .bind(input.name.trim())
            .bind(&input.config)
            .bind(input.position)
            .fetch_one(&mut *tx)
            .await
            .map_err(|err| match &err {
                sqlx::Error::Database(db) if db.constraint() == Some("uq_requirements_fixed_type") => {
                    ApplicationError::already_exists(&format!(
                        "this program already has a {:?} requirement; only custom_field can repeat",
                        input.requirement_type
                    ))
                }
                sqlx::Error::Database(db) if db.constraint() == Some("requirements_pkey") => {
                    ApplicationError::already_exists(&format!("a requirement with id {id} already exists"))
                }
                _ => db_error(err),
            })?;
        let created = Requirement::from(created);

        bump_version(&mut tx, input.program_id, current).await?;
        record_change(&mut tx, input.program_id, &input.actor, "requirement_created", None, Some(snapshot(&created)))
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(created)
    }

    async fn update(&self, program_id: Uuid, id: Uuid, input: UpdateRequirement) -> Result<Requirement, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = lock_program(&mut tx, program_id).await?;

        let query = format!("SELECT {COLUMNS} FROM requirements WHERE id = $1 AND program_id = $2 FOR UPDATE");
        let existing: Requirement = sqlx::query_as::<_, RequirementDbModel>(&query)
            .bind(id)
            .bind(program_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_error)?
            .map(Requirement::from)
            .ok_or_else(|| ApplicationError::not_found("Requirement", id))?;

        let name = input.name.as_deref().map(str::trim).unwrap_or(&existing.name);
        let config = input.config.as_ref().unwrap_or(&existing.config);
        let position = input.position.unwrap_or(existing.position);
        // Nothing differs: write nothing, bump nothing.
        if name == existing.name && *config == existing.config && position == existing.position {
            tx.rollback().await.map_err(db_error)?;
            return Ok(existing);
        }

        let query = format!(
            "UPDATE requirements SET name = $3, config = $4, position = $5, update_time = now() \
             WHERE id = $1 AND program_id = $2 RETURNING {COLUMNS}"
        );
        let updated = Requirement::from(
            sqlx::query_as::<_, RequirementDbModel>(&query)
                .bind(id)
                .bind(program_id)
                .bind(name)
                .bind(config)
                .bind(position)
                .fetch_one(&mut *tx)
                .await
                .map_err(db_error)?,
        );

        bump_version(&mut tx, program_id, current).await?;
        record_change(&mut tx, program_id, &input.actor, "requirement_updated", Some(snapshot(&existing)), Some(snapshot(&updated)))
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(updated)
    }

    async fn delete(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<(), ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = lock_program(&mut tx, program_id).await?;

        let query = format!("DELETE FROM requirements WHERE id = $1 AND program_id = $2 RETURNING {COLUMNS}");
        let removed = sqlx::query_as::<_, RequirementDbModel>(&query)
            .bind(id)
            .bind(program_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_error)?
            .map(Requirement::from)
            .ok_or_else(|| ApplicationError::not_found("Requirement", id))?;

        bump_version(&mut tx, program_id, current).await?;
        record_change(&mut tx, program_id, actor, "requirement_deleted", Some(snapshot(&removed)), None)
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(())
    }

    async fn reorder(&self, program_id: Uuid, positions: Vec<(Uuid, i32)>, actor: &str) -> Result<Vec<Requirement>, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = lock_program(&mut tx, program_id).await?;

        let ids: Vec<Uuid> = positions.iter().map(|(id, _)| *id).collect();
        let owned: i64 = sqlx::query_scalar("SELECT count(*) FROM requirements WHERE program_id = $1 AND id = ANY($2)")
            .bind(program_id)
            .bind(&ids)
            .fetch_one(&mut *tx)
            .await
            .map_err(db_error)?;
        if owned as usize != ids.len() {
            return Err(ApplicationError::not_found_with_message("a requirement in the list does not belong to this program"));
        }

        // One statement for every move; only rows whose position really changes are written.
        let payload: Vec<Value> = positions.iter().map(|(id, position)| json!({ "id": id, "position": position })).collect();
        let moved: Vec<(Uuid, i32, i32)> = sqlx::query_as(
            "UPDATE requirements r SET position = x.position, update_time = now() \
             FROM (SELECT (v->>'id')::uuid AS id, (v->>'position')::int AS position \
                   FROM jsonb_array_elements($2::jsonb) AS v) x, \
                  (SELECT id, position FROM requirements WHERE program_id = $1) old \
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
            record_change(&mut tx, program_id, actor, "requirements_reordered", Some(Value::Array(before)), Some(Value::Array(after)))
                .await
                .map_err(db_error)?;
        }

        let query = format!("SELECT {COLUMNS} FROM requirements WHERE program_id = $1 ORDER BY position, id");
        let rows = sqlx::query_as::<_, RequirementDbModel>(&query)
            .bind(program_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(rows.into_iter().map(Requirement::from).collect())
    }
}
