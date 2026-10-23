use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchUpdateRequirementsRequest, BatchUpdateRequirementsResponse, CreateRequirementRequest,
    CreateRequirementResponse, DeleteRequirementRequest, GetRequirementRequest, GetRequirementResponse,
    ListRequirementsRequest, ListRequirementsResponse, UpdateRequirementRequest, UpdateRequirementResponse,
};
use crate::adapters::inputs::grpc_apis::requirement::requirement_mappers::{
    BatchUpdateRequirementsGrpcMapper, CreateRequirementGrpcMapper, DeleteRequirementGrpcMapper,
    GetRequirementGrpcMapper, ListRequirementsGrpcMapper, UpdateRequirementGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use buffa_types::google::protobuf::Empty;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct RequirementServiceImpl {
    app_state: Arc<AppState>,
}

impl RequirementServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl RequirementServiceImpl {
    pub async fn list_requirements(
        &self,
        request: ServiceRequest<'_, ListRequirementsRequest>,
    ) -> ServiceResult<ListRequirementsResponse> {
        let input = ListRequirementsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .requirement
            .list_requirements
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListRequirementsGrpcMapper::output_success(output))
    }

    pub async fn get_requirement(
        &self,
        request: ServiceRequest<'_, GetRequirementRequest>,
    ) -> ServiceResult<GetRequirementResponse> {
        let input = GetRequirementGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .requirement
            .get_requirement
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetRequirementGrpcMapper::output_success(output))
    }

    pub async fn create_requirement(
        &self,
        actor: String,
        request: ServiceRequest<'_, CreateRequirementRequest>,
    ) -> ServiceResult<CreateRequirementResponse> {
        let input = CreateRequirementGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .requirement
            .create_requirement
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(CreateRequirementGrpcMapper::output_success(output))
    }

    pub async fn update_requirement(
        &self,
        actor: String,
        request: ServiceRequest<'_, UpdateRequirementRequest>,
    ) -> ServiceResult<UpdateRequirementResponse> {
        let input = UpdateRequirementGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .requirement
            .update_requirement
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(UpdateRequirementGrpcMapper::output_success(output))
    }

    pub async fn delete_requirement(
        &self,
        actor: String,
        request: ServiceRequest<'_, DeleteRequirementRequest>,
    ) -> ServiceResult<Empty> {
        let input = DeleteRequirementGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        self
            .app_state
            .use_cases
            .requirement
            .delete_requirement
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(Empty::default())
    }

    pub async fn batch_update_requirements(
        &self,
        actor: String,
        request: ServiceRequest<'_, BatchUpdateRequirementsRequest>,
    ) -> ServiceResult<BatchUpdateRequirementsResponse> {
        let input = BatchUpdateRequirementsGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .requirement
            .batch_update_requirements
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(BatchUpdateRequirementsGrpcMapper::output_success(output))
    }
}
