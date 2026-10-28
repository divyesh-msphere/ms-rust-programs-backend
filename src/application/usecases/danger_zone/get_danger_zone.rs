use super::{DangerZoneInput, DangerZoneOutput};
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::danger_zone::DangerZoneRepository;
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

pub struct GetDangerZoneUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    danger_zone_repository: Arc<dyn DangerZoneRepository>,
}

impl GetDangerZoneUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, danger_zone_repository: Arc<dyn DangerZoneRepository>) -> Self {
        Self { program_repository, danger_zone_repository }
    }
}

#[async_trait::async_trait]
impl UseCase<DangerZoneOutput, DangerZoneInput> for GetDangerZoneUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<DangerZoneInput>) -> Result<DangerZoneOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetDangerZoneUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetDangerZoneUseCase::get_program"))
            .await?;

        let danger_zone = self
            .danger_zone_repository
            .get(program.id)
            .instrument(info_span!("GetDangerZoneUseCase::get"))
            .await?;
        Ok(DangerZoneOutput { danger_zone })
    }
}
