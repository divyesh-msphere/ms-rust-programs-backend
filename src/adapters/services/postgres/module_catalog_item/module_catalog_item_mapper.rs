use crate::adapters::services::postgres::module_catalog_item::module_catalog_item_model::ModuleCatalogItemDbModel;
use crate::application::mappers::db_mapper::DbMapper;
use crate::domain::module_catalog_item::ModuleCatalogItem;

pub struct ModuleCatalogItemDbMapper;

impl DbMapper<ModuleCatalogItem, ModuleCatalogItemDbModel> for ModuleCatalogItemDbMapper {
    fn to_entity(model: ModuleCatalogItemDbModel) -> ModuleCatalogItem {
        ModuleCatalogItem {
            id: model.id,
            key: model.key,
            name: model.name,
            kind: model.kind,
            parent_id: model.parent_id,
            ancestry: model.ancestry,
            position: model.position,
            ui_component: model.ui_component,
            rule_family: model.rule_family,
            universal: model.universal,
            requires_publish: model.requires_publish,
            permission_key: model.permission_key,
            source: model.source,
            handler_key: model.handler_key,
            api_base: model.api_base,
            selectable: model.selectable,
            allow_components: model.allow_components,
            default_mode: model.default_mode,
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }

    fn to_db(entity: ModuleCatalogItem) -> ModuleCatalogItemDbModel {
        ModuleCatalogItemDbModel {
            id: entity.id,
            key: entity.key,
            name: entity.name,
            kind: entity.kind,
            parent_id: entity.parent_id,
            ancestry: entity.ancestry,
            position: entity.position,
            ui_component: entity.ui_component,
            rule_family: entity.rule_family,
            universal: entity.universal,
            requires_publish: entity.requires_publish,
            permission_key: entity.permission_key,
            source: entity.source,
            handler_key: entity.handler_key,
            api_base: entity.api_base,
            selectable: entity.selectable,
            allow_components: entity.allow_components,
            default_mode: entity.default_mode,
            create_time: entity.create_time,
            update_time: entity.update_time,
        }
    }
}
