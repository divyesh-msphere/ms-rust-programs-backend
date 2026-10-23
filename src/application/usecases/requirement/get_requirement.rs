use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::requirement::{Requirement, RequirementRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct GetRequirementUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    requirement_repository: Arc<dyn RequirementRepository>,
}

impl GetRequirementUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, requirement_repository: Arc<dyn RequirementRepository>) -> Self {
        Self { program_repository, requirement_repository }
    }
}

#[derive(Debug)]
pub struct GetRequirementInput {
    pub program: ResourceIdentifier,
    pub requirement: Uuid,
}

#[async_trait::async_trait]
impl UseCase<Requirement, GetRequirementInput> for GetRequirementUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetRequirementInput>) -> Result<Requirement, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetRequirementUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetRequirementUseCase::get_program"))
            .await?;

        self.requirement_repository
            .get(program.id, input.requirement)
            .instrument(info_span!("GetRequirementUseCase::get"))
            .await
    }
}
