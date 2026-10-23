use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::approval_step::{ApprovalStepRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct DeleteApprovalStepUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    approval_step_repository: Arc<dyn ApprovalStepRepository>,
}

impl DeleteApprovalStepUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, approval_step_repository: Arc<dyn ApprovalStepRepository>) -> Self {
        Self { program_repository, approval_step_repository }
    }
}

#[derive(Debug)]
pub struct DeleteApprovalStepInput {
    pub program: ResourceIdentifier,
    pub approval_step: Uuid,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<(), DeleteApprovalStepInput> for DeleteApprovalStepUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<DeleteApprovalStepInput>) -> Result<(), ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in DeleteApprovalStepUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("DeleteApprovalStepUseCase::get_program"))
            .await?;

        self.approval_step_repository
            .delete(program.id, input.approval_step, &input.actor)
            .instrument(info_span!("DeleteApprovalStepUseCase::delete"))
            .await
    }
}
