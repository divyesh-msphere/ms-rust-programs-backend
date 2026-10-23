use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::approval_step::{ApprovalStep, ApprovalStepRepository, validate_position};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct BatchUpdateApprovalStepsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    approval_step_repository: Arc<dyn ApprovalStepRepository>,
}

impl BatchUpdateApprovalStepsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, approval_step_repository: Arc<dyn ApprovalStepRepository>) -> Self {
        Self { program_repository, approval_step_repository }
    }
}

#[derive(Debug)]
pub struct BatchUpdateApprovalStepsInput {
    pub program: ResourceIdentifier,
    pub positions: Vec<(Uuid, i32)>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Vec<ApprovalStep>, BatchUpdateApprovalStepsInput> for BatchUpdateApprovalStepsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<BatchUpdateApprovalStepsInput>) -> Result<Vec<ApprovalStep>, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in BatchUpdateApprovalStepsUseCase"))?;

        if input.positions.is_empty() {
            return Err(ApplicationError::validation_error("approval_steps must list at least one approval step"));
        }
        let mut seen = std::collections::HashSet::new();
        for (id, position) in &input.positions {
            validate_position(*position)?;
            if !seen.insert(*id) {
                return Err(ApplicationError::validation_error(&format!("approval_step {id} is listed twice")));
            }
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("BatchUpdateApprovalStepsUseCase::get_program"))
            .await?;

        self.approval_step_repository
            .reorder(program.id, input.positions, &input.actor)
            .instrument(info_span!("BatchUpdateApprovalStepsUseCase::reorder"))
            .await
    }
}
