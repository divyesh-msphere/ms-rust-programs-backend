use crate::adapters::services::postgres::db_error::db_error;
use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::program::program_repository::record_event;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::enrollment::{
    ACTIVE, BatchOutcome, Enrollment, EnrollmentFilter, EnrollmentRepository, NewEnrollment, UpdateEnrollment,
    active_delta, can_change,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection, Postgres, QueryBuilder};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

const COLUMNS: &str = "id, program_id, application_id, store_id, store_name, store_resolved, website_id, website_name, \
    customer_group, status, enrolled_time, contracts_count, last_contract_time, create_time, update_time";

#[derive(Debug, FromRow)]
struct EnrollmentDbModel {
    id: Uuid,
    program_id: Uuid,
    application_id: Option<Uuid>,
    store_id: String,
    store_name: Option<String>,
    store_resolved: bool,
    website_id: String,
    website_name: Option<String>,
    customer_group: Option<String>,
    status: String,
    enrolled_time: Option<DateTime<Utc>>,
    contracts_count: i32,
    last_contract_time: Option<DateTime<Utc>>,
    create_time: DateTime<Utc>,
    update_time: DateTime<Utc>,
}

impl From<EnrollmentDbModel> for Enrollment {
    fn from(model: EnrollmentDbModel) -> Self {
        Self {
            id: model.id,
            program_id: model.program_id,
            application_id: model.application_id,
            store_id: model.store_id,
            store_name: model.store_name,
            store_resolved: model.store_resolved,
            website_id: model.website_id,
            website_name: model.website_name,
            customer_group: model.customer_group,
            status: model.status,
            enrolled_time: model.enrolled_time,
            contracts_count: model.contracts_count,
            last_contract_time: model.last_contract_time,
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }
}

fn clean(text: Option<&str>) -> Option<String> {
    text.map(str::trim).filter(|t| !t.is_empty()).map(String::from)
}

fn snapshot(enrollment: &Enrollment) -> Value {
    json!({
        "id": enrollment.id,
        "store_id": enrollment.store_id,
        "website_id": enrollment.website_id,
        "status": enrollment.status,
        "customer_group": enrollment.customer_group,
        "application_id": enrollment.application_id,
    })
}

/// The Overview counters, in the same transaction as the change.
async fn bump_enrollment_stats(conn: &mut PgConnection, program_id: Uuid, total: i32, active: i32) -> Result<(), ApplicationError> {
    sqlx::query(
        "INSERT INTO program_stats (program_id, enrollments_total, active_enrollments) \
         VALUES ($1, GREATEST($2, 0), GREATEST($3, 0)) \
         ON CONFLICT (program_id) DO UPDATE SET \
            enrollments_total = GREATEST(program_stats.enrollments_total + $2, 0), \
            active_enrollments = GREATEST(program_stats.active_enrollments + $3, 0), \
            update_time = now()",
    )
    .bind(program_id)
    .bind(total)
    .bind(active)
    .execute(conn)
    .await
    .map_err(db_error)?;
    Ok(())
}

/// One insert. `None` means a live (pending, active or suspended) enrollment already exists for
/// the store and website: the unique index decided, so two requests at once cannot both win.
async fn insert_live(conn: &mut PgConnection, input: &NewEnrollment, id: Uuid) -> Result<Option<Enrollment>, sqlx::Error> {
    let query = format!(
        "INSERT INTO enrollments (id, program_id, application_id, store_id, store_name, store_resolved, website_id, \
            website_name, customer_group, status, enrolled_time) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, CASE WHEN $10 = 'active' THEN now() END) \
         ON CONFLICT (program_id, store_id, website_id) WHERE status IN ('pending', 'active', 'suspended') DO NOTHING \
         RETURNING {COLUMNS}"
    );
    let store_name = clean(input.store_name.as_deref());
    let row = sqlx::query_as::<_, EnrollmentDbModel>(&query)
        .bind(id)
        .bind(input.program_id)
        .bind(input.application_id)
        .bind(input.store_id.trim())
        .bind(&store_name)
        .bind(store_name.is_some())
        .bind(input.website_id.trim())
        .bind(clean(input.website_name.as_deref()))
        .bind(clean(input.customer_group.as_deref()))
        .bind(&input.status)
        .fetch_optional(conn)
        .await?;
    Ok(row.map(Enrollment::from))
}

/// An approved application enrolls its store, in the approving transaction. A store that is
/// already enrolled on that website (an admin got there first) is left as it is.
pub(crate) async fn enroll_after_approval(
    conn: &mut PgConnection,
    program_id: Uuid,
    application_id: Uuid,
    store_id: &str,
    store_name: Option<&str>,
    website_id: &str,
    website_name: Option<&str>,
    actor: &str,
) -> Result<(), ApplicationError> {
    let input = NewEnrollment {
        id: None,
        program_id,
        application_id: Some(application_id),
        store_id: store_id.to_string(),
        store_name: store_name.map(String::from),
        website_id: website_id.to_string(),
        website_name: website_name.map(String::from),
        customer_group: None,
        status: ACTIVE.to_string(),
        actor: actor.to_string(),
    };
    let id = Uuid::now_v7();
    let Some(created) = insert_live(conn, &input, id).await.map_err(db_error)? else {
        return Ok(());
    };
    bump_enrollment_stats(conn, program_id, 1, 1).await?;
    record_event(
        conn,
        program_id,
        actor,
        "created",
        "enrollment",
        &id.to_string(),
        None,
        Some(json!({ "enrollment": snapshot(&created), "source": "application", "application_id": application_id })),
    )
    .await
    .map_err(db_error)?;
    Ok(())
}

fn push_filters(qb: &mut QueryBuilder<'_, Postgres>, program_id: Uuid, filter: &EnrollmentFilter) {
    qb.push(" WHERE program_id = ").push_bind(program_id);
    if let Some(website) = &filter.website_id {
        qb.push(" AND website_id = ").push_bind(website.clone());
    }
    if !filter.statuses.is_empty() {
        qb.push(" AND status = ANY(").push_bind(filter.statuses.clone()).push(")");
    }
    if let Some(prefix) = &filter.store_prefix {
        // A prefix match on the lower-cased name (idx_enrollments_program_store_name).
        let escaped = prefix.to_lowercase().replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        qb.push(" AND lower(store_name) LIKE ").push_bind(format!("{escaped}%"));
    }
}

pub struct EnrollmentRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl EnrollmentRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

#[async_trait]
impl EnrollmentRepository for EnrollmentRepositoryPostgresImpl {
    async fn list(
        &self,
        program_id: Uuid,
        filter: &EnrollmentFilter,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<Enrollment>, Option<Uuid>, i64), ApplicationError> {
        let pool = self.db_connection.get_pool();

        let mut count = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM enrollments");
        push_filters(&mut count, program_id, filter);
        let total_size: i64 = count.build_query_scalar().fetch_one(pool).await.map_err(db_error)?;

        // One table, no joins; newest first is a backward range of the id index.
        let mut qb = QueryBuilder::<Postgres>::new(format!("SELECT {COLUMNS} FROM enrollments"));
        push_filters(&mut qb, program_id, filter);
        if let Some(token) = page_token {
            qb.push(" AND id < ").push_bind(token);
        }
        qb.push(" ORDER BY id DESC LIMIT ").push_bind(page_size + 1);
        let mut rows = qb.build_query_as::<EnrollmentDbModel>().fetch_all(pool).await.map_err(db_error)?;

        let next_page_token = if rows.len() as i64 > page_size {
            rows.pop();
            rows.last().map(|row| row.id)
        } else {
            None
        };
        Ok((rows.into_iter().map(Enrollment::from).collect(), next_page_token, total_size))
    }

    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<Enrollment, ApplicationError> {
        let query = format!("SELECT {COLUMNS} FROM enrollments WHERE id = $1 AND program_id = $2");
        sqlx::query_as::<_, EnrollmentDbModel>(&query)
            .bind(id)
            .bind(program_id)
            .fetch_optional(self.db_connection.get_pool())
            .await
            .map_err(db_error)?
            .map(Enrollment::from)
            .ok_or_else(|| ApplicationError::not_found("Enrollment", id))
    }

