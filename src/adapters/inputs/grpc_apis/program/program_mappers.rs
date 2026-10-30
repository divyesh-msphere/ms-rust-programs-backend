use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    CreateProgramRequest, CreateProgramResponse, DeleteProgramRequest, GetProgramRequest, GetProgramResponse,
    ListProgramsRequest, ListProgramsResponse, ModuleConfig, ProgramDomain, ProgramLastActivity, ProgramListSummary, ProgramStats, OptionConfig, Program as GrpcProgram, SettingConfig, UndeleteProgramRequest,
    UndeleteProgramResponse, UpdateProgramRequest, UpdateProgramResponse,
};
use crate::adapters::inputs::grpc_apis::program_module::program_module_mappers::to_grpc_program_module;
use crate::adapters::inputs::grpc_apis::shared::json_struct::buffa_value_to_json;
use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::program::create_program::{CreateProgramInput, CreateProgramOutput};
use crate::application::usecases::program::delete_program::DeleteProgramInput;
use crate::application::usecases::program::get_program::{GetProgramInput, GetProgramOutput};
use crate::application::usecases::program::list_programs::{ListProgramsInput, ListProgramsOutput};
use crate::application::usecases::program::undelete_program::{UndeleteProgramInput, UndeleteProgramOutput};
use crate::application::usecases::program::update_program::{UpdateProgramInput, UpdateProgramOutput};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::{Program, ProgramListItem};
use crate::domain::program_setting::{ModuleChange, ModuleMode, OptionInput, SettingInput, SettingType};

fn to_option(config: OptionConfig) -> OptionInput {
    OptionInput { code: config.code, label: config.label }
}

