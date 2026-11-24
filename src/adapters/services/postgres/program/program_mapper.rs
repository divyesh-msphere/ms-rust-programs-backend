use crate::adapters::services::postgres::program::program_model::ProgramDbModel;
use crate::application::mappers::db_mapper::DbMapper;
use crate::domain::aep::ResourceState;
use crate::domain::program::{Branding, Program, Renewal};

pub struct ProgramDbMapper;

impl DbMapper<Program, ProgramDbModel> for ProgramDbMapper {
    fn to_entity(model: ProgramDbModel) -> Program {
        Program {
            id: model.id,
            name: model.name,
            slug: model.slug,
            description: model.description,
            about: model.about,
            highlights: model.highlights,
            branding: Branding {
                brand_color: model.brand_color,
                text_color: model.text_color,
                logo_icon: model.logo_icon,
            },
            renewal: Renewal {
                term_months: model.renewal_term_months,
                grace_period_days: model.renewal_grace_days,
                reminder_days: model.renewal_reminder_days,
                auto_renew: model.auto_renew,
                enabled: model.renewal_enabled,
            },
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
            about: entity.about,
            highlights: entity.highlights,
            brand_color: entity.branding.brand_color,
            text_color: entity.branding.text_color,
            logo_icon: entity.branding.logo_icon,
            renewal_term_months: entity.renewal.term_months,
            renewal_grace_days: entity.renewal.grace_period_days,
            renewal_reminder_days: entity.renewal.reminder_days,
            auto_renew: entity.renewal.auto_renew,
            renewal_enabled: entity.renewal.enabled,
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
