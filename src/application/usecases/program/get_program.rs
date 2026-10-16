use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::{Program, ProgramRepository};

pub struct GetProgramUseCase {
    program_repository: Arc<dyn ProgramRepository>,
}

impl GetProgramUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>) -> Self {
        Self { program_repository }
    }
}

#[derive(Debug)]
pub struct GetProgramInput {
    pub identifier: ResourceIdentifier,
}

#[derive(Debug)]
pub struct GetProgramOutput {
    pub program: Program,
}

#[async_trait::async_trait]
impl UseCase<GetProgramOutput, GetProgramInput> for GetProgramUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetProgramInput>) -> Result<GetProgramOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in GetProgramUseCase")
        })?;

        let program = self
            .program_repository
            .get(&input.identifier, false)
            .instrument(info_span!("GetProgramUseCase::get"))
            .await?;

        Ok(GetProgramOutput { program })
    }
}
