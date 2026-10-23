use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::requirement::{Requirement, RequirementRepository, UpdateRequirement, validate_config, validate_name, validate_position};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use serde_json::Value;
use uuid::Uuid;

pub struct UpdateRequirementUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    requirement_repository: Arc<dyn RequirementRepository>,
}

impl UpdateRequirementUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, requirement_repository: Arc<dyn RequirementRepository>) -> Self {
        Self { program_repository, requirement_repository }
    }
}

#[derive(Debug)]
pub struct UpdateRequirementInput {
    pub program: ResourceIdentifier,
    pub requirement: Uuid,
    pub name: Option<String>,
    pub config: Option<Value>,
    pub position: Option<i32>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<Requirement, UpdateRequirementInput> for UpdateRequirementUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<UpdateRequirementInput>) -> Result<Requirement, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in UpdateRequirementUseCase"))?;

        if let Some(name) = &input.name {
            validate_name(name)?;
        }
        if let Some(config) = &input.config {
            validate_config(config)?;
        }
        if let Some(position) = input.position {
            validate_position(position)?;
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("UpdateRequirementUseCase::get_program"))
            .await?;

        self.requirement_repository
            .update(
                program.id,
                input.requirement,
                UpdateRequirement { name: input.name, config: input.config, position: input.position, actor: input.actor },
            )
            .instrument(info_span!("UpdateRequirementUseCase::update"))
            .await
    }
}
