use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    AuditLogEntry as GrpcAuditLogEntry, FunnelEntry, GetOverviewRequest, GetOverviewResponse, ListAuditLogsRequest,
    ListAuditLogsResponse, OverviewStats, PublishedRules, RuleFamilyStatus,
};
use crate::adapters::inputs::grpc_apis::program::program_mappers::to_grpc_program;
use crate::application::errors::application_error::ApplicationError;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::audit_log::list_audit_logs::{ListAuditLogsInput, ListAuditLogsOutput};
use crate::application::usecases::program_overview::get_overview::{GetOverviewInput, GetOverviewOutput};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program_overview::AuditEntry;
use crate::adapters::inputs::grpc_apis::shared::json_struct::json_to_buffa_value;

fn to_grpc_audit(entry: AuditEntry, with_detail: bool) -> GrpcAuditLogEntry {
    let summary = entry.summary();
    GrpcAuditLogEntry {
        path: entry.path(),
        id: entry.id.to_string(),
        create_time: format_datetime(entry.create_time),
        actor: entry.actor,
        action: entry.action,
        summary,
        target_type: entry.target_type,
        target_id: entry.target_id,
        before: if with_detail { entry.before.map(json_to_buffa_value).into() } else { None.into() },
        after: if with_detail { entry.after.map(json_to_buffa_value).into() } else { None.into() },
        ..Default::default()
    }
}

pub struct GetOverviewGrpcMapper;

impl GrpcApiMapper<GetOverviewOutput, GetOverviewInput, GetOverviewResponse, GetOverviewRequest> for GetOverviewGrpcMapper {
    fn input(request: GetOverviewRequest) -> Result<GetOverviewInput, ApplicationError> {
        Ok(GetOverviewInput { program: ResourceIdentifier::parse(&request.program) })
    }

    fn output_success(output: GetOverviewOutput) -> GetOverviewResponse {
        let overview = output.overview;
        let families: Vec<RuleFamilyStatus> = overview
            .rules
            .into_iter()
            .map(|r| RuleFamilyStatus {
                module_key: r.module_key,
                name: r.name,
                rule_type: r.rule_type,
                state: r.state,
                draft_version: r.draft_version,
                published_version: r.published_version,
                ..Default::default()
            })
            .collect();
        // Nothing to publish (in sync, or no rules saved yet) is not drift.
        let in_sync = families.iter().filter(|f| f.state == "in_sync" || f.state == "empty").count();
        let total = families.len();

        GetOverviewResponse {
            config_version: output.program.config_version,
            published_config_version: overview.published_config_version,
            unpublished_changes: output.unpublished_changes,
            program: Some(to_grpc_program(output.program)).into(),
            stats: Some(OverviewStats {
                total_applications: overview.stats.total_applications,
                pending_approvals: overview.stats.pending_approvals,
                active_enrollments: overview.stats.active_enrollments,
                enrollments_total: overview.stats.enrollments_total,
                ..Default::default()
            })
            .into(),
            funnel: overview.funnel.into_iter().map(|(status, count)| FunnelEntry { status, count, ..Default::default() }).collect(),
            published_rules: Some(PublishedRules {
                total: total as i32,
                in_sync: in_sync as i32,
                drift_detected: families.iter().any(|f| f.state == "never" || f.state == "drifted"),
                families,
                ..Default::default()
            })
            .into(),
            recent_activity: overview.recent_activity.into_iter().map(|e| to_grpc_audit(e, false)).collect(),
            ..Default::default()
        }
    }
}

pub struct ListAuditLogsGrpcMapper;

impl GrpcApiMapper<ListAuditLogsOutput, ListAuditLogsInput, ListAuditLogsResponse, ListAuditLogsRequest> for ListAuditLogsGrpcMapper {
    fn input(request: ListAuditLogsRequest) -> Result<ListAuditLogsInput, ApplicationError> {
        Ok(ListAuditLogsInput {
            program: ResourceIdentifier::parse(&request.program),
            page_size: request.max_page_size,
            page_token: request.page_token.filter(|t| !t.trim().is_empty()),
            actions: request
                .action
                .map(|a| a.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect())
                .unwrap_or_default(),
        })
    }

    fn output_success(output: ListAuditLogsOutput) -> ListAuditLogsResponse {
        ListAuditLogsResponse {
            results: output.entries.into_iter().map(|e| to_grpc_audit(e, true)).collect(),
            next_page_token: output.next_page_token,
            ..Default::default()
        }
    }
}
