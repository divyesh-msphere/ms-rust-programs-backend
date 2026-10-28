use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchUpdateSettingValuesRequest, BatchUpdateSettingValuesResponse, ListSettingValuesRequest, ListSettingValuesResponse,
};
use crate::adapters::inputs::grpc_apis::program_setting_value::program_setting_value_mappers::{
    BatchUpdateSettingValuesGrpcMapper, ListSettingValuesGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct ProgramSettingValueServiceImpl {
    app_state: Arc<AppState>,
}

impl ProgramSettingValueServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl ProgramSettingValueServiceImpl {
    pub async fn list_setting_values(
        &self,
        request: ServiceRequest<'_, ListSettingValuesRequest>,
    ) -> ServiceResult<ListSettingValuesResponse> {
        let input = ListSettingValuesGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .program_setting_value
            .list_setting_values
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListSettingValuesGrpcMapper::output_success(output))
    }

    pub async fn batch_update_setting_values(
        &self,
        actor: String,
        request: ServiceRequest<'_, BatchUpdateSettingValuesRequest>,
    ) -> ServiceResult<BatchUpdateSettingValuesResponse> {
        let input = BatchUpdateSettingValuesGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .program_setting_value
            .batch_update_setting_values
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(BatchUpdateSettingValuesGrpcMapper::output_success(output))
    }
}
