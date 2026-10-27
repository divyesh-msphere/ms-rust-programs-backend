use crate::adapters::services::postgres::db_error::db_error;
use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::adapters::services::postgres::program::program_repository::record_change;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::store_discount::{
    DiscountChange, DiscountConfig, DiscountLevels, LevelTarget, MODULE_KEY, StoreDiscountFilter, StoreDiscountRepository,
    StoreDiscountRow, resolve, validate_config, validate_resolved, validate_website_id,
};
use async_trait::async_trait;
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection, Postgres, QueryBuilder};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

/// The enrollments that are listed: a store that has left (`ended`) is not.
const LIVE: &str = "('pending', 'active', 'suspended')";

#[derive(Debug, FromRow)]
struct EnrollmentRow {
    id: Uuid,
    store_id: String,
    store_name: Option<String>,
    website_id: String,
    website_name: Option<String>,
    status: String,
}

type Configs = HashMap<String, DiscountConfig>;

pub struct StoreDiscountRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl StoreDiscountRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

/// The program must use the Store Discounts module; its config holds the PROGRAM level.
async fn module_config(conn: &mut PgConnection, program_id: Uuid, lock: bool) -> Result<(Uuid, DiscountConfig), ApplicationError> {
    let query = format!(
        "SELECT pm.module_id, pm.config FROM program_modules pm \
         JOIN module_definitions md ON md.id = pm.module_id \
         WHERE pm.program_id = $1 AND md.key = $2 AND pm.enabled{}",
        if lock { " FOR UPDATE OF pm" } else { "" }
    );
    let row: Option<(Uuid, Value)> = sqlx::query_as(&query)
        .bind(program_id)
        .bind(MODULE_KEY)
        .fetch_optional(conn)
        .await
        .map_err(db_error)?;
    let (module_id, config) = row.ok_or_else(|| {
        ApplicationError::failed_precondition(
            "this program does not use Store Discounts: switch the \"Store Discounts Control Panel\" module on first",
        )
    })?;
    Ok((module_id, config.get("discount").map(DiscountConfig::from_json).unwrap_or_default()))
}

/// The domain and store levels of the given websites and stores: unique-index probes.
async fn level_configs(
    conn: &mut PgConnection,
    program_id: Uuid,
    websites: &[String],
    store_keys: &[String],
) -> Result<(Configs, Configs), ApplicationError> {
    let rows: Vec<(String, String, Value)> = sqlx::query_as(
        "SELECT scope_type, scope_id, config FROM program_overrides \
         WHERE program_id = $1 AND ((scope_type = 'domain' AND scope_id = ANY($2)) \
                                 OR (scope_type = 'store' AND scope_id = ANY($3)))",
    )
    .bind(program_id)
    .bind(websites)
    .bind(store_keys)
    .fetch_all(conn)
    .await
    .map_err(db_error)?;
    let (mut domains, mut stores) = (Configs::new(), Configs::new());
    for (scope_type, scope_id, config) in rows {
        let config = DiscountConfig::from_json(&config);
        if scope_type == "domain" {
            domains.insert(scope_id, config);
        } else {
            stores.insert(scope_id, config);
        }
    }
    Ok((domains, stores))
}

fn build_rows(
    program_id: Uuid,
    enrollments: Vec<EnrollmentRow>,
    program: &DiscountConfig,
    domains: &Configs,
    stores: &Configs,
) -> Vec<StoreDiscountRow> {
    enrollments
        .into_iter()
        .map(|e| {
            let domain = domains.get(&e.website_id).cloned().unwrap_or_default();
            let store = stores.get(&e.id.to_string()).cloned().unwrap_or_default();
            StoreDiscountRow::build(
                e.id,
                program_id,
                (e.store_id, e.store_name, e.website_id, e.website_name, e.status),
                program,
                &domain,
                store,
            )
        })
        .collect()
}

