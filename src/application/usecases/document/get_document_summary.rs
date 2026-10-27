use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{DocumentRepository, DocumentSummary};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};


pub struct GetDocumentSummaryUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
}

impl GetDocumentSummaryUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>) -> Self {
        Self { program_repository, document_repository }
    }
}

#[derive(Debug)]
pub struct GetDocumentSummaryInput {
    pub program: ResourceIdentifier,
}

#[async_trait::async_trait]
impl UseCase<DocumentSummary, GetDocumentSummaryInput> for GetDocumentSummaryUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetDocumentSummaryInput>) -> Result<DocumentSummary, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetDocumentSummaryUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetDocumentSummaryUseCase::get_program"))
            .await?;

        self.document_repository
            .summary(program.id)
            .instrument(info_span!("GetDocumentSummaryUseCase::summary"))
            .await
    }
}
