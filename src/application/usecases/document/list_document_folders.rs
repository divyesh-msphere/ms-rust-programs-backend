use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{DocumentRepository, Folder};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};


pub struct ListDocumentFoldersUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
}

impl ListDocumentFoldersUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>) -> Self {
        Self { program_repository, document_repository }
    }
}

#[derive(Debug)]
pub struct ListDocumentFoldersInput {
    pub program: ResourceIdentifier,
}

#[async_trait::async_trait]
impl UseCase<Vec<Folder>, ListDocumentFoldersInput> for ListDocumentFoldersUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListDocumentFoldersInput>) -> Result<Vec<Folder>, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListDocumentFoldersUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListDocumentFoldersUseCase::get_program"))
            .await?;

        self.document_repository
            .list_folders(program.id)
            .instrument(info_span!("ListDocumentFoldersUseCase::list_folders"))
            .await
    }
}
