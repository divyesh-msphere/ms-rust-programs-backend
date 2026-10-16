use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    GetModuleDefinitionRequest, GetModuleDefinitionResponse, ListModuleDefinitionsRequest,
    ListModuleDefinitionsResponse, ModuleDefinition as GrpcModuleDefinition,
    ModuleDefinitionTreeNode as GrpcModuleDefinitionTreeNode,
};
use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::module_definition::get_module_definition::{
    GetModuleDefinitionInput, GetModuleDefinitionOutput,
};
use crate::application::usecases::module_definition::get_module_definition_tree::{
    GetModuleDefinitionTreeInput, GetModuleDefinitionTreeOutput,
};
use crate::application::usecases::module_definition::list_module_definitions::{
    ListModuleDefinitionsInput, ListModuleDefinitionsOutput,
};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::module_definition::{ModuleDefinition, ModuleDefinitionNode};

fn to_grpc_module_definition(module_definition: ModuleDefinition) -> GrpcModuleDefinition {
    GrpcModuleDefinition {
        path: format!("module-definitions/{}", module_definition.id),
        id: module_definition.id.to_string(),
        key: module_definition.key,
        name: module_definition.name,
        kind: module_definition.kind,
        parent_id: module_definition.parent_id.map(|id| id.to_string()),
        ancestry: module_definition.ancestry,
        position: module_definition.position,
        ui_component: module_definition.ui_component,
        rule_family: module_definition.rule_family,
        universal: module_definition.universal,
        requires_publish: module_definition.requires_publish,
        permission_key: module_definition.permission_key,
        source: module_definition.source,
        handler_key: module_definition.handler_key,
        api_base: module_definition.api_base,
        selectable: module_definition.selectable,
        allow_components: module_definition.allow_components,
        create_time: format_datetime(module_definition.create_time),
        update_time: format_datetime(module_definition.update_time),
        ..Default::default()
    }
}

pub struct ListModuleDefinitionsGrpcMapper;

impl
    GrpcApiMapper<
        ListModuleDefinitionsOutput,
        ListModuleDefinitionsInput,
        ListModuleDefinitionsResponse,
        ListModuleDefinitionsRequest,
    > for ListModuleDefinitionsGrpcMapper
{
    fn input(request: ListModuleDefinitionsRequest) -> Result<ListModuleDefinitionsInput, ApplicationError> {
        Ok(ListModuleDefinitionsInput {
            parent_id: parse::optional_uuid("parent_id", request.parent_id.as_deref())?,
            page_size: request.max_page_size,
            page_token: parse::optional_uuid("page_token", request.page_token.as_deref())?,
        })
    }

    fn output_success(output: ListModuleDefinitionsOutput) -> ListModuleDefinitionsResponse {
        ListModuleDefinitionsResponse {
            results: output.module_definitions.into_iter().map(to_grpc_module_definition).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            total_size: output.total_size,
            ..Default::default()
        }
    }
}

pub struct GetModuleDefinitionGrpcMapper;

impl
    GrpcApiMapper<
        GetModuleDefinitionOutput,
        GetModuleDefinitionInput,
        GetModuleDefinitionResponse,
        GetModuleDefinitionRequest,
    > for GetModuleDefinitionGrpcMapper
{
    fn input(request: GetModuleDefinitionRequest) -> Result<GetModuleDefinitionInput, ApplicationError> {
        Ok(GetModuleDefinitionInput {
            identifier: ResourceIdentifier::parse(&request.module_definition),
        })
    }

    fn output_success(output: GetModuleDefinitionOutput) -> GetModuleDefinitionResponse {
        GetModuleDefinitionResponse {
            module_definition: Some(to_grpc_module_definition(output.module_definition)).into(),
            ..Default::default()
        }
    }
}

fn to_grpc_tree_node(node: ModuleDefinitionNode) -> GrpcModuleDefinitionTreeNode {
    let module_definition = node.module_definition;
    GrpcModuleDefinitionTreeNode {
        path: format!("module-definitions/{}", module_definition.id),
        id: module_definition.id.to_string(),
        key: module_definition.key,
        name: module_definition.name,
        kind: module_definition.kind,
        parent_id: module_definition.parent_id.map(|id| id.to_string()),
        ancestry: module_definition.ancestry,
        position: module_definition.position,
        ui_component: module_definition.ui_component,
        rule_family: module_definition.rule_family,
        universal: module_definition.universal,
        requires_publish: module_definition.requires_publish,
        permission_key: module_definition.permission_key,
        source: module_definition.source,
        handler_key: module_definition.handler_key,
        api_base: module_definition.api_base,
        selectable: module_definition.selectable,
        allow_components: module_definition.allow_components,
        create_time: format_datetime(module_definition.create_time),
        update_time: format_datetime(module_definition.update_time),
        children: node.children.into_iter().map(to_grpc_tree_node).collect(),
        ..Default::default()
    }
}

/// The `full_tree = true` form of `ListModuleDefinitions`: same request and
/// response messages, but the nested tree comes back in `tree_results`.
pub struct GetModuleDefinitionTreeGrpcMapper;

impl
    GrpcApiMapper<
        GetModuleDefinitionTreeOutput,
        GetModuleDefinitionTreeInput,
        ListModuleDefinitionsResponse,
        ListModuleDefinitionsRequest,
    > for GetModuleDefinitionTreeGrpcMapper
{
    fn input(request: ListModuleDefinitionsRequest) -> Result<GetModuleDefinitionTreeInput, ApplicationError> {
        Ok(GetModuleDefinitionTreeInput {
            root_id: parse::optional_uuid("parent_id", request.parent_id.as_deref())?,
        })
    }

    fn output_success(output: GetModuleDefinitionTreeOutput) -> ListModuleDefinitionsResponse {
        ListModuleDefinitionsResponse {
            tree_results: output.tree.into_iter().map(to_grpc_tree_node).collect(),
            full_tree: true,
            ..Default::default()
        }
    }
}
