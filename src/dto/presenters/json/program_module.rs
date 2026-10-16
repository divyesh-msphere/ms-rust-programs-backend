use serde::Serialize;

/// One entry of a program's module manifest. `source` tells the frontend how
/// to draw it (`premade` → call `api_base`, `container` → render its
/// components, `default` → Overview).
#[derive(Debug, Serialize, Clone)]
pub struct ProgramModulePresenter {
    pub path: String,
    pub module_id: String,
    pub key: String,
    pub name: String,
    pub kind: String,
    pub parent_id: Option<String>,
    pub ancestry: String,
    pub source: String,
    pub handler_key: Option<String>,
    pub api_base: Option<String>,
    pub ui_component: Option<String>,
    pub allow_components: bool,
    pub requires_publish: bool,
    pub enabled: bool,
    /// `rule`, `settings`, or null until the admin chooses (always null when the
    /// module is not configurable).
    pub mode: Option<String>,
    /// The leaf's settings, when `mode` is `settings`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<Vec<ProgramSettingPresenter>>,
    /// The leaf's rule-set summary, when `mode` is `rule` (the frontend renders the rule engine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<ProgramRulePresenter>,
    pub position: i32,
    pub update_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ListProgramModulesPresenter {
    pub results: Vec<ProgramModulePresenter>,
}

/// One setting of a `settings` leaf, with what the frontend needs to draw it.
#[derive(Debug, Serialize, Clone)]
pub struct ProgramSettingPresenter {
    pub id: String,
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    /// The control to draw: `yes_no`, `number_input`, `text_input`, `radio_group`, `checkbox_group`
    /// or `dropdown`.
    pub render: String,
    pub required: bool,
    #[serde(rename = "default")]
    pub default_value: Option<serde_json::Value>,
    pub position: i32,
    /// `manual` or `library` for radio / checkbox / dropdown; null for the other types.
    pub options_source: Option<String>,
    pub library_id: Option<String>,
    /// Empty for boolean / number / text.
    pub options: Vec<SettingOptionPresenter>,
    /// `not_found` or `unavailable` when the library's options could not be read.
    pub library_error: Option<String>,
    /// The saved value of a boolean / number / text setting; null until one is saved.
    pub value: Option<serde_json::Value>,
    /// The saved choice of a radio / checkbox / dropdown setting.
    pub option_ids: Vec<String>,
    /// Saved options that are no longer on offer (deactivated, or gone from their library):
    /// warn the admin and let them pick again.
    pub missing_option_ids: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SettingOptionPresenter {
    pub id: String,
    pub code: String,
    pub label: String,
    /// `manual` or `library`.
    pub source: String,
    pub active: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProgramRulePresenter {
    pub rule_type: Option<String>,
    pub draft_version: Option<i32>,
    pub published_version: Option<i32>,
}
