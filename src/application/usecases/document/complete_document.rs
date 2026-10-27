use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{Document, DocumentRepository, UPLOADING};
use crate::domain::program::ProgramRepository;
use crate::domain::object_storage::ObjectStorage;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct CompleteDocumentUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
    object_storage: Arc<dyn ObjectStorage>,
}

impl CompleteDocumentUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>, object_storage: Arc<dyn ObjectStorage>) -> Self {
        Self { program_repository, document_repository, object_storage }
    }
}

#[derive(Debug)]
pub struct CompleteDocumentInput {
    pub program: ResourceIdentifier,
    pub document: Uuid,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Document, CompleteDocumentInput> for CompleteDocumentUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<CompleteDocumentInput>) -> Result<Document, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in CompleteDocumentUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("CompleteDocumentUseCase::get_program"))
            .await?;

        let document = self
            .document_repository
            .get(program.id, input.document)
            .instrument(info_span!("CompleteDocumentUseCase::get"))
            .await?;
        // Completing twice is harmless: a file past `uploading` is returned as it is.
        if document.status != UPLOADING {
            return Ok(document);
        }

        // The storage is asked what really arrived, never the browser.
        let info = self
            .object_storage
            .head(&document.file_key)
            .instrument(info_span!("CompleteDocumentUseCase::head"))
            .await?
            .ok_or_else(|| ApplicationError::conflict("the file has not reached the storage yet: send it to the upload URL first"))?;
        if info.size_bytes != document.size_bytes {
            return Err(ApplicationError::unprocessable_entity(&format!(
                "the uploaded file is {} bytes but {} were declared",
                info.size_bytes, document.size_bytes
            )));
        }
        let essence = |t: &str| t.split(';').next().unwrap_or("").trim().to_lowercase();
        if let Some(stored) = &info.content_type
            && essence(stored) != essence(&document.content_type)
        {
            return Err(ApplicationError::unprocessable_entity(&format!(
                "the uploaded file is {stored} but {} was declared",
                document.content_type
            )));
        }
        if let (Some(declared), Some(stored)) = (&document.checksum, &info.checksum_sha256)
            && !declared.eq_ignore_ascii_case(stored)
        {
            return Err(ApplicationError::unprocessable_entity("the uploaded file does not match the declared SHA-256"));
        }

        self.document_repository
            .complete(program.id, input.document, &input.actor)
            .instrument(info_span!("CompleteDocumentUseCase::complete"))
            .await
    }
}
