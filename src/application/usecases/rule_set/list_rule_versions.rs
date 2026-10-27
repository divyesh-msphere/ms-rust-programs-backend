use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::rule_set::{RuleSetRepository, RuleVersion};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;

pub struct ListRuleVersionsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    rule_set_repository: Arc<dyn RuleSetRepository>,
}

impl ListRuleVersionsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, rule_set_repository: Arc<dyn RuleSetRepository>) -> Self {
        Self { program_repository, rule_set_repository }
    }
}

#[derive(Debug)]
pub struct ListRuleVersionsInput {
    pub program: ResourceIdentifier,
    pub rule_set: ResourceIdentifier,
    pub page_size: Option<i64>,
    pub page_token: Option<Uuid>,
}

#[derive(Debug)]
pub struct ListRuleVersionsOutput {
    pub versions: Vec<RuleVersion>,
    pub next_page_token: Option<Uuid>,
}

#[async_trait::async_trait]
impl UseCase<ListRuleVersionsOutput, ListRuleVersionsInput> for ListRuleVersionsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ListRuleVersionsInput>) -> Result<ListRuleVersionsOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ListRuleVersionsUseCase"))?;

        let page_size = input.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page_size < 1 {
            return Err(ApplicationError::validation_error("max_page_size must be at least 1"));
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ListRuleVersionsUseCase::get_program"))
            .await?;

        let (versions, next_page_token) = self
            .rule_set_repository
            .list_versions(program.id, &input.rule_set, page_size.min(MAX_PAGE_SIZE), input.page_token)
            .instrument(info_span!("ListRuleVersionsUseCase::list_versions"))
            .await?;

        Ok(ListRuleVersionsOutput { versions, next_page_token })
    }
}
