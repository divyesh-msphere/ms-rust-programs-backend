use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    CreateSignoffRequest, CreateSignoffResponse, GetSignoffRequest, GetSignoffResponse, ListSignoffsRequest,
    ListSignoffsResponse,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::grpc_apis::signoff::signoff_mappers::{
    CreateSignoffGrpcMapper, GetSignoffGrpcMapper, ListSignoffsGrpcMapper,
};
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct SignoffServiceImpl {
    app_state: Arc<AppState>,
}

impl SignoffServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl SignoffServiceImpl {
    pub async fn list_signoffs(
        &self,
        request: ServiceRequest<'_, ListSignoffsRequest>,
    ) -> ServiceResult<ListSignoffsResponse> {
        let input = ListSignoffsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .signoff
            .list_signoffs
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListSignoffsGrpcMapper::output_success(output))
    }

    pub async fn get_signoff(
        &self,
        request: ServiceRequest<'_, GetSignoffRequest>,
    ) -> ServiceResult<GetSignoffResponse> {
        let input = GetSignoffGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .signoff
            .get_signoff
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetSignoffGrpcMapper::output_success(output))
    }

    pub async fn create_signoff(
        &self,
        actor: String,
        request: ServiceRequest<'_, CreateSignoffRequest>,
    ) -> ServiceResult<CreateSignoffResponse> {
        let input = CreateSignoffGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .signoff
            .create_signoff
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(CreateSignoffGrpcMapper::output_success(output))
    }
}
