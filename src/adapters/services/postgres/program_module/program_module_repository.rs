use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::program_module::program_module_mapper::ProgramModuleDbMapper;
use crate::adapters::services::postgres::program_module::program_module_model::ProgramModuleDbModel;
use crate::application::errors::application_error::ApplicationError;
use crate::application::mappers::db_mapper::DbMapper;
use crate::domain::program_module::{ProgramModule, ProgramModuleRepository};
use crate::domain::program_setting::{OptionsSource, ProgramRule, ProgramSetting, SettingOption, SettingType};
use async_trait::async_trait;
use ms_telemetry_rs::tracing::error;
use serde_json::Value;
use sqlx::{FromRow, PgConnection};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub const COLUMNS: &str = "pm.module_id, md.key, md.name, md.kind, md.parent_id, \
    md.ancestry::text AS ancestry, md.source, md.handler_key, md.api_base, md.ui_component, \
    md.allow_components, md.requires_publish, pm.enabled, pm.mode, pm.position, pm.update_time";

#[derive(Debug, FromRow)]
struct FieldRow {
    id: Uuid,
    module_id: Uuid,
    key: String,
    name: String,
    field_type: String,
    options_source: Option<String>,
    library_id: Option<Uuid>,
    required: bool,
    default_value: Option<Value>,
    position: i32,
}

#[derive(Debug, FromRow)]
struct OptionRow {
    id: Uuid,
    field_id: Uuid,
    code: String,
    label: String,
}

#[derive(Debug, FromRow)]
struct ValueRow {
    field_id: Uuid,
    value: Option<Value>,
    option_ids: Vec<Uuid>,
}

#[derive(Debug, FromRow)]
struct RuleRow {
    module_id: Uuid,
    rule_type: Option<String>,
    draft_version: Option<i32>,
    published_version: Option<i32>,
}

/// Shared with the program repository so `create` / `update` return the manifest from inside
/// their own transaction.
///
/// One indexed read of the program's module rows joined with the (tiny) registry, plus -- only
/// when some module needs them -- its settings, those settings' manual options, and its rule
/// sets: each a single query on an index that starts with `program_id`. Library options are
/// NOT read here (they live in global-settings); the use case fills them in.
pub async fn fetch_modules(
    conn: &mut PgConnection,
    program_id: Uuid,
    enabled_only: bool,
) -> Result<Vec<ProgramModule>, sqlx::Error> {
    // Order: shallowest first, so every parent precedes its children and a client builds the
    // tree in ONE pass; then the program's own position.
    let query = format!(
        "SELECT {COLUMNS} FROM program_modules pm \
         JOIN module_definitions md ON md.id = pm.module_id \
         WHERE pm.program_id = $1 AND (pm.enabled OR NOT $2) \
         ORDER BY nlevel(md.ancestry), pm.position, md.id"
    );
    let models = sqlx::query_as::<_, ProgramModuleDbModel>(&query)
        .bind(program_id)
        .bind(enabled_only)
        .fetch_all(&mut *conn)
        .await?;

    let mut modules: Vec<ProgramModule> = models.into_iter().map(ProgramModuleDbMapper::to_entity).collect();

    if modules.iter().any(|m| m.mode.as_deref() == Some("settings")) {
        let mut by_module = load_settings(&mut *conn, program_id).await?;
        for module in modules.iter_mut().filter(|m| m.mode.as_deref() == Some("settings")) {
            module.settings = by_module.remove(&module.module_id).unwrap_or_default();
        }
    }

    if modules.iter().any(|m| m.mode.as_deref() == Some("rule")) {
        let rules = sqlx::query_as::<_, RuleRow>(
            "SELECT module_id, rule_type, draft_version, published_version FROM rule_sets WHERE program_id = $1",
        )
        .bind(program_id)
        .fetch_all(&mut *conn)
        .await?;
        let mut by_module: HashMap<Uuid, ProgramRule> = rules
            .into_iter()
            .map(|r| {
                (r.module_id, ProgramRule {
                    rule_type: r.rule_type,
                    draft_version: r.draft_version,
                    published_version: r.published_version,
                })
            })
            .collect();
        for module in modules.iter_mut().filter(|m| m.mode.as_deref() == Some("rule")) {
            module.rule = by_module.remove(&module.module_id);
        }
    }

    Ok(modules)
}

