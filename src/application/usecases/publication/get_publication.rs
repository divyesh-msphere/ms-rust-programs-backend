use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::publication::{Publication, PublicationRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct GetPublicationUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    publication_repository: Arc<dyn PublicationRepository>,
}

impl GetPublicationUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, publication_repository: Arc<dyn PublicationRepository>) -> Self {
        Self { program_repository, publication_repository }
    }
}

#[derive(Debug)]
pub struct GetPublicationInput {
    pub program: ResourceIdentifier,
    pub publication: Uuid,
}

#[async_trait::async_trait]
impl UseCase<Publication, GetPublicationInput> for GetPublicationUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetPublicationInput>) -> Result<Publication, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetPublicationUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetPublicationUseCase::get_program"))
            .await?;

        self.publication_repository
            .get(program.id, input.publication)
            .instrument(info_span!("GetPublicationUseCase::get"))
            .await
    }
}
