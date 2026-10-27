use crate::adapters::services::postgres::db_error::db_error;
use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::program::program_repository::record_change;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::rule_set::{NewRuleVersion, RuleSet, RuleSetRepository, RuleVersion, SaveOutcome};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use std::sync::Arc;
use uuid::Uuid;

const VERSION_FULL: &str = "id, version, status, definition, compiled, valid, validation_errors, content_hash, size_bytes, created_by, create_time";
const VERSION_LIGHT: &str = "id, version, status, NULL::jsonb AS definition, NULL::jsonb AS compiled, valid, validation_errors, content_hash, size_bytes, created_by, create_time";

#[derive(Debug, FromRow)]
struct VersionRow {
    id: Uuid,
    version: i32,
    status: String,
    definition: Option<Value>,
    compiled: Option<Value>,
    valid: bool,
    validation_errors: Value,
    content_hash: String,
    size_bytes: i32,
    created_by: Option<String>,
    create_time: DateTime<Utc>,
}

fn to_version(row: VersionRow, program_id: Uuid, module_key: &str) -> RuleVersion {
    RuleVersion {
        id: row.id,
        program_id,
        module_key: module_key.to_string(),
        version: row.version,
        status: row.status,
        definition: row.definition,
        compiled: row.compiled,
        valid: row.valid,
        validation_errors: row.validation_errors,
        content_hash: row.content_hash,
        size_bytes: row.size_bytes,
        created_by: row.created_by,
        create_time: row.create_time,
    }
}

#[derive(Debug, FromRow)]
struct Leaf {
    module_id: Uuid,
    key: String,
    name: String,
    rule_family: Option<String>,
    mode: Option<String>,
    enabled: bool,
}

#[derive(Debug, FromRow)]
struct SetRow {
    id: Uuid,
    rule_type: Option<String>,
    draft_version: Option<i32>,
    published_version: Option<i32>,
    update_time: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct ListRow {
    module_id: Uuid,
    key: String,
    name: String,
    rule_set_id: Option<Uuid>,
    rule_type: Option<String>,
    draft_version: Option<i32>,
    published_version: Option<i32>,
    update_time: Option<DateTime<Utc>>,
}

pub struct RuleSetRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl RuleSetRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

/// The rule leaf an identifier names: a module key, or the id of its rule set. It must be a module
/// this program chose and made a rule leaf.
async fn resolve_leaf(conn: &mut PgConnection, program_id: Uuid, rule_set: &ResourceIdentifier) -> Result<Leaf, ApplicationError> {
    let base = "SELECT md.id AS module_id, md.key, md.name, md.rule_family, pm.mode, pm.enabled FROM module_definitions md \
                JOIN program_modules pm ON pm.module_id = md.id AND pm.program_id = $1";
    let leaf = match rule_set {
        ResourceIdentifier::Name(key) => sqlx::query_as::<_, Leaf>(&format!("{base} WHERE md.key = $2")).bind(program_id).bind(key).fetch_optional(&mut *conn).await,
        ResourceIdentifier::Id(id) => sqlx::query_as::<_, Leaf>(&format!("{base} JOIN rule_sets rs ON rs.module_id = md.id AND rs.program_id = pm.program_id WHERE rs.id = $2"))
            .bind(program_id)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await,
    }
    .map_err(db_error)?;
    let leaf = leaf.ok_or_else(|| match rule_set {
        ResourceIdentifier::Name(key) => ApplicationError::not_found_with_message(&format!("this program has no module {key:?}")),
        ResourceIdentifier::Id(id) => ApplicationError::not_found("Rule set", id),
    })?;
    if !leaf.enabled || leaf.mode.as_deref() != Some("rule") {
        return Err(ApplicationError::failed_precondition(&format!(
            "{} is not a rule module of this program (switch it on and make its mode \"rule\")",
            leaf.key
        )));
    }
    Ok(leaf)
}

async fn set_row(conn: &mut PgConnection, program_id: Uuid, module_id: Uuid, lock: bool) -> Result<Option<SetRow>, ApplicationError> {
    let query = format!(
        "SELECT id, rule_type, draft_version, published_version, update_time FROM rule_sets \
         WHERE program_id = $1 AND module_id = $2{}",
        if lock { " FOR UPDATE" } else { "" }
    );
    sqlx::query_as::<_, SetRow>(&query).bind(program_id).bind(module_id).fetch_optional(conn).await.map_err(db_error)
}

async fn version_row(conn: &mut PgConnection, rule_set_id: Uuid, version: i32) -> Result<Option<VersionRow>, ApplicationError> {
    let query = format!("SELECT {VERSION_FULL} FROM rule_versions WHERE rule_set_id = $1 AND version = $2");
    sqlx::query_as::<_, VersionRow>(&query).bind(rule_set_id).bind(version).fetch_optional(conn).await.map_err(db_error)
}

#[async_trait]
impl RuleSetRepository for RuleSetRepositoryPostgresImpl {
    async fn list(&self, program_id: Uuid) -> Result<Vec<RuleSet>, ApplicationError> {
        let rows = sqlx::query_as::<_, ListRow>(
            "SELECT pm.module_id, md.key, md.name, rs.id AS rule_set_id, COALESCE(rs.rule_type, md.rule_family) AS rule_type, \
                    rs.draft_version, rs.published_version, rs.update_time \
             FROM program_modules pm \
             JOIN module_definitions md ON md.id = pm.module_id \
             LEFT JOIN rule_sets rs ON rs.program_id = pm.program_id AND rs.module_id = pm.module_id \
             WHERE pm.program_id = $1 AND pm.enabled AND pm.mode = 'rule' \
             ORDER BY pm.position, md.key",
        )
        .bind(program_id)
        .fetch_all(self.db_connection.get_pool())
        .await
        .map_err(db_error)?;
        Ok(rows
            .into_iter()
            .map(|r| RuleSet {
                program_id,
                module_id: r.module_id,
                module_key: r.key,
                module_name: r.name,
                rule_set_id: r.rule_set_id,
                rule_type: r.rule_type,
                draft_version: r.draft_version,
                published_version: r.published_version,
                draft: None,
                update_time: r.update_time,
            })
            .collect())
    }

