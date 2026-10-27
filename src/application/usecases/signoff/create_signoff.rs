use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::signoff::{NewSignoff, Signoff, SignoffRepository, validate_comment, validate_decision};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct CreateSignoffUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    signoff_repository: Arc<dyn SignoffRepository>,
}

impl CreateSignoffUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, signoff_repository: Arc<dyn SignoffRepository>) -> Self {
        Self { program_repository, signoff_repository }
    }
}

#[derive(Debug)]
pub struct CreateSignoffInput {
    pub program: ResourceIdentifier,
    pub application: Uuid,
    pub approval_step_id: Option<Uuid>,
    pub decision: String,
    pub comment: Option<String>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Signoff, CreateSignoffInput> for CreateSignoffUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<CreateSignoffInput>) -> Result<Signoff, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in CreateSignoffUseCase"))?;

        validate_decision(&input.decision)?;
        validate_comment(&input.comment)?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("CreateSignoffUseCase::get_program"))
            .await?;

        self.signoff_repository
            .create(NewSignoff {
                program_id: program.id,
                application_id: input.application,
                approval_step_id: input.approval_step_id,
                decision: input.decision,
                comment: input.comment,
                actor: input.actor,
            })
            .instrument(info_span!("CreateSignoffUseCase::create"))
            .await
    }
}
