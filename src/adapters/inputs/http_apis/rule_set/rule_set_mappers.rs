use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::rule_set::create_rule_version::CreateRuleVersionInput;
use crate::application::usecases::rule_set::execute_rule::{ExecuteRuleInput, ExecuteRuleOutput, VersionSelector};
use crate::application::usecases::rule_set::get_rule_set::GetRuleSetInput;
use crate::application::usecases::rule_set::get_rule_version::GetRuleVersionInput;
use crate::application::usecases::rule_set::list_rule_sets::ListRuleSetsInput;
use crate::application::usecases::rule_set::list_rule_versions::{ListRuleVersionsInput, ListRuleVersionsOutput};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::rule_set::{RuleSet, RuleVersion, SaveOutcome};
use crate::dto::payloads::json::rule_set::{
    CreateRuleVersionBodyPayload, CreateRuleVersionPathPayload, ExecuteRuleBodyPayload, ExecuteRulePathPayload, GetRuleSetPayload, GetRuleVersionPayload,
    ListRuleSetsPayload, ListRuleVersionsPathPayload, ListRuleVersionsQueryPayload,
};
use crate::dto::presenters::json::rule_set::{
    ExecuteRulePresenter, RuleSetPresenter, RuleSetsPresenter, RuleVersionPresenter, RuleVersionsPresenter, SaveRuleVersionPresenter,
};

pub fn version_presenter(version: RuleVersion) -> RuleVersionPresenter {
    RuleVersionPresenter {
        path: format!("programs/{}/rule-sets/{}/versions/{}", version.program_id, version.module_key, version.version),
        id: version.id.to_string(),
        version: version.version,
        status: version.status,
        definition: version.definition,
        compiled: version.compiled,
        valid: version.valid,
        validation_errors: version.validation_errors,
        input_schema: version.input_schema,
        content_hash: version.content_hash,
        size_bytes: version.size_bytes,
        created_by: version.created_by,
        create_time: format_datetime(version.create_time),
    }
}

pub fn rule_set_presenter(rule_set: RuleSet) -> RuleSetPresenter {
    let state = rule_set.state();
    RuleSetPresenter {
        path: format!("programs/{}/rule-sets/{}", rule_set.program_id, rule_set.module_key),
        id: rule_set.rule_set_id.map(|id| id.to_string()),
        module_key: rule_set.module_key,
        module_name: rule_set.module_name,
        rule_type: rule_set.rule_type,
        draft_version: rule_set.draft_version,
        published_version: rule_set.published_version,
        state,
        draft: rule_set.draft.map(version_presenter),
        update_time: rule_set.update_time.map(format_datetime),
    }
}

pub struct ListRuleSetsMapper;

impl ListRuleSetsMapper {
    pub fn input(path: ListRuleSetsPayload) -> Result<ListRuleSetsInput, ApplicationError> {
        Ok(ListRuleSetsInput { program: ResourceIdentifier::parse(&path.program) })
    }

    pub fn output_success(output: Vec<RuleSet>) -> RuleSetsPresenter {
        RuleSetsPresenter { results: output.into_iter().map(rule_set_presenter).collect() }
    }
}

pub struct GetRuleSetMapper;

impl GetRuleSetMapper {
    pub fn input(path: GetRuleSetPayload) -> Result<GetRuleSetInput, ApplicationError> {
        Ok(GetRuleSetInput {
            program: ResourceIdentifier::parse(&path.program),
            rule_set: ResourceIdentifier::parse(&path.rule_set),
        })
    }

    pub fn output_success(output: RuleSet) -> RuleSetPresenter {
        rule_set_presenter(output)
    }
}

pub struct CreateRuleVersionMapper;

impl CreateRuleVersionMapper {
    pub fn input(
        path: CreateRuleVersionPathPayload,
        body: CreateRuleVersionBodyPayload,
        actor: String,
    ) -> Result<CreateRuleVersionInput, ApplicationError> {
        Ok(CreateRuleVersionInput {
            program: ResourceIdentifier::parse(&path.program),
            rule_set: ResourceIdentifier::parse(&path.rule_set),
            rule_type: body.rule_type,
            definition: body.definition,
            compiled: body.compiled,
            valid: body.valid,
            validation_errors: body.validation_errors,
            input_schema: body.input_schema,
            actor,
        })
    }

    pub fn output_success(output: SaveOutcome) -> SaveRuleVersionPresenter {
        SaveRuleVersionPresenter { version: version_presenter(output.version), created: output.created }
    }
}

pub struct ExecuteRuleMapper;

impl ExecuteRuleMapper {
    pub fn input(path: ExecuteRulePathPayload, body: ExecuteRuleBodyPayload) -> Result<ExecuteRuleInput, ApplicationError> {
        let version = match &body.version {
            None | Some(serde_json::Value::Null) => VersionSelector::Published,
            Some(serde_json::Value::String(s)) => VersionSelector::parse(Some(s))?,
            Some(serde_json::Value::Number(n)) => VersionSelector::parse(Some(&n.to_string()))?,
            Some(_) => {
                return Err(ApplicationError::validation_error(
                    "version must be \"published\", \"draft\" or a version number",
                ));
            }
        };
        Ok(ExecuteRuleInput {
            program: ResourceIdentifier::parse(&path.program),
            rule_set: ResourceIdentifier::parse(&path.rule_set),
            rule_type: body.r#type.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()),
            version,
            input: body.input,
            trace: body.trace.unwrap_or(false),
        })
    }

    pub fn output_success(output: ExecuteRuleOutput) -> ExecuteRulePresenter {
        ExecuteRulePresenter {
            program: format!("programs/{}", output.program_id),
            module: output.module_key,
            r#type: output.rule_type,
            version: output.version,
            status: output.status,
            result: output.run.result,
            duration_ms: output.duration_ms,
            trace: output.run.trace,
        }
    }
}

pub struct ListRuleVersionsMapper;

impl ListRuleVersionsMapper {
    pub fn input(
        path: ListRuleVersionsPathPayload,
        query: ListRuleVersionsQueryPayload,
    ) -> Result<ListRuleVersionsInput, ApplicationError> {
        Ok(ListRuleVersionsInput {
            program: ResourceIdentifier::parse(&path.program),
            rule_set: ResourceIdentifier::parse(&path.rule_set),
            page_size: query.max_page_size,
            page_token: parse::optional_uuid("page_token", query.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListRuleVersionsOutput) -> RuleVersionsPresenter {
        RuleVersionsPresenter {
            results: output.versions.into_iter().map(version_presenter).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
        }
    }
}

pub struct GetRuleVersionMapper;

impl GetRuleVersionMapper {
    pub fn input(path: GetRuleVersionPayload) -> Result<GetRuleVersionInput, ApplicationError> {
        Ok(GetRuleVersionInput {
            program: ResourceIdentifier::parse(&path.program),
            rule_set: ResourceIdentifier::parse(&path.rule_set),
            version: path
                .version
                .trim()
                .parse::<i32>()
                .map_err(|_| ApplicationError::validation_error("version must be a whole number"))?,
        })
    }

    pub fn output_success(output: RuleVersion) -> RuleVersionPresenter {
        version_presenter(output)
    }
}
