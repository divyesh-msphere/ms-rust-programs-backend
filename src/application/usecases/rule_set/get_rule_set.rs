use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::rule_set::{RuleSet, RuleSetRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};


pub struct GetRuleSetUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    rule_set_repository: Arc<dyn RuleSetRepository>,
}

impl GetRuleSetUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, rule_set_repository: Arc<dyn RuleSetRepository>) -> Self {
        Self { program_repository, rule_set_repository }
    }
}

#[derive(Debug)]
pub struct GetRuleSetInput {
    pub program: ResourceIdentifier,
    pub rule_set: ResourceIdentifier,
}

#[async_trait::async_trait]
impl UseCase<RuleSet, GetRuleSetInput> for GetRuleSetUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetRuleSetInput>) -> Result<RuleSet, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetRuleSetUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetRuleSetUseCase::get_program"))
            .await?;

        self.rule_set_repository
            .get(program.id, &input.rule_set)
            .instrument(info_span!("GetRuleSetUseCase::get"))
            .await
    }
}
