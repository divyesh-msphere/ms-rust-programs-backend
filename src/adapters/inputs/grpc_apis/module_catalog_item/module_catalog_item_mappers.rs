use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    GetModuleCatalogItemRequest, GetModuleCatalogItemResponse, ListModuleCatalogRequest,
    ListModuleCatalogResponse, ModuleCatalogItem as GrpcModuleCatalogItem,
    ModuleCatalogTreeNode as GrpcModuleCatalogTreeNode,
};
use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::module_catalog_item::get_module_catalog_item::{
    GetModuleCatalogItemInput, GetModuleCatalogItemOutput,
};
use crate::application::usecases::module_catalog_item::get_module_catalog_tree::{
    GetModuleCatalogTreeInput, GetModuleCatalogTreeOutput,
};
use crate::application::usecases::module_catalog_item::list_module_catalog::{
    ListModuleCatalogInput, ListModuleCatalogOutput,
};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::module_catalog_item::{ModuleCatalogItem, ModuleCatalogItemNode};

fn to_grpc_module_catalog_item(module_catalog_item: ModuleCatalogItem) -> GrpcModuleCatalogItem {
    GrpcModuleCatalogItem {
        path: format!("module-catalog/{}", module_catalog_item.id),
        id: module_catalog_item.id.to_string(),
        key: module_catalog_item.key,
        name: module_catalog_item.name,
        kind: module_catalog_item.kind,
        parent_id: module_catalog_item.parent_id.map(|id| id.to_string()),
        ancestry: module_catalog_item.ancestry,
        position: module_catalog_item.position,
        ui_component: module_catalog_item.ui_component,
        rule_family: module_catalog_item.rule_family,
        universal: module_catalog_item.universal,
        requires_publish: module_catalog_item.requires_publish,
        permission_key: module_catalog_item.permission_key,
        source: module_catalog_item.source,
        handler_key: module_catalog_item.handler_key,
        api_base: module_catalog_item.api_base,
        selectable: module_catalog_item.selectable,
        allow_components: module_catalog_item.allow_components,
        default_mode: module_catalog_item.default_mode,
        create_time: format_datetime(module_catalog_item.create_time),
        update_time: format_datetime(module_catalog_item.update_time),
        ..Default::default()
    }
}

pub struct ListModuleCatalogGrpcMapper;

impl
    GrpcApiMapper<
        ListModuleCatalogOutput,
        ListModuleCatalogInput,
        ListModuleCatalogResponse,
        ListModuleCatalogRequest,
    > for ListModuleCatalogGrpcMapper
{
    fn input(request: ListModuleCatalogRequest) -> Result<ListModuleCatalogInput, ApplicationError> {
        Ok(ListModuleCatalogInput {
            parent_id: parse::optional_uuid("parent_id", request.parent_id.as_deref())?,
            page_size: request.max_page_size,
            page_token: parse::optional_uuid("page_token", request.page_token.as_deref())?,
        })
    }

    fn output_success(output: ListModuleCatalogOutput) -> ListModuleCatalogResponse {
        ListModuleCatalogResponse {
            results: output.module_catalog.into_iter().map(to_grpc_module_catalog_item).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            total_size: output.total_size,
            ..Default::default()
        }
    }
}

pub struct GetModuleCatalogItemGrpcMapper;

impl
    GrpcApiMapper<
        GetModuleCatalogItemOutput,
        GetModuleCatalogItemInput,
        GetModuleCatalogItemResponse,
        GetModuleCatalogItemRequest,
    > for GetModuleCatalogItemGrpcMapper
{
    fn input(request: GetModuleCatalogItemRequest) -> Result<GetModuleCatalogItemInput, ApplicationError> {
        Ok(GetModuleCatalogItemInput {
            identifier: ResourceIdentifier::parse(&request.module_catalog_item),
        })
    }

    fn output_success(output: GetModuleCatalogItemOutput) -> GetModuleCatalogItemResponse {
        GetModuleCatalogItemResponse {
            module_catalog_item: Some(to_grpc_module_catalog_item(output.module_catalog_item)).into(),
            ..Default::default()
        }
    }
}

fn to_grpc_tree_node(node: ModuleCatalogItemNode) -> GrpcModuleCatalogTreeNode {
    let module_catalog_item = node.module_catalog_item;
    GrpcModuleCatalogTreeNode {
        path: format!("module-catalog/{}", module_catalog_item.id),
        id: module_catalog_item.id.to_string(),
        key: module_catalog_item.key,
        name: module_catalog_item.name,
        kind: module_catalog_item.kind,
        parent_id: module_catalog_item.parent_id.map(|id| id.to_string()),
        ancestry: module_catalog_item.ancestry,
        position: module_catalog_item.position,
        ui_component: module_catalog_item.ui_component,
        rule_family: module_catalog_item.rule_family,
        universal: module_catalog_item.universal,
        requires_publish: module_catalog_item.requires_publish,
        permission_key: module_catalog_item.permission_key,
        source: module_catalog_item.source,
        handler_key: module_catalog_item.handler_key,
        api_base: module_catalog_item.api_base,
        selectable: module_catalog_item.selectable,
        allow_components: module_catalog_item.allow_components,
        default_mode: module_catalog_item.default_mode,
        create_time: format_datetime(module_catalog_item.create_time),
        update_time: format_datetime(module_catalog_item.update_time),
        children: node.children.into_iter().map(to_grpc_tree_node).collect(),
        ..Default::default()
    }
}

/// The `full_tree = true` form of `ListModuleCatalog`: same request and
/// response messages, but the nested tree comes back in `tree_results`.
pub struct GetModuleCatalogTreeGrpcMapper;

impl
    GrpcApiMapper<
        GetModuleCatalogTreeOutput,
        GetModuleCatalogTreeInput,
        ListModuleCatalogResponse,
        ListModuleCatalogRequest,
    > for GetModuleCatalogTreeGrpcMapper
{
    fn input(request: ListModuleCatalogRequest) -> Result<GetModuleCatalogTreeInput, ApplicationError> {
        Ok(GetModuleCatalogTreeInput {
            root_id: parse::optional_uuid("parent_id", request.parent_id.as_deref())?,
        })
    }

    fn output_success(output: GetModuleCatalogTreeOutput) -> ListModuleCatalogResponse {
        ListModuleCatalogResponse {
            tree_results: output.tree.into_iter().map(to_grpc_tree_node).collect(),
            full_tree: true,
            ..Default::default()
        }
    }
}
