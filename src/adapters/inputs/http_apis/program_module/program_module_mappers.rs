use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::program_module::list_program_modules::{
    ListProgramModulesInput, ListProgramModulesOutput,
};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program_module::ProgramModule;
use crate::dto::payloads::json::program_module::{ListProgramModulesPathPayload, ListProgramModulesQueryPayload};
use crate::domain::program_setting::ProgramSetting;
use crate::dto::presenters::json::program_module::{
    ListProgramModulesPresenter, ProgramModulePresenter, ProgramRulePresenter, ProgramSettingPresenter,
    SettingOptionPresenter,
};

fn setting_to_presenter(setting: ProgramSetting) -> ProgramSettingPresenter {
    ProgramSettingPresenter {
        id: setting.id.to_string(),
        key: setting.key,
        name: setting.name,
        field_type: setting.field_type.as_str().to_string(),
        render: setting.field_type.render().to_string(),
        required: setting.required,
        default_value: setting.default_value,
        position: setting.position,
        options_source: setting.options_source.map(|s| s.as_str().to_string()),
        library_id: setting.library_id.map(|id| id.to_string()),
        options: setting
            .options
            .into_iter()
            .map(|o| SettingOptionPresenter {
                id: o.id.to_string(),
                code: o.code,
                label: o.label,
                source: o.source,
                active: o.active,
            })
            .collect(),
        library_error: setting.library_error,
        value: setting.value,
        option_ids: setting.option_ids.into_iter().map(|id| id.to_string()).collect(),
        missing_option_ids: setting.missing_option_ids.into_iter().map(|id| id.to_string()).collect(),
    }
}

pub fn to_presenter(program_path: &str, module: ProgramModule) -> ProgramModulePresenter {
    let is_settings = module.mode.as_deref() == Some("settings");
    ProgramModulePresenter {
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
        settings: is_settings.then(|| module.settings.into_iter().map(setting_to_presenter).collect()),
        rule: module.rule.map(|r| ProgramRulePresenter {
            rule_type: r.rule_type,
            draft_version: r.draft_version,
            published_version: r.published_version,
        }),
        position: module.position,
        update_time: format_datetime(module.update_time),
    }
}

pub struct ListProgramModulesMapper;

impl ListProgramModulesMapper {
    pub fn input(
        path: ListProgramModulesPathPayload,
        query: ListProgramModulesQueryPayload,
    ) -> Result<ListProgramModulesInput, ApplicationError> {
        Ok(ListProgramModulesInput {
            program: ResourceIdentifier::parse(&path.program),
            enabled_only: query.enabled_only.unwrap_or(true),
            include_options: query.include_options.unwrap_or(true),
        })
    }

    pub fn output_success(program_path: &str, output: ListProgramModulesOutput) -> ListProgramModulesPresenter {
        ListProgramModulesPresenter {
            results: output.modules.into_iter().map(|m| to_presenter(program_path, m)).collect(),
        }
    }
}
