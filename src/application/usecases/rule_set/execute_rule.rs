use crate::application::errors::application_error::{ApplicationError, ApplicationErrorKind};
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::ProgramRepository;
use crate::domain::rule_engine::{RuleEngine, RuleRun, RuleRunError};
use crate::domain::rule_set::{RuleSetRepository, RuleVersion, check_input, parse_input_schema};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Instant;
use tracing::{Instrument, info_span, instrument};
use uuid::Uuid;

/// Which version of the rule to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionSelector {
    /// What is live (the default).
    Published,
    /// The newest save, for a "Test rule" button.
    Draft,
    Number(i32),
}

impl VersionSelector {
    /// `published`, `draft` or a version number ("3"). Empty means published.
    pub fn parse(raw: Option<&str>) -> Result<Self, ApplicationError> {
        match raw.map(str::trim).filter(|r| !r.is_empty()) {
            None | Some("published") => Ok(Self::Published),
            Some("draft") => Ok(Self::Draft),
            Some(number) => number.parse::<i32>().ok().filter(|n| *n > 0).map(Self::Number).ok_or_else(|| {
                ApplicationError::validation_error("version must be \"published\", \"draft\" or a version number (1, 2, 3, ...)")
            }),
        }
    }
}

pub struct ExecuteRuleUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    rule_set_repository: Arc<dyn RuleSetRepository>,
    rule_engine: Arc<dyn RuleEngine>,
}

impl ExecuteRuleUseCase {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        rule_set_repository: Arc<dyn RuleSetRepository>,
        rule_engine: Arc<dyn RuleEngine>,
    ) -> Self {
        Self { program_repository, rule_set_repository, rule_engine }
    }
}

#[derive(Debug)]
pub struct ExecuteRuleInput {
    pub program: ResourceIdentifier,
    /// The rule leaf: its module key (or the id of its rule set).
    pub rule_set: ResourceIdentifier,
    /// The rule family the caller expects (`pre_enrollment`, ...), checked against the module.
    pub rule_type: Option<String>,
    pub version: VersionSelector,
    pub input: Value,
    pub trace: bool,
}

#[derive(Debug)]
pub struct ExecuteRuleOutput {
    pub program_id: Uuid,
    pub module_key: String,
    pub rule_type: Option<String>,
    pub version: i32,
    pub status: String,
    pub run: RuleRun,
    pub duration_ms: f64,
}

fn coded(kind: ApplicationErrorKind, code: &str, msg: &str, details: Value) -> ApplicationError {
    ApplicationError::coded(kind, code, msg, details)
}

/// Checks the input against what the version says it reads, if it says.
fn check_against_schema(version: &RuleVersion, input: &Value) -> Result<(), ApplicationError> {
    let Some(schema) = &version.input_schema else { return Ok(()) };
    let fields = parse_input_schema(schema)?;
    check_input(&fields, input).map_err(|mismatch| {
        coded(
            ApplicationErrorKind::UnprocessableEntity,
            "INPUT_MISMATCH",
            &mismatch.message(),
            json!({
                "required_fields": mismatch.required_fields,
                "optional_fields": mismatch.optional_fields,
                "missing": mismatch.missing,
                "wrong_type": mismatch.wrong_type.iter()
                    .map(|(field, expected, got)| json!({ "field": field, "expected": expected, "got": got }))
                    .collect::<Vec<_>>(),
            }),
        )
    })
}

#[async_trait::async_trait]
impl UseCase<ExecuteRuleOutput, ExecuteRuleInput> for ExecuteRuleUseCase {
    #[instrument(skip(self, input), fields(rule_set = ?input.as_ref().map(|i| &i.rule_set)))]
    async fn execute(&self, input: Option<ExecuteRuleInput>) -> Result<ExecuteRuleOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ExecuteRuleUseCase"))?;
        if !input.input.is_object() {
            return Err(coded(
                ApplicationErrorKind::InvalidArgument,
                "INVALID_BODY",
                "Validation error: input must be a JSON object",
                json!({}),
            ));
        }

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ExecuteRuleUseCase::get_program"))
            .await?;