    async fn create(&self, input: NewEnrollment) -> Result<Enrollment, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let id = input.id.unwrap_or_else(Uuid::now_v7);

        let created = insert_live(&mut tx, &input, id)
            .await
            .map_err(|err| match &err {
                sqlx::Error::Database(db) if db.constraint() == Some("enrollments_pkey") => {
                    ApplicationError::already_exists(&format!("an enrollment with id {id} already exists"))
                }
                _ => db_error(err),
            })?
            .ok_or_else(|| {
                ApplicationError::already_exists(
                    "this store is already enrolled on this website (pending, active or suspended); end that enrollment first",
                )
            })?;

        bump_enrollment_stats(&mut tx, input.program_id, 1, i32::from(created.status == ACTIVE)).await?;
        record_event(&mut tx, input.program_id, &input.actor, "created", "enrollment", &id.to_string(), None, Some(snapshot(&created)))
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(created)
    }

    async fn batch_create(
        &self,
        program_id: Uuid,
        items: Vec<NewEnrollment>,
        actor: &str,
    ) -> Result<Vec<BatchOutcome>, ApplicationError> {
        if items.is_empty() {
            return Ok(Vec::new());
        }
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;

        let ids: Vec<Uuid> = items.iter().map(|_| Uuid::now_v7()).collect();
        let payload: Vec<Value> = items
            .iter()
            .zip(&ids)
            .map(|(item, id)| {
                json!({
                    "id": id,
                    "store_id": item.store_id.trim(),
                    "store_name": clean(item.store_name.as_deref()),
                    "website_id": item.website_id.trim(),
                    "website_name": clean(item.website_name.as_deref()),
                    "customer_group": clean(item.customer_group.as_deref()),
                    "status": item.status,
                })
            })
            .collect();

        // One statement for every item. A store already enrolled on the website (or listed twice
        // in the batch) conflicts with the live-enrollment index and is skipped, not failed.
        let inserted: Vec<(Uuid, String, String, String)> = sqlx::query_as(
            "INSERT INTO enrollments (id, program_id, store_id, store_name, store_resolved, website_id, website_name, \
                customer_group, status, enrolled_time) \
             SELECT x.id, $1, x.store_id, x.store_name, x.store_name IS NOT NULL, x.website_id, x.website_name, \
                    x.customer_group, x.status, CASE WHEN x.status = 'active' THEN now() END \
             FROM jsonb_to_recordset($2::jsonb) AS x(id uuid, store_id text, store_name text, website_id text, \
                    website_name text, customer_group text, status text) \
             ON CONFLICT (program_id, store_id, website_id) WHERE status IN ('pending', 'active', 'suspended') DO NOTHING \
             RETURNING id, store_id, website_id, status",
        )
        .bind(program_id)
        .bind(Value::Array(payload))
        .fetch_all(&mut *tx)
        .await
        .map_err(db_error)?;

        let created: HashMap<(String, String), Uuid> =
            inserted.iter().map(|(id, store, website, _)| ((store.clone(), website.clone()), *id)).collect();
        let created_ids: HashSet<Uuid> = created.values().copied().collect();
        let active = inserted.iter().filter(|(_, _, _, status)| status == ACTIVE).count() as i32;

        // Each item gets its own outcome; only the row that really went in is "created".
        let mut taken: HashSet<Uuid> = HashSet::new();
        let outcomes: Vec<BatchOutcome> = items
            .iter()
            .map(|item| {
                let key = (item.store_id.trim().to_string(), item.website_id.trim().to_string());
                match created.get(&key) {
                    Some(id) if created_ids.contains(id) && taken.insert(*id) => BatchOutcome::Created(*id),
                    _ => BatchOutcome::AlreadyEnrolled,
                }
            })
            .collect();

        if !inserted.is_empty() {
            bump_enrollment_stats(&mut tx, program_id, inserted.len() as i32, active).await?;
            let new_ids: Vec<Uuid> = inserted.iter().map(|(id, ..)| *id).collect();
            record_event(
                &mut tx,
                program_id,
                actor,
                "batch_created",
                "enrollment",
                &program_id.to_string(),
                None,
                Some(json!({ "requested": items.len(), "created": inserted.len(), "enrollment_ids": new_ids })),
            )
            .await
            .map_err(db_error)?;
        }
        tx.commit().await.map_err(db_error)?;
        Ok(outcomes)
    }

