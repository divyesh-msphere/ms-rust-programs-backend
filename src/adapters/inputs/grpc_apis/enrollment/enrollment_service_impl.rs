use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchCreateEnrollmentsRequest, BatchCreateEnrollmentsResponse, CreateEnrollmentRequest, CreateEnrollmentResponse,
    GetEnrollmentRequest, GetEnrollmentResponse, ListEnrollmentsRequest, ListEnrollmentsResponse,
    UpdateEnrollmentRequest, UpdateEnrollmentResponse,
};
use crate::adapters::inputs::grpc_apis::enrollment::enrollment_mappers::{
    BatchCreateEnrollmentsGrpcMapper, CreateEnrollmentGrpcMapper, GetEnrollmentGrpcMapper, ListEnrollmentsGrpcMapper,
    UpdateEnrollmentGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct EnrollmentServiceImpl {
    app_state: Arc<AppState>,
}

impl EnrollmentServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl EnrollmentServiceImpl {
    pub async fn list_enrollments(
        &self,
        request: ServiceRequest<'_, ListEnrollmentsRequest>,
    ) -> ServiceResult<ListEnrollmentsResponse> {
        let input = ListEnrollmentsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .enrollment
            .list_enrollments
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListEnrollmentsGrpcMapper::output_success(output))
    }

    pub async fn get_enrollment(
        &self,
        request: ServiceRequest<'_, GetEnrollmentRequest>,
    ) -> ServiceResult<GetEnrollmentResponse> {
        let input = GetEnrollmentGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .enrollment
            .get_enrollment
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetEnrollmentGrpcMapper::output_success(output))
    }

    pub async fn create_enrollment(
        &self,
        actor: String,
        request: ServiceRequest<'_, CreateEnrollmentRequest>,
    ) -> ServiceResult<CreateEnrollmentResponse> {
        let input = CreateEnrollmentGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .enrollment
            .create_enrollment
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(CreateEnrollmentGrpcMapper::output_success(output))
    }

    pub async fn batch_create_enrollments(
        &self,
        actor: String,
        request: ServiceRequest<'_, BatchCreateEnrollmentsRequest>,
    ) -> ServiceResult<BatchCreateEnrollmentsResponse> {
        let input = BatchCreateEnrollmentsGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .enrollment
            .batch_create_enrollments
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(BatchCreateEnrollmentsGrpcMapper::output_success(output))
    }

    pub async fn update_enrollment(
        &self,
        actor: String,
        request: ServiceRequest<'_, UpdateEnrollmentRequest>,
    ) -> ServiceResult<UpdateEnrollmentResponse> {
        let input = UpdateEnrollmentGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .enrollment
            .update_enrollment
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(UpdateEnrollmentGrpcMapper::output_success(output))
    }
}