    async fn get(&self, program_id: Uuid, rule_set: &ResourceIdentifier) -> Result<RuleSet, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        let leaf = resolve_leaf(&mut conn, program_id, rule_set).await?;
        let row = set_row(&mut conn, program_id, leaf.module_id, false).await?;
        let draft = match row.as_ref().and_then(|r| r.draft_version.map(|v| (r.id, v))) {
            Some((id, version)) => version_row(&mut conn, id, version).await?.map(|v| to_version(v, program_id, &leaf.key)),
            None => None,
        };
        Ok(RuleSet {
            program_id,
            module_id: leaf.module_id,
            module_key: leaf.key,
            module_name: leaf.name,
            rule_set_id: row.as_ref().map(|r| r.id),
            rule_type: row.as_ref().and_then(|r| r.rule_type.clone()).or(leaf.rule_family),
            draft_version: row.as_ref().and_then(|r| r.draft_version),
            published_version: row.as_ref().and_then(|r| r.published_version),
            draft,
            update_time: row.map(|r| r.update_time),
        })
    }

    async fn save_version(&self, input: NewRuleVersion) -> Result<SaveOutcome, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;

        // Rules are program configuration: the program lock serializes them with every other
        // change, and the version bump is what makes "unpublished changes" show.
        let current: i32 = sqlx::query_scalar("SELECT config_version FROM programs WHERE id = $1 AND state = 'ACTIVE' FOR UPDATE")
            .bind(input.program_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_error)?
            .ok_or_else(|| ApplicationError::not_found("Program", input.program_id))?;

        let leaf = resolve_leaf(&mut tx, input.program_id, &input.rule_set).await?;
        let existing = set_row(&mut tx, input.program_id, leaf.module_id, true).await?;
        let rule_set_id = match &existing {
            Some(row) => row.id,
            None => {
                let id = Uuid::now_v7();
                sqlx::query("INSERT INTO rule_sets (id, program_id, module_id, rule_type) VALUES ($1, $2, $3, $4)")
                    .bind(id)
                    .bind(input.program_id)
                    .bind(leaf.module_id)
                    .bind(input.rule_type.as_deref().map(str::trim).map(String::from).or(leaf.rule_family.clone()))
                    .execute(&mut *tx)
                    .await
                    .map_err(db_error)?;
                id
            }
        };
        let old_draft = existing.as_ref().and_then(|r| r.draft_version);

