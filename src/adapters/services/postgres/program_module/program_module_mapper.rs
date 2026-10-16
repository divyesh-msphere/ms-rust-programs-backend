use crate::adapters::services::postgres::program_module::program_module_model::ProgramModuleDbModel;
use crate::application::mappers::db_mapper::DbMapper;
use crate::domain::program_module::ProgramModule;

pub struct ProgramModuleDbMapper;

impl DbMapper<ProgramModule, ProgramModuleDbModel> for ProgramModuleDbMapper {
    fn to_entity(model: ProgramModuleDbModel) -> ProgramModule {
        ProgramModule {
            module_id: model.module_id,
            key: model.key,
            name: model.name,
            kind: model.kind,
            parent_id: model.parent_id,
            ancestry: model.ancestry,
            source: model.source,
            handler_key: model.handler_key,
            api_base: model.api_base,
            ui_component: model.ui_component,
            allow_components: model.allow_components,
            requires_publish: model.requires_publish,
            enabled: model.enabled,
            mode: model.mode,
            settings: Vec::new(),
            rule: None,
            position: model.position,
            update_time: model.update_time,
        }
    }

    fn to_db(entity: ProgramModule) -> ProgramModuleDbModel {
        ProgramModuleDbModel {
            module_id: entity.module_id,
            key: entity.key,
            name: entity.name,
            kind: entity.kind,
            parent_id: entity.parent_id,
            ancestry: entity.ancestry,
            source: entity.source,
            handler_key: entity.handler_key,
            api_base: entity.api_base,
            ui_component: entity.ui_component,
            allow_components: entity.allow_components,
            requires_publish: entity.requires_publish,
            enabled: entity.enabled,
            mode: entity.mode,
            position: entity.position,
            update_time: entity.update_time,
        }
    }
}
