use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{DocumentRepository};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct DeleteDocumentUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
}

impl DeleteDocumentUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>) -> Self {
        Self { program_repository, document_repository }
    }
}

#[derive(Debug)]
pub struct DeleteDocumentInput {
    pub program: ResourceIdentifier,
    pub document: Uuid,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<(), DeleteDocumentInput> for DeleteDocumentUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<DeleteDocumentInput>) -> Result<(), ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in DeleteDocumentUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("DeleteDocumentUseCase::get_program"))
            .await?;

        self.document_repository
            .delete(program.id, input.document, &input.actor)
            .instrument(info_span!("DeleteDocumentUseCase::delete"))
            .await
    }
}
