use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::publication::{Publication, PublicationRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};


pub struct CreatePublicationUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    publication_repository: Arc<dyn PublicationRepository>,
}

impl CreatePublicationUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, publication_repository: Arc<dyn PublicationRepository>) -> Self {
        Self { program_repository, publication_repository }
    }
}

#[derive(Debug)]
pub struct CreatePublicationInput {
    pub program: ResourceIdentifier,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Publication, CreatePublicationInput> for CreatePublicationUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<CreatePublicationInput>) -> Result<Publication, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in CreatePublicationUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("CreatePublicationUseCase::get_program"))
            .await?;

        self.publication_repository
            .publish(program.id, &input.actor)
            .instrument(info_span!("CreatePublicationUseCase::publish"))
            .await
    }
}
