use crate::adapters::inputs::http_apis::program::program_mappers::to_presenter as program_to_presenter;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::audit_log::list_audit_logs::{ListAuditLogsInput, ListAuditLogsOutput};
use crate::application::usecases::program_overview::get_overview::{GetOverviewInput, GetOverviewOutput};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program_overview::AuditEntry;
use crate::dto::payloads::json::program_overview::{GetOverviewPayload, ListAuditLogsPathPayload, ListAuditLogsQueryPayload};
use crate::dto::presenters::json::program_overview::{
    AuditEntryPresenter, FunnelEntryPresenter, ListAuditLogsPresenter, OverviewPresenter, PublishedRulesPresenter,
    RuleFamilyPresenter, StatsPresenter,
};

/// `with_detail` adds the before / after of the change (the History view); the Overview's short
/// activity list leaves them out.
pub fn audit_to_presenter(entry: AuditEntry, with_detail: bool) -> AuditEntryPresenter {
    let summary = entry.summary();
    AuditEntryPresenter {
        path: entry.path(),
        id: entry.id.to_string(),
        create_time: format_datetime(entry.create_time),
        actor: entry.actor,
        action: entry.action,
        summary,
        target_type: entry.target_type,
        target_id: entry.target_id,
        before: if with_detail { entry.before } else { None },
        after: if with_detail { entry.after } else { None },
    }
}

pub struct GetOverviewMapper;

impl GetOverviewMapper {
    pub fn input(payload: GetOverviewPayload) -> Result<GetOverviewInput, ApplicationError> {
        Ok(GetOverviewInput { program: ResourceIdentifier::parse(&payload.program) })
    }

    pub fn output_success(output: GetOverviewOutput) -> OverviewPresenter {
        let overview = output.overview;
        let families: Vec<RuleFamilyPresenter> = overview
            .rules
            .into_iter()
            .map(|r| RuleFamilyPresenter {
                module_key: r.module_key,
                name: r.name,
                rule_type: r.rule_type,
                state: r.state,
                draft_version: r.draft_version,
                published_version: r.published_version,
            })
            .collect();
        let in_sync = families.iter().filter(|f| f.state == "in_sync").count();

        OverviewPresenter {
            config_version: output.program.config_version,
            published_config_version: overview.published_config_version,
            unpublished_changes: output.unpublished_changes,
            program: program_to_presenter(output.program),
            stats: StatsPresenter {
                total_applications: overview.stats.total_applications,
                pending_approvals: overview.stats.pending_approvals,
                active_enrollments: overview.stats.active_enrollments,
                enrollments_total: overview.stats.enrollments_total,
            },
            funnel: overview.funnel.into_iter().map(|(status, count)| FunnelEntryPresenter { status, count }).collect(),
            published_rules: PublishedRulesPresenter {
                total: families.len(),
                in_sync,
                drift_detected: in_sync != families.len(),
                families,
            },
            recent_activity: overview.recent_activity.into_iter().map(|e| audit_to_presenter(e, false)).collect(),
        }
    }
}

pub struct ListAuditLogsMapper;

impl ListAuditLogsMapper {
    pub fn input(
        path: ListAuditLogsPathPayload,
        query: ListAuditLogsQueryPayload,
    ) -> Result<ListAuditLogsInput, ApplicationError> {
        Ok(ListAuditLogsInput {
            program: ResourceIdentifier::parse(&path.program),
            page_size: query.max_page_size,
            page_token: query.page_token.filter(|t| !t.trim().is_empty()),
            actions: query
                .action
                .map(|a| a.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect())
                .unwrap_or_default(),
        })
    }

    pub fn output_success(output: ListAuditLogsOutput) -> ListAuditLogsPresenter {
        ListAuditLogsPresenter {
            results: output.entries.into_iter().map(|e| audit_to_presenter(e, true)).collect(),
            next_page_token: output.next_page_token,
        }
    }
}
