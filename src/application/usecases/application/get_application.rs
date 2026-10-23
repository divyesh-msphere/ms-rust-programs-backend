use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::application::{Application, ApplicationRepository};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct GetApplicationUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    application_repository: Arc<dyn ApplicationRepository>,
}

impl GetApplicationUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, application_repository: Arc<dyn ApplicationRepository>) -> Self {
        Self { program_repository, application_repository }
    }
}

#[derive(Debug)]
pub struct GetApplicationInput {
    pub program: ResourceIdentifier,
    pub application: Uuid,
}

#[async_trait::async_trait]
impl UseCase<Application, GetApplicationInput> for GetApplicationUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetApplicationInput>) -> Result<Application, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetApplicationUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetApplicationUseCase::get_program"))
            .await?;

        self.application_repository
            .get(program.id, input.application)
            .instrument(info_span!("GetApplicationUseCase::get"))
            .await
    }
}
