use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{Document, DocumentPolicy, DocumentRepository, NewDocument, clean_file_name};
use crate::domain::program::ProgramRepository;
use crate::domain::object_storage::ObjectStorage;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use crate::domain::object_storage::PresignedRequest;
use uuid::Uuid;


pub struct CreateDocumentUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
    object_storage: Arc<dyn ObjectStorage>,
    policy: DocumentPolicy,
}

impl CreateDocumentUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>, object_storage: Arc<dyn ObjectStorage>, policy: DocumentPolicy) -> Self {
        Self { program_repository, document_repository, object_storage, policy }
    }
}

#[derive(Debug)]
pub struct CreateDocumentInput {
    pub program: ResourceIdentifier,
    pub folder_id: Option<Uuid>,
    pub name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub checksum: Option<String>,
    pub actor: String,
}

#[derive(Debug)]
pub struct CreateDocumentOutput {
    /// The new file, still `uploading` (hidden from every list until it is completed).
    pub document: Document,
    /// Where and how to send the bytes.
    pub upload: PresignedRequest,
}

#[async_trait::async_trait]
impl UseCase<CreateDocumentOutput, CreateDocumentInput> for CreateDocumentUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<CreateDocumentInput>) -> Result<CreateDocumentOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in CreateDocumentUseCase"))?;

        // Refused before any row exists: a file that is too big or of a type that is not allowed never starts.
        let name = clean_file_name(&input.name)?;
        self.policy.validate_upload(&input.content_type, input.size_bytes, &input.checksum)?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("CreateDocumentUseCase::get_program"))
            .await?;

        let document = self
            .document_repository
            .create(
                NewDocument {
                    program_id: program.id,
                    folder_id: input.folder_id,
                    name,
                    content_type: input.content_type,
                    size_bytes: input.size_bytes,
                    checksum: input.checksum,
                    uploaded_by: input.actor,
                },
                self.policy.max_files_per_program,
            )
            .instrument(info_span!("CreateDocumentUseCase::create"))
            .await?;

        // The browser uploads straight to the storage with this: type, size and checksum are signed in.
        let upload = self.object_storage.presign_upload(&document.file_key, &document.content_type, document.size_bytes, document.checksum.as_deref())?;
        Ok(CreateDocumentOutput { document, upload })
    }
}
