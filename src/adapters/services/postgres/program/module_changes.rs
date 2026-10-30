// The write side of "what the admin built on a leaf": which modules a program has, the MODE of
// each configurable leaf (`rule` or `settings`) and the settings of a `settings` leaf.
//
// Everything here runs on the CALLER's transaction connection, so a program create / update and
// all of its module changes commit or roll back together. Nothing here calls another service
// (global-settings libraries are validated by the use case BEFORE the transaction opens).
use crate::application::errors::application_error::ApplicationError;
use crate::domain::notifications::default_triggers;
use crate::domain::program_setting::{
    ModuleChange, ModuleMode, OptionInput, SettingInput, code_for_option, key_from_name,
};
use ms_telemetry_rs::tracing::error;
use sqlx::{FromRow, PgConnection};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

fn db_error(err: sqlx::Error) -> ApplicationError {
    error!("DB error: {:?}", err);
    ApplicationError::internal_error(Some(err))
}

/// Every module key must exist and be selectable -- unless the program already has it (Overview
/// is not selectable but every program has it).
pub async fn check_keys(conn: &mut PgConnection, program_id: Uuid, keys: &[String]) -> Result<(), ApplicationError> {
    if keys.is_empty() {
        return Ok(());
    }
    let known: Vec<String> = sqlx::query_scalar(
        "SELECT md.key FROM module_definitions md WHERE md.key = ANY($1) AND (md.selectable OR EXISTS \
         (SELECT 1 FROM program_modules pm WHERE pm.program_id = $2 AND pm.module_id = md.id))",
    )
    .bind(keys)
    .bind(program_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(db_error)?;

    let unknown: Vec<&str> = keys.iter().filter(|key| !known.contains(key)).map(String::as_str).collect();
    if unknown.is_empty() {
        Ok(())
    } else {
        Err(ApplicationError::unprocessable_entity(&format!(
            "unknown or non-selectable modules: {}",
            unknown.join(", ")
        )))
    }
}

/// Gives the program the default modules (Overview), the premade pages every program has (Danger zone,
/// `universal`) with their ancestors, every named module with its ancestors (so the tree is connected) and, for a named group, everything under it. `<@` is the ltree
/// "descendant of" operator. Modules the program already has are left exactly as they are.
pub async fn ensure_modules(conn: &mut PgConnection, program_id: Uuid, keys: &[String]) -> Result<(), ApplicationError> {
    let had_notifications = has_notifications(conn, program_id).await?;
    sqlx::query(
        "INSERT INTO program_modules (program_id, module_id, enabled, position, mode) \
         SELECT $1, md.id, true, md.position, md.default_mode FROM module_definitions md \
         WHERE md.source = 'default' \
            OR EXISTS (SELECT 1 FROM module_definitions u WHERE u.source = 'premade' AND u.universal \
                       AND u.ancestry <@ md.ancestry) \
            OR EXISTS (SELECT 1 FROM module_definitions s WHERE s.key = ANY($2) \
                       AND (s.ancestry <@ md.ancestry OR md.ancestry <@ s.ancestry)) \
         ON CONFLICT (program_id, module_id) DO NOTHING",
    )
    .bind(program_id)
    .bind(keys)
    .execute(&mut *conn)
    .await
    .map_err(db_error)?;
    // Only when the page arrives: an admin who emptied the grid must not get the defaults back on the
    // next program update.
    if !had_notifications && has_notifications(conn, program_id).await? {
        seed_default_notifications(conn, program_id).await?;
    }
    seed_default_rules(conn, program_id).await
}

async fn has_notifications(conn: &mut PgConnection, program_id: Uuid) -> Result<bool, ApplicationError> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM program_modules pm JOIN module_definitions md ON md.id = pm.module_id \
         WHERE pm.program_id = $1 AND md.handler_key = 'notifications')",
    )
    .bind(program_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(db_error)
}

