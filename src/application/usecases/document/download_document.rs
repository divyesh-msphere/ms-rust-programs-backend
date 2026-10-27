use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{Document, DocumentRepository, can_download};
use crate::domain::program::ProgramRepository;
use crate::domain::object_storage::ObjectStorage;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use crate::domain::object_storage::PresignedRequest;
use uuid::Uuid;


pub struct DownloadDocumentUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
    object_storage: Arc<dyn ObjectStorage>,
}

impl DownloadDocumentUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>, object_storage: Arc<dyn ObjectStorage>) -> Self {
        Self { program_repository, document_repository, object_storage }
    }
}

#[derive(Debug)]
pub struct DownloadDocumentInput {
    pub program: ResourceIdentifier,
    pub document: Uuid,
}

#[derive(Debug)]
pub struct DownloadDocumentOutput {
    pub document: Document,
    /// A short-lived link: the browser downloads from the storage, not from this service.
    pub request: PresignedRequest,
}

#[async_trait::async_trait]
impl UseCase<DownloadDocumentOutput, DownloadDocumentInput> for DownloadDocumentUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<DownloadDocumentInput>) -> Result<DownloadDocumentOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in DownloadDocumentUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("DownloadDocumentUseCase::get_program"))
            .await?;

        let document = self
            .document_repository
            .get(program.id, input.document)
            .instrument(info_span!("DownloadDocumentUseCase::get"))
            .await?;
        can_download(&document.status)?;
        let request = self.object_storage.presign_download(&document.file_key, &document.name, &document.content_type)?;
        Ok(DownloadDocumentOutput { document, request })
    }
}
