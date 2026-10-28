use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    GetModuleCatalogItemRequest, GetModuleCatalogItemResponse, ListModuleCatalogRequest,
    ListModuleCatalogResponse,
};
use crate::adapters::inputs::grpc_apis::module_catalog_item::module_catalog_item_mappers::{
    GetModuleCatalogItemGrpcMapper, GetModuleCatalogTreeGrpcMapper, ListModuleCatalogGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct ModuleCatalogItemServiceImpl {
    app_state: Arc<AppState>,
}

impl ModuleCatalogItemServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl ModuleCatalogItemServiceImpl {
    pub async fn list_module_catalog(
        &self,
        request: ServiceRequest<'_, ListModuleCatalogRequest>,
    ) -> ServiceResult<ListModuleCatalogResponse> {
        let request = request.to_owned_message();

        // `full_tree = true` swaps the paginated flat list for the nested tree.
        if request.full_tree.unwrap_or(false) {
            let tree_input = GetModuleCatalogTreeGrpcMapper::input(request)
                .map_err(GrpcErrorResponse::map_response_error)?;

            let tree_output = self
                .app_state
                .use_cases
                .module_catalog_item
                .get_module_catalog_tree
                .execute(Some(tree_input))
                .await
                .map_err(GrpcErrorResponse::map_response_error)?;

            return Response::ok(GetModuleCatalogTreeGrpcMapper::output_success(tree_output));
        }

        let input = ListModuleCatalogGrpcMapper::input(request)
            .map_err(GrpcErrorResponse::map_response_error)?;

        let output = self
            .app_state
            .use_cases
            .module_catalog_item
            .list_module_catalog
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListModuleCatalogGrpcMapper::output_success(output))
    }

    pub async fn get_module_catalog_item(
        &self,
        request: ServiceRequest<'_, GetModuleCatalogItemRequest>,
    ) -> ServiceResult<GetModuleCatalogItemResponse> {
        let input = GetModuleCatalogItemGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;

        let output = self
            .app_state
            .use_cases
            .module_catalog_item
            .get_module_catalog_item
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetModuleCatalogItemGrpcMapper::output_success(output))
    }
}
