use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::signoff::{Signoff, SignoffRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct GetSignoffUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    signoff_repository: Arc<dyn SignoffRepository>,
}

impl GetSignoffUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, signoff_repository: Arc<dyn SignoffRepository>) -> Self {
        Self { program_repository, signoff_repository }
    }
}

#[derive(Debug)]
pub struct GetSignoffInput {
    pub program: ResourceIdentifier,
    pub application: Uuid,
    pub signoff: Uuid,
}

#[async_trait::async_trait]
impl UseCase<Signoff, GetSignoffInput> for GetSignoffUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetSignoffInput>) -> Result<Signoff, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetSignoffUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetSignoffUseCase::get_program"))
            .await?;

        self.signoff_repository
            .get(program.id, input.application, input.signoff)
            .instrument(info_span!("GetSignoffUseCase::get"))
            .await
    }
}
