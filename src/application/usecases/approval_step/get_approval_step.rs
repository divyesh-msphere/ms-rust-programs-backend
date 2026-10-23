use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::approval_step::{ApprovalStep, ApprovalStepRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct GetApprovalStepUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    approval_step_repository: Arc<dyn ApprovalStepRepository>,
}

impl GetApprovalStepUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, approval_step_repository: Arc<dyn ApprovalStepRepository>) -> Self {
        Self { program_repository, approval_step_repository }
    }
}

#[derive(Debug)]
pub struct GetApprovalStepInput {
    pub program: ResourceIdentifier,
    pub approval_step: Uuid,
}

#[async_trait::async_trait]
impl UseCase<ApprovalStep, GetApprovalStepInput> for GetApprovalStepUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetApprovalStepInput>) -> Result<ApprovalStep, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetApprovalStepUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetApprovalStepUseCase::get_program"))
            .await?;

        self.approval_step_repository
            .get(program.id, input.approval_step)
            .instrument(info_span!("GetApprovalStepUseCase::get"))
            .await
    }
}
