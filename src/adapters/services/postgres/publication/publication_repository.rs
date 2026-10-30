use crate::adapters::services::postgres::db_error::db_error;
use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::program::program_repository::record_change;
use crate::adapters::services::postgres::program_module::program_module_repository::fetch_modules;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::publication::{Publication, PublicationRepository};
use crate::domain::store_discount::{DiscountConfig, MODULE_KEY, resolve};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct PublicationRow {
    id: Uuid,
    program_id: Uuid,
    config_version: i32,
    config_hash: String,
    summary: Option<Value>,
    bundle: Option<Value>,
    created_by: Option<String>,
    create_time: DateTime<Utc>,
}

impl From<PublicationRow> for Publication {
    fn from(row: PublicationRow) -> Self {
        Self {
            id: row.id,
            program_id: row.program_id,
            config_version: row.config_version,
            config_hash: row.config_hash,
            summary: row.summary.unwrap_or_else(|| json!({})),
            bundle: row.bundle,
            created_by: row.created_by,
            create_time: row.create_time,
        }
    }
}

#[derive(Debug, FromRow)]
struct PendingRule {
    id: Uuid,
    key: String,
    draft_version: i32,
    published_version: Option<i32>,
    valid: bool,
    has_input_schema: bool,
}

pub struct PublicationRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl PublicationRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

/// The store -> discount map pricing reads: the three levels worked out once, for the stores that are
/// active and really get a discount. `null` when the program does not use Store Discounts.
async fn store_discounts_section(conn: &mut PgConnection, program_id: Uuid) -> Result<Value, ApplicationError> {
    let module: Option<Value> = sqlx::query_scalar(
        "SELECT pm.config FROM program_modules pm JOIN module_definitions md ON md.id = pm.module_id \
         WHERE pm.program_id = $1 AND md.key = $2 AND pm.enabled",
    )
    .bind(program_id)
    .bind(MODULE_KEY)
    .fetch_optional(&mut *conn)
    .await
    .map_err(db_error)?;
    let Some(module_config) = module else {
        return Ok(Value::Null);
    };
    let program = module_config.get("discount").map(DiscountConfig::from_json).unwrap_or_default();

    let overrides: Vec<(String, String, Value)> =
        sqlx::query_as("SELECT scope_type, scope_id, config FROM program_overrides WHERE program_id = $1")
            .bind(program_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(db_error)?;
    let (mut domains, mut stores): (HashMap<String, DiscountConfig>, HashMap<String, DiscountConfig>) = Default::default();
    for (scope_type, scope_id, config) in overrides {
        let config = DiscountConfig::from_json(&config);
        if scope_type == "domain" {
            domains.insert(scope_id, config);
        } else {
            stores.insert(scope_id, config);
        }
    }

    let enrollments: Vec<(Uuid, String, String)> = sqlx::query_as(
        "SELECT id, store_id, website_id FROM enrollments WHERE program_id = $1 AND status = 'active' ORDER BY id",
    )
    .bind(program_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(db_error)?;
    let empty = DiscountConfig::default();
    let resolved: Vec<Value> = enrollments
        .into_iter()
        .filter_map(|(id, store_id, website_id)| {
            let r = resolve(&program, domains.get(&website_id).unwrap_or(&empty), stores.get(&id.to_string()).unwrap_or(&empty)).resolved;
            (r.enabled && r.amount > 0.0).then(|| {
                json!({ "store_id": store_id, "website_id": website_id, "type": r.discount_type, "amount": r.amount, "currency_code": r.currency_code })
            })
        })
        .collect();

    Ok(json!({
        "program": program.to_json(),
        "domains": domains.iter().map(|(k, v)| (k.clone(), v.to_json())).collect::<serde_json::Map<_, _>>(),
        "stores": resolved,
    }))
}

/// Everything a consumer needs, read inside the publishing transaction so it is one consistent snapshot.
async fn build_bundle(conn: &mut PgConnection, program_id: Uuid, config_version: i32, config_hash: &str) -> Result<Value, ApplicationError> {
    let program: Value = sqlx::query_scalar(
        "SELECT jsonb_build_object('id', id, 'name', name, 'slug', slug, 'level', level, 'active', active) FROM programs WHERE id = $1",
    )
    .bind(program_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(db_error)?;

    let modules = fetch_modules(conn, program_id, true).await.map_err(db_error)?;
    let modules = serde_json::to_value(modules).map_err(|e| ApplicationError::internal_error(Some(e)))?;

    let requirements: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(jsonb_build_object('id', id, 'type', type, 'name', name, 'config', config, 'position', position) \
                ORDER BY position, id), '[]'::jsonb) FROM requirements WHERE program_id = $1",
    )
    .bind(program_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(db_error)?;
    let approval_steps: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(jsonb_build_object('id', id, 'name', name, 'approver_role', approver_role, 'position', position) \
                ORDER BY position, id), '[]'::jsonb) FROM approval_steps WHERE program_id = $1",
    )
    .bind(program_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(db_error)?;

    // The PUBLISHED version of every rule leaf (the ones just promoted included).
    let rules: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(jsonb_build_object('module_key', md.key, 'rule_type', COALESCE(rs.rule_type, md.rule_family), \
                'version', rv.version, 'definition', rv.definition, 'compiled', rv.compiled, \
                'input_schema', rv.input_schema) ORDER BY pm.position, md.key), '[]'::jsonb) \
         FROM rule_sets rs \
         JOIN program_modules pm ON pm.program_id = rs.program_id AND pm.module_id = rs.module_id \
         JOIN module_definitions md ON md.id = rs.module_id \
         JOIN rule_versions rv ON rv.rule_set_id = rs.id AND rv.version = rs.published_version \
         WHERE rs.program_id = $1 AND pm.enabled AND pm.mode = 'rule' AND rs.published_version IS NOT NULL",
    )
    .bind(program_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(db_error)?;

    let store_discounts = store_discounts_section(conn, program_id).await?;

    let count = |v: &Value| v.as_array().map_or(0, Vec::len);
    let summary = json!({
        "modules": count(&modules),
        "requirements": count(&requirements),
        "approval_steps": count(&approval_steps),
        "store_discount_stores": store_discounts.get("stores").map_or(0, count),
        "rules": rules.as_array().map(|a| a.iter().map(|r| json!({ "module_key": r["module_key"], "version": r["version"] })).collect::<Vec<_>>()).unwrap_or_default(),
    });
    Ok(json!({
        "program": program,
        "config_version": config_version,
        "config_hash": config_hash,
        "modules": modules,
        "requirements": requirements,
        "approval_steps": approval_steps,
        "store_discounts": store_discounts,
        "rules": rules,
        "summary": summary,
    }))
}

