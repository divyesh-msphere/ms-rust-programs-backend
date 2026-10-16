use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    CreateProgramRequest, CreateProgramResponse, DeleteProgramRequest, GetProgramRequest, GetProgramResponse,
    ListProgramsRequest, ListProgramsResponse, UndeleteProgramRequest, UndeleteProgramResponse,
    UpdateProgramRequest, UpdateProgramResponse,
};
use crate::adapters::inputs::grpc_apis::program::program_mappers::{
    CreateProgramGrpcMapper, DeleteProgramGrpcMapper, GetProgramGrpcMapper, ListProgramsGrpcMapper,
    UndeleteProgramGrpcMapper, UpdateProgramGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct ProgramServiceImpl {
    app_state: Arc<AppState>,
}

impl ProgramServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl ProgramServiceImpl {
    pub async fn list_programs(
        &self,
        request: ServiceRequest<'_, ListProgramsRequest>,
    ) -> ServiceResult<ListProgramsResponse> {
        let input = ListProgramsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .program
            .list_programs
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListProgramsGrpcMapper::output_success(output))
    }

    pub async fn get_program(
        &self,
        request: ServiceRequest<'_, GetProgramRequest>,
    ) -> ServiceResult<GetProgramResponse> {
        let input = GetProgramGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .program
            .get_program
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetProgramGrpcMapper::output_success(output))
    }

    pub async fn create_program(
        &self,
        actor: String,
        request: ServiceRequest<'_, CreateProgramRequest>,
    ) -> ServiceResult<CreateProgramResponse> {
        let input = CreateProgramGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .program
            .create_program
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(CreateProgramGrpcMapper::output_success(output))
    }

    pub async fn update_program(
        &self,
        actor: String,
        request: ServiceRequest<'_, UpdateProgramRequest>,
    ) -> ServiceResult<UpdateProgramResponse> {
        let input = UpdateProgramGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .program
            .update_program
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(UpdateProgramGrpcMapper::output_success(output))
    }

    pub async fn delete_program(
        &self,
        actor: String,
        request: ServiceRequest<'_, DeleteProgramRequest>,
    ) -> ServiceResult<buffa_types::google::protobuf::Empty> {
        let input = DeleteProgramGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        self.app_state
            .use_cases
            .program
            .delete_program
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(buffa_types::google::protobuf::Empty::default())
    }

    pub async fn undelete_program(
        &self,
        actor: String,
        request: ServiceRequest<'_, UndeleteProgramRequest>,
    ) -> ServiceResult<UndeleteProgramResponse> {
        let input = UndeleteProgramGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .program
            .undelete_program
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(UndeleteProgramGrpcMapper::output_success(output))
    }
}