fn to_setting(config: SettingConfig) -> Result<SettingInput, ApplicationError> {
    Ok(SettingInput {
        id: parse::optional_uuid("settings.id", config.id.as_deref())?,
        name: config.name,
        field_type: SettingType::parse(&config.r#type)?,
        required: config.required.unwrap_or(false),
        // An unset or null default means "no default".
        default_value: config.default_value.into_option().map(buffa_value_to_json).filter(|v| !v.is_null()),
        library_id: parse::optional_uuid("settings.library_id", config.library_id.as_deref())?,
        options: config.options.into_iter().map(to_option).collect(),
    })
}

/// The `module_configs` / `modules` entries of a request as domain changes.
pub fn to_module_changes(configs: Vec<ModuleConfig>) -> Result<Vec<ModuleChange>, ApplicationError> {
    configs
        .into_iter()
        .map(|config| {
            Ok(ModuleChange {
                key: config.key,
                enabled: config.enabled,
                mode: config.mode.as_deref().map(ModuleMode::parse).transpose()?,
                settings: config
                    .settings
                    .into_option()
                    .map(|list| list.items.into_iter().map(to_setting).collect::<Result<Vec<_>, _>>())
                    .transpose()?,
            })
        })
        .collect()
}

pub fn to_grpc_program(program: Program) -> GrpcProgram {
    GrpcProgram {
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
        ..Default::default()
    }
}

fn to_grpc_list_row(item: ProgramListItem) -> GrpcProgram {
    let stats = item.stats;
    let mut program = to_grpc_program(item.program);
    program.stats = Some(ProgramStats {
        active_enrollments: stats.active_enrollments,
        total_enrollments: stats.total_enrollments,
        pending_approvals: stats.pending_approvals,
        domains: stats.domains.into_iter().map(|d| ProgramDomain { website_id: d.website_id, name: d.name, ..Default::default() }).collect(),
        last_activity: stats
            .last_activity
            .map(|a| ProgramLastActivity { summary: a.summary(), time: format_datetime(a.time), actor: a.actor, action: a.action, ..Default::default() })
            .into(),
        ..Default::default()
    }).into();
    program
}

/// The caller of a mapper that needs the actor passes it in separately, since
/// the `GrpcApiMapper` trait only sees the request message.
pub struct ListProgramsGrpcMapper;

impl GrpcApiMapper<ListProgramsOutput, ListProgramsInput, ListProgramsResponse, ListProgramsRequest>
    for ListProgramsGrpcMapper
{
    fn input(request: ListProgramsRequest) -> Result<ListProgramsInput, ApplicationError> {
        Ok(ListProgramsInput {
            page_size: request.max_page_size,
            page_token: parse::optional_uuid("page_token", request.page_token.as_deref())?,
            show_deleted: request.show_deleted.unwrap_or(false),
            show_archived: request.show_archived.unwrap_or(false),
        })
    }

    fn output_success(output: ListProgramsOutput) -> ListProgramsResponse {
        ListProgramsResponse {
            results: output.programs.into_iter().map(to_grpc_list_row).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            total_size: output.total_size,
            summary: Some(ProgramListSummary {
                live_programs: output.summary.live_programs,
                not_live_programs: output.summary.not_live_programs,
                active_enrollments: output.summary.active_enrollments,
                total_enrollments: output.summary.total_enrollments,
                pending_approvals: output.summary.pending_approvals,
                programs_with_pending_approvals: output.summary.programs_with_pending_approvals,
                ..Default::default()
            }).into(),
            ..Default::default()
        }
    }
}

pub struct GetProgramGrpcMapper;

impl GrpcApiMapper<GetProgramOutput, GetProgramInput, GetProgramResponse, GetProgramRequest>
    for GetProgramGrpcMapper
{
    fn input(request: GetProgramRequest) -> Result<GetProgramInput, ApplicationError> {
        Ok(GetProgramInput { identifier: ResourceIdentifier::parse(&request.program) })
    }

    fn output_success(output: GetProgramOutput) -> GetProgramResponse {
        GetProgramResponse { program: Some(to_grpc_program(output.program)).into(), ..Default::default() }
    }
}

pub struct CreateProgramGrpcMapper;

impl CreateProgramGrpcMapper {
    pub fn input(request: CreateProgramRequest, actor: String) -> Result<CreateProgramInput, ApplicationError> {
        Ok(CreateProgramInput {
            id: parse::optional_uuid("id", request.id.as_deref())?,
            name: request.name,
            slug: request.slug,
            description: request.description,
            level: request.level,
            modules: {
                // Plain keys first, then the richer entries (the same module in both is rejected).
                let mut modules: Vec<ModuleChange> = request.modules.into_iter().map(ModuleChange::plain).collect();
                modules.extend(to_module_changes(request.module_configs)?);
                modules
            },
            actor,
        })
    }

    pub fn output_success(output: CreateProgramOutput) -> CreateProgramResponse {
        let program_path = format!("programs/{}", output.program.id);
        CreateProgramResponse {
            program: Some(to_grpc_program(output.program)).into(),
            modules: output.modules.into_iter().map(|m| to_grpc_program_module(&program_path, m)).collect(),
            ..Default::default()
        }
    }
}

pub struct UpdateProgramGrpcMapper;

impl UpdateProgramGrpcMapper {
    pub fn input(request: UpdateProgramRequest, actor: String) -> Result<UpdateProgramInput, ApplicationError> {
        Ok(UpdateProgramInput {
            identifier: ResourceIdentifier::parse(&request.program),
            name: request.name,
            // gRPC has no null: an empty string clears the description.
            description: request.description.map(|d| if d.is_empty() { None } else { Some(d) }),
            level: request.level,
            active: request.active,
            modules: (!request.modules.is_empty()).then(|| to_module_changes(request.modules)).transpose()?,
            actor,
        })
    }

    pub fn output_success(output: UpdateProgramOutput) -> UpdateProgramResponse {
        let program_path = format!("programs/{}", output.program.id);
        UpdateProgramResponse {
            program: Some(to_grpc_program(output.program)).into(),
            // Only when the request touched modules.
            modules: output
                .modules
                .unwrap_or_default()
                .into_iter()
                .map(|m| to_grpc_program_module(&program_path, m))
                .collect(),
            ..Default::default()
        }
    }
}

pub struct DeleteProgramGrpcMapper;

impl DeleteProgramGrpcMapper {
    pub fn input(request: DeleteProgramRequest, actor: String) -> Result<DeleteProgramInput, ApplicationError> {
        Ok(DeleteProgramInput { identifier: ResourceIdentifier::parse(&request.program), actor })
    }
}

pub struct UndeleteProgramGrpcMapper;

impl UndeleteProgramGrpcMapper {
    pub fn input(request: UndeleteProgramRequest, actor: String) -> Result<UndeleteProgramInput, ApplicationError> {
        Ok(UndeleteProgramInput { identifier: ResourceIdentifier::parse(&request.program), actor })
    }

    pub fn output_success(output: UndeleteProgramOutput) -> UndeleteProgramResponse {
        UndeleteProgramResponse { program: Some(to_grpc_program(output.program)).into(), ..Default::default() }
    }
}
