use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::application::{Application, ApplicationRepository};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct SubmitApplicationUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    application_repository: Arc<dyn ApplicationRepository>,
}

impl SubmitApplicationUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, application_repository: Arc<dyn ApplicationRepository>) -> Self {
        Self { program_repository, application_repository }
    }
}

#[derive(Debug)]
pub struct SubmitApplicationInput {
    pub program: ResourceIdentifier,
    pub application: Uuid,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Application, SubmitApplicationInput> for SubmitApplicationUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<SubmitApplicationInput>) -> Result<Application, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in SubmitApplicationUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("SubmitApplicationUseCase::get_program"))
            .await?;

        self.application_repository
            .submit(program.id, input.application, &input.actor)
            .instrument(info_span!("SubmitApplicationUseCase::submit"))
            .await
    }
}
    