async fn load_settings(
    conn: &mut PgConnection,
    program_id: Uuid,
) -> Result<HashMap<Uuid, Vec<ProgramSetting>>, sqlx::Error> {
    // The partial index (program_id, module_id, position, id) WHERE state = 'ACTIVE'.
    let fields = sqlx::query_as::<_, FieldRow>(
        "SELECT id, module_id, key, name, field_type, options_source, library_id, required, \
                default_value, position \
         FROM program_fields WHERE program_id = $1 AND state = 'ACTIVE' \
         ORDER BY module_id, position, id",
    )
    .bind(program_id)
    .fetch_all(&mut *conn)
    .await?;

    let manual_ids: Vec<Uuid> = fields
        .iter()
        .filter(|f| OptionsSource::parse(f.options_source.as_deref().unwrap_or("manual")) == OptionsSource::Manual)
        .map(|f| f.id)
        .collect();

    let mut options_by_field: HashMap<Uuid, Vec<SettingOption>> = HashMap::new();
    if !manual_ids.is_empty() {
        let options = sqlx::query_as::<_, OptionRow>(
            "SELECT id, field_id, code, label FROM field_options \
             WHERE field_id = ANY($1) AND active ORDER BY field_id, position, id",
        )
        .bind(&manual_ids)
        .fetch_all(&mut *conn)
        .await?;
        for option in options {
            options_by_field.entry(option.field_id).or_default().push(SettingOption {
                id: option.id,
                code: option.code,
                label: option.label,
                source: "manual".to_string(),
                active: true,
            });
        }
    }

    // The saved values: one row per setting, on the primary key's `program_id` prefix.
    let mut values: HashMap<Uuid, ValueRow> = sqlx::query_as::<_, ValueRow>(
        "SELECT field_id, value, option_ids FROM program_field_values \
         WHERE program_id = $1 AND scope_type = 'program' AND scope_id = ''",
    )
    .bind(program_id)
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| (row.field_id, row))
    .collect();

    let mut by_module: HashMap<Uuid, Vec<ProgramSetting>> = HashMap::new();
    for field in fields {
        let saved = values.remove(&field.id);
        let Ok(field_type) = SettingType::parse(&field.field_type) else { continue };
        let source = field.options_source.as_deref().map(OptionsSource::parse);
        by_module.entry(field.module_id).or_default().push(ProgramSetting {
            id: field.id,
            key: field.key,
            name: field.name,
            field_type,
            required: field.required,
            default_value: field.default_value,
            position: field.position,
            options_source: source,
            library_id: field.library_id,
            options: options_by_field.remove(&field.id).unwrap_or_default(),
            library_error: None,
            value: saved.as_ref().and_then(|row| row.value.clone()),
            option_ids: saved.map(|row| row.option_ids).unwrap_or_default(),
            // Filled in by the use case once library options are known.
            missing_option_ids: Vec::new(),
        });
    }
    Ok(by_module)
}

pub struct ProgramModuleRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl ProgramModuleRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

#[async_trait]
impl ProgramModuleRepository for ProgramModuleRepositoryPostgresImpl {
    async fn list(&self, program_id: Uuid, enabled_only: bool) -> Result<Vec<ProgramModule>, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(|err| {
            error!("DB error: {:?}", err);
            ApplicationError::internal_error(Some(err))
        })?;
        fetch_modules(&mut conn, program_id, enabled_only).await.map_err(|err| {
            error!("DB error: {:?}", err);
            ApplicationError::internal_error(Some(err))
        })
    }

    async fn library_ids_in_use(&self) -> Result<Vec<Uuid>, ApplicationError> {
        // The partial index on library_id (live settings only).
        sqlx::query_scalar("SELECT DISTINCT library_id FROM program_fields WHERE library_id IS NOT NULL AND state = 'ACTIVE'")
            .fetch_all(self.db_connection.get_pool())
            .await
            .map_err(|err| {
                error!("DB error: {:?}", err);
                ApplicationError::internal_error(Some(err))
            })
    }
}
