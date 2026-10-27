use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    CreateRuleVersionRequest, CreateRuleVersionResponse, GetRuleSetRequest, GetRuleSetResponse, GetRuleVersionRequest,
    GetRuleVersionResponse, ListRuleSetsRequest, ListRuleSetsResponse, ListRuleVersionsRequest, ListRuleVersionsResponse,
};
use crate::adapters::inputs::grpc_apis::rule_set::rule_set_mappers::{
    CreateRuleVersionGrpcMapper, GetRuleSetGrpcMapper, GetRuleVersionGrpcMapper, ListRuleSetsGrpcMapper,
    ListRuleVersionsGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct RuleSetServiceImpl {
    app_state: Arc<AppState>,
}

impl RuleSetServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl RuleSetServiceImpl {
    pub async fn list_rule_sets(
        &self,
        request: ServiceRequest<'_, ListRuleSetsRequest>,
    ) -> ServiceResult<ListRuleSetsResponse> {
        let input = ListRuleSetsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .rule_set
            .list_rule_sets
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListRuleSetsGrpcMapper::output_success(output))
    }

    pub async fn get_rule_set(
        &self,
        request: ServiceRequest<'_, GetRuleSetRequest>,
    ) -> ServiceResult<GetRuleSetResponse> {
        let input = GetRuleSetGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .rule_set
            .get_rule_set
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetRuleSetGrpcMapper::output_success(output))
    }

    pub async fn create_rule_version(
        &self,
        actor: String,
        request: ServiceRequest<'_, CreateRuleVersionRequest>,
    ) -> ServiceResult<CreateRuleVersionResponse> {
        let input = CreateRuleVersionGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .rule_set
            .create_rule_version
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(CreateRuleVersionGrpcMapper::output_success(output))
    }

    pub async fn list_rule_versions(
        &self,
        request: ServiceRequest<'_, ListRuleVersionsRequest>,
    ) -> ServiceResult<ListRuleVersionsResponse> {
        let input = ListRuleVersionsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .rule_set
            .list_rule_versions
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListRuleVersionsGrpcMapper::output_success(output))
    }

    pub async fn get_rule_version(
        &self,
        request: ServiceRequest<'_, GetRuleVersionRequest>,
    ) -> ServiceResult<GetRuleVersionResponse> {
        let input = GetRuleVersionGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .rule_set
            .get_rule_version
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetRuleVersionGrpcMapper::output_success(output))
    }
}