async fn fetch_enrollments(conn: &mut PgConnection, program_id: Uuid, ids: &[Uuid]) -> Result<Vec<EnrollmentRow>, ApplicationError> {
    let query = format!(
        "SELECT id, store_id, store_name, website_id, website_name, status FROM enrollments \
         WHERE program_id = $1 AND id = ANY($2) AND status IN {LIVE}"
    );
    sqlx::query_as::<_, EnrollmentRow>(&query)
        .bind(program_id)
        .bind(ids)
        .fetch_all(conn)
        .await
        .map_err(db_error)
}

/// Locks the program so two admins cannot interleave, like every other configuration change.
async fn lock_program(conn: &mut PgConnection, program_id: Uuid) -> Result<i32, ApplicationError> {
    sqlx::query_scalar("SELECT config_version FROM programs WHERE id = $1 AND state = 'ACTIVE' FOR UPDATE")
        .bind(program_id)
        .fetch_optional(conn)
        .await
        .map_err(db_error)?
        .ok_or_else(|| ApplicationError::not_found("Program", program_id))
}

/// Discounts are program configuration (the module requires publish): the version drives "unpublished changes".
async fn bump_version(conn: &mut PgConnection, program_id: Uuid, current: i32) -> Result<(), ApplicationError> {
    let version = current + 1;
    sqlx::query("UPDATE programs SET config_version = $2, config_hash = md5(id::text || ':' || $2::text), update_time = now() WHERE id = $1")
        .bind(program_id)
        .bind(version)
        .execute(conn)
        .await
        .map_err(db_error)?;
    Ok(())
}

/// What one store would end up with, refused when it makes no sense.
fn check_store(program: &DiscountConfig, domain: &DiscountConfig, new_store: &DiscountConfig) -> Result<(), ApplicationError> {
    validate_config(new_store)?;
    validate_resolved(&resolve(program, domain, new_store).resolved)
}

fn push_filters(qb: &mut QueryBuilder<'_, Postgres>, program_id: Uuid, filter: &StoreDiscountFilter) {
    qb.push(" WHERE program_id = ").push_bind(program_id);
    qb.push(format!(" AND status IN {LIVE}"));
    if let Some(website) = &filter.website_id {
        qb.push(" AND website_id = ").push_bind(website.clone());
    }
    if let Some(prefix) = &filter.store_prefix {
        let escaped = prefix.to_lowercase().replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        qb.push(" AND lower(store_name) LIKE ").push_bind(format!("{escaped}%"));
    }
}

#[async_trait]
impl StoreDiscountRepository for StoreDiscountRepositoryPostgresImpl {
    async fn list(
        &self,
        program_id: Uuid,
        filter: &StoreDiscountFilter,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<StoreDiscountRow>, Option<Uuid>, i64), ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        let (_, program) = module_config(&mut conn, program_id, false).await?;

        let mut count = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM enrollments");
        push_filters(&mut count, program_id, filter);
        let total_size: i64 = count.build_query_scalar().fetch_one(&mut *conn).await.map_err(db_error)?;

        let mut qb = QueryBuilder::<Postgres>::new("SELECT id, store_id, store_name, website_id, website_name, status FROM enrollments");
        push_filters(&mut qb, program_id, filter);
        if let Some(token) = page_token {
            qb.push(" AND id < ").push_bind(token);
        }
        qb.push(" ORDER BY id DESC LIMIT ").push_bind(page_size + 1);
        let mut enrollments = qb.build_query_as::<EnrollmentRow>().fetch_all(&mut *conn).await.map_err(db_error)?;

        let next_page_token = if enrollments.len() as i64 > page_size {
            enrollments.pop();
            enrollments.last().map(|e| e.id)
        } else {
            None
        };