        // 404 when the module is not on the program; 400 FAILED_PRECONDITION when it is not a rule module.
        let rule_set = self
            .rule_set_repository
            .get(program.id, &input.rule_set)
            .instrument(info_span!("ExecuteRuleUseCase::get_rule_set"))
            .await?;

        if let Some(expected) = &input.rule_type
            && rule_set.rule_type.as_deref().map(str::to_lowercase) != Some(expected.trim().to_lowercase())
        {
            return Err(coded(
                ApplicationErrorKind::UnprocessableEntity,
                "TYPE_MISMATCH",
                &format!(
                    "{} is a {:?} rule, not {:?}",
                    rule_set.module_key,
                    rule_set.rule_type.clone().unwrap_or_default(),
                    expected
                ),
                json!({ "expected_type": rule_set.rule_type }),
            ));
        }

        let number = match &input.version {
            VersionSelector::Published => rule_set.published_version.ok_or_else(|| {
                coded(
                    ApplicationErrorKind::NotFound,
                    "NOT_FOUND",
                    &format!("{} has no published version yet; publish the program, or run the draft", rule_set.module_key),
                    json!({}),
                )
            })?,
            VersionSelector::Draft => rule_set.draft_version.ok_or_else(|| {
                coded(
                    ApplicationErrorKind::NotFound,
                    "NOT_FOUND",
                    &format!("{} has no saved rule yet", rule_set.module_key),
                    json!({}),
                )
            })?,
            VersionSelector::Number(n) => *n,
        };
        let version = self
            .rule_set_repository
            .get_version(program.id, &input.rule_set, number)
            .instrument(info_span!("ExecuteRuleUseCase::get_version"))
            .await?;

        if !version.valid {
            return Err(coded(
                ApplicationErrorKind::UnprocessableEntity,
                "RULE_INVALID",
                &format!("version {} of {} is incomplete and cannot run", version.version, rule_set.module_key),
                json!({ "validation_errors": version.validation_errors }),
            ));
        }
        let Some(graph) = &version.compiled else {
            return Err(coded(
                ApplicationErrorKind::UnprocessableEntity,
                "RULE_INVALID",
                &format!("version {} of {} has no compiled decision graph to run", version.version, rule_set.module_key),
                json!({ "validation_errors": [] }),
            ));
        };

        check_against_schema(&version, &input.input)?;

        let started = Instant::now();
        let run = self
            .rule_engine
            .run(graph, input.input, input.trace)
            .instrument(info_span!("ExecuteRuleUseCase::run"))
            .await
            .map_err(|error| match error {
                RuleRunError::Timeout => coded(
                    ApplicationErrorKind::DeadlineExceeded,
                    "RULE_TIMEOUT",
                    "the rule took too long to run and was stopped",
                    json!({}),
                ),
                RuleRunError::BadGraph(message) => coded(
                    ApplicationErrorKind::UnprocessableEntity,
                    "RULE_INVALID",
                    &message,
                    json!({ "validation_errors": [] }),
                ),
                RuleRunError::Failed { node, message } => coded(
                    ApplicationErrorKind::UnprocessableEntity,
                    "RULE_EXECUTION_FAILED",
                    &match &node {
                        Some(node) => format!("the rule failed at node {node}: {message}"),
                        None => format!("the rule failed: {message}"),
                    },
                    json!({ "node_id": node, "message": message }),
                ),
            })?;
        let duration_ms = (started.elapsed().as_secs_f64() * 1000.0 * 1000.0).round() / 1000.0;

        Ok(ExecuteRuleOutput {
            program_id: program.id,
            module_key: rule_set.module_key,
            rule_type: rule_set.rule_type,
            version: version.version,
            status: version.status,
            run,
            duration_ms,
        })
    }
}
