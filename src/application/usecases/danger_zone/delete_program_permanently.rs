use super::DangerZoneInput;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::danger_zone::DangerZoneRepository;
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

pub struct DeleteProgramPermanentlyUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    danger_zone_repository: Arc<dyn DangerZoneRepository>,
}

impl DeleteProgramPermanentlyUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, danger_zone_repository: Arc<dyn DangerZoneRepository>) -> Self {
        Self { program_repository, danger_zone_repository }
    }
}

#[async_trait::async_trait]
impl UseCase<(), DangerZoneInput> for DeleteProgramPermanentlyUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<DangerZoneInput>) -> Result<(), ApplicationError> {
        let input = input
            .ok_or_else(|| ApplicationError::validation_error("Input is required in DeleteProgramPermanentlyUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("DeleteProgramPermanentlyUseCase::get_program"))
            .await?;

        // The repository refuses (409) while the program has operational history.
        self.danger_zone_repository
            .delete_permanently(program.id, &input.actor)
            .instrument(info_span!("DeleteProgramPermanentlyUseCase::delete"))
            .await
    }
}
