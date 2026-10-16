use crate::adapters::inputs::http_apis::program_module::program_module_mappers::to_presenter as module_to_presenter;
use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::program::create_program::{CreateProgramInput, CreateProgramOutput};
use crate::application::usecases::program::delete_program::DeleteProgramInput;
use crate::application::usecases::program::get_program::{GetProgramInput, GetProgramOutput};
use crate::application::usecases::program::list_programs::{ListProgramsInput, ListProgramsOutput};
use crate::application::usecases::program::undelete_program::{UndeleteProgramInput, UndeleteProgramOutput};
use crate::application::usecases::program::update_program::{UpdateProgramInput, UpdateProgramOutput};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::Program;
use crate::domain::program_setting::{ModuleChange, ModuleMode, OptionInput, SettingInput, SettingType};
use crate::dto::payloads::json::program::{
    CreateProgramBodyPayload, CreateProgramQueryPayload, DeleteProgramPayload, GetProgramPayload,
    ListProgramsPayload, ModulePayload, OptionPayload, SettingPayload, UndeleteProgramPayload, UpdateProgramBodyPayload, UpdateProgramPathPayload,
};
use crate::dto::presenters::json::program::{ListProgramsPresenter, ProgramPresenter, ProgramWithModulesPresenter};

pub fn to_presenter(program: Program) -> ProgramPresenter {
    ProgramPresenter {
        path: format!("programs/{}", program.id),
        id: program.id.to_string(),
        name: program.name,
        slug: program.slug,
        description: program.description,
        level: program.level,
        active: program.active,
        config_version: program.config_version,
        config_hash: program.config_hash,
        create_time: format_datetime(program.create_time),
        update_time: format_datetime(program.update_time),
        state: program.state.as_str().to_string(),
    }
}

fn to_option(payload: OptionPayload) -> OptionInput {
    OptionInput { code: payload.code, label: payload.label }
}

fn to_setting(payload: SettingPayload) -> Result<SettingInput, ApplicationError> {
    Ok(SettingInput {
        id: parse::optional_uuid("settings[].id", payload.id.as_deref())?,
        name: payload.name,
        field_type: SettingType::parse(&payload.field_type)?,
        required: payload.required.unwrap_or(false),
        default_value: payload.default_value,
        library_id: parse::optional_uuid("settings[].library_id", payload.library_id.as_deref())?,
        options: payload.options.unwrap_or_default().into_iter().map(to_option).collect(),
    })
}

/// The module entries of a create / update request as domain changes.
pub fn to_module_changes(payloads: Vec<ModulePayload>) -> Result<Vec<ModuleChange>, ApplicationError> {
    payloads
        .into_iter()
        .map(|payload| match payload {
            ModulePayload::Key(key) => Ok(ModuleChange::plain(key)),
            ModulePayload::Config(config) => Ok(ModuleChange {
                key: config.key,
                enabled: config.enabled,
                mode: config.mode.as_deref().map(ModuleMode::parse).transpose()?,
                settings: config
                    .settings
                    .map(|settings| settings.into_iter().map(to_setting).collect::<Result<Vec<_>, _>>())
                    .transpose()?,
            }),
        })
        .collect()
}

fn modules_presenters(program_id: &str, modules: Vec<crate::domain::program_module::ProgramModule>) -> Vec<crate::dto::presenters::json::program_module::ProgramModulePresenter> {
    let program_path = format!("programs/{program_id}");
    modules.into_iter().map(|m| module_to_presenter(&program_path, m)).collect()
}

pub struct ListProgramsMapper;

impl ListProgramsMapper {
    pub fn input(payload: ListProgramsPayload) -> Result<ListProgramsInput, ApplicationError> {
        Ok(ListProgramsInput {
            page_size: payload.max_page_size,
            page_token: parse::optional_uuid("page_token", payload.page_token.as_deref())?,
            show_deleted: payload.show_deleted.unwrap_or(false),
        })
    }

    pub fn output_success(output: ListProgramsOutput) -> ListProgramsPresenter {
        ListProgramsPresenter {
            results: output.programs.into_iter().map(to_presenter).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            total_size: output.total_size,
        }
    }
}

pub struct GetProgramMapper;

impl GetProgramMapper {
    pub fn input(payload: GetProgramPayload) -> Result<GetProgramInput, ApplicationError> {
        Ok(GetProgramInput { identifier: ResourceIdentifier::parse(&payload.program) })
    }

    pub fn output_success(output: GetProgramOutput) -> ProgramPresenter {
        to_presenter(output.program)
    }
}

pub struct CreateProgramMapper;

impl CreateProgramMapper {
    pub fn input(
        body: CreateProgramBodyPayload,
        query: CreateProgramQueryPayload,
        actor: String,
    ) -> Result<CreateProgramInput, ApplicationError> {
        Ok(CreateProgramInput {
            id: parse::optional_uuid("id", query.id.as_deref())?,
            name: body.name,
            slug: body.slug,
            description: body.description,
            level: body.level,
            modules: to_module_changes(body.modules.unwrap_or_default())?,
            actor,
        })
    }

    pub fn output_success(output: CreateProgramOutput) -> ProgramWithModulesPresenter {
        let program_id = output.program.id.to_string();
        ProgramWithModulesPresenter {
            program: to_presenter(output.program),
            modules: Some(modules_presenters(&program_id, output.modules)),
        }
    }
}

pub struct UpdateProgramMapper;

impl UpdateProgramMapper {
    pub fn input(
        path: UpdateProgramPathPayload,
        body: UpdateProgramBodyPayload,
        actor: String,
    ) -> Result<UpdateProgramInput, ApplicationError> {
        Ok(UpdateProgramInput {
            identifier: ResourceIdentifier::parse(&path.program),
            name: body.name,
            description: body.description,
            level: body.level,
            active: body.active,
            modules: body.modules.map(to_module_changes).transpose()?,
            actor,
        })
    }

    /// The program, plus the module manifest when the request touched modules.
    pub fn output_success(output: UpdateProgramOutput) -> ProgramWithModulesPresenter {
        let program_id = output.program.id.to_string();
        ProgramWithModulesPresenter {
            program: to_presenter(output.program),
            modules: output.modules.map(|modules| modules_presenters(&program_id, modules)),
        }
    }
}

pub struct DeleteProgramMapper;

impl DeleteProgramMapper {
    pub fn input(payload: DeleteProgramPayload, actor: String) -> Result<DeleteProgramInput, ApplicationError> {
        Ok(DeleteProgramInput { identifier: ResourceIdentifier::parse(&payload.program), actor })
    }
}

pub struct UndeleteProgramMapper;

impl UndeleteProgramMapper {
    pub fn input(payload: UndeleteProgramPayload, actor: String) -> Result<UndeleteProgramInput, ApplicationError> {
        Ok(UndeleteProgramInput { identifier: ResourceIdentifier::parse(&payload.program), actor })
    }

    pub fn output_success(output: UndeleteProgramOutput) -> ProgramPresenter {
        to_presenter(output.program)
    }
}
