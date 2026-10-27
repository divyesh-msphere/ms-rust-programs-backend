use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::rule_set::{RuleSetRepository, RuleVersion};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};


pub struct GetRuleVersionUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    rule_set_repository: Arc<dyn RuleSetRepository>,
}

impl GetRuleVersionUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, rule_set_repository: Arc<dyn RuleSetRepository>) -> Self {
        Self { program_repository, rule_set_repository }
    }
}

#[derive(Debug)]
pub struct GetRuleVersionInput {
    pub program: ResourceIdentifier,
    pub rule_set: ResourceIdentifier,
    pub version: i32,
}

#[async_trait::async_trait]
impl UseCase<RuleVersion, GetRuleVersionInput> for GetRuleVersionUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetRuleVersionInput>) -> Result<RuleVersion, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetRuleVersionUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetRuleVersionUseCase::get_program"))
            .await?;

        self.rule_set_repository
            .get_version(program.id, &input.rule_set, input.version)
            .instrument(info_span!("GetRuleVersionUseCase::get_version"))
            .await
    }
}
