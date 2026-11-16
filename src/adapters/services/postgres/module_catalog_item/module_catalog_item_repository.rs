use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::module_catalog_item::module_catalog_item_mapper::ModuleCatalogItemDbMapper;
use crate::adapters::services::postgres::module_catalog_item::module_catalog_item_model::ModuleCatalogItemDbModel;
use crate::application::errors::application_error::ApplicationError;
use crate::application::mappers::db_mapper::DbMapper;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::module_catalog_item::{ModuleCatalogItem, ModuleCatalogItemRepository};
use async_trait::async_trait;
use ms_telemetry_rs::tracing::error;
use std::sync::Arc;
use uuid::Uuid;

const COLUMNS: &str = "id, key, name, kind, parent_id, ancestry::text AS ancestry, position, \
                       ui_component, rule_family, universal, requires_publish, permission_key, \
                       source, handler_key, api_base, selectable, allow_components, default_mode, \
                       EXISTS (SELECT 1 FROM module_definitions c WHERE c.parent_id = module_definitions.id) \
                           AS has_children, \
                       (SELECT COUNT(*) FROM module_definitions c WHERE c.parent_id = module_definitions.id) \
                           AS children_count, \
                       icon, module_in, \
                       create_time, update_time";

pub struct ModuleCatalogItemRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl ModuleCatalogItemRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

fn db_error(err: sqlx::Error) -> ApplicationError {
    error!("DB error: {:?}", err);
    ApplicationError::internal_error(Some(err))
}

#[async_trait]
impl ModuleCatalogItemRepository for ModuleCatalogItemRepositoryPostgresImpl {
    async fn get(&self, identifier: ResourceIdentifier) -> Result<ModuleCatalogItem, ApplicationError> {
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

        let builder = sqlx::query_as::<_, ModuleCatalogItemDbModel>(&query);
        let builder = match &identifier {
            ResourceIdentifier::Id(id) => builder.bind(*id),
            ResourceIdentifier::Name(key) => builder.bind(key.clone()),
        };

        let model = builder
            .fetch_optional(self.db_connection.get_pool())
            .await
            .map_err(db_error)?;

        match model {
            Some(model) => Ok(ModuleCatalogItemDbMapper::to_entity(model)),
            None => Err(ApplicationError::not_found_with_message(&format!(
                "Module catalog item with {label} {} not found",
                match identifier {
                    ResourceIdentifier::Id(id) => id.to_string(),
                    ResourceIdentifier::Name(key) => key,
                }
            ))),
        }
    }

    async fn list_enabled_under(
        &self,
        program: &str,
        root: ResourceIdentifier,
    ) -> Result<Vec<ModuleCatalogItem>, ApplicationError> {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM programs WHERE id::text = $1 OR slug = $1)")
            .bind(program)
            .fetch_one(self.db_connection.get_pool())
            .await
            .map_err(db_error)?;
        if !exists {
            return Err(ApplicationError::not_found_with_message(&format!("Program {program} not found")));
        }
        let root = self.get(root).await?;

        // `<@` is the ltree "is at or below": the item itself and everything under it, narrowed to the
        // modules the program has switched on.
        let query = format!(
            "SELECT {COLUMNS} FROM module_definitions \
             WHERE ancestry <@ (SELECT ancestry FROM module_definitions WHERE id = $1) \
               AND EXISTS (SELECT 1 FROM program_modules pm JOIN programs p ON p.id = pm.program_id \
                           WHERE (p.id::text = $2 OR p.slug = $2) AND pm.module_id = module_definitions.id \
                             AND pm.enabled) \
             ORDER BY nlevel(ancestry), position, id"
        );
        let rows = sqlx::query_as::<_, ModuleCatalogItemDbModel>(&query)
            .bind(root.id)
            .bind(program)
            .fetch_all(self.db_connection.get_pool())
            .await
            .map_err(db_error)?;
        Ok(rows.into_iter().map(ModuleCatalogItemDbMapper::to_entity).collect())
    }

    async fn list(
        &self,
        parent_id: Option<Uuid>,
        search: Option<&str>,
        keys: &[String],
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<ModuleCatalogItem>, Option<Uuid>, i64), ApplicationError> {
        // Without `search`: the direct children of `parent_id` (`IS NOT DISTINCT FROM` matches
        // NULL = NULL, so one statement serves "children of X" and "roots").
        // With `search`: a case-insensitive contains match on the name at any depth, narrowed to
        // the descendants of `parent_id` when one is given. The text is lower-cased and its LIKE
        // wildcards escaped here, so it is only ever a literal.
        let pattern = search.map(|text| {
            let escaped = text.to_lowercase().replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            format!("%{escaped}%")
        });
        // `$3` (keys) wins: exactly those modules (by key or id), no parent scoping, no descendants.
        // Else the tree browse. `search` ($2) narrows in every case.
        const FILTER: &str = "( \
              ( cardinality($3::text[]) > 0 AND (key = ANY($3) OR id::text = ANY($3)) \
                AND ($2::text IS NULL OR lower(name) LIKE $2 ESCAPE '\\') ) \
           OR ( cardinality($3::text[]) = 0 AND ( \
                ($2::text IS NULL AND parent_id IS NOT DISTINCT FROM $1) \
                OR ($2::text IS NOT NULL AND lower(name) LIKE $2 ESCAPE '\\' \
                    AND ($1::uuid IS NULL OR (id <> $1 AND ancestry <@ \
                         (SELECT ancestry FROM module_definitions WHERE id = $1)))) ) ) )";

        let total_size: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM module_definitions WHERE {FILTER}"))
            .bind(parent_id)
            .bind(pattern.clone())
            .bind(keys)
            .fetch_one(self.db_connection.get_pool())
            .await
            .map_err(db_error)?;

        // Keyset pagination in display order: the token is the last id of the previous page.
        let query = format!(
            "SELECT {COLUMNS} FROM module_definitions \
             WHERE {FILTER} \
               AND ($4::uuid IS NULL OR (position, id) > \
                    (SELECT position, id FROM module_definitions WHERE id = $4)) \
             ORDER BY position, id LIMIT $5"
        );

        let mut rows = sqlx::query_as::<_, ModuleCatalogItemDbModel>(&query)
            .bind(parent_id)
            .bind(pattern)
            .bind(keys)
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
            rows.into_iter().map(ModuleCatalogItemDbMapper::to_entity).collect(),
            next_page_token,
            total_size,
        ))
    }

    async fn list_descendants(&self, root: Option<Uuid>) -> Result<Vec<ModuleCatalogItem>, ApplicationError> {
        let models = match root {
            // The whole tree: a tiny table, read in one pass, in display order.
            None => {
                let query = format!("SELECT {COLUMNS} FROM module_definitions ORDER BY position, id");
                sqlx::query_as::<_, ModuleCatalogItemDbModel>(&query)
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
                    return Err(ApplicationError::not_found("Module catalog item", root_id));
                }

                let query = format!(
                    "SELECT {COLUMNS} FROM module_definitions \
                     WHERE ancestry <@ (SELECT ancestry FROM module_definitions WHERE id = $1) \
                       AND id <> $1 \
                     ORDER BY position, id"
                );
                sqlx::query_as::<_, ModuleCatalogItemDbModel>(&query)
                    .bind(root_id)
                    .fetch_all(self.db_connection.get_pool())
                    .await
                    .map_err(db_error)?
            }
        };

        Ok(models.into_iter().map(ModuleCatalogItemDbMapper::to_entity).collect())
    }
}
