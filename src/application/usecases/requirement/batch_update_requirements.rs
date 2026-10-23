use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::requirement::{Requirement, RequirementRepository, validate_position};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

pub struct BatchUpdateRequirementsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    requirement_repository: Arc<dyn RequirementRepository>,
}

impl BatchUpdateRequirementsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, requirement_repository: Arc<dyn RequirementRepository>) -> Self {
        Self { program_repository, requirement_repository }
    }
}

#[derive(Debug)]
pub struct BatchUpdateRequirementsInput {
    pub program: ResourceIdentifier,
    pub positions: Vec<(Uuid, i32)>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Vec<Requirement>, BatchUpdateRequirementsInput> for BatchUpdateRequirementsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<BatchUpdateRequirementsInput>) -> Result<Vec<Requirement>, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in BatchUpdateRequirementsUseCase"))?;

        if input.positions.is_empty() {
            return Err(ApplicationError::validation_error("requirements must list at least one requirement"));
        }
        let mut seen = std::collections::HashSet::new();
        for (id, position) in &input.positions {
            validate_position(*position)?;
            if !seen.insert(*id) {
                return Err(ApplicationError::validation_error(&format!("requirement {id} is listed twice")));
            }
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("BatchUpdateRequirementsUseCase::get_program"))
            .await?;

        self.requirement_repository
            .reorder(program.id, input.positions, &input.actor)
            .instrument(info_span!("BatchUpdateRequirementsUseCase::reorder"))
            .await
    }
}
