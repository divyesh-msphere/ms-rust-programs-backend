use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::requirement::{RequirementRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct DeleteRequirementUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    requirement_repository: Arc<dyn RequirementRepository>,
}

impl DeleteRequirementUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, requirement_repository: Arc<dyn RequirementRepository>) -> Self {
        Self { program_repository, requirement_repository }
    }
}

#[derive(Debug)]
pub struct DeleteRequirementInput {
    pub program: ResourceIdentifier,
    pub requirement: Uuid,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<(), DeleteRequirementInput> for DeleteRequirementUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<DeleteRequirementInput>) -> Result<(), ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in DeleteRequirementUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("DeleteRequirementUseCase::get_program"))
            .await?;

        self.requirement_repository
            .delete(program.id, input.requirement, &input.actor)
            .instrument(info_span!("DeleteRequirementUseCase::delete"))
            .await
    }
}
