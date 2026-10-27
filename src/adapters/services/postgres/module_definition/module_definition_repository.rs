use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::module_definition::module_definition_mapper::ModuleDefinitionDbMapper;
use crate::adapters::services::postgres::module_definition::module_definition_model::ModuleDefinitionDbModel;
use crate::application::errors::application_error::ApplicationError;
use crate::application::mappers::db_mapper::DbMapper;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::module_definition::{ModuleDefinition, ModuleDefinitionRepository};
use async_trait::async_trait;
use ms_telemetry_rs::tracing::error;
use std::sync::Arc;
use uuid::Uuid;

const COLUMNS: &str = "id, key, name, kind, parent_id, ancestry::text AS ancestry, position, \
                       ui_component, rule_family, universal, requires_publish, permission_key, \
                       source, handler_key, api_base, selectable, allow_components, default_mode, \
                       create_time, update_time";

pub struct ModuleDefinitionRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl ModuleDefinitionRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

fn db_error(err: sqlx::Error) -> ApplicationError {
    error!("DB error: {:?}", err);
    ApplicationError::internal_error(Some(err))
}

#[async_trait]
impl ModuleDefinitionRepository for ModuleDefinitionRepositoryPostgresImpl {
    async fn get(&self, identifier: ResourceIdentifier) -> Result<ModuleDefinition, ApplicationError> {
        // Both lookups hit a unique index (the primary key, or the `key`
        // unique constraint), so either is a single-row index probe.
        let (query, label) = match &identifier {
            ResourceIdentifier::Id(_) => (
                format!("SELECT {COLUMNS} FROM module_definitions WHERE id = $1"),
                "id",
            ),
            ResourceIdentifier::Name(_) => (
                format!("SELECT {COLUMNS} FROM module_definitions WHERE key = $1"),
                "key",
            ),
        };

        let builder = sqlx::query_as::<_, ModuleDefinitionDbModel>(&query);
        let builder = match &identifier {
            ResourceIdentifier::Id(id) => builder.bind(*id),
            ResourceIdentifier::Name(key) => builder.bind(key.clone()),
        };

        let model = builder
            .fetch_optional(self.db_connection.get_pool())
            .await
            .map_err(db_error)?;

        match model {
            Some(model) => Ok(ModuleDefinitionDbMapper::to_entity(model)),
            None => Err(ApplicationError::not_found_with_message(&format!(
                "Module definition with {label} {} not found",
                match identifier {
                    ResourceIdentifier::Id(id) => id.to_string(),
                    ResourceIdentifier::Name(key) => key,
                }
            ))),
        }
    }

    async fn list(
        &self,
        parent_id: Option<Uuid>,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<ModuleDefinition>, Option<Uuid>, i64), ApplicationError> {
        // `IS NOT DISTINCT FROM` matches NULL = NULL, so one statement serves
        // both "children of X" and "roots", and both are an index range scan
        // over (parent_id, position, id).
        let total_size: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM module_definitions WHERE parent_id IS NOT DISTINCT FROM $1",
        )
        .bind(parent_id)
        .fetch_one(self.db_connection.get_pool())
        .await
        .map_err(db_error)?;

        // Keyset pagination in display order: the token is the last id of the
        // previous page, and the row comparison resumes after that row's
        // (position, id).
        let query = format!(
            "SELECT {COLUMNS} FROM module_definitions \
             WHERE parent_id IS NOT DISTINCT FROM $1 \
               AND ($2::uuid IS NULL OR (position, id) > \
                    (SELECT position, id FROM module_definitions WHERE id = $2)) \
             ORDER BY position, id LIMIT $3"
        );

        let mut rows = sqlx::query_as::<_, ModuleDefinitionDbModel>(&query)
            .bind(parent_id)
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
            rows.into_iter().map(ModuleDefinitionDbMapper::to_entity).collect(),
            next_page_token,
            total_size,
        ))
    }

    async fn list_descendants(&self, root: Option<Uuid>) -> Result<Vec<ModuleDefinition>, ApplicationError> {
        let models = match root {
            // The whole tree: a tiny table, read in one pass, in display order.
            None => {
                let query = format!("SELECT {COLUMNS} FROM module_definitions ORDER BY position, id");
                sqlx::query_as::<_, ModuleDefinitionDbModel>(&query)
                    .fetch_all(self.db_connection.get_pool())
                    .await
                    .map_err(db_error)?
            }
            // A subtree: `<@` is the ltree "descendant of" operator, served by
            // the GiST index on `ancestry`. The scalar subquery doubles as the
            // existence check (no row -> NULL -> nothing matches), and the
            // explicit probe below turns that into a proper NotFound.
            Some(root_id) => {
                let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM module_definitions WHERE id = $1)")
                    .bind(root_id)
                    .fetch_one(self.db_connection.get_pool())
                    .await
                    .map_err(db_error)?;
                if !exists {
                    return Err(ApplicationError::not_found("Module definition", root_id));
                }

                let query = format!(
                    "SELECT {COLUMNS} FROM module_definitions \
                     WHERE ancestry <@ (SELECT ancestry FROM module_definitions WHERE id = $1) \
                       AND id <> $1 \
                     ORDER BY position, id"
                );
                sqlx::query_as::<_, ModuleDefinitionDbModel>(&query)
                    .bind(root_id)
                    .fetch_all(self.db_connection.get_pool())
                    .await
                    .map_err(db_error)?
            }
        };

        Ok(models.into_iter().map(ModuleDefinitionDbMapper::to_entity).collect())
    }
}
