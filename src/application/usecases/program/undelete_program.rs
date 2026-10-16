use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::{Program, ProgramRepository};

pub struct UndeleteProgramUseCase {
    program_repository: Arc<dyn ProgramRepository>,
}

impl UndeleteProgramUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>) -> Self {
        Self { program_repository }
    }
}

#[derive(Debug)]
pub struct UndeleteProgramInput {
    pub identifier: ResourceIdentifier,
    pub actor: String,
}

#[derive(Debug)]
pub struct UndeleteProgramOutput {
    pub program: Program,
}

#[async_trait::async_trait]
impl UseCase<UndeleteProgramOutput, UndeleteProgramInput> for UndeleteProgramUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<UndeleteProgramInput>) -> Result<UndeleteProgramOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in UndeleteProgramUseCase")
        })?;

        // A deleted program is only visible to this lookup.
        let existing = self
            .program_repository
            .get(&input.identifier, true)
            .instrument(info_span!("UndeleteProgramUseCase::get"))
            .await?;

        let program = self
            .program_repository
            .undelete(existing.id, &input.actor)
            .instrument(info_span!("UndeleteProgramUseCase::undelete"))
            .await?;

        Ok(UndeleteProgramOutput { program })
    }
}
