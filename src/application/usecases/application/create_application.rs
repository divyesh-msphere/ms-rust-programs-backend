use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::application::{Application, ApplicationRepository, NewApplication, validate_new};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use serde_json::Value;
use uuid::Uuid;


pub struct CreateApplicationUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    application_repository: Arc<dyn ApplicationRepository>,
}

impl CreateApplicationUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, application_repository: Arc<dyn ApplicationRepository>) -> Self {
        Self { program_repository, application_repository }
    }
}

#[derive(Debug)]
pub struct CreateApplicationInput {
    pub program: ResourceIdentifier,
    pub id: Option<Uuid>,
    pub store_id: String,
    pub store_name: Option<String>,
    pub website_id: String,
    pub website_name: Option<String>,
    pub answers: Value,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Application, CreateApplicationInput> for CreateApplicationUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<CreateApplicationInput>) -> Result<Application, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in CreateApplicationUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("CreateApplicationUseCase::get_program"))
            .await?;

        let new_application = NewApplication {
            id: input.id,
            program_id: program.id,
            store_id: input.store_id,
            store_name: input.store_name,
            website_id: input.website_id,
            website_name: input.website_name,
            answers: input.answers,
            actor: input.actor,
        };
        validate_new(&new_application)?;

        self.application_repository
            .create(new_application)
            .instrument(info_span!("CreateApplicationUseCase::create"))
            .await
    }
}
