use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    ArchiveProgramRequest, ArchiveProgramResponse, DangerZone as GrpcDangerZone, DeleteBlockers as GrpcDeleteBlockers,
    DeleteProgramPermanentlyRequest, GetDangerZoneRequest, GetDangerZoneResponse, UnarchiveProgramRequest,
    UnarchiveProgramResponse,
};
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::danger_zone::{DangerZoneInput, DangerZoneOutput};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::danger_zone::DangerZone;

fn to_grpc_danger_zone(page: DangerZone) -> GrpcDangerZone {
    GrpcDangerZone {
        path: page.path(),
        archived: page.archived(),
        archive_time: page.archive_time.map(format_datetime),
        can_archive: page.can_archive(),
        can_unarchive: page.can_unarchive(),
        can_delete: page.can_delete(),
        delete_blocked_reason: page.blockers.reason(),
        delete_blockers: Some(GrpcDeleteBlockers {
            enrollments: page.blockers.enrollments,
            applications: page.blockers.applications,
            publications: page.blockers.publications,
            documents: page.blockers.documents,
            ..Default::default()
        })
        .into(),
        ..Default::default()
    }
}

fn input(program: &str, actor: String) -> Result<DangerZoneInput, ApplicationError> {
    Ok(DangerZoneInput { program: ResourceIdentifier::parse(program), actor })
}

pub struct GetDangerZoneGrpcMapper;

impl GetDangerZoneGrpcMapper {
    pub fn input(request: GetDangerZoneRequest, actor: String) -> Result<DangerZoneInput, ApplicationError> {
        input(&request.program, actor)
    }

    pub fn output_success(output: DangerZoneOutput) -> GetDangerZoneResponse {
        GetDangerZoneResponse { danger_zone: Some(to_grpc_danger_zone(output.danger_zone)).into(), ..Default::default() }
    }
}

pub struct ArchiveProgramGrpcMapper;

impl ArchiveProgramGrpcMapper {
    pub fn input(request: ArchiveProgramRequest, actor: String) -> Result<DangerZoneInput, ApplicationError> {
        input(&request.program, actor)
    }

    pub fn output_success(output: DangerZoneOutput) -> ArchiveProgramResponse {
        ArchiveProgramResponse { danger_zone: Some(to_grpc_danger_zone(output.danger_zone)).into(), ..Default::default() }
    }
}

pub struct UnarchiveProgramGrpcMapper;

impl UnarchiveProgramGrpcMapper {
    pub fn input(request: UnarchiveProgramRequest, actor: String) -> Result<DangerZoneInput, ApplicationError> {
        input(&request.program, actor)
    }

    pub fn output_success(output: DangerZoneOutput) -> UnarchiveProgramResponse {
        UnarchiveProgramResponse { danger_zone: Some(to_grpc_danger_zone(output.danger_zone)).into(), ..Default::default() }
    }
}

pub struct DeleteProgramPermanentlyGrpcMapper;

impl DeleteProgramPermanentlyGrpcMapper {
    pub fn input(request: DeleteProgramPermanentlyRequest, actor: String) -> Result<DangerZoneInput, ApplicationError> {
        input(&request.program, actor)
    }
}
