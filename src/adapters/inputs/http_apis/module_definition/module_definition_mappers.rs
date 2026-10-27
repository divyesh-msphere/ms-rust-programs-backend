use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
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
use crate::dto::payloads::json::module_definition::{
    GetModuleDefinitionPayload, ListModuleDefinitionsPayload,
};
use crate::dto::presenters::json::module_definition::{
    ListModuleDefinitionsPresenter, ModuleDefinitionPresenter, ModuleDefinitionTreeNodePresenter,
    ModuleDefinitionTreePresenter,
};

pub fn to_presenter(module_definition: ModuleDefinition) -> ModuleDefinitionPresenter {
    ModuleDefinitionPresenter {
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
        default_mode: module_definition.default_mode,
        create_time: format_datetime(module_definition.create_time),
        update_time: format_datetime(module_definition.update_time),
    }
}

pub struct ListModuleDefinitionsMapper;

impl ListModuleDefinitionsMapper {
    pub fn input(payload: ListModuleDefinitionsPayload) -> Result<ListModuleDefinitionsInput, ApplicationError> {
        Ok(ListModuleDefinitionsInput {
            parent_id: parse::optional_uuid("parent_id", payload.parent_id.as_deref())?,
            page_size: payload.max_page_size,
            page_token: parse::optional_uuid("page_token", payload.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListModuleDefinitionsOutput) -> ListModuleDefinitionsPresenter {
        ListModuleDefinitionsPresenter {
            results: output.module_definitions.into_iter().map(to_presenter).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            total_size: output.total_size,
        }
    }
}

pub struct GetModuleDefinitionMapper;

impl GetModuleDefinitionMapper {
    pub fn input(payload: GetModuleDefinitionPayload) -> Result<GetModuleDefinitionInput, ApplicationError> {
        Ok(GetModuleDefinitionInput {
            identifier: ResourceIdentifier::parse(&payload.module_definition),
        })
    }

    pub fn output_success(output: GetModuleDefinitionOutput) -> ModuleDefinitionPresenter {
        to_presenter(output.module_definition)
    }
}

fn to_tree_node_presenter(node: ModuleDefinitionNode) -> ModuleDefinitionTreeNodePresenter {
    ModuleDefinitionTreeNodePresenter {
        module_definition: to_presenter(node.module_definition),
        children: node.children.into_iter().map(to_tree_node_presenter).collect(),
    }
}

pub struct GetModuleDefinitionTreeMapper;

impl GetModuleDefinitionTreeMapper {
    pub fn input(payload: ListModuleDefinitionsPayload) -> Result<GetModuleDefinitionTreeInput, ApplicationError> {
        Ok(GetModuleDefinitionTreeInput {
            root_id: parse::optional_uuid("parent_id", payload.parent_id.as_deref())?,
        })
    }

    pub fn output_success(output: GetModuleDefinitionTreeOutput) -> ModuleDefinitionTreePresenter {
        ModuleDefinitionTreePresenter {
            results: output.tree.into_iter().map(to_tree_node_presenter).collect(),
        }
    }
}
