use crate::adapters::services::postgres::program::program_model::ProgramDbModel;
use crate::application::mappers::db_mapper::DbMapper;
use crate::domain::aep::ResourceState;
use crate::domain::program::Program;

pub struct ProgramDbMapper;

impl DbMapper<Program, ProgramDbModel> for ProgramDbMapper {
    fn to_entity(model: ProgramDbModel) -> Program {
        Program {
            id: model.id,
            name: model.name,
            slug: model.slug,
            description: model.description,
            level: model.level,
            active: model.active,
            config_version: model.config_version,
            config_hash: model.config_hash,
            state: ResourceState::parse(&model.state),
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }

    fn to_db(entity: Program) -> ProgramDbModel {
        ProgramDbModel {
            id: entity.id,
            name: entity.name,
            slug: entity.slug,
            description: entity.description,
            level: entity.level,
            active: entity.active,
            config_version: entity.config_version,
            config_hash: entity.config_hash,
            state: entity.state.as_str().to_string(),
            create_time: entity.create_time,
            update_time: entity.update_time,
        }
    }
}
