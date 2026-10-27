use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{DocumentRepository, Folder, NewFolder, clean_folder_name};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct CreateDocumentFolderUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
}

impl CreateDocumentFolderUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>) -> Self {
        Self { program_repository, document_repository }
    }
}

#[derive(Debug)]
pub struct CreateDocumentFolderInput {
    pub program: ResourceIdentifier,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Folder, CreateDocumentFolderInput> for CreateDocumentFolderUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<CreateDocumentFolderInput>) -> Result<Folder, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in CreateDocumentFolderUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("CreateDocumentFolderUseCase::get_program"))
            .await?;

        let name = clean_folder_name(&input.name)?;
        self.document_repository
            .create_folder(NewFolder { program_id: program.id, parent_id: input.parent_id, name, actor: input.actor })
            .instrument(info_span!("CreateDocumentFolderUseCase::create_folder"))
            .await
    }
}