/// A program that just got the Notifications page starts with the usual trigger points.
async fn seed_default_notifications(conn: &mut PgConnection, program_id: Uuid) -> Result<(), ApplicationError> {
    for (position, trigger) in default_triggers().into_iter().enumerate() {
        sqlx::query(
            "INSERT INTO program_notification_triggers (program_id, key, label, email, sms, push, in_app, position) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (program_id, key) DO NOTHING",
        )
        .bind(program_id)
        .bind(&trigger.key)
        .bind(&trigger.label)
        .bind(trigger.email)
        .bind(trigger.sms)
        .bind(trigger.push)
        .bind(trigger.in_app)
        .bind(i32::try_from(position).unwrap_or(i32::MAX))
        .execute(&mut *conn)
        .await
        .map_err(db_error)?;
    }
    Ok(())
}

#[derive(Debug, FromRow)]
struct DefaultRuleRow {
    module_id: Uuid,
    rule_family: Option<String>,
    default_rule: serde_json::Value,
    default_input_schema: Option<serde_json::Value>,
}

/// A rule page that has a starter rule (`module_definitions.default_rule`) and nothing saved yet gets
/// it as draft 1, so the rule editor opens on a working rule instead of an empty canvas. It runs on
/// the caller's transaction, so the pages and their starter rules appear together. The version is a
/// normal one: the admin's first save becomes version 2, and it can be published as it is. Nothing is
/// written for a page whose rule set already has a version.
async fn seed_default_rules(conn: &mut PgConnection, program_id: Uuid) -> Result<(), ApplicationError> {
    let pending = sqlx::query_as::<_, DefaultRuleRow>(
        "SELECT pm.module_id, md.rule_family, md.default_rule, md.default_input_schema FROM program_modules pm \
         JOIN module_definitions md ON md.id = pm.module_id \
         LEFT JOIN rule_sets rs ON rs.program_id = pm.program_id AND rs.module_id = pm.module_id \
         WHERE pm.program_id = $1 AND pm.mode = 'rule' AND md.default_rule IS NOT NULL \
           AND rs.draft_version IS NULL AND rs.published_version IS NULL",
    )
    .bind(program_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(db_error)?;

    for rule in pending {
        let rule_set_id: Uuid = sqlx::query_scalar(
            "INSERT INTO rule_sets (id, program_id, module_id, rule_type, draft_version) VALUES ($1, $2, $3, $4, 1) \
             ON CONFLICT (program_id, module_id) DO UPDATE SET draft_version = 1, update_time = now() RETURNING id",
        )
        .bind(Uuid::now_v7())
        .bind(program_id)
        .bind(rule.module_id)
        .bind(&rule.rule_family)
        .fetch_one(&mut *conn)
        .await
        .map_err(db_error)?;
        // The starter rule is both the model and the graph: it is already a decision graph, and it is complete.
        sqlx::query(
            "INSERT INTO rule_versions (id, rule_set_id, version, status, definition, compiled, valid, validation_errors, \
                input_schema, content_hash, size_bytes, created_by) \
             VALUES ($1, $2, 1, 'draft', $3, $3, true, '[]'::jsonb, $4, md5($3::jsonb::text || COALESCE($4::jsonb::text, '')), \
                octet_length($3::jsonb::text), 'system')",
        )
        .bind(Uuid::now_v7())
        .bind(rule_set_id)
        .bind(&rule.default_rule)
        .bind(&rule.default_input_schema)
        .execute(&mut *conn)
        .await
        .map_err(db_error)?;
    }
    Ok(())
}

#[derive(Debug, FromRow)]
struct ModuleRow {
    id: Uuid,
    allow_components: bool,
    source: String,
    rule_family: Option<String>,
}

#[derive(Debug, FromRow)]
struct ProgramModuleRow {
    mode: Option<String>,
    enabled: bool,
}

#[derive(Debug, FromRow)]
struct ExistingField {
    id: Uuid,
    key: String,
    field_type: String,
}

/// Applies the entries that do something (a mode, settings, or an on/off switch). Returns whether
/// anything was changed.
pub async fn apply_changes(
    conn: &mut PgConnection,
    program_id: Uuid,
    changes: &[ModuleChange],
) -> Result<bool, ApplicationError> {
    let mut changed = false;

    for change in changes.iter().filter(|c| c.has_changes()) {
        let module = sqlx::query_as::<_, ModuleRow>(
            "SELECT id, allow_components, source, rule_family FROM module_definitions WHERE key = $1",
        )
        .bind(&change.key)
        .fetch_optional(&mut *conn)
        .await
        .map_err(db_error)?
        .ok_or_else(|| ApplicationError::unprocessable_entity(&format!("unknown module {:?}", change.key)))?;

        // Lock the row: two admins cannot change one leaf's mode at the same time.
        let current = sqlx::query_as::<_, ProgramModuleRow>(
            "SELECT mode, enabled FROM program_modules WHERE program_id = $1 AND module_id = $2 FOR UPDATE",
        )
        .bind(program_id)
        .bind(module.id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(db_error)?
        .ok_or_else(|| {
            ApplicationError::unprocessable_entity(&format!("module {:?} is not part of this program", change.key))
        })?;

        if let Some(enabled) = change.enabled
            && enabled != current.enabled
        {
            sqlx::query("UPDATE program_modules SET enabled = $3, update_time = now() WHERE program_id = $1 AND module_id = $2")
                .bind(program_id)
                .bind(module.id)
                .bind(enabled)
                .execute(&mut *conn)
                .await
                .map_err(db_error)?;
            changed = true;
        }

        let current_mode = current.mode.as_deref().and_then(|m| ModuleMode::parse(m).ok());
        let mut target_mode = current_mode;

        if let Some(mode) = change.mode {
            if !module.allow_components {
                return Err(ApplicationError::unprocessable_entity(&format!(
                    "module {:?} is not configurable: it takes neither a rule nor settings",
                    change.key
                )));
            }
            if module.source == "premade" && mode == ModuleMode::Rule {
                return Err(ApplicationError::unprocessable_entity(&format!(
                    "module {:?} is a premade module: it can only use settings, not a rule",
                    change.key
                )));
            }

            if current_mode != Some(mode) {
                // A leaf that already holds something cannot silently become something else.
                if current_mode == Some(ModuleMode::Settings) {
                    let live: i64 = sqlx::query_scalar(
                        "SELECT COUNT(*) FROM program_fields WHERE program_id = $1 AND module_id = $2 AND state = 'ACTIVE'",
                    )
                    .bind(program_id)
                    .bind(module.id)
                    .fetch_one(&mut *conn)
                    .await
                    .map_err(db_error)?;
                    if live > 0 {
                        return Err(ApplicationError::failed_precondition(&format!(
                            "module {:?} still has settings; remove them first (send settings: []) before changing its mode",
                            change.key
                        )));
                    }
                }
                if current_mode == Some(ModuleMode::Rule) {
                    let versioned: i64 = sqlx::query_scalar(
                        "SELECT COUNT(*) FROM rule_sets WHERE program_id = $1 AND module_id = $2 \
                         AND (draft_version IS NOT NULL OR published_version IS NOT NULL)",
                    )
                    .bind(program_id)
                    .bind(module.id)
                    .fetch_one(&mut *conn)
                    .await
                    .map_err(db_error)?;
                    if versioned > 0 {
                        return Err(ApplicationError::failed_precondition(&format!(
                            "module {:?} already has a rule with versions, so its mode cannot change",
                            change.key
                        )));
                    }
                    sqlx::query("DELETE FROM rule_sets WHERE program_id = $1 AND module_id = $2")
                        .bind(program_id)
                        .bind(module.id)
                        .execute(&mut *conn)
                        .await
                        .map_err(db_error)?;
                }

                sqlx::query(
                    "UPDATE program_modules SET mode = $3, version = version + 1, update_time = now() \
                     WHERE program_id = $1 AND module_id = $2",
                )
                .bind(program_id)
                .bind(module.id)
                .bind(mode.as_str())
                .execute(&mut *conn)
                .await
                .map_err(db_error)?;
                changed = true;
            }
            target_mode = Some(mode);

            if mode == ModuleMode::Rule {
                // One rule set per rule leaf. The rule engine itself is a front-end component, so
                // there is nothing to validate here beyond the row existing.
                sqlx::query(
                    "INSERT INTO rule_sets (id, program_id, module_id, rule_type) VALUES ($1, $2, $3, $4) \
                     ON CONFLICT (program_id, module_id) DO NOTHING",
                )
                .bind(Uuid::now_v7())
                .bind(program_id)
                .bind(module.id)
                .bind(&module.rule_family)
                .execute(&mut *conn)
                .await
                .map_err(db_error)?;
            }
        }

        if let Some(settings) = &change.settings {
            if target_mode != Some(ModuleMode::Settings) {
                return Err(ApplicationError::unprocessable_entity(&format!(
                    "module {:?}: settings need mode \"settings\" (set it in the same entry)",
                    change.key
                )));
            }
            if replace_settings(conn, program_id, module.id, &change.key, settings).await? {
                changed = true;
            }
        }
    }

    seed_default_rules(conn, program_id).await?;
    Ok(changed)
}

/// Makes the module's settings exactly `inputs`: an input with an `id` keeps and updates that
/// setting, one without creates a new setting, and a stored setting that is not listed is removed
/// (soft delete, so its saved values survive). Options are matched by `code`: a removed option is
/// deactivated, never deleted, so a saved value can never lose the option it points at.
async fn replace_settings(
    conn: &mut PgConnection,
    program_id: Uuid,
    module_id: Uuid,
    module_key: &str,
    inputs: &[SettingInput],
) -> Result<bool, ApplicationError> {
    let existing = sqlx::query_as::<_, ExistingField>(
        "SELECT id, key, field_type FROM program_fields WHERE program_id = $1 AND module_id = $2 AND state = 'ACTIVE'",
    )
    .bind(program_id)
    .bind(module_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(db_error)?;
    let by_id: HashMap<Uuid, &ExistingField> = existing.iter().map(|f| (f.id, f)).collect();

    // Every id must be one of THIS module's settings.
    for input in inputs {
        if let Some(id) = input.id
            && !by_id.contains_key(&id)
        {
            return Err(ApplicationError::unprocessable_entity(&format!(
                "module {module_key:?}: setting {id} does not exist here"
            )));
        }
    }

    // Final keys: an existing setting keeps its key (renaming never changes it); a new one gets
    // a key from its name. They must all differ.
    let kept: HashSet<Uuid> = inputs.iter().filter_map(|i| i.id).collect();
    let mut keys: Vec<String> = Vec::with_capacity(inputs.len());
    let mut seen = HashSet::new();
    for input in inputs {
        let key = match input.id {
            Some(id) => by_id[&id].key.clone(),
            None => key_from_name(&input.name),
        };
        if !seen.insert(key.clone()) {
            return Err(ApplicationError::unprocessable_entity(&format!(
                "module {module_key:?}: a setting named like {:?} already exists",
                input.name.trim()
            )));
        }
        keys.push(key);
    }

    let mut changed = false;

    // 1. Remove what is no longer listed.
    let removed: Vec<Uuid> = existing.iter().filter(|f| !kept.contains(&f.id)).map(|f| f.id).collect();
    if !removed.is_empty() {
        sqlx::query("UPDATE program_fields SET state = 'DELETED', update_time = now() WHERE id = ANY($1)")
            .bind(&removed)
            .execute(&mut *conn)
            .await
            .map_err(db_error)?;
        changed = true;
    }

    // 2. Update the kept ones and create the new ones.
    for (position, (input, key)) in inputs.iter().zip(keys.iter()).enumerate() {
        let options_source = input
            .field_type
            .is_choice()
            .then(|| if input.library_id.is_some() { "library" } else { "manual" });
        let library_id = if options_source == Some("library") { input.library_id } else { None };

        let field_id = match input.id {
            Some(id) => {
                let current = by_id[&id];
                if current.field_type != input.field_type.as_str() {
                    let has_values: bool =
                        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM program_field_values WHERE field_id = $1)")
                            .bind(id)
                            .fetch_one(&mut *conn)
                            .await
                            .map_err(db_error)?;
                    if has_values {
                        return Err(ApplicationError::failed_precondition(&format!(
                            "module {module_key:?}: setting {:?} already has saved values, so its type cannot change",
                            input.name.trim()
                        )));
                    }
                }
                sqlx::query(
                    "UPDATE program_fields SET name = $2, field_type = $3, options_source = $4, library_id = $5, \
                            required = $6, default_value = $7, position = $8, update_time = now() \
                     WHERE id = $1",
                )
                .bind(id)
                .bind(input.name.trim())
                .bind(input.field_type.as_str())
                .bind(options_source)
                .bind(library_id)
                .bind(input.required)
                .bind(&input.default_value)
                .bind(position as i32)
                .execute(&mut *conn)
                .await
                .map_err(db_error)?;
                id
            }
            None => {
                let id = Uuid::now_v7();
                sqlx::query(
                    "INSERT INTO program_fields \
                        (id, program_id, module_id, name, key, field_type, options_source, library_id, required, default_value, position) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                )
                .bind(id)
                .bind(program_id)
                .bind(module_id)
                .bind(input.name.trim())
                .bind(key)
                .bind(input.field_type.as_str())
                .bind(options_source)
                .bind(library_id)
                .bind(input.required)
                .bind(&input.default_value)
                .bind(position as i32)
                .execute(&mut *conn)
                .await
                .map_err(|err| {
                    if let sqlx::Error::Database(db_err) = &err
                        && db_err.is_unique_violation()
                    {
                        return ApplicationError::unprocessable_entity(&format!(
                            "module {module_key:?}: a setting named like {:?} already exists",
                            input.name.trim()
                        ));
                    }
                    db_error(err)
                })?;
                id
            }
        };
        changed = true;

        // 3. Options: the admin's own values only when there is no library.
        if options_source == Some("manual") {
            sync_manual_options(conn, field_id, &input.options).await?;
        } else {
            sqlx::query("UPDATE field_options SET active = false, update_time = now() WHERE field_id = $1 AND active")
                .bind(field_id)
                .execute(&mut *conn)
                .await
                .map_err(db_error)?;
        }
    }

    Ok(changed)
}

/// Upserts the listed options by `code` (two statements however many there are), then deactivates
/// the ones that are no longer listed.
async fn sync_manual_options(
    conn: &mut PgConnection,
    field_id: Uuid,
    options: &[OptionInput],
) -> Result<(), ApplicationError> {
    let ids: Vec<Uuid> = options.iter().map(|_| Uuid::now_v7()).collect();
    let codes: Vec<String> = options.iter().map(code_for_option).collect();
    let labels: Vec<String> = options.iter().map(|o| o.label.trim().to_string()).collect();
    let positions: Vec<i32> = (0..options.len() as i32).collect();

    sqlx::query(
        "INSERT INTO field_options (id, field_id, code, label, position) \
         SELECT u.id, $2, u.code, u.label, u.position \
         FROM unnest($1::uuid[], $3::text[], $4::text[], $5::int[]) AS u(id, code, label, position) \
         ON CONFLICT (field_id, code) DO UPDATE \
            SET label = EXCLUDED.label, position = EXCLUDED.position, active = true, update_time = now()",
    )
    .bind(&ids)
    .bind(field_id)
    .bind(&codes)
    .bind(&labels)
    .bind(&positions)
    .execute(&mut *conn)
    .await
    .map_err(db_error)?;

    sqlx::query(
        "UPDATE field_options SET active = false, update_time = now() \
         WHERE field_id = $1 AND active AND NOT (code = ANY($2))",
    )
    .bind(field_id)
    .bind(&codes)
    .execute(&mut *conn)
    .await
    .map_err(db_error)?;
    Ok(())
}
