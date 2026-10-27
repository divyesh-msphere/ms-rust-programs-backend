use crate::adapters::services::postgres::application::application_repository::bump_stats;
use crate::adapters::services::postgres::db_error::db_error;
use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::enrollment::enrollment_repository::enroll_after_approval;
use crate::adapters::services::postgres::program::program_repository::record_event;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::application::{APPROVED, is_pending};
use crate::domain::signoff::{NewSignoff, Signoff, SignoffRepository, outcome};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{FromRow, PgConnection};
use std::sync::Arc;
use uuid::Uuid;

const COLUMNS: &str = "so.id, so.application_id, a.program_id, so.approval_step_id, s.name AS approval_step_name, \
    so.decision, so.comment, so.actor, so.create_time";

#[derive(Debug, FromRow)]
struct SignoffDbModel {
    id: Uuid,
    application_id: Uuid,
    program_id: Uuid,
    approval_step_id: Uuid,
    approval_step_name: String,
    decision: String,
    comment: Option<String>,
    actor: String,
    create_time: DateTime<Utc>,
}

impl From<SignoffDbModel> for Signoff {
    fn from(model: SignoffDbModel) -> Self {
        Self {
            id: model.id,
            application_id: model.application_id,
            program_id: model.program_id,
            approval_step_id: model.approval_step_id,
            approval_step_name: model.approval_step_name,
            decision: model.decision,
            comment: model.comment,
            actor: model.actor,
            create_time: model.create_time,
        }
    }
}

#[derive(Debug, FromRow)]
struct LockedApplication {
    status: String,
    current_step_id: Option<Uuid>,
    store_id: String,
    store_name: Option<String>,
    website_id: String,
    website_name: Option<String>,
}

#[derive(Debug, FromRow)]
struct NextStep {
    id: Uuid,
}

pub struct SignoffRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl SignoffRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

async fn fetch_one(conn: &mut PgConnection, program_id: Uuid, application_id: Uuid, id: Uuid) -> Result<Signoff, ApplicationError> {
    let query = format!(
        "SELECT {COLUMNS} FROM signoffs so \
         JOIN applications a ON a.id = so.application_id \
         JOIN approval_steps s ON s.id = so.approval_step_id \
         WHERE so.id = $1 AND so.application_id = $2 AND a.program_id = $3"
    );
    sqlx::query_as::<_, SignoffDbModel>(&query)
        .bind(id)
        .bind(application_id)
        .bind(program_id)
        .fetch_optional(conn)
        .await
        .map_err(db_error)?
        .map(Signoff::from)
        .ok_or_else(|| ApplicationError::not_found("Signoff", id))
}

#[async_trait]
impl SignoffRepository for SignoffRepositoryPostgresImpl {
    async fn list(&self, program_id: Uuid, application_id: Uuid) -> Result<Vec<Signoff>, ApplicationError> {
        let pool = self.db_connection.get_pool();
        // The application must belong to the program: a wrong pair is a 404, not an empty list.
        let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM applications WHERE id = $1 AND program_id = $2)")
            .bind(application_id)
            .bind(program_id)
            .fetch_one(pool)
            .await
            .map_err(db_error)?;
        if !exists {
            return Err(ApplicationError::not_found("Application", application_id));
        }

