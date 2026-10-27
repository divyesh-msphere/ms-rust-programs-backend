use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{DocumentRepository, Folder, UpdateFolder, clean_folder_name};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct UpdateDocumentFolderUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
}

impl UpdateDocumentFolderUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>) -> Self {
        Self { program_repository, document_repository }
    }
}

#[derive(Debug)]
pub struct UpdateDocumentFolderInput {
    pub program: ResourceIdentifier,
    pub folder: Uuid,
    pub name: Option<String>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Folder, UpdateDocumentFolderInput> for UpdateDocumentFolderUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<UpdateDocumentFolderInput>) -> Result<Folder, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in UpdateDocumentFolderUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("UpdateDocumentFolderUseCase::get_program"))
            .await?;

        let name = input.name.as_deref().map(clean_folder_name).transpose()?;
        self.document_repository
            .update_folder(program.id, input.folder, UpdateFolder { name, actor: input.actor })
            .instrument(info_span!("UpdateDocumentFolderUseCase::update_folder"))
            .await
    }
}
