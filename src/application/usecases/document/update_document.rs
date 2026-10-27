use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{Document, DocumentRepository, UpdateDocument, clean_file_name};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct UpdateDocumentUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
}

impl UpdateDocumentUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>) -> Self {
        Self { program_repository, document_repository }
    }
}

#[derive(Debug)]
pub struct UpdateDocumentInput {
    pub program: ResourceIdentifier,
    pub document: Uuid,
    pub name: Option<String>,
    pub folder_id: Option<Uuid>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Document, UpdateDocumentInput> for UpdateDocumentUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<UpdateDocumentInput>) -> Result<Document, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in UpdateDocumentUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("UpdateDocumentUseCase::get_program"))
            .await?;

        let name = input.name.as_deref().map(clean_file_name).transpose()?;
        self.document_repository
            .update(program.id, input.document, UpdateDocument { name, folder_id: input.folder_id, actor: input.actor })
            .instrument(info_span!("UpdateDocumentUseCase::update"))
            .await
    }
}