        // idx_signoffs_application_id_id: one index range, in the order the decisions were made.
        let query = format!(
            "SELECT {COLUMNS} FROM signoffs so \
             JOIN applications a ON a.id = so.application_id \
             JOIN approval_steps s ON s.id = so.approval_step_id \
             WHERE so.application_id = $1 ORDER BY so.id"
        );
        let rows = sqlx::query_as::<_, SignoffDbModel>(&query)
            .bind(application_id)
            .fetch_all(pool)
            .await
            .map_err(db_error)?;
        Ok(rows.into_iter().map(Signoff::from).collect())
    }

    async fn get(&self, program_id: Uuid, application_id: Uuid, id: Uuid) -> Result<Signoff, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        fetch_one(&mut conn, program_id, application_id, id).await
    }

    async fn create(&self, input: NewSignoff) -> Result<Signoff, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;

        // The row lock makes two decisions on the same application take turns, so the step it is
        // waiting at cannot change under us. The program is not locked.
        let locked = sqlx::query_as::<_, LockedApplication>(
            "SELECT status, current_step_id, store_id, store_name, website_id, website_name \
             FROM applications WHERE id = $1 AND program_id = $2 FOR UPDATE",
        )
        .bind(input.application_id)
        .bind(input.program_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?
        .ok_or_else(|| ApplicationError::not_found("Application", input.application_id))?;

        if !is_pending(&locked.status) {
            return Err(ApplicationError::failed_precondition(&format!(
                "cannot decide an application that is {}; only a submitted or in-review one can be",
                locked.status
            )));
        }
        let step_id = locked.current_step_id.ok_or_else(|| {
            ApplicationError::failed_precondition(
                "this application is not waiting at an approval step (its program had no approval steps when it was submitted)",
            )
        })?;

        if let Some(expected) = input.approval_step_id
            && expected != step_id
        {
            return Err(ApplicationError::conflict(
                "the application is no longer waiting at that approval step; reload it and decide again",
            ));
        }

        // The step after this one, by (position, id). None: this is the last one.
        let next: Option<NextStep> = sqlx::query_as(
            "SELECT s2.id FROM approval_steps s1 \
             JOIN approval_steps s2 ON s2.program_id = s1.program_id AND (s2.position, s2.id) > (s1.position, s1.id) \
             WHERE s1.id = $1 ORDER BY s2.position, s2.id LIMIT 1",
        )
        .bind(step_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?;
        let result = outcome(&input.decision, next.is_some())?;

        let signoff_id = Uuid::now_v7();
        sqlx::query("INSERT INTO signoffs (id, application_id, approval_step_id, decision, comment, actor) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(signoff_id)
            .bind(input.application_id)
            .bind(step_id)
            .bind(&input.decision)
            .bind(input.comment.as_deref().map(str::trim).filter(|c| !c.is_empty()))
            .bind(&input.actor)
            .execute(&mut *tx)
            .await
            .map_err(|err| match &err {
                sqlx::Error::Database(db) if db.is_unique_violation() => {
                    ApplicationError::conflict("this approval step already has a decision for the application")
                }
                _ => db_error(err),
            })?;

        let next_step = if result.moves_to_next_step { next.map(|n| n.id) } else { None };
        let decided_for_good = !is_pending(result.status);
        sqlx::query(
            "UPDATE applications SET status = $3, current_step_id = $4, update_time = now(), \
                decided_time = CASE WHEN $5 THEN now() ELSE decided_time END \
             WHERE id = $1 AND program_id = $2",
        )
        .bind(input.application_id)
        .bind(input.program_id)
        .bind(result.status)
        .bind(next_step)
        .bind(decided_for_good)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;

        // Only a final decision takes it out of "pending approvals".
        if decided_for_good {
            bump_stats(&mut tx, input.program_id, 0, -1).await?;
        }

        // The last approval enrolls the store, in this same transaction.
        if result.status == APPROVED {
            enroll_after_approval(
                &mut tx,
                input.program_id,
                input.application_id,
                &locked.store_id,
                locked.store_name.as_deref(),
                &locked.website_id,
                locked.website_name.as_deref(),
                &input.actor,
            )
            .await?;
        }

        record_event(
            &mut tx,
            input.program_id,
            &input.actor,
            result.action,
            "application",
            &input.application_id.to_string(),
            Some(json!({ "status": locked.status, "current_step_id": step_id })),
            Some(json!({
                "status": result.status,
                "current_step_id": next_step,
                "signoff_id": signoff_id,
                "decision": input.decision,
                "comment": input.comment,
            })),
        )
        .await
        .map_err(db_error)?;

        let created = fetch_one(&mut tx, input.program_id, input.application_id, signoff_id).await?;
        tx.commit().await.map_err(db_error)?;
        Ok(created)
    }
}
