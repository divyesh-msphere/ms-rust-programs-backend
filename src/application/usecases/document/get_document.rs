use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{Document, DocumentRepository};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct GetDocumentUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
}

impl GetDocumentUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>) -> Self {
        Self { program_repository, document_repository }
    }
}

#[derive(Debug)]
pub struct GetDocumentInput {
    pub program: ResourceIdentifier,
    pub document: Uuid,
}

#[async_trait::async_trait]
impl UseCase<Document, GetDocumentInput> for GetDocumentUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetDocumentInput>) -> Result<Document, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetDocumentUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetDocumentUseCase::get_program"))
            .await?;

        self.document_repository
            .get(program.id, input.document)
            .instrument(info_span!("GetDocumentUseCase::get"))
            .await
    }
}
