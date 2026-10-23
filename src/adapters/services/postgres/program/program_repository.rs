use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::program::module_changes::{apply_changes, check_keys, ensure_modules};
use crate::adapters::services::postgres::program::program_mapper::ProgramDbMapper;
use crate::adapters::services::postgres::program::program_model::ProgramDbModel;
use crate::adapters::services::postgres::program_module::program_module_repository::fetch_modules;
use crate::application::errors::application_error::ApplicationError;
use crate::application::mappers::db_mapper::DbMapper;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::{NewProgram, Program, ProgramRepository, UpdateProgram};
use crate::domain::program_module::ProgramModule;
use async_trait::async_trait;
use ms_telemetry_rs::tracing::error;
use serde_json::{Value, json};
use sqlx::PgConnection;
use std::sync::Arc;
use uuid::Uuid;

const COLUMNS: &str = "id, name, slug, description, level, active, config_version, config_hash, \
                       state, create_time, update_time";

pub struct ProgramRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl ProgramRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

fn db_error(err: sqlx::Error) -> ApplicationError {
    error!("DB error: {:?}", err);
    ApplicationError::internal_error(Some(err))
}

/// Maps a unique violation to the right message: the PRIMARY KEY (a client-supplied
/// `?id=` that already exists) and the live-slug index are different conflicts.
fn create_conflict(err: sqlx::Error, slug: &str) -> ApplicationError {
    if let sqlx::Error::Database(db_err) = &err
        && db_err.is_unique_violation()
    {
        return if db_err.constraint() == Some("programs_pkey") {
            ApplicationError::already_exists("A program with this id already exists")
        } else {
            ApplicationError::already_exists(&format!("A program with slug {slug:?} already exists"))
        };
    }
    db_error(err)
}

fn snapshot(program: &Program) -> Value {
    json!({
        "name": program.name,
        "slug": program.slug,
        "description": program.description,
        "level": program.level,
        "active": program.active,
        "config_version": program.config_version,
        "state": program.state.as_str(),
    })
}

/// Audit entry + outbox event for one program change. Both ride in the
/// caller's transaction, so a change and its record commit or roll back
/// together — an event can never describe a change that did not happen.
pub(crate) async fn record_change(
    conn: &mut PgConnection,
    program_id: Uuid,
    actor: &str,
    action: &str,
    before: Option<Value>,
    after: Option<Value>,
) -> Result<(), sqlx::Error> {
    record_event(conn, program_id, actor, action, "program", &program_id.to_string(), before, after).await
}

/// The same record for a change to something that belongs to the program (an application, ...).
/// The outbox topic is `<target_type>.<action>` and the event is keyed by the program.
pub(crate) async fn record_event(
    conn: &mut PgConnection,
    program_id: Uuid,
    actor: &str,
    action: &str,
    target_type: &str,
    target_id: &str,
    before: Option<Value>,
    after: Option<Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO audit_log (id, program_id, actor, action, target_type, target_id, before, after) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(Uuid::now_v7())
    .bind(program_id)
    .bind(actor)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(before)
    .bind(after.clone())
    .execute(&mut *conn)
    .await?;

    sqlx::query("INSERT INTO outbox (id, topic, event_key, payload) VALUES ($1, $2, $3, $4)")
        .bind(Uuid::now_v7())
        .bind(format!("{target_type}.{action}"))
        .bind(program_id.to_string())
        .bind(json!({ "program_id": program_id, "target_id": target_id, "actor": actor, "after": after }))
        .execute(&mut *conn)
        .await?;

    Ok(())
}