        // The page only: three small probes into the unique index, however many rows.
        let websites: Vec<String> = enrollments.iter().map(|e| e.website_id.clone()).collect::<HashSet<_>>().into_iter().collect();
        let store_keys: Vec<String> = enrollments.iter().map(|e| e.id.to_string()).collect();
        let (domains, stores) = level_configs(&mut conn, program_id, &websites, &store_keys).await?;
        Ok((build_rows(program_id, enrollments, &program, &domains, &stores), next_page_token, total_size))
    }

    async fn get(&self, program_id: Uuid, enrollment_id: Uuid) -> Result<StoreDiscountRow, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        let (_, program) = module_config(&mut conn, program_id, false).await?;
        let enrollments = fetch_enrollments(&mut conn, program_id, &[enrollment_id]).await?;
        if enrollments.is_empty() {
            return Err(ApplicationError::not_found("Enrollment", enrollment_id));
        }
        let websites: Vec<String> = enrollments.iter().map(|e| e.website_id.clone()).collect();
        let (domains, stores) = level_configs(&mut conn, program_id, &websites, &[enrollment_id.to_string()]).await?;
        build_rows(program_id, enrollments, &program, &domains, &stores)
            .pop()
            .ok_or_else(|| ApplicationError::not_found("Enrollment", enrollment_id))
    }

    async fn update_store(
        &self,
        program_id: Uuid,
        enrollment_id: Uuid,
        change: &DiscountChange,
        actor: &str,
    ) -> Result<StoreDiscountRow, ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = lock_program(&mut tx, program_id).await?;
        let (_, program) = module_config(&mut tx, program_id, false).await?;

        let enrollments = fetch_enrollments(&mut tx, program_id, &[enrollment_id]).await?;
        let Some(enrollment) = enrollments.first() else {
            return Err(ApplicationError::not_found("Enrollment", enrollment_id));
        };
        let (domains, stores) =
            level_configs(&mut tx, program_id, &[enrollment.website_id.clone()], &[enrollment_id.to_string()]).await?;
        let domain = domains.get(&enrollment.website_id).cloned().unwrap_or_default();
        let old = stores.get(&enrollment_id.to_string()).cloned().unwrap_or_default();
        let new = change.apply(&old);

        // Nothing differs: write nothing, bump nothing.
        if new == old {
            tx.rollback().await.map_err(db_error)?;
            let stores = HashMap::from([(enrollment_id.to_string(), old)]);
            return Ok(build_rows(program_id, enrollments, &program, &domains, &stores).remove(0));
        }
        check_store(&program, &domain, &new)?;

        write_store_levels(&mut tx, program_id, &[(enrollment_id, new.clone())]).await?;
        bump_version(&mut tx, program_id, current).await?;
        record_change(
            &mut tx,
            program_id,
            actor,
            "store_discount_updated",
            Some(json!({ "enrollment_id": enrollment_id, "store_id": enrollment.store_id, "website_id": enrollment.website_id, "discount": old.to_json() })),
            Some(json!({ "enrollment_id": enrollment_id, "store_id": enrollment.store_id, "website_id": enrollment.website_id, "discount": new.to_json() })),
        )
        .await
        .map_err(db_error)?;

        let stores = HashMap::from([(enrollment_id.to_string(), new)]);
        let row = build_rows(program_id, enrollments, &program, &domains, &stores).remove(0);
        tx.commit().await.map_err(db_error)?;
        Ok(row)
    }

    async fn batch_update_stores(
        &self,
        program_id: Uuid,
        enrollment_ids: &[Uuid],
        change: &DiscountChange,
        actor: &str,
    ) -> Result<(usize, Vec<StoreDiscountRow>), ApplicationError> {
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = lock_program(&mut tx, program_id).await?;
        let (_, program) = module_config(&mut tx, program_id, false).await?;

        let mut enrollments = fetch_enrollments(&mut tx, program_id, enrollment_ids).await?;
        if enrollments.len() != enrollment_ids.len() {
            let found: HashSet<Uuid> = enrollments.iter().map(|e| e.id).collect();
            let missing: Vec<String> = enrollment_ids.iter().filter(|id| !found.contains(id)).map(|id| id.to_string()).collect();
            let shown = missing.iter().take(5).cloned().collect::<Vec<_>>().join(", ");
            let more = if missing.len() > 5 { format!(" (and {} more)", missing.len() - 5) } else { String::new() };
            return Err(ApplicationError::not_found_with_message(&format!(
                "not enrolled stores of this program: {shown}{more}"
            )));
        }
        // The caller's order, so the response lines up with what was sent.
        let position: HashMap<Uuid, usize> = enrollment_ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();
        enrollments.sort_by_key(|e| position[&e.id]);

        let websites: Vec<String> = enrollments.iter().map(|e| e.website_id.clone()).collect::<HashSet<_>>().into_iter().collect();
        let store_keys: Vec<String> = enrollment_ids.iter().map(Uuid::to_string).collect();
        let (domains, mut stores) = level_configs(&mut tx, program_id, &websites, &store_keys).await?;

        // Work every store out first: one invalid result refuses the whole batch.
        let mut changed: Vec<(Uuid, DiscountConfig)> = Vec::new();
        for e in &enrollments {
            let old = stores.get(&e.id.to_string()).cloned().unwrap_or_default();
            let new = change.apply(&old);
            if new == old {
                continue;
            }
            let domain = domains.get(&e.website_id).cloned().unwrap_or_default();
            check_store(&program, &domain, &new).map_err(|err| {
                ApplicationError::unprocessable_entity(&format!("store {} ({}): {}", e.store_id, e.website_id, err.message()))
            })?;
            changed.push((e.id, new.clone()));
            stores.insert(e.id.to_string(), new);
        }

        if !changed.is_empty() {
            write_store_levels(&mut tx, program_id, &changed).await?;
            bump_version(&mut tx, program_id, current).await?;
            let ids: Vec<Uuid> = changed.iter().map(|(id, _)| *id).collect();
            record_change(
                &mut tx,
                program_id,
                actor,
                "store_discounts_updated",
                None,
                Some(json!({ "requested": enrollment_ids.len(), "updated": changed.len(), "enrollment_ids": ids, "change": change_summary(change) })),
            )
            .await
            .map_err(db_error)?;
        }

        let rows = build_rows(program_id, enrollments, &program, &domains, &stores);
        tx.commit().await.map_err(db_error)?;
        Ok((changed.len(), rows))
    }

    async fn levels(&self, program_id: Uuid) -> Result<DiscountLevels, ApplicationError> {
        let mut conn = self.db_connection.get_pool().acquire().await.map_err(db_error)?;
        let (_, program) = module_config(&mut conn, program_id, false).await?;
        let rows: Vec<(String, Value)> = sqlx::query_as(
            "SELECT scope_id, config FROM program_overrides WHERE program_id = $1 AND scope_type = 'domain' ORDER BY scope_id",
        )
        .bind(program_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(db_error)?;
        Ok(DiscountLevels {
            program,
            domains: rows.into_iter().map(|(website, config)| (website, DiscountConfig::from_json(&config))).collect(),
        })
    }

    async fn update_level(
        &self,
        program_id: Uuid,
        target: &LevelTarget,
        change: &DiscountChange,
        actor: &str,
    ) -> Result<DiscountConfig, ApplicationError> {
        if let LevelTarget::Domain(website) = target {
            validate_website_id(website)?;
        }
        let mut tx = self.db_connection.get_pool().begin().await.map_err(db_error)?;
        let current = lock_program(&mut tx, program_id).await?;
        let (module_id, program) = module_config(&mut tx, program_id, true).await?;

        let (old, level_name, website) = match target {
            LevelTarget::Program => (program.clone(), "program", None),
            LevelTarget::Domain(website) => {
                let (domains, _) = level_configs(&mut tx, program_id, std::slice::from_ref(website), &[]).await?;
                (domains.get(website).cloned().unwrap_or_default(), "domain", Some(website.clone()))
            }
        };
        let new = change.apply(&old);
        if new == old {
            tx.rollback().await.map_err(db_error)?;
            return Ok(old);
        }
        validate_config(&new)?;
        let resolution = match target {
            LevelTarget::Program => resolve(&new, &DiscountConfig::default(), &DiscountConfig::default()),
            LevelTarget::Domain(_) => resolve(&program, &new, &DiscountConfig::default()),
        };
        validate_resolved(&resolution.resolved)?;

        match target {
            LevelTarget::Program => {
                // The program level lives in the module's own config, under `discount`.
                sqlx::query(
                    "UPDATE program_modules SET \
                        config = CASE WHEN $3 THEN config - 'discount' ELSE jsonb_set(config, '{discount}', $4::jsonb, true) END, \
                        version = version + 1, update_time = now() \
                     WHERE program_id = $1 AND module_id = $2",
                )
                .bind(program_id)
                .bind(module_id)
                .bind(new.is_empty())
                .bind(new.to_json())
                .execute(&mut *tx)
                .await
                .map_err(db_error)?;
            }
            LevelTarget::Domain(website) => {
                if new.is_empty() {
                    sqlx::query("DELETE FROM program_overrides WHERE program_id = $1 AND scope_type = 'domain' AND scope_id = $2")
                        .bind(program_id)
                        .bind(website)
                        .execute(&mut *tx)
                        .await
                        .map_err(db_error)?;
                } else {
                    upsert_override(&mut tx, program_id, "domain", website, &new).await?;
                }
            }
        }
        bump_version(&mut tx, program_id, current).await?;
        record_change(
            &mut tx,
            program_id,
            actor,
            "store_discount_level_updated",
            Some(json!({ "level": level_name, "website_id": website, "discount": old.to_json() })),
            Some(json!({ "level": level_name, "website_id": website, "discount": new.to_json() })),
        )
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(new)
    }
}

