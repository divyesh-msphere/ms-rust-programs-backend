use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    GetModuleDefinitionRequest, GetModuleDefinitionResponse, ListModuleDefinitionsRequest,
    ListModuleDefinitionsResponse,
};
use crate::adapters::inputs::grpc_apis::module_definition::module_definition_mappers::{
    GetModuleDefinitionGrpcMapper, GetModuleDefinitionTreeGrpcMapper, ListModuleDefinitionsGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct ModuleDefinitionServiceImpl {
    app_state: Arc<AppState>,
}

impl ModuleDefinitionServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl ModuleDefinitionServiceImpl {
    pub async fn list_module_definitions(
        &self,
        request: ServiceRequest<'_, ListModuleDefinitionsRequest>,
    ) -> ServiceResult<ListModuleDefinitionsResponse> {
        let request = request.to_owned_message();

        // `full_tree = true` swaps the paginated flat list for the nested tree.
        if request.full_tree.unwrap_or(false) {
            let tree_input = GetModuleDefinitionTreeGrpcMapper::input(request)
                .map_err(GrpcErrorResponse::map_response_error)?;

            let tree_output = self
                .app_state
                .use_cases
                .module_definition
                .get_module_definition_tree
                .execute(Some(tree_input))
                .await
                .map_err(GrpcErrorResponse::map_response_error)?;

            return Response::ok(GetModuleDefinitionTreeGrpcMapper::output_success(tree_output));
        }

        let input = ListModuleDefinitionsGrpcMapper::input(request)
            .map_err(GrpcErrorResponse::map_response_error)?;

        let output = self
            .app_state
            .use_cases
            .module_definition
            .list_module_definitions
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListModuleDefinitionsGrpcMapper::output_success(output))
    }

    pub async fn get_module_definition(
        &self,
        request: ServiceRequest<'_, GetModuleDefinitionRequest>,
    ) -> ServiceResult<GetModuleDefinitionResponse> {
        let input = GetModuleDefinitionGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;

        let output = self
            .app_state
            .use_cases
            .module_definition
            .get_module_definition
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetModuleDefinitionGrpcMapper::output_success(output))
    }
}