    async fn update(&self, program_id: Uuid, id: Uuid, input: UpdateEnrollment) -> Result<Enrollment, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;

        let query = format!("SELECT {COLUMNS} FROM enrollments WHERE id = $1 AND program_id = $2 FOR UPDATE");
        let existing: Enrollment = sqlx::query_as::<_, EnrollmentDbModel>(&query)
            .bind(id)
            .bind(program_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_error)?
            .map(Enrollment::from)
            .ok_or_else(|| ApplicationError::not_found("Enrollment", id))?;

        let new_status = input.status.as_deref().filter(|s| *s != existing.status);
        if let Some(to) = new_status
            && !can_change(&existing.status, to)
        {
            return Err(ApplicationError::conflict(&format!(
                "an enrollment cannot move from {} to {to}",
                existing.status
            )));
        }
        let new_group = match input.customer_group.as_deref().map(str::trim) {
            None => existing.customer_group.clone(),
            Some("") => None,
            Some(group) => Some(group.to_string()),
        };
        // Nothing differs: write nothing.
        if new_status.is_none() && new_group == existing.customer_group {
            tx.rollback().await.map_err(db_error)?;
            return Ok(existing);
        }

        let status = new_status.unwrap_or(&existing.status).to_string();
        let query = format!(
            "UPDATE enrollments SET status = $3, customer_group = $4, update_time = now(), \
                enrolled_time = CASE WHEN $3 = 'active' AND enrolled_time IS NULL THEN now() ELSE enrolled_time END \
             WHERE id = $1 AND program_id = $2 RETURNING {COLUMNS}"
        );
        let updated = Enrollment::from(
            sqlx::query_as::<_, EnrollmentDbModel>(&query)
                .bind(id)
                .bind(program_id)
                .bind(&status)
                .bind(&new_group)
                .fetch_one(&mut *tx)
                .await
                .map_err(db_error)?,
        );

        let delta = active_delta(&existing.status, &updated.status);
        if delta != 0 {
            bump_enrollment_stats(&mut tx, program_id, 0, delta).await?;
        }
        let action = if new_status.is_some() { "status_changed" } else { "updated" };
        record_event(
            &mut tx,
            program_id,
            &input.actor,
            action,
            "enrollment",
            &id.to_string(),
            Some(snapshot(&existing)),
            Some(snapshot(&updated)),
        )
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(updated)
    }
}
