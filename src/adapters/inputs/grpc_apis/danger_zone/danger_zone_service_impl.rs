use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    ArchiveProgramRequest, ArchiveProgramResponse, DeleteProgramPermanentlyRequest, GetDangerZoneRequest,
    GetDangerZoneResponse, UnarchiveProgramRequest, UnarchiveProgramResponse,
};
use crate::adapters::inputs::grpc_apis::danger_zone::danger_zone_mappers::{
    ArchiveProgramGrpcMapper, DeleteProgramPermanentlyGrpcMapper, GetDangerZoneGrpcMapper, UnarchiveProgramGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct DangerZoneServiceImpl {
    app_state: Arc<AppState>,
}

impl DangerZoneServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    pub async fn get_danger_zone(
        &self,
        actor: String,
        request: ServiceRequest<'_, GetDangerZoneRequest>,
    ) -> ServiceResult<GetDangerZoneResponse> {
        let input = GetDangerZoneGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .danger_zone
            .get_danger_zone
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;
        Response::ok(GetDangerZoneGrpcMapper::output_success(output))
    }

    pub async fn archive_program(
        &self,
        actor: String,
        request: ServiceRequest<'_, ArchiveProgramRequest>,
    ) -> ServiceResult<ArchiveProgramResponse> {
        let input = ArchiveProgramGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .danger_zone
            .archive_program
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;
        Response::ok(ArchiveProgramGrpcMapper::output_success(output))
    }

    pub async fn unarchive_program(
        &self,
        actor: String,
        request: ServiceRequest<'_, UnarchiveProgramRequest>,
    ) -> ServiceResult<UnarchiveProgramResponse> {
        let input = UnarchiveProgramGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .danger_zone
            .unarchive_program
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;
        Response::ok(UnarchiveProgramGrpcMapper::output_success(output))
    }

    pub async fn delete_program_permanently(
        &self,
        actor: String,
        request: ServiceRequest<'_, DeleteProgramPermanentlyRequest>,
    ) -> ServiceResult<buffa_types::google::protobuf::Empty> {
        let input = DeleteProgramPermanentlyGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        self.app_state
            .use_cases
            .danger_zone
            .delete_program_permanently
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;
        Response::ok(buffa_types::google::protobuf::Empty::default())
    }
}
