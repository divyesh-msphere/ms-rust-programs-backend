use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::document::{Document, DocumentRepository, ScanResult};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

/// The scanner's callback. It names the document, not the program.
pub struct ScanDocumentUseCase {
    document_repository: Arc<dyn DocumentRepository>,
}

impl ScanDocumentUseCase {
    pub fn new(document_repository: Arc<dyn DocumentRepository>) -> Self {
        Self { document_repository }
    }
}

#[derive(Debug)]
pub struct ScanDocumentInput {
    pub document: Uuid,
    /// `clean`, `suspicious` or `infected`.
    pub result: String,
    pub scanner: String,
}

#[async_trait::async_trait]
impl UseCase<Document, ScanDocumentInput> for ScanDocumentUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ScanDocumentInput>) -> Result<Document, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ScanDocumentUseCase"))?;
        let result = ScanResult::parse(&input.result)?;
        let scanner = input.scanner.trim();
        let scanner = if scanner.is_empty() { "scanner" } else { scanner };
        self.document_repository
            .apply_scan(input.document, result, scanner)
            .instrument(info_span!("ScanDocumentUseCase::apply_scan"))
            .await
    }
}
