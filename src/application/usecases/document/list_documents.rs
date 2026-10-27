use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{Document, DocumentFilter, DocumentRepository, Location};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;

pub struct ListDocumentsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
}

impl ListDocumentsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>) -> Self {
        Self { program_repository, document_repository }
    }
}

#[derive(Debug)]
pub struct ListDocumentsInput {
    pub program: ResourceIdentifier,
    pub folder_id: Option<Uuid>,
    pub location: Option<String>,
    pub q: Option<String>,
    pub page_size: Option<i64>,
    pub page_token: Option<Uuid>,
}

#[derive(Debug)]
pub struct ListDocumentsOutput {
    pub documents: Vec<Document>,
    pub next_page_token: Option<Uuid>,
}

#[async_trait::async_trait]
impl UseCase<ListDocumentsOutput, ListDocumentsInput> for ListDocumentsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListDocumentsInput>) -> Result<ListDocumentsOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListDocumentsUseCase"))?;

        let filter = DocumentFilter {
            folder_id: input.folder_id,
            location: input.location.as_deref().map(str::trim).filter(|l| !l.is_empty()).map(Location::parse).transpose()?,
            name_prefix: input.q.map(|q| q.trim().to_string()).filter(|q| !q.is_empty()),
        };
        let page_size = input.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page_size < 1 {
            return Err(ApplicationError::validation_error("max_page_size must be at least 1"));
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListDocumentsUseCase::get_program"))
            .await?;

        let (documents, next_page_token) = self
            .document_repository
            .list(program.id, &filter, page_size.min(MAX_PAGE_SIZE), input.page_token)
            .instrument(info_span!("ListDocumentsUseCase::list"))
            .await?;

        Ok(ListDocumentsOutput { documents, next_page_token })
    }
}