async fn upsert_override(
    conn: &mut PgConnection,
    program_id: Uuid,
    scope_type: &str,
    scope_id: &str,
    config: &DiscountConfig,
) -> Result<(), ApplicationError> {
    sqlx::query(
        "INSERT INTO program_overrides (id, program_id, scope_type, scope_id, config) VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (program_id, scope_type, scope_id) DO UPDATE SET config = EXCLUDED.config, update_time = now()",
    )
    .bind(Uuid::now_v7())
    .bind(program_id)
    .bind(scope_type)
    .bind(scope_id)
    .bind(config.to_json())
    .execute(conn)
    .await
    .map_err(db_error)?;
    Ok(())
}

/// Writes the store level of every given store: one statement for the non-empty configs, one
/// for the ones that went back to "inherits everything" (their row is removed).
async fn write_store_levels(conn: &mut PgConnection, program_id: Uuid, changes: &[(Uuid, DiscountConfig)]) -> Result<(), ApplicationError> {
    let upserts: Vec<Value> = changes
        .iter()
        .filter(|(_, config)| !config.is_empty())
        .map(|(id, config)| json!({ "id": Uuid::now_v7(), "scope_id": id.to_string(), "config": config.to_json() }))
        .collect();
    let deletes: Vec<String> = changes.iter().filter(|(_, config)| config.is_empty()).map(|(id, _)| id.to_string()).collect();

    if !upserts.is_empty() {
        sqlx::query(
            "INSERT INTO program_overrides (id, program_id, scope_type, scope_id, config) \
             SELECT x.id, $1, 'store', x.scope_id, x.config \
             FROM jsonb_to_recordset($2::jsonb) AS x(id uuid, scope_id text, config jsonb) \
             ON CONFLICT (program_id, scope_type, scope_id) DO UPDATE SET config = EXCLUDED.config, update_time = now()",
        )
        .bind(program_id)
        .bind(Value::Array(upserts))
        .execute(&mut *conn)
        .await
        .map_err(db_error)?;
    }
    if !deletes.is_empty() {
        sqlx::query("DELETE FROM program_overrides WHERE program_id = $1 AND scope_type = 'store' AND scope_id = ANY($2)")
            .bind(program_id)
            .bind(&deletes)
            .execute(&mut *conn)
            .await
            .map_err(db_error)?;
    }
    Ok(())
}

fn change_summary(change: &DiscountChange) -> Value {
    match change {
        DiscountChange::RevertAll => json!("revert_all"),
        DiscountChange::Patch(patch) => {
            let field = |set: bool, revert: bool| if revert { "revert" } else if set { "set" } else { "keep" };
            json!({
                "enabled": field(matches!(patch.enabled, Some(Some(_))), matches!(patch.enabled, Some(None))),
                "type": field(matches!(patch.discount_type, Some(Some(_))), matches!(patch.discount_type, Some(None))),
                "amount": field(matches!(patch.amount, Some(Some(_))), matches!(patch.amount, Some(None))),
                "currency_code": field(matches!(patch.currency_code, Some(Some(_))), matches!(patch.currency_code, Some(None))),
            })
        }
    }
}