        // The hash of the definition: saving what is already the newest version creates nothing.
        let hash: String = sqlx::query_scalar("SELECT md5($1::jsonb::text)").bind(&input.definition).fetch_one(&mut *tx).await.map_err(db_error)?;
        if let Some(version) = old_draft
            && let Some(latest) = version_row(&mut tx, rule_set_id, version).await?
            && latest.content_hash == hash
            && latest.valid == input.valid
        {
            tx.rollback().await.map_err(db_error)?;
            return Ok(SaveOutcome { version: to_version(latest, input.program_id, &leaf.key), created: false });
        }

        let next: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) + 1 FROM rule_versions WHERE rule_set_id = $1")
            .bind(rule_set_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(db_error)?;
        let query = format!(
            "INSERT INTO rule_versions (id, rule_set_id, version, status, definition, compiled, valid, validation_errors, \
                content_hash, size_bytes, created_by) \
             VALUES ($1, $2, $3, 'draft', $4, $5, $6, $7, $8, octet_length($4::jsonb::text), $9) RETURNING {VERSION_FULL}"
        );
        let created = sqlx::query_as::<_, VersionRow>(&query)
            .bind(Uuid::now_v7())
            .bind(rule_set_id)
            .bind(next)
            .bind(&input.definition)
            .bind(&input.compiled)
            .bind(input.valid)
            .bind(&input.validation_errors)
            .bind(&hash)
            .bind(&input.actor)
            .fetch_one(&mut *tx)
            .await
            .map_err(db_error)?;

        // Older unpublished drafts are replaced; a published version is never touched.
        sqlx::query("UPDATE rule_versions SET status = 'superseded' WHERE rule_set_id = $1 AND status = 'draft' AND version < $2")
            .bind(rule_set_id)
            .bind(next)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        sqlx::query("UPDATE rule_sets SET draft_version = $2, update_time = now() WHERE id = $1")
            .bind(rule_set_id)
            .bind(next)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;

        let new_version = current + 1;
        sqlx::query("UPDATE programs SET config_version = $2, config_hash = md5(id::text || ':' || $2::text), update_time = now() WHERE id = $1")
            .bind(input.program_id)
            .bind(new_version)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        record_change(
            &mut tx,
            input.program_id,
            &input.actor,
            "rule_version_saved",
            Some(json!({ "module_key": leaf.key, "draft_version": old_draft })),
            Some(json!({ "module_key": leaf.key, "version": next, "valid": input.valid, "content_hash": hash })),
        )
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(SaveOutcome { version: to_version(created, input.program_id, &leaf.key), created: true })
    }

    async fn list_versions(
        &self,
        program_id: Uuid,
        rule_set: &ResourceIdentifier,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<RuleVersion>, Option<Uuid>), ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        let leaf = resolve_leaf(&mut conn, program_id, rule_set).await?;
        let Some(row) = set_row(&mut conn, program_id, leaf.module_id, false).await? else {
            return Ok((Vec::new(), None));
        };

        // idx_rule_versions_rule_set_id_id: newest first is a backward index range.
        let query = format!(
            "SELECT {VERSION_LIGHT} FROM rule_versions WHERE rule_set_id = $1 AND ($2::uuid IS NULL OR id < $2) \
             ORDER BY id DESC LIMIT $3"
        );
        let mut rows = sqlx::query_as::<_, VersionRow>(&query)
            .bind(row.id)
            .bind(page_token)
            .bind(page_size + 1)
            .fetch_all(&mut *conn)
            .await
            .map_err(db_error)?;
        let next_page_token = if rows.len() as i64 > page_size {
            rows.pop();
            rows.last().map(|r| r.id)
        } else {
            None
        };
        Ok((rows.into_iter().map(|r| to_version(r, program_id, &leaf.key)).collect(), next_page_token))
    }

    async fn get_version(&self, program_id: Uuid, rule_set: &ResourceIdentifier, version: i32) -> Result<RuleVersion, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        let leaf = resolve_leaf(&mut conn, program_id, rule_set).await?;
        let not_found = || ApplicationError::not_found("Rule version", version);
        let row = set_row(&mut conn, program_id, leaf.module_id, false).await?.ok_or_else(not_found)?;
        let found = version_row(&mut conn, row.id, version).await?.ok_or_else(not_found)?;
        Ok(to_version(found, program_id, &leaf.key))
    }
}
