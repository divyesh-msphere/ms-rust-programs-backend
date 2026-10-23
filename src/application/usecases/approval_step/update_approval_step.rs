use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::approval_step::{
    ApprovalStep, ApprovalStepRepository, UpdateApprovalStep, validate_approver_role, validate_name, validate_position,
};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct UpdateApprovalStepUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    approval_step_repository: Arc<dyn ApprovalStepRepository>,
}

impl UpdateApprovalStepUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, approval_step_repository: Arc<dyn ApprovalStepRepository>) -> Self {
        Self { program_repository, approval_step_repository }
    }
}

#[derive(Debug)]
pub struct UpdateApprovalStepInput {
    pub program: ResourceIdentifier,
    pub approval_step: Uuid,
    pub name: Option<String>,
    pub approver_role: Option<String>,
    pub position: Option<i32>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<ApprovalStep, UpdateApprovalStepInput> for UpdateApprovalStepUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<UpdateApprovalStepInput>) -> Result<ApprovalStep, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in UpdateApprovalStepUseCase"))?;

        if let Some(name) = &input.name {
            validate_name(name)?;
        }
        if let Some(role) = &input.approver_role {
            validate_approver_role(role)?;
        }
        if let Some(position) = input.position {
            validate_position(position)?;
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("UpdateApprovalStepUseCase::get_program"))
            .await?;

        self.approval_step_repository
            .update(
                program.id,
                input.approval_step,
                UpdateApprovalStep {
                    name: input.name,
                    approver_role: input.approver_role,
                    position: input.position,
                    actor: input.actor,
                },
            )
            .instrument(info_span!("UpdateApprovalStepUseCase::update"))
            .await
    }
}
