use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::approval_step::{
    ApprovalStep, ApprovalStepRepository, NewApprovalStep, validate_approver_role, validate_name, validate_position,
};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct CreateApprovalStepUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    approval_step_repository: Arc<dyn ApprovalStepRepository>,
}

impl CreateApprovalStepUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, approval_step_repository: Arc<dyn ApprovalStepRepository>) -> Self {
        Self { program_repository, approval_step_repository }
    }
}

#[derive(Debug)]
pub struct CreateApprovalStepInput {
    pub program: ResourceIdentifier,
    pub id: Option<Uuid>,
    pub name: String,
    pub approver_role: String,
    pub position: Option<i32>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<ApprovalStep, CreateApprovalStepInput> for CreateApprovalStepUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<CreateApprovalStepInput>) -> Result<ApprovalStep, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in CreateApprovalStepUseCase"))?;

        validate_name(&input.name)?;
        validate_approver_role(&input.approver_role)?;
        if let Some(position) = input.position {
            validate_position(position)?;
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("CreateApprovalStepUseCase::get_program"))
            .await?;

        self.approval_step_repository
            .create(NewApprovalStep {
                id: input.id,
                program_id: program.id,
                name: input.name,
                approver_role: input.approver_role,
                position: input.position,
                actor: input.actor,
            })
            .instrument(info_span!("CreateApprovalStepUseCase::create"))
            .await
    }
}
