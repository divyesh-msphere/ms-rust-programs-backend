use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::rule_set::{NewRuleVersion, RuleSetRepository, SaveOutcome, validate_new};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};
use serde_json::{Value, json};
use uuid::Uuid;


pub struct CreateRuleVersionUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    rule_set_repository: Arc<dyn RuleSetRepository>,
}

impl CreateRuleVersionUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, rule_set_repository: Arc<dyn RuleSetRepository>) -> Self {
        Self { program_repository, rule_set_repository }
    }
}

#[derive(Debug)]
pub struct CreateRuleVersionInput {
    pub program: ResourceIdentifier,
    pub rule_set: ResourceIdentifier,
    pub rule_type: Option<String>,
    pub definition: Value,
    pub compiled: Option<Value>,
    /// Defaults to "no validation errors were sent".
    pub valid: Option<bool>,
    pub validation_errors: Option<Value>,
    /// The input the rule reads (see `RuleVersion::input_schema`).
    pub input_schema: Option<Value>,
    pub actor: String,
}

#[async_trait::async_trait]
impl UseCase<SaveOutcome, CreateRuleVersionInput> for CreateRuleVersionUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<CreateRuleVersionInput>) -> Result<SaveOutcome, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in CreateRuleVersionUseCase"))?;

        let validation_errors = input.validation_errors.unwrap_or_else(|| json!([]));
        let valid = input.valid.unwrap_or_else(|| validation_errors.as_array().is_some_and(Vec::is_empty));
        let new_version = NewRuleVersion {
            program_id: Uuid::nil(),
            rule_set: input.rule_set,
            rule_type: input.rule_type.map(|t| t.trim().to_string()),
            definition: input.definition,
            compiled: input.compiled,
            valid,
            validation_errors,
            input_schema: input.input_schema,
            actor: input.actor,
        };
        validate_new(&new_version)?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("CreateRuleVersionUseCase::get_program"))
            .await?;

        self.rule_set_repository
            .save_version(NewRuleVersion { program_id: program.id, ..new_version })
            .instrument(info_span!("CreateRuleVersionUseCase::save_version"))
            .await
    }
}
