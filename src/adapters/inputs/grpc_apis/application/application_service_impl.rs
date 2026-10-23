use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    CreateApplicationRequest, CreateApplicationResponse, GetApplicationRequest, GetApplicationResponse,
    ListApplicationsRequest, ListApplicationsResponse, SubmitApplicationRequest, SubmitApplicationResponse,
    WithdrawApplicationRequest, WithdrawApplicationResponse,
};
use crate::adapters::inputs::grpc_apis::application::application_mappers::{
    CreateApplicationGrpcMapper, GetApplicationGrpcMapper, ListApplicationsGrpcMapper, SubmitApplicationGrpcMapper,
    WithdrawApplicationGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct ApplicationServiceImpl {
    app_state: Arc<AppState>,
}

impl ApplicationServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl ApplicationServiceImpl {
    pub async fn list_applications(
        &self,
        request: ServiceRequest<'_, ListApplicationsRequest>,
    ) -> ServiceResult<ListApplicationsResponse> {
        let input = ListApplicationsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .application
            .list_applications
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListApplicationsGrpcMapper::output_success(output))
    }

    pub async fn get_application(
        &self,
        request: ServiceRequest<'_, GetApplicationRequest>,
    ) -> ServiceResult<GetApplicationResponse> {
        let input = GetApplicationGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .application
            .get_application
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetApplicationGrpcMapper::output_success(output))
    }

    pub async fn create_application(
        &self,
        actor: String,
        request: ServiceRequest<'_, CreateApplicationRequest>,
    ) -> ServiceResult<CreateApplicationResponse> {
        let input = CreateApplicationGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .application
            .create_application
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(CreateApplicationGrpcMapper::output_success(output))
    }

    pub async fn submit_application(
        &self,
        actor: String,
        request: ServiceRequest<'_, SubmitApplicationRequest>,
    ) -> ServiceResult<SubmitApplicationResponse> {
        let input = SubmitApplicationGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .application
            .submit_application
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(SubmitApplicationGrpcMapper::output_success(output))
    }

    pub async fn withdraw_application(
        &self,
        actor: String,
        request: ServiceRequest<'_, WithdrawApplicationRequest>,
    ) -> ServiceResult<WithdrawApplicationResponse> {
        let input = WithdrawApplicationGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .application
            .withdraw_application
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(WithdrawApplicationGrpcMapper::output_success(output))
    }
}
