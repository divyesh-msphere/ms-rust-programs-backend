use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    GetOverviewRequest, GetOverviewResponse, ListAuditLogsRequest, ListAuditLogsResponse,
};
use crate::adapters::inputs::grpc_apis::program_overview::program_overview_mappers::{
    GetOverviewGrpcMapper, ListAuditLogsGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct ProgramOverviewServiceImpl {
    app_state: Arc<AppState>,
}

impl ProgramOverviewServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl ProgramOverviewServiceImpl {
    pub async fn get_overview(&self, request: ServiceRequest<'_, GetOverviewRequest>) -> ServiceResult<GetOverviewResponse> {
        let input = GetOverviewGrpcMapper::input(request.to_owned_message()).map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .program_overview
            .get_overview
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;
        Response::ok(GetOverviewGrpcMapper::output_success(output))
    }

    pub async fn list_audit_logs(&self, request: ServiceRequest<'_, ListAuditLogsRequest>) -> ServiceResult<ListAuditLogsResponse> {
        let input = ListAuditLogsGrpcMapper::input(request.to_owned_message()).map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .audit_log
            .list_audit_logs
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;
        Response::ok(ListAuditLogsGrpcMapper::output_success(output))
    }
}
