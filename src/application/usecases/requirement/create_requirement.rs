use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::requirement::{NewRequirement, Requirement, RequirementRepository, validate_config, validate_name, validate_position, validate_type};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use serde_json::Value;
use uuid::Uuid;

pub struct CreateRequirementUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    requirement_repository: Arc<dyn RequirementRepository>,
}

impl CreateRequirementUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, requirement_repository: Arc<dyn RequirementRepository>) -> Self {
        Self { program_repository, requirement_repository }
    }
}

#[derive(Debug)]
pub struct CreateRequirementInput {
    pub program: ResourceIdentifier,
    pub id: Option<Uuid>,
    pub requirement_type: String,
    pub name: String,
    pub config: Value,
    pub position: Option<i32>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Requirement, CreateRequirementInput> for CreateRequirementUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<CreateRequirementInput>) -> Result<Requirement, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in CreateRequirementUseCase"))?;

        validate_type(&input.requirement_type)?;
        validate_name(&input.name)?;
        validate_config(&input.config)?;
        if let Some(position) = input.position {
            validate_position(position)?;
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("CreateRequirementUseCase::get_program"))
            .await?;

        self.requirement_repository
            .create(NewRequirement {
                id: input.id,
                program_id: program.id,
                requirement_type: input.requirement_type,
                name: input.name,
                config: input.config,
                position: input.position,
                actor: input.actor,
            })
            .instrument(info_span!("CreateRequirementUseCase::create"))
            .await
    }
}
