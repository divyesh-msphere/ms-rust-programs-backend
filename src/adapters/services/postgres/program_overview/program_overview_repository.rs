use crate::adapters::services::postgres::db_connection::PostgresDbConnection;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::program_overview::{
    AuditEntry, FUNNEL_STATUSES, Overview, ProgramOverviewRepository, ProgramStats, RuleSync, rule_state,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ms_telemetry_rs::tracing::error;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize, Default)]
struct StatsJson {
    total_applications: i32,
    pending_approvals: i32,
    active_enrollments: i32,
    enrollments_total: i32,
}

#[derive(Debug, Deserialize)]
struct RuleJson {
    module_key: String,
    name: String,
    rule_type: Option<String>,
    draft_version: Option<i32>,
    published_version: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct ActivityJson {
    id: Uuid,
    create_time: DateTime<Utc>,
    actor: String,
    action: String,
    target_type: String,
    target_id: String,
}

#[derive(Debug, Deserialize)]
struct OverviewJson {
    stats: Option<StatsJson>,
    funnel: HashMap<String, i64>,
    rules: Vec<RuleJson>,
    published_config_version: Option<i32>,
    recent: Vec<ActivityJson>,
}

pub struct ProgramOverviewRepositoryPostgresImpl {
    db_connection: Arc<PostgresDbConnection>,
}

impl ProgramOverviewRepositoryPostgresImpl {
    pub fn new(db_connection: Arc<PostgresDbConnection>) -> Self {
        Self { db_connection }
    }
}

#[async_trait]
impl ProgramOverviewRepository for ProgramOverviewRepositoryPostgresImpl {
    async fn get(&self, program_id: Uuid, recent_activity: i64) -> Result<Overview, ApplicationError> {
        // ONE round trip for the whole page. Each piece is an indexed read that starts at
        // `program_id`: the counters row (primary key), the applications-by-status counts (index-only
        // on (program_id, status, id)), the rule leaves with their rule sets, the latest publication
        // (index-only, newest first) and the latest audit entries (newest first).
        let document: Value = sqlx::query_scalar(
            "SELECT jsonb_build_object( \
               'stats', (SELECT jsonb_build_object('total_applications', total_applications, \
                                                    'pending_approvals', pending_approvals, \
                                                    'active_enrollments', active_enrollments, \
                                                    'enrollments_total', enrollments_total) \
                         FROM program_stats WHERE program_id = $1), \
               'funnel', COALESCE((SELECT jsonb_object_agg(status, n) \
                                   FROM (SELECT status, COUNT(*) AS n FROM applications \
                                         WHERE program_id = $1 GROUP BY status) f), '{}'::jsonb), \
               'rules', COALESCE((SELECT jsonb_agg(jsonb_build_object('module_key', md.key, 'name', md.name, \
                                         'rule_type', rs.rule_type, 'draft_version', rs.draft_version, \
                                         'published_version', rs.published_version) ORDER BY pm.position, md.key) \
                                  FROM program_modules pm \
                                  JOIN module_definitions md ON md.id = pm.module_id \
                                  JOIN rule_sets rs ON rs.program_id = pm.program_id AND rs.module_id = pm.module_id \
                                  WHERE pm.program_id = $1 AND pm.enabled AND pm.mode = 'rule'), '[]'::jsonb), \
               'published_config_version', (SELECT config_version FROM publications \
                                            WHERE program_id = $1 ORDER BY id DESC LIMIT 1), \
               'recent', COALESCE((SELECT jsonb_agg(jsonb_build_object('id', id, 'create_time', create_time, \
                                         'actor', actor, 'action', action, 'target_type', target_type, \
                                         'target_id', target_id) ORDER BY create_time DESC, id DESC) \
                                   FROM (SELECT id, create_time, actor, action, target_type, target_id \
                                         FROM audit_log WHERE program_id = $1 \
                                         ORDER BY create_time DESC, id DESC LIMIT $2) a), '[]'::jsonb) \
             )",
        )
        .bind(program_id)
        .bind(recent_activity)
        .fetch_one(self.db_connection.get_pool())
        .await
        .map_err(|err| {
            error!("DB error: {:?}", err);
            ApplicationError::internal_error(Some(err))
        })?;

        let parsed: OverviewJson = serde_json::from_value(document).map_err(|err| {
            error!("the overview document could not be read: {:?}", err);
            ApplicationError::internal_error(Some(err))
        })?;

        let stats = parsed.stats.unwrap_or_default();
        let funnel = FUNNEL_STATUSES
            .iter()
            .map(|status| (status.to_string(), parsed.funnel.get(*status).copied().unwrap_or(0)))
            .collect();

        Ok(Overview {
            stats: ProgramStats {
                total_applications: stats.total_applications,
                pending_approvals: stats.pending_approvals,
                active_enrollments: stats.active_enrollments,
                enrollments_total: stats.enrollments_total,
            },
            funnel,
            rules: parsed
                .rules
                .into_iter()
                .map(|r| RuleSync {
                    state: rule_state(r.draft_version, r.published_version).to_string(),
                    module_key: r.module_key,
                    name: r.name,
                    rule_type: r.rule_type,
                    draft_version: r.draft_version,
                    published_version: r.published_version,
                })
                .collect(),
            published_config_version: parsed.published_config_version,
            recent_activity: parsed
                .recent
                .into_iter()
                .map(|a| AuditEntry {
                    id: a.id,
                    program_id,
                    create_time: a.create_time,
                    actor: a.actor,
                    action: a.action,
                    target_type: a.target_type,
                    target_id: a.target_id,
                    before: None,
                    after: None,
                })
                .collect(),
        })
    }
}
