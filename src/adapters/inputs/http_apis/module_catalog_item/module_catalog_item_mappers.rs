use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
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
use crate::dto::payloads::json::module_catalog_item::{
    GetModuleCatalogItemPayload, ListModuleCatalogPayload,
};
use crate::dto::presenters::json::module_catalog_item::{
    ListModuleCatalogPresenter, ModuleCatalogItemPresenter, ModuleCatalogTreeNodePresenter,
    ModuleCatalogTreePresenter,
};

pub fn to_presenter(module_catalog_item: ModuleCatalogItem) -> ModuleCatalogItemPresenter {
    ModuleCatalogItemPresenter {
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
    }
}

pub struct ListModuleCatalogMapper;

impl ListModuleCatalogMapper {
    pub fn input(payload: ListModuleCatalogPayload) -> Result<ListModuleCatalogInput, ApplicationError> {
        Ok(ListModuleCatalogInput {
            parent_id: parse::optional_uuid("parent_id", payload.parent_id.as_deref())?,
            page_size: payload.max_page_size,
            page_token: parse::optional_uuid("page_token", payload.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListModuleCatalogOutput) -> ListModuleCatalogPresenter {
        ListModuleCatalogPresenter {
            results: output.module_catalog.into_iter().map(to_presenter).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            total_size: output.total_size,
        }
    }
}

pub struct GetModuleCatalogItemMapper;

impl GetModuleCatalogItemMapper {
    pub fn input(payload: GetModuleCatalogItemPayload) -> Result<GetModuleCatalogItemInput, ApplicationError> {
        Ok(GetModuleCatalogItemInput {
            identifier: ResourceIdentifier::parse(&payload.module_catalog_item),
        })
    }

    pub fn output_success(output: GetModuleCatalogItemOutput) -> ModuleCatalogItemPresenter {
        to_presenter(output.module_catalog_item)
    }
}

fn to_tree_node_presenter(node: ModuleCatalogItemNode) -> ModuleCatalogTreeNodePresenter {
    ModuleCatalogTreeNodePresenter {
        module_catalog_item: to_presenter(node.module_catalog_item),
        children: node.children.into_iter().map(to_tree_node_presenter).collect(),
    }
}

pub struct GetModuleCatalogTreeMapper;

impl GetModuleCatalogTreeMapper {
    pub fn input(payload: ListModuleCatalogPayload) -> Result<GetModuleCatalogTreeInput, ApplicationError> {
        Ok(GetModuleCatalogTreeInput {
            root_id: parse::optional_uuid("parent_id", payload.parent_id.as_deref())?,
        })
    }

    pub fn output_success(output: GetModuleCatalogTreeOutput) -> ModuleCatalogTreePresenter {
        ModuleCatalogTreePresenter {
            results: output.tree.into_iter().map(to_tree_node_presenter).collect(),
        }
    }
}
