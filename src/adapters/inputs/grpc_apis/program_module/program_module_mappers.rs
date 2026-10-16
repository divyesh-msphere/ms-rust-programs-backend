use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    ListProgramModulesRequest, ListProgramModulesResponse, ProgramModule as GrpcProgramModule,
    ProgramRule as GrpcProgramRule, ProgramSetting as GrpcProgramSetting, SettingOption as GrpcSettingOption,
};
use crate::adapters::inputs::grpc_apis::shared::json_struct::json_to_buffa_value;
use crate::application::errors::application_error::ApplicationError;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::program_module::list_program_modules::{
    ListProgramModulesInput, ListProgramModulesOutput,
};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program_module::ProgramModule;
use crate::domain::program_setting::ProgramSetting;

fn to_grpc_setting(setting: ProgramSetting) -> GrpcProgramSetting {
    GrpcProgramSetting {
        id: setting.id.to_string(),
        key: setting.key,
        name: setting.name,
        r#type: setting.field_type.as_str().to_string(),
        render: setting.field_type.render().to_string(),
        required: setting.required,
        default_value: setting.default_value.map(json_to_buffa_value).into(),
        position: setting.position,
        options_source: setting.options_source.map(|s| s.as_str().to_string()),
        library_id: setting.library_id.map(|id| id.to_string()),
        options: setting
            .options
            .into_iter()
            .map(|o| GrpcSettingOption {
                id: o.id.to_string(),
                code: o.code,
                label: o.label,
                source: o.source,
                active: o.active,
                ..Default::default()
            })
            .collect(),
        library_error: setting.library_error,
        value: setting.value.map(json_to_buffa_value).into(),
        option_ids: setting.option_ids.into_iter().map(|id| id.to_string()).collect(),
        missing_option_ids: setting.missing_option_ids.into_iter().map(|id| id.to_string()).collect(),
        ..Default::default()
    }
}

pub fn to_grpc_program_module(program_path: &str, module: ProgramModule) -> GrpcProgramModule {
    let is_settings = module.mode.as_deref() == Some("settings");
    GrpcProgramModule {
        path: format!("{program_path}/modules/{}", module.module_id),
        module_id: module.module_id.to_string(),
        key: module.key,
        name: module.name,
        kind: module.kind,
        parent_id: module.parent_id.map(|id| id.to_string()),
        ancestry: module.ancestry,
        source: module.source,
        handler_key: module.handler_key,
        api_base: module.api_base,
        ui_component: module.ui_component,
        allow_components: module.allow_components,
        requires_publish: module.requires_publish,
        enabled: module.enabled,
        mode: module.mode,
        settings: if is_settings { module.settings.into_iter().map(to_grpc_setting).collect() } else { Vec::new() },
        rule: module
            .rule
            .map(|r| GrpcProgramRule {
                rule_type: r.rule_type,
                draft_version: r.draft_version,
                published_version: r.published_version,
                ..Default::default()
            })
            .into(),
        position: module.position,
        update_time: format_datetime(module.update_time),
        ..Default::default()
    }
}

pub struct ListProgramModulesGrpcMapper;

impl
    GrpcApiMapper<
        ListProgramModulesOutput,
        ListProgramModulesInput,
        ListProgramModulesResponse,
        ListProgramModulesRequest,
    > for ListProgramModulesGrpcMapper
{
    fn input(request: ListProgramModulesRequest) -> Result<ListProgramModulesInput, ApplicationError> {
        Ok(ListProgramModulesInput {
            program: ResourceIdentifier::parse(&request.program),
            enabled_only: request.enabled_only.unwrap_or(true),
            include_options: request.include_options.unwrap_or(true),
        })
    }

    fn output_success(output: ListProgramModulesOutput) -> ListProgramModulesResponse {
        // The program path is not known to this mapper; the service impl
        // passes it through `output_with_path` instead.
        ListProgramModulesResponse {
            results: output.modules.into_iter().map(|m| to_grpc_program_module("programs", m)).collect(),
            ..Default::default()
        }
    }
}

impl ListProgramModulesGrpcMapper {
    pub fn output_with_path(program_path: &str, output: ListProgramModulesOutput) -> ListProgramModulesResponse {
        ListProgramModulesResponse {
            results: output.modules.into_iter().map(|m| to_grpc_program_module(program_path, m)).collect(),
            ..Default::default()
        }
    }
}