#[async_trait]
impl PublicationRepository for PublicationRepositoryPostgresImpl {
    async fn publish(&self, program_id: Uuid, actor: &str) -> Result<Publication, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;

        // Lock the program: nobody changes its configuration while it is being snapshotted.
        let (config_version, config_hash): (i32, String) =
            sqlx::query_as("SELECT config_version, config_hash FROM programs WHERE id = $1 AND state = 'ACTIVE' FOR UPDATE")
                .bind(program_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(db_error)?
                .ok_or_else(|| ApplicationError::not_found("Program", program_id))?;

        let last_published: Option<i32> =
            sqlx::query_scalar("SELECT config_version FROM publications WHERE program_id = $1 ORDER BY id DESC LIMIT 1")
                .bind(program_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(db_error)?;

        let rules: Vec<PendingRule> = sqlx::query_as(
            "SELECT rs.id, md.key, rs.draft_version, rs.published_version, rv.valid, (rv.input_schema IS NOT NULL) AS has_input_schema \
             FROM rule_sets rs \
             JOIN program_modules pm ON pm.program_id = rs.program_id AND pm.module_id = rs.module_id \
             JOIN module_definitions md ON md.id = rs.module_id \
             JOIN rule_versions rv ON rv.rule_set_id = rs.id AND rv.version = rs.draft_version \
             WHERE rs.program_id = $1 AND pm.enabled AND pm.mode = 'rule' AND rs.draft_version IS NOT NULL",
        )
        .bind(program_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(db_error)?;

        let invalid: Vec<&str> = rules.iter().filter(|r| !r.valid).map(|r| r.key.as_str()).collect();
        if !invalid.is_empty() {
            return Err(ApplicationError::failed_precondition(&format!(
                "cannot publish: the newest draft of these rules is invalid, fix and save it first: {}",
                invalid.join(", ")
            )));
        }
        let changed: Vec<&PendingRule> = rules.iter().filter(|r| r.published_version != Some(r.draft_version)).collect();
        // A rule goes live with the input it reads, so Execute can tell a caller what to send.
        let no_schema: Vec<&str> = changed.iter().filter(|r| !r.has_input_schema).map(|r| r.key.as_str()).collect();
        if !no_schema.is_empty() {
            return Err(ApplicationError::failed_precondition(&format!(
                "cannot publish: these rules do not say what input they read (input_schema); save a new version with it first: {}",
                no_schema.join(", ")
            )));
        }
        if last_published == Some(config_version) && changed.is_empty() {
            return Err(ApplicationError::failed_precondition(&format!(
                "there is nothing new to publish: the program has not changed since version {config_version} was published"
            )));
        }

        // The drafts become the live versions; what was live is superseded.
        if !changed.is_empty() {
            let set_ids: Vec<Uuid> = changed.iter().map(|r| r.id).collect();
            sqlx::query("UPDATE rule_versions SET status = 'superseded' WHERE rule_set_id = ANY($1) AND status = 'published'")
                .bind(&set_ids)
                .execute(&mut *tx)
                .await
                .map_err(db_error)?;
            sqlx::query(
                "UPDATE rule_versions rv SET status = 'published' FROM rule_sets rs \
                 WHERE rs.id = ANY($1) AND rv.rule_set_id = rs.id AND rv.version = rs.draft_version",
            )
            .bind(&set_ids)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
            sqlx::query("UPDATE rule_sets SET published_version = draft_version, update_time = now() WHERE id = ANY($1)")
                .bind(&set_ids)
                .execute(&mut *tx)
                .await
                .map_err(db_error)?;
        }

        let bundle = build_bundle(&mut tx, program_id, config_version, &config_hash).await?;
        let id = Uuid::now_v7();
        sqlx::query("INSERT INTO publications (id, program_id, config_version, config_hash, bundle, created_by) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(id)
            .bind(program_id)
            .bind(config_version)
            .bind(&config_hash)
            .bind(&bundle)
            .bind(actor)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;

        record_change(
            &mut tx,
            program_id,
            actor,
            "published",
            None,
            Some(json!({
                "publication_id": id,
                "config_version": config_version,
                "rules_published": changed.iter().map(|r| json!({ "module_key": r.key, "version": r.draft_version })).collect::<Vec<_>>(),
                "summary": bundle["summary"],
            })),
        )
        .await
        .map_err(db_error)?;

        let publication = Publication {
            id,
            program_id,
            config_version,
            config_hash,
            summary: bundle["summary"].clone(),
            bundle: Some(bundle),
            created_by: Some(actor.to_string()),
            create_time: sqlx::query_scalar("SELECT create_time FROM publications WHERE id = $1").bind(id).fetch_one(&mut *tx).await.map_err(db_error)?,
        };
        tx.commit().await.map_err(db_error)?;
        Ok(publication)
    }

    async fn list(
        &self,
        program_id: Uuid,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<Publication>, Option<Uuid>), ApplicationError> {
        // idx_publications_program_id_id: newest first is a backward index range.
        let mut rows = sqlx::query_as::<_, PublicationRow>(
            "SELECT id, program_id, config_version, config_hash, bundle->'summary' AS summary, NULL::jsonb AS bundle, created_by, create_time \
             FROM publications WHERE program_id = $1 AND ($2::uuid IS NULL OR id < $2) ORDER BY id DESC LIMIT $3",
        )
        .bind(program_id)
        .bind(page_token)
        .bind(page_size + 1)
        .fetch_all(self.db_connection.get_pool())
        .await
        .map_err(db_error)?;
        let next_page_token = if rows.len() as i64 > page_size {
            rows.pop();
            rows.last().map(|r| r.id)
        } else {
            None
        };
        Ok((rows.into_iter().map(Publication::from).collect(), next_page_token))
    }

    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<Publication, ApplicationError> {
        sqlx::query_as::<_, PublicationRow>(
            "SELECT id, program_id, config_version, config_hash, bundle->'summary' AS summary, bundle, created_by, create_time \
             FROM publications WHERE id = $1 AND program_id = $2",
        )
        .bind(id)
        .bind(program_id)
        .fetch_optional(self.db_connection.get_pool())
        .await
        .map_err(db_error)?
        .map(Publication::from)
        .ok_or_else(|| ApplicationError::not_found("Publication", id))
    }
}
