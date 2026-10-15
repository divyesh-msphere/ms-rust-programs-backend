use crate::adapters::services::postgres::module_definition::module_definition_model::ModuleDefinitionDbModel;
use crate::application::mappers::db_mapper::DbMapper;
use crate::domain::module_definition::ModuleDefinition;

pub struct ModuleDefinitionDbMapper;

impl DbMapper<ModuleDefinition, ModuleDefinitionDbModel> for ModuleDefinitionDbMapper {
    fn to_entity(model: ModuleDefinitionDbModel) -> ModuleDefinition {
        ModuleDefinition {
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
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }

    fn to_db(entity: ModuleDefinition) -> ModuleDefinitionDbModel {
        ModuleDefinitionDbModel {
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
            create_time: entity.create_time,
            update_time: entity.update_time,
        }
    }
}
