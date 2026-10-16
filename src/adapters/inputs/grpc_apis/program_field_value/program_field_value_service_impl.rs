use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchUpdateFieldValuesRequest, BatchUpdateFieldValuesResponse, ListFieldValuesRequest, ListFieldValuesResponse,
};
use crate::adapters::inputs::grpc_apis::program_field_value::program_field_value_mappers::{
    BatchUpdateFieldValuesGrpcMapper, ListFieldValuesGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct ProgramFieldValueServiceImpl {
    app_state: Arc<AppState>,
}

impl ProgramFieldValueServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl ProgramFieldValueServiceImpl {
    pub async fn list_field_values(
        &self,
        request: ServiceRequest<'_, ListFieldValuesRequest>,
    ) -> ServiceResult<ListFieldValuesResponse> {
        let input = ListFieldValuesGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .program_field_value
            .list_field_values
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListFieldValuesGrpcMapper::output_success(output))
    }

    pub async fn batch_update_field_values(
        &self,
        actor: String,
        request: ServiceRequest<'_, BatchUpdateFieldValuesRequest>,
    ) -> ServiceResult<BatchUpdateFieldValuesResponse> {
        let input = BatchUpdateFieldValuesGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .program_field_value
            .batch_update_field_values
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(BatchUpdateFieldValuesGrpcMapper::output_success(output))
    }
}
