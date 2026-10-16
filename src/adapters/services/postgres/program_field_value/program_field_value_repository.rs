use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::program::program_repository::record_change;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::program_field_value::{
    ProgramFieldValueRepository, SaveOutcome, SettingDefinition, StoredValue, ValueChange,
};
use crate::domain::program_setting::{OptionsSource, SettingType};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ms_telemetry_rs::tracing::error;
use serde_json::{Map, Value, json};
use sqlx::FromRow;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

fn db_error(err: sqlx::Error) -> ApplicationError {
    error!("DB error: {:?}", err);
    ApplicationError::internal_error(Some(err))
}

#[derive(Debug, FromRow)]
struct DefinitionRow {
    id: Uuid,
    module_key: String,
    key: String,
    name: String,
    field_type: String,
    required: bool,
    options_source: Option<String>,
    library_id: Option<Uuid>,
    manual_option_ids: Vec<Uuid>,
}

#[derive(Debug, FromRow)]
struct StoredRow {
    field_id: Uuid,
    module_key: String,
    key: String,
    name: String,
    field_type: String,
    value: Option<Value>,
    option_ids: Vec<Uuid>,
    update_time: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct ExistingRow {
    field_id: Uuid,
    value: Option<Value>,
    option_ids: Vec<Uuid>,
}

pub struct ProgramFieldValueRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl ProgramFieldValueRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

#[async_trait]
impl ProgramFieldValueRepository for ProgramFieldValueRepositoryPostgresImpl {
    async fn definitions(&self, program_id: Uuid, setting_ids: &[Uuid]) -> Result<Vec<SettingDefinition>, ApplicationError> {
        // One statement: the settings, their module key, and the ids of their own ACTIVE
        // options (the options a manual choice may pick from).
        let rows = sqlx::query_as::<_, DefinitionRow>(
            "SELECT pf.id, md.key AS module_key, pf.key, pf.name, pf.field_type, pf.required, \
                    pf.options_source, pf.library_id, \
                    COALESCE(array_agg(fo.id) FILTER (WHERE fo.active), '{}') AS manual_option_ids \
             FROM program_fields pf \
             JOIN module_definitions md ON md.id = pf.module_id \
             LEFT JOIN field_options fo ON fo.field_id = pf.id \
             WHERE pf.program_id = $1 AND pf.id = ANY($2) AND pf.state = 'ACTIVE' \
             GROUP BY pf.id, md.key",
        )
        .bind(program_id)
        .bind(setting_ids)
        .fetch_all(self.db_connection.get_pool())
        .await
        .map_err(db_error)?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                Some(SettingDefinition {
                    id: row.id,
                    module_key: row.module_key,
                    key: row.key,
                    name: row.name,
                    field_type: SettingType::parse(&row.field_type).ok()?,
                    required: row.required,
                    options_source: row.options_source.as_deref().map(OptionsSource::parse),
                    library_id: row.library_id,
                    manual_option_ids: row.manual_option_ids.into_iter().collect(),
                })
            })
            .collect())
    }

    async fn list(&self, program_id: Uuid, module_key: Option<&str>) -> Result<(i32, Vec<StoredValue>), ApplicationError> {
        let pool = self.db_connection.get_pool();
        let version: i32 = sqlx::query_scalar("SELECT config_version FROM programs WHERE id = $1 AND state = 'ACTIVE'")
            .bind(program_id)
            .fetch_optional(pool)
            .await
            .map_err(db_error)?
            .ok_or_else(|| ApplicationError::not_found("Program", program_id))?;

        let rows = sqlx::query_as::<_, StoredRow>(
            "SELECT pfv.field_id, md.key AS module_key, pf.key, pf.name, pf.field_type, pfv.value, \
                    pfv.option_ids, pfv.update_time \
             FROM program_field_values pfv \
             JOIN program_fields pf ON pf.id = pfv.field_id AND pf.state = 'ACTIVE' \
             JOIN module_definitions md ON md.id = pf.module_id \
             WHERE pfv.program_id = $1 AND pfv.scope_type = 'program' AND pfv.scope_id = '' \
               AND ($2::text IS NULL OR md.key = $2) \
             ORDER BY md.key, pf.position, pf.id",
        )
        .bind(program_id)
        .bind(module_key)
        .fetch_all(pool)
        .await
        .map_err(db_error)?;

        let values = rows
            .into_iter()
            .filter_map(|row| {
                Some(StoredValue {
                    setting_id: row.field_id,
                    module_key: row.module_key,
                    key: row.key,
                    name: row.name,
                    field_type: SettingType::parse(&row.field_type).ok()?,
                    value: row.value,
                    option_ids: row.option_ids,
                    update_time: row.update_time,
                })
            })
            .collect();
        Ok((version, values))
    }

    async fn save(
        &self,
        program_id: Uuid,
        expected_version: Option<i32>,
        changes: Vec<ValueChange>,
        actor: &str,
    ) -> Result<SaveOutcome, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;

        // Lock the program: two admins saving the same page cannot interleave, and the
        // version they read is the version they are checked against.
        let current: i32 = sqlx::query_scalar("SELECT config_version FROM programs WHERE id = $1 AND state = 'ACTIVE' FOR UPDATE")
            .bind(program_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_error)?
            .ok_or_else(|| ApplicationError::not_found("Program", program_id))?;

        if let Some(expected) = expected_version
            && expected != current
        {
            return Err(ApplicationError::conflict(&format!(
                "the program changed since you loaded it (you have version {expected}, it is now {current}); reload and try again"
            )));
        }

        // The settings must still be live ones (one could have been removed since validation).
        let ids: Vec<Uuid> = changes.iter().map(ValueChange::setting_id).collect();
        let live: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM program_fields WHERE program_id = $1 AND id = ANY($2) AND state = 'ACTIVE'")
            .bind(program_id)
            .bind(&ids)
            .fetch_all(&mut *tx)
            .await
            .map_err(db_error)?;
        if live.len() != ids.len() {
            return Err(ApplicationError::conflict("a setting was removed while you were saving; reload and try again"));
        }

        let existing: HashMap<Uuid, ExistingRow> = sqlx::query_as::<_, ExistingRow>(
            "SELECT field_id, value, option_ids FROM program_field_values \
             WHERE program_id = $1 AND field_id = ANY($2) AND scope_type = 'program' AND scope_id = ''",
        )
        .bind(program_id)
        .bind(&ids)
        .fetch_all(&mut *tx)
        .await
        .map_err(db_error)?
        .into_iter()
        .map(|row| (row.field_id, row))
        .collect();

        // Only what actually differs is written, so saving a page nobody edited changes nothing,
        // bumps nothing, and shows no "unpublished changes".
        let mut upserts = Vec::new();
        let mut deletes: Vec<Uuid> = Vec::new();
        let (mut before, mut after) = (Map::new(), Map::new());
        for change in &changes {
            let old = existing.get(&change.setting_id());
            match change {
                ValueChange::Set { setting_id, value, option_ids, library_id } => {
                    let same = old.is_some_and(|row| row.value == *value && row.option_ids == *option_ids);
                    if same {
                        continue;
                    }
                    upserts.push(json!({
                        "field_id": setting_id,
                        "value": value,
                        "option_ids": option_ids,
                        "library_id": library_id,
                    }));
                    after.insert(setting_id.to_string(), json!({ "value": value, "option_ids": option_ids }));
                    before.insert(
                        setting_id.to_string(),
                        old.map(|row| json!({ "value": row.value, "option_ids": row.option_ids })).unwrap_or(Value::Null),
                    );
                }
                ValueChange::Clear { setting_id } => {
                    let Some(row) = old else { continue };
                    deletes.push(*setting_id);
                    before.insert(setting_id.to_string(), json!({ "value": row.value, "option_ids": row.option_ids }));
                    after.insert(setting_id.to_string(), Value::Null);
                }
            }
        }

        let changed = upserts.len() + deletes.len();
        if changed == 0 {
            tx.rollback().await.map_err(db_error)?;
            return Ok(SaveOutcome { version: current, changed: 0 });
        }

        if !upserts.is_empty() {
            // One statement for every upsert, however many there are.
            sqlx::query(
                "INSERT INTO program_field_values (program_id, field_id, scope_type, scope_id, value, option_ids, library_id) \
                 SELECT $1, (x->>'field_id')::uuid, 'program', '', NULLIF(x->'value', 'null'::jsonb), \
                        ARRAY(SELECT jsonb_array_elements_text(x->'option_ids'))::uuid[], \
                        NULLIF(x->>'library_id', '')::uuid \
                 FROM jsonb_array_elements($2::jsonb) AS x \
                 ON CONFLICT (program_id, field_id, scope_type, scope_id) DO UPDATE \
                    SET value = EXCLUDED.value, option_ids = EXCLUDED.option_ids, \
                        library_id = EXCLUDED.library_id, update_time = now()",
            )
            .bind(program_id)
            .bind(Value::Array(upserts))
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        }
        if !deletes.is_empty() {
            sqlx::query(
                "DELETE FROM program_field_values \
                 WHERE program_id = $1 AND field_id = ANY($2) AND scope_type = 'program' AND scope_id = ''",
            )
            .bind(program_id)
            .bind(&deletes)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        }

        // The version drives the "unpublished changes" check.
        let version = current + 1;
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
            "values_updated",
            Some(Value::Object(before)),
            Some(Value::Object(after)),
        )
        .await
        .map_err(db_error)?;

        tx.commit().await.map_err(db_error)?;
        Ok(SaveOutcome { version, changed })
    }
}
