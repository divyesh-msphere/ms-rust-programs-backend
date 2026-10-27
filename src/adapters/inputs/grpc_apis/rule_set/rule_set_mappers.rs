use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    CreateRuleVersionRequest, CreateRuleVersionResponse, GetRuleSetRequest, GetRuleSetResponse, GetRuleVersionRequest,
    GetRuleVersionResponse, ListRuleSetsRequest, ListRuleSetsResponse, ListRuleVersionsRequest, ListRuleVersionsResponse,
    RuleSet as GrpcRuleSet, RuleVersion as GrpcRuleVersion,
};
use crate::adapters::inputs::grpc_apis::shared::json_struct::{buffa_value_to_json, json_to_buffa_value};
use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::rule_set::create_rule_version::CreateRuleVersionInput;
use crate::application::usecases::rule_set::get_rule_set::GetRuleSetInput;
use crate::application::usecases::rule_set::get_rule_version::GetRuleVersionInput;
use crate::application::usecases::rule_set::list_rule_sets::ListRuleSetsInput;
use crate::application::usecases::rule_set::list_rule_versions::{ListRuleVersionsInput, ListRuleVersionsOutput};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::rule_set::{RuleSet, RuleVersion, SaveOutcome};

fn to_grpc_version(version: RuleVersion) -> GrpcRuleVersion {
    GrpcRuleVersion {
        path: format!("programs/{}/rule-sets/{}/versions/{}", version.program_id, version.module_key, version.version),
        id: version.id.to_string(),
        version: version.version,
        status: version.status,
        definition: version.definition.map(json_to_buffa_value).into(),
        compiled: version.compiled.map(json_to_buffa_value).into(),
        valid: version.valid,
        validation_errors: Some(json_to_buffa_value(version.validation_errors)).into(),
        content_hash: version.content_hash,
        size_bytes: version.size_bytes,
        created_by: version.created_by,
        create_time: format_datetime(version.create_time),
        ..Default::default()
    }
}

fn to_grpc_rule_set(rule_set: RuleSet) -> GrpcRuleSet {
    let state = rule_set.state();
    GrpcRuleSet {
        path: format!("programs/{}/rule-sets/{}", rule_set.program_id, rule_set.module_key),
        id: rule_set.rule_set_id.map(|id| id.to_string()),
        module_key: rule_set.module_key,
        module_name: rule_set.module_name,
        rule_type: rule_set.rule_type,
        draft_version: rule_set.draft_version,
        published_version: rule_set.published_version,
        state: state.to_string(),
        draft: rule_set.draft.map(to_grpc_version).into(),
        update_time: rule_set.update_time.map(format_datetime),
        ..Default::default()
    }
}

pub struct ListRuleSetsGrpcMapper;

impl ListRuleSetsGrpcMapper {
    pub fn input(request: ListRuleSetsRequest) -> Result<ListRuleSetsInput, ApplicationError> {
        Ok(ListRuleSetsInput { program: ResourceIdentifier::parse(&request.program) })
    }

    pub fn output_success(output: Vec<RuleSet>) -> ListRuleSetsResponse {
        ListRuleSetsResponse { results: output.into_iter().map(to_grpc_rule_set).collect(), ..Default::default() }
    }
}

pub struct GetRuleSetGrpcMapper;

impl GetRuleSetGrpcMapper {
    pub fn input(request: GetRuleSetRequest) -> Result<GetRuleSetInput, ApplicationError> {
        Ok(GetRuleSetInput {
            program: ResourceIdentifier::parse(&request.program),
            rule_set: ResourceIdentifier::parse(&request.rule_set),
        })
    }

    pub fn output_success(output: RuleSet) -> GetRuleSetResponse {
        GetRuleSetResponse { rule_set: Some(to_grpc_rule_set(output)).into(), ..Default::default() }
    }
}

pub struct CreateRuleVersionGrpcMapper;

impl CreateRuleVersionGrpcMapper {
    pub fn input(request: CreateRuleVersionRequest, actor: String) -> Result<CreateRuleVersionInput, ApplicationError> {
        Ok(CreateRuleVersionInput {
            program: ResourceIdentifier::parse(&request.program),
            rule_set: ResourceIdentifier::parse(&request.rule_set),
            rule_type: request.rule_type,
            definition: request
                .definition
                .into_option()
                .map(buffa_value_to_json)
                .ok_or_else(|| ApplicationError::validation_error("definition is required"))?,
            compiled: request.compiled.into_option().map(buffa_value_to_json),
            valid: request.valid,
            validation_errors: request.validation_errors.into_option().map(buffa_value_to_json),
            actor,
        })
    }

    pub fn output_success(output: SaveOutcome) -> CreateRuleVersionResponse {
        CreateRuleVersionResponse { version: Some(to_grpc_version(output.version)).into(), created: output.created, ..Default::default() }
    }
}

pub struct ListRuleVersionsGrpcMapper;

impl ListRuleVersionsGrpcMapper {
    pub fn input(request: ListRuleVersionsRequest) -> Result<ListRuleVersionsInput, ApplicationError> {
        Ok(ListRuleVersionsInput {
            program: ResourceIdentifier::parse(&request.program),
            rule_set: ResourceIdentifier::parse(&request.rule_set),
            page_size: request.max_page_size,
            page_token: parse::optional_uuid("page_token", request.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListRuleVersionsOutput) -> ListRuleVersionsResponse {
        ListRuleVersionsResponse {
            results: output.versions.into_iter().map(to_grpc_version).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            ..Default::default()
        }
    }
}

pub struct GetRuleVersionGrpcMapper;

impl GetRuleVersionGrpcMapper {
    pub fn input(request: GetRuleVersionRequest) -> Result<GetRuleVersionInput, ApplicationError> {
        Ok(GetRuleVersionInput {
            program: ResourceIdentifier::parse(&request.program),
            rule_set: ResourceIdentifier::parse(&request.rule_set),
            version: request.version,
        })
    }

    pub fn output_success(output: RuleVersion) -> GetRuleVersionResponse {
        GetRuleVersionResponse { version: Some(to_grpc_version(output)).into(), ..Default::default() }
    }
}
