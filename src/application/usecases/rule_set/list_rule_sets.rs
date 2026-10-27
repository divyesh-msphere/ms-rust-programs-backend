use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::rule_set::{RuleSet, RuleSetRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};


pub struct ListRuleSetsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    rule_set_repository: Arc<dyn RuleSetRepository>,
}

impl ListRuleSetsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, rule_set_repository: Arc<dyn RuleSetRepository>) -> Self {
        Self { program_repository, rule_set_repository }
    }
}

#[derive(Debug)]
pub struct ListRuleSetsInput {
    pub program: ResourceIdentifier,
}

#[async_trait::async_trait]
impl UseCase<Vec<RuleSet>, ListRuleSetsInput> for ListRuleSetsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListRuleSetsInput>) -> Result<Vec<RuleSet>, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListRuleSetsUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListRuleSetsUseCase::get_program"))
            .await?;

        self.rule_set_repository
            .list(program.id)
            .instrument(info_span!("ListRuleSetsUseCase::list"))
            .await
    }
}
