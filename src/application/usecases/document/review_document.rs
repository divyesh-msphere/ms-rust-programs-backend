use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{Document, DocumentRepository, ReviewDecision};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;


pub struct ReviewDocumentUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    document_repository: Arc<dyn DocumentRepository>,
}

impl ReviewDocumentUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, document_repository: Arc<dyn DocumentRepository>) -> Self {
        Self { program_repository, document_repository }
    }
}

#[derive(Debug)]
pub struct ReviewDocumentInput {
    pub program: ResourceIdentifier,
    pub document: Uuid,
    pub decision: String,
    pub note: Option<String>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Document, ReviewDocumentInput> for ReviewDocumentUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ReviewDocumentInput>) -> Result<Document, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ReviewDocumentUseCase"))?;

        let decision = ReviewDecision::parse(&input.decision)?;
        if input.note.as_ref().is_some_and(|n| n.chars().count() > 1000) {
            return Err(ApplicationError::unprocessable_entity("note must be at most 1000 characters"));
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ReviewDocumentUseCase::get_program"))
            .await?;

        self.document_repository
            .review(program.id, input.document, decision, input.note, &input.actor)
            .instrument(info_span!("ReviewDocumentUseCase::review"))
            .await
    }
}
