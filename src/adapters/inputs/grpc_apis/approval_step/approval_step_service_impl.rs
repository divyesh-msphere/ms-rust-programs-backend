use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchUpdateApprovalStepsRequest, BatchUpdateApprovalStepsResponse, CreateApprovalStepRequest,
    CreateApprovalStepResponse, DeleteApprovalStepRequest, GetApprovalStepRequest, GetApprovalStepResponse,
    ListApprovalStepsRequest, ListApprovalStepsResponse, UpdateApprovalStepRequest, UpdateApprovalStepResponse,
};
use crate::adapters::inputs::grpc_apis::approval_step::approval_step_mappers::{
    BatchUpdateApprovalStepsGrpcMapper, CreateApprovalStepGrpcMapper, DeleteApprovalStepGrpcMapper,
    GetApprovalStepGrpcMapper, ListApprovalStepsGrpcMapper, UpdateApprovalStepGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use buffa_types::google::protobuf::Empty;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct ApprovalStepServiceImpl {
    app_state: Arc<AppState>,
}

impl ApprovalStepServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl ApprovalStepServiceImpl {
    pub async fn list_approval_steps(
        &self,
        request: ServiceRequest<'_, ListApprovalStepsRequest>,
    ) -> ServiceResult<ListApprovalStepsResponse> {
        let input = ListApprovalStepsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .approval_step
            .list_approval_steps
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListApprovalStepsGrpcMapper::output_success(output))
    }

    pub async fn get_approval_step(
        &self,
        request: ServiceRequest<'_, GetApprovalStepRequest>,
    ) -> ServiceResult<GetApprovalStepResponse> {
        let input = GetApprovalStepGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .approval_step
            .get_approval_step
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetApprovalStepGrpcMapper::output_success(output))
    }

    pub async fn create_approval_step(
        &self,
        actor: String,
        request: ServiceRequest<'_, CreateApprovalStepRequest>,
    ) -> ServiceResult<CreateApprovalStepResponse> {
        let input = CreateApprovalStepGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .approval_step
            .create_approval_step
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(CreateApprovalStepGrpcMapper::output_success(output))
    }

    pub async fn update_approval_step(
        &self,
        actor: String,
        request: ServiceRequest<'_, UpdateApprovalStepRequest>,
    ) -> ServiceResult<UpdateApprovalStepResponse> {
        let input = UpdateApprovalStepGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .approval_step
            .update_approval_step
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(UpdateApprovalStepGrpcMapper::output_success(output))
    }

    pub async fn delete_approval_step(
        &self,
        actor: String,
        request: ServiceRequest<'_, DeleteApprovalStepRequest>,
    ) -> ServiceResult<Empty> {
        let input = DeleteApprovalStepGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        self
            .app_state
            .use_cases
            .approval_step
            .delete_approval_step
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(Empty::default())
    }

    pub async fn batch_update_approval_steps(
        &self,
        actor: String,
        request: ServiceRequest<'_, BatchUpdateApprovalStepsRequest>,
    ) -> ServiceResult<BatchUpdateApprovalStepsResponse> {
        let input = BatchUpdateApprovalStepsGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .approval_step
            .batch_update_approval_steps
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(BatchUpdateApprovalStepsGrpcMapper::output_success(output))
    }
}
