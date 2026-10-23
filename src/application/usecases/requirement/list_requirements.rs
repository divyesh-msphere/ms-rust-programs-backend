use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::requirement::{Requirement, RequirementRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

pub struct ListRequirementsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    requirement_repository: Arc<dyn RequirementRepository>,
}

impl ListRequirementsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, requirement_repository: Arc<dyn RequirementRepository>) -> Self {
        Self { program_repository, requirement_repository }
    }
}

#[derive(Debug)]
pub struct ListRequirementsInput {
    pub program: ResourceIdentifier,
}

#[async_trait::async_trait]
impl UseCase<Vec<Requirement>, ListRequirementsInput> for ListRequirementsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListRequirementsInput>) -> Result<Vec<Requirement>, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListRequirementsUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListRequirementsUseCase::get_program"))
            .await?;

        self.requirement_repository
            .list(program.id)
            .instrument(info_span!("ListRequirementsUseCase::list"))
            .await
    }
}
