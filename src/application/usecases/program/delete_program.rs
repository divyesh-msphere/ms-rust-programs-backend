use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;

pub struct DeleteProgramUseCase {
    program_repository: Arc<dyn ProgramRepository>,
}

impl DeleteProgramUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>) -> Self {
        Self { program_repository }
    }
}

#[derive(Debug)]
pub struct DeleteProgramInput {
    pub identifier: ResourceIdentifier,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<(), DeleteProgramInput> for DeleteProgramUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<DeleteProgramInput>) -> Result<(), ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in DeleteProgramUseCase")
        })?;

        let existing = self
            .program_repository
            .get(&input.identifier, false)
            .instrument(info_span!("DeleteProgramUseCase::get"))
            .await?;

        self.program_repository
            .delete(existing.id, &input.actor)
            .instrument(info_span!("DeleteProgramUseCase::delete"))
            .await
    }
}
