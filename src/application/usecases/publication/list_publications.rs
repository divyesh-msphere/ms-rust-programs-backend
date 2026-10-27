use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::publication::{Publication, PublicationRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;

pub struct ListPublicationsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    publication_repository: Arc<dyn PublicationRepository>,
}

impl ListPublicationsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, publication_repository: Arc<dyn PublicationRepository>) -> Self {
        Self { program_repository, publication_repository }
    }
}

#[derive(Debug)]
pub struct ListPublicationsInput {
    pub program: ResourceIdentifier,
    pub page_size: Option<i64>,
    pub page_token: Option<Uuid>,
}

#[derive(Debug)]
pub struct ListPublicationsOutput {
    pub publications: Vec<Publication>,
    pub next_page_token: Option<Uuid>,
}

#[async_trait::async_trait]
impl UseCase<ListPublicationsOutput, ListPublicationsInput> for ListPublicationsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListPublicationsInput>) -> Result<ListPublicationsOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListPublicationsUseCase"))?;

        let page_size = input.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page_size < 1 {
            return Err(ApplicationError::validation_error("max_page_size must be at least 1"));
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListPublicationsUseCase::get_program"))
            .await?;

        let (publications, next_page_token) = self
            .publication_repository
            .list(program.id, page_size.min(MAX_PAGE_SIZE), input.page_token)
            .instrument(info_span!("ListPublicationsUseCase::list"))
            .await?;

        Ok(ListPublicationsOutput { publications, next_page_token })
    }
}