#[async_trait]
impl ProgramRepository for ProgramRepositoryPostgresImpl {
    async fn get(&self, identifier: &ResourceIdentifier, include_deleted: bool) -> Result<Program, ApplicationError> {
        let model = match identifier {
            // Primary-key probe.
            ResourceIdentifier::Id(id) => {
                let query = format!("SELECT {COLUMNS} FROM programs WHERE id = $1 AND (state = 'ACTIVE' OR $2)");
                sqlx::query_as::<_, ProgramDbModel>(&query)
                    .bind(id)
                    .bind(include_deleted)
                    .fetch_optional(self.db_connection.get_pool())
                    .await
            }
            // The partial unique index on lower(slug) among LIVE rows: one index probe.
            // Only when a deleted program is wanted (Undelete) and no live one matches
            // does it fall back to the newest deleted one, because a deleted slug can
            // repeat. The two-step shape keeps the common path on the index (a single
            // `state = 'ACTIVE' OR $2` query cannot use a partial index).
            ResourceIdentifier::Name(slug) => {
                let live = format!("SELECT {COLUMNS} FROM programs WHERE lower(slug) = lower($1) AND state = 'ACTIVE'");
                let found = sqlx::query_as::<_, ProgramDbModel>(&live)
                    .bind(slug)
                    .fetch_optional(self.db_connection.get_pool())
                    .await;
                match found {
                    Ok(None) if include_deleted => {
                        let deleted = format!(
                            "SELECT {COLUMNS} FROM programs WHERE lower(slug) = lower($1) AND state = 'DELETED' \
                             ORDER BY id DESC LIMIT 1"
                        );
                        sqlx::query_as::<_, ProgramDbModel>(&deleted)
                            .bind(slug)
                            .fetch_optional(self.db_connection.get_pool())
                            .await
                    }
                    other => other,
                }
            }
        }
        .map_err(db_error)?;

        match model {
            Some(model) => Ok(ProgramDbMapper::to_entity(model)),
            None => Err(ApplicationError::not_found_with_message(&format!(
                "Program with {} not found",
                match identifier {
                    ResourceIdentifier::Id(id) => format!("ID {id}"),
                    ResourceIdentifier::Name(slug) => format!("slug {slug}"),
                }
            ))),
        }
    }

    async fn list(
        &self,
        page_size: i64,
        page_token: Option<Uuid>,
        show_deleted: bool,
    ) -> Result<(Vec<Program>, Option<Uuid>, i64), ApplicationError> {
        // `state` is a LITERAL in the query text (not a bound parameter) so the planner
        // can use the partial indexes: an index-only scan over live (or deleted) rows,
        // never the whole table.
        let state = if show_deleted { "DELETED" } else { "ACTIVE" };

        let total_size: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM programs WHERE state = '{state}'"))
            .fetch_one(self.db_connection.get_pool())
            .await
            .map_err(db_error)?;

        // Keyset on the primary key; UUIDv7 ids are in creation order.
        let query = format!(
            "SELECT {COLUMNS} FROM programs \
             WHERE state = '{state}' AND ($1::uuid IS NULL OR id > $1) \
             ORDER BY id LIMIT $2"
        );

        let mut rows = sqlx::query_as::<_, ProgramDbModel>(&query)
            .bind(page_token)
            .bind(page_size + 1)
            .fetch_all(self.db_connection.get_pool())
            .await
            .map_err(db_error)?;

        let next_page_token = if rows.len() as i64 > page_size {
            rows.pop();
            rows.last().map(|row| row.id)
        } else {
            None
        };

        Ok((
            rows.into_iter().map(ProgramDbMapper::to_entity).collect(),
            next_page_token,
            total_size,
        ))
    }

    async fn create(&self, input: NewProgram) -> Result<(Program, Vec<ProgramModule>), ApplicationError> {
        let id = input.id.unwrap_or_else(Uuid::now_v7);
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;

        // Every picked key must exist and be selectable (Overview is added automatically and
        // is not selectable).
        let keys: Vec<String> = input.modules.iter().map(|m| m.key.clone()).collect();
        check_keys(&mut tx, id, &keys).await?;

        let insert = format!(
            "INSERT INTO programs (id, name, slug, description, level, config_hash) \
             VALUES ($1, $2, $3, $4, $5, md5($1::text || ':1')) RETURNING {COLUMNS}"
        );
        let model = sqlx::query_as::<_, ProgramDbModel>(&insert)
            .bind(id)
            .bind(&input.name)
            .bind(&input.slug)
            .bind(&input.description)
            .bind(&input.level)
            .fetch_one(&mut *tx)
            .await
            .map_err(|err| create_conflict(err, &input.slug))?;

        // The default modules (Overview) plus every picked module with its ancestors and, for a
        // picked group, everything under it; then the mode and settings of the leaves that
        // came with one. All in this transaction.
        ensure_modules(&mut tx, id, &keys).await?;
        apply_changes(&mut tx, id, &input.modules).await?;

        // Overview counters: created with the program so the page is always a
        // single read, never an "if missing" branch.
        sqlx::query("INSERT INTO program_stats (program_id) VALUES ($1)")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;

        let program = ProgramDbMapper::to_entity(model);
        record_change(&mut tx, id, &input.actor, "created", None, Some(snapshot(&program)))
            .await
            .map_err(db_error)?;

        let modules = fetch_modules(&mut tx, id, false).await.map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;

        Ok((program, modules))
    }

    async fn update(
        &self,
        id: Uuid,
        input: UpdateProgram,
    ) -> Result<(Program, Option<Vec<ProgramModule>>), ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;

