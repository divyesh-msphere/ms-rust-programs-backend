use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::approval_step::{ApprovalStep, ApprovalStepRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

pub struct ListApprovalStepsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    approval_step_repository: Arc<dyn ApprovalStepRepository>,
}

impl ListApprovalStepsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, approval_step_repository: Arc<dyn ApprovalStepRepository>) -> Self {
        Self { program_repository, approval_step_repository }
    }
}

#[derive(Debug)]
pub struct ListApprovalStepsInput {
    pub program: ResourceIdentifier,
}

#[async_trait::async_trait]
impl UseCase<Vec<ApprovalStep>, ListApprovalStepsInput> for ListApprovalStepsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListApprovalStepsInput>) -> Result<Vec<ApprovalStep>, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListApprovalStepsUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListApprovalStepsUseCase::get_program"))
            .await?;

        self.approval_step_repository
            .list(program.id)
            .instrument(info_span!("ListApprovalStepsUseCase::list"))
            .await
    }
}
