use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::program::program_repository::record_event;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::application::{
    Application, ApplicationRepository, NewApplication, can_submit, can_withdraw, is_pending,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ms_telemetry_rs::tracing::error;
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use std::sync::Arc;
use uuid::Uuid;

/// A list row: no answers, no snapshot. The step name comes from the step's primary key.
const LIST_COLUMNS: &str = "a.id, a.program_id, a.store_id, a.store_name, a.website_id, a.website_name, a.status, \
    a.current_step_id, s.name AS current_step_name, NULL::jsonb AS answers, NULL::jsonb AS requirements_snapshot, \
    a.submitted_time, a.decided_time, a.create_time, a.update_time";
const DETAIL_COLUMNS: &str = "a.id, a.program_id, a.store_id, a.store_name, a.website_id, a.website_name, a.status, \
    a.current_step_id, s.name AS current_step_name, a.answers, a.requirements_snapshot, \
    a.submitted_time, a.decided_time, a.create_time, a.update_time";

fn db_error(err: sqlx::Error) -> ApplicationError {
    error!("DB error: {:?}", err);
    ApplicationError::internal_error(Some(err))
}

#[derive(Debug, FromRow)]
struct ApplicationDbModel {
    id: Uuid,
    program_id: Uuid,
    store_id: String,
    store_name: Option<String>,
    website_id: String,
    website_name: Option<String>,
    status: String,
    current_step_id: Option<Uuid>,
    current_step_name: Option<String>,
    answers: Option<Value>,
    requirements_snapshot: Option<Value>,
    submitted_time: Option<DateTime<Utc>>,
    decided_time: Option<DateTime<Utc>>,
    create_time: DateTime<Utc>,
    update_time: DateTime<Utc>,
}

impl From<ApplicationDbModel> for Application {
    fn from(model: ApplicationDbModel) -> Self {
        Self {
            id: model.id,
            program_id: model.program_id,
            store_id: model.store_id,
            store_name: model.store_name,
            website_id: model.website_id,
            website_name: model.website_name,
            status: model.status,
            current_step_id: model.current_step_id,
            current_step_name: model.current_step_name,
            answers: model.answers,
            requirements_snapshot: model.requirements_snapshot,
            submitted_time: model.submitted_time,
            decided_time: model.decided_time,
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }
}

pub struct ApplicationRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl ApplicationRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

async fn fetch_detail(conn: &mut PgConnection, program_id: Uuid, id: Uuid) -> Result<Application, ApplicationError> {
    let query = format!(
        "SELECT {DETAIL_COLUMNS} FROM applications a LEFT JOIN approval_steps s ON s.id = a.current_step_id \
         WHERE a.id = $1 AND a.program_id = $2"
    );
    sqlx::query_as::<_, ApplicationDbModel>(&query)
        .bind(id)
        .bind(program_id)
        .fetch_optional(conn)
        .await
        .map_err(db_error)?
        .map(Application::from)
        .ok_or_else(|| ApplicationError::not_found("Application", id))
}

/// The row-level lock that makes two decisions on the same application take turns. The program
/// is not locked: applications are data, not configuration, and many arrive at once.
async fn lock_status(conn: &mut PgConnection, program_id: Uuid, id: Uuid) -> Result<String, ApplicationError> {
    sqlx::query_scalar("SELECT status FROM applications WHERE id = $1 AND program_id = $2 FOR UPDATE")
        .bind(id)
        .bind(program_id)
        .fetch_optional(conn)
        .await
        .map_err(db_error)?
        .ok_or_else(|| ApplicationError::not_found("Application", id))
}

/// The Overview counters, in the same transaction as the change. A program created before the
/// counters row existed gets one.
async fn bump_stats(conn: &mut PgConnection, program_id: Uuid, total: i32, pending: i32) -> Result<(), ApplicationError> {
    sqlx::query(
        "INSERT INTO program_stats (program_id, total_applications, pending_approvals) \
         VALUES ($1, GREATEST($2, 0), GREATEST($3, 0)) \
         ON CONFLICT (program_id) DO UPDATE SET \
            total_applications = GREATEST(program_stats.total_applications + $2, 0), \
            pending_approvals = GREATEST(program_stats.pending_approvals + $3, 0), \
            update_time = now()",
    )
    .bind(program_id)
    .bind(total)
    .bind(pending)
    .execute(conn)
    .await
    .map_err(db_error)?;
    Ok(())
}

fn not_allowed(verb: &str, status: &str) -> ApplicationError {
    ApplicationError::failed_precondition(&format!("cannot {verb} an application that is already {status}"))
}

fn summary(application: &Application) -> Value {
    json!({
        "id": application.id,
        "store_id": application.store_id,
        "website_id": application.website_id,
        "status": application.status,
        "current_step_id": application.current_step_id,
    })
}

#[async_trait]
impl ApplicationRepository for ApplicationRepositoryPostgresImpl {
    async fn list(
        &self,
        program_id: Uuid,
        statuses: &[String],
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<Application>, Option<Uuid>), ApplicationError> {
        // Two query shapes so each plans for its own index: (program_id, id) for "All",
        // (program_id, status, id) for a status chip. Newest first = a backward index range.
        let mut rows = if statuses.is_empty() {
            let query = format!(
                "SELECT {LIST_COLUMNS} FROM applications a LEFT JOIN approval_steps s ON s.id = a.current_step_id \
                 WHERE a.program_id = $1 AND ($2::uuid IS NULL OR a.id < $2) ORDER BY a.id DESC LIMIT $3"
            );
            sqlx::query_as::<_, ApplicationDbModel>(&query)
                .bind(program_id)
                .bind(page_token)
                .bind(page_size + 1)
                .fetch_all(self.db_connection.get_pool())
                .await
        } else {
            let query = format!(
                "SELECT {LIST_COLUMNS} FROM applications a LEFT JOIN approval_steps s ON s.id = a.current_step_id \
                 WHERE a.program_id = $1 AND a.status = ANY($2) AND ($3::uuid IS NULL OR a.id < $3) \
                 ORDER BY a.id DESC LIMIT $4"
            );
            sqlx::query_as::<_, ApplicationDbModel>(&query)
                .bind(program_id)
                .bind(statuses)
                .bind(page_token)
                .bind(page_size + 1)
                .fetch_all(self.db_connection.get_pool())
                .await
        }
        .map_err(db_error)?;

        let next_page_token = if rows.len() as i64 > page_size {
            rows.pop();
            rows.last().map(|row| row.id)
        } else {
            None
        };
        Ok((rows.into_iter().map(Application::from).collect(), next_page_token))
    }

    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<Application, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        fetch_detail(&mut conn, program_id, id).await
    }

    async fn create(&self, input: NewApplication) -> Result<Application, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let id = input.id.unwrap_or_else(Uuid::now_v7);

        sqlx::query(
            "INSERT INTO applications (id, program_id, store_id, store_name, website_id, website_name, status, answers) \
             VALUES ($1, $2, $3, $4, $5, $6, 'draft', $7)",
        )
        .bind(id)
        .bind(input.program_id)
        .bind(input.store_id.trim())
        .bind(input.store_name.as_deref().map(str::trim).filter(|n| !n.is_empty()))
        .bind(input.website_id.trim())
        .bind(input.website_name.as_deref().map(str::trim).filter(|n| !n.is_empty()))
        .bind(&input.answers)
        .execute(&mut *tx)
        .await
        .map_err(|err| match &err {
            sqlx::Error::Database(db) if db.constraint() == Some("uq_applications_open") => ApplicationError::already_exists(
                "this store already has an open application (draft, submitted or in review) for this website",
            ),
            sqlx::Error::Database(db) if db.constraint() == Some("applications_pkey") => {
                ApplicationError::already_exists(&format!("an application with id {id} already exists"))
            }
            _ => db_error(err),
        })?;

        bump_stats(&mut tx, input.program_id, 1, 0).await?;
        let created = fetch_detail(&mut tx, input.program_id, id).await?;
        record_event(&mut tx, input.program_id, &input.actor, "created", "application", &id.to_string(), None, Some(summary(&created)))
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(created)
    }

    async fn submit(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<Application, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let status = lock_status(&mut tx, program_id, id).await?;
        if !can_submit(&status) {
            return Err(not_allowed("submit", &status));
        }

        // It enters the first step; a program with no steps leaves it unassigned.
        let first_step: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM approval_steps WHERE program_id = $1 ORDER BY position, id LIMIT 1",
        )
        .bind(program_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?;

        // Freeze what is required today: the program may change its requirements later.
        sqlx::query(
            "UPDATE applications SET status = 'submitted', current_step_id = $3, submitted_time = now(), update_time = now(), \
                requirements_snapshot = jsonb_build_object('requirements', COALESCE((SELECT jsonb_agg( \
                    jsonb_build_object('id', r.id, 'type', r.type, 'name', r.name, 'config', r.config, 'position', r.position) \
                    ORDER BY r.position, r.id) FROM requirements r WHERE r.program_id = $2), '[]'::jsonb)) \
             WHERE id = $1 AND program_id = $2",
        )
        .bind(id)
        .bind(program_id)
        .bind(first_step)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;

        bump_stats(&mut tx, program_id, 0, 1).await?;
        let submitted = fetch_detail(&mut tx, program_id, id).await?;
        record_event(
            &mut tx,
            program_id,
            actor,
            "submitted",
            "application",
            &id.to_string(),
            Some(json!({ "status": status })),
            Some(summary(&submitted)),
        )
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(submitted)
    }

    async fn withdraw(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<Application, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let status = lock_status(&mut tx, program_id, id).await?;
        if !can_withdraw(&status) {
            return Err(not_allowed("withdraw", &status));
        }

        sqlx::query(
            "UPDATE applications SET status = 'withdrawn', current_step_id = NULL, decided_time = now(), update_time = now() \
             WHERE id = $1 AND program_id = $2",
        )
        .bind(id)
        .bind(program_id)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;

        // Only an application that was waiting for a decision was counted as pending.
        bump_stats(&mut tx, program_id, 0, if is_pending(&status) { -1 } else { 0 }).await?;
        let withdrawn = fetch_detail(&mut tx, program_id, id).await?;
        record_event(
            &mut tx,
            program_id,
            actor,
            "withdrawn",
            "application",
            &id.to_string(),
            Some(json!({ "status": status })),
            Some(summary(&withdrawn)),
        )
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(withdrawn)
    }
}