        // Lock the row so two admins cannot interleave a read-modify-write.
        let select = format!("SELECT {COLUMNS} FROM programs WHERE id = $1 AND state = 'ACTIVE' FOR UPDATE");
        let current = sqlx::query_as::<_, ProgramDbModel>(&select)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_error)?
            .map(ProgramDbMapper::to_entity)
            .ok_or_else(|| ApplicationError::not_found("Program", id))?;

        let name = input.name.clone().unwrap_or_else(|| current.name.clone());
        let description = match &input.description {
            Some(value) => value.clone(),
            None => current.description.clone(),
        };
        let level = input.level.clone().unwrap_or_else(|| current.level.clone());
        let active = input.active.unwrap_or(current.active);

        // Module changes first: they may fail, and then nothing else should have happened.
        let module_changes = input.modules.as_deref().filter(|changes| !changes.is_empty());
        let mut modules_changed = false;
        if let Some(changes) = module_changes {
            let keys: Vec<String> = changes.iter().map(|m| m.key.clone()).collect();
            check_keys(&mut tx, id, &keys).await?;
            // A key the program does not have yet adds that module (and its ancestors).
            ensure_modules(&mut tx, id, &keys).await?;
            modules_changed = apply_changes(&mut tx, id, changes).await?;
        }

        // Only a change that other parts of the system react to bumps the configuration version
        // (which is what the Publish drift check reads): the level, the active flag, or any
        // module / mode / setting change.
        let config_changed = level != current.level || active != current.active || modules_changed;
        let config_version = current.config_version + i32::from(config_changed);

        let update = format!(
            "UPDATE programs SET name = $2, description = $3, level = $4, active = $5, \
                    config_version = $6, config_hash = md5(id::text || ':' || $6::text), \
                    update_time = now() \
             WHERE id = $1 RETURNING {COLUMNS}"
        );
        let model = sqlx::query_as::<_, ProgramDbModel>(&update)
            .bind(id)
            .bind(&name)
            .bind(&description)
            .bind(&level)
            .bind(active)
            .bind(config_version)
            .fetch_one(&mut *tx)
            .await
            .map_err(db_error)?;

        let program = ProgramDbMapper::to_entity(model);
        let mut after = snapshot(&program);
        if let Some(changes) = module_changes {
            after["modules_changed"] = json!(changes.iter().map(|m| m.key.as_str()).collect::<Vec<_>>());
        }
        record_change(&mut tx, id, &input.actor, "updated", Some(snapshot(&current)), Some(after))
            .await
            .map_err(db_error)?;

        let modules = if module_changes.is_some() {
            Some(fetch_modules(&mut tx, id, false).await.map_err(db_error)?)
        } else {
            None
        };
        tx.commit().await.map_err(db_error)?;

        Ok((program, modules))
    }

    async fn delete(&self, id: Uuid, actor: &str) -> Result<(), ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;

        let result = sqlx::query("UPDATE programs SET state = 'DELETED', update_time = now() WHERE id = $1 AND state = 'ACTIVE'")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        if result.rows_affected() == 0 {
            return Err(ApplicationError::not_found("Program", id));
        }

        record_change(&mut tx, id, actor, "deleted", Some(json!({ "state": "ACTIVE" })), Some(json!({ "state": "DELETED" })))
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;

        Ok(())
    }

    async fn undelete(&self, id: Uuid, actor: &str) -> Result<Program, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;

        let update = format!(
            "UPDATE programs SET state = 'ACTIVE', update_time = now() \
             WHERE id = $1 AND state = 'DELETED' RETURNING {COLUMNS}"
        );
        let model = sqlx::query_as::<_, ProgramDbModel>(&update)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|err| {
                // The live slug may have been taken while this program was deleted.
                if let sqlx::Error::Database(db_err) = &err
                    && db_err.is_unique_violation()
                {
                    return ApplicationError::already_exists(
                        "another live program already uses this program's slug; rename or delete it first",
                    );
                }
                db_error(err)
            })?;

        let Some(model) = model else {
            let state: Option<String> = sqlx::query_scalar("SELECT state FROM programs WHERE id = $1")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(db_error)?;
            return Err(match state {
                Some(_) => ApplicationError::validation_error("program is not deleted; nothing to undelete"),
                None => ApplicationError::not_found("Program", id),
            });
        };

        let program = ProgramDbMapper::to_entity(model);
        record_change(&mut tx, id, actor, "undeleted", Some(json!({ "state": "DELETED" })), Some(snapshot(&program)))
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;

        Ok(program)
    }
}
