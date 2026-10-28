use crate::domain::aep::ResourceIdentifier;
use crate::domain::danger_zone::{DangerZone, DangerZoneRepository};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;

pub mod archive_program;
pub mod delete_program_permanently;
pub mod get_danger_zone;
pub mod unarchive_program;

use archive_program::ArchiveProgramUseCase;
use delete_program_permanently::DeleteProgramPermanentlyUseCase;
use get_danger_zone::GetDangerZoneUseCase;
use unarchive_program::UnarchiveProgramUseCase;

/// Every Danger zone call names a program and who is asking.
#[derive(Debug)]
pub struct DangerZoneInput {
    pub program: ResourceIdentifier,
    pub actor: String,
}

#[derive(Debug)]
pub struct DangerZoneOutput {
    pub danger_zone: DangerZone,
}

pub struct DangerZoneUseCases {
    pub get_danger_zone: GetDangerZoneUseCase,
    pub archive_program: ArchiveProgramUseCase,
    pub unarchive_program: UnarchiveProgramUseCase,
    pub delete_program_permanently: DeleteProgramPermanentlyUseCase,
}

impl DangerZoneUseCases {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        danger_zone_repository: Arc<dyn DangerZoneRepository>,
    ) -> Self {
        Self {
            get_danger_zone: GetDangerZoneUseCase::new(program_repository.clone(), danger_zone_repository.clone()),
            archive_program: ArchiveProgramUseCase::new(program_repository.clone(), danger_zone_repository.clone()),
            unarchive_program: UnarchiveProgramUseCase::new(program_repository.clone(), danger_zone_repository.clone()),
            delete_program_permanently: DeleteProgramPermanentlyUseCase::new(program_repository, danger_zone_repository),
        }
    }
}
