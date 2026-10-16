// domain/program_setting.rs
//
// What the admin builds on a configurable leaf: a MODE (`rule` or `settings`) and, for
// `settings`, a list of typed settings. The leaf's own name is fixed by the module, so only
// the settings carry names.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;

pub const MAX_SETTINGS_PER_MODULE: usize = 50;
pub const MAX_OPTIONS_PER_SETTING: usize = 100;
pub const MAX_NAME_LEN: usize = 80;
pub const MAX_LABEL_LEN: usize = 120;
pub const MAX_CODE_LEN: usize = 64;
pub const MAX_MODULE_CHANGES: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleMode {
    Rule,
    Settings,
}

impl ModuleMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModuleMode::Rule => "rule",
            ModuleMode::Settings => "settings",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApplicationError> {
        match value {
            "rule" => Ok(ModuleMode::Rule),
            "settings" => Ok(ModuleMode::Settings),
            other => Err(ApplicationError::unprocessable_entity(&format!(
                "mode {other:?} must be one of: rule, settings"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingType {
    Boolean,
    Number,
    Text,
    Radio,
    Checkbox,
    Dropdown,
}

impl SettingType {
    pub const ALL: [&'static str; 6] = ["boolean", "number", "text", "radio", "checkbox", "dropdown"];

    pub fn as_str(&self) -> &'static str {
        match self {
            SettingType::Boolean => "boolean",
            SettingType::Number => "number",
            SettingType::Text => "text",
            SettingType::Radio => "radio",
            SettingType::Checkbox => "checkbox",
            SettingType::Dropdown => "dropdown",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ApplicationError> {
        match value {
            "boolean" => Ok(SettingType::Boolean),
            "number" => Ok(SettingType::Number),
            "text" => Ok(SettingType::Text),
            "radio" => Ok(SettingType::Radio),
            "checkbox" => Ok(SettingType::Checkbox),
            "dropdown" => Ok(SettingType::Dropdown),
            other => Err(ApplicationError::unprocessable_entity(&format!(
                "type {other:?} must be one of: {}",
                Self::ALL.join(", ")
            ))),
        }
    }

    /// Radio, checkbox and dropdown pick from a list of options.
    pub fn is_choice(&self) -> bool {
        matches!(self, SettingType::Radio | SettingType::Checkbox | SettingType::Dropdown)
    }

    /// The control the frontend draws: a Yes/No switch for a boolean, a dropdown for a
    /// dropdown, and so on.
    pub fn render(&self) -> &'static str {
        match self {
            SettingType::Boolean => "yes_no",
            SettingType::Number => "number_input",
            SettingType::Text => "text_input",
            SettingType::Radio => "radio_group",
            SettingType::Checkbox => "checkbox_group",
            SettingType::Dropdown => "dropdown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionsSource {
    Manual,
    Library,
}

impl OptionsSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            OptionsSource::Manual => "manual",
            OptionsSource::Library => "library",
        }
    }

    pub fn parse(value: &str) -> Self {
        // The app is the only writer of this column.
        if value == "library" { OptionsSource::Library } else { OptionsSource::Manual }
    }
}

// ---------------------------------------------------------------------------------------
// Input (what a create / update request carries)
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct OptionInput {
    /// The identity of the option. Omitted means "derive it from the label".
    pub code: Option<String>,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct SettingInput {
    /// Present for an existing setting (kept and updated); absent for a new one.
    pub id: Option<Uuid>,
    pub name: String,
    pub field_type: SettingType,
    pub required: bool,
    pub default_value: Option<Value>,
    /// The global-settings library that is the ONLY source of the options.
    pub library_id: Option<Uuid>,
    /// The admin's own values (the ONLY source when there is no library).
    pub options: Vec<OptionInput>,
}

#[derive(Debug, Clone)]
pub struct ModuleChange {
    pub key: String,
    /// `Some(false)` switches the module off (its data is kept).
    pub enabled: Option<bool>,
    pub mode: Option<ModuleMode>,
    /// `Some` REPLACES the module's settings: items with an `id` are kept and updated,
    /// items without one are created, and the rest are removed. `None` leaves them alone.
    pub settings: Option<Vec<SettingInput>>,
}

impl ModuleChange {
    pub fn plain(key: String) -> Self {
        Self { key, enabled: None, mode: None, settings: None }
    }

    /// True when this entry does more than name a module to include.
    pub fn has_changes(&self) -> bool {
        self.enabled.is_some() || self.mode.is_some() || self.settings.is_some()
    }
}

// ---------------------------------------------------------------------------------------
// Output (what a manifest carries)
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingOption {
    /// The option's id: ours for a manual option, the library's for a library option.
    pub id: Uuid,
    pub code: String,
    pub label: String,
    /// `manual` or `library`.
    pub source: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramSetting {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub field_type: SettingType,
    pub required: bool,
    pub default_value: Option<Value>,
    pub position: i32,
    pub options_source: Option<OptionsSource>,
    pub library_id: Option<Uuid>,
    pub options: Vec<SettingOption>,
    /// Set when the library's options could not be read: `not_found` or `unavailable`.
    pub library_error: Option<String>,
    /// The saved value of a boolean / number / text setting (`None` until one is saved).
    pub value: Option<Value>,
    /// The saved choice of a radio / checkbox / dropdown setting (option ids).
    pub option_ids: Vec<Uuid>,
    /// Saved options that are no longer on offer (deactivated, or gone from the library).
    pub missing_option_ids: Vec<Uuid>,
}

/// The rule engine is a front-end component; this is only the leaf's rule-set summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramRule {
    pub rule_type: Option<String>,
    pub draft_version: Option<i32>,
    pub published_version: Option<i32>,
}

// ---------------------------------------------------------------------------------------
// Keys and codes
// ---------------------------------------------------------------------------------------

fn slugify(text: &str, max_len: usize) -> String {
    let mut out = String::new();
    let mut last_underscore = true;
    for c in text.trim().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_underscore = false;
        } else if !last_underscore {
            out.push('_');
            last_underscore = true;
        }
    }
    let trimmed = out.trim_end_matches('_');
    trimmed.chars().take(max_len).collect::<String>().trim_end_matches('_').to_string()
}

/// A setting's key, generated from its name (so nobody types an identifier).
pub fn key_from_name(name: &str) -> String {
    slugify(name, MAX_CODE_LEN)
}

/// An option's code, from its explicit `code` or else its label.
pub fn code_for_option(option: &OptionInput) -> String {
    match option.code.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
        Some(code) => code.to_string(),
        None => slugify(&option.label, MAX_CODE_LEN),
    }
}

fn valid_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= MAX_CODE_LEN
        && code.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

// ---------------------------------------------------------------------------------------
// Validation (pure: no database, no network)
// ---------------------------------------------------------------------------------------

fn unprocessable(message: String) -> ApplicationError {
    ApplicationError::unprocessable_entity(&message)
}

fn check_default(setting: &SettingInput, codes: &HashSet<String>) -> Result<(), ApplicationError> {
    let Some(default) = &setting.default_value else { return Ok(()) };
    if default.is_null() {
        return Ok(());
    }
    let name = &setting.name;
    let manual_choice = setting.field_type.is_choice() && setting.library_id.is_none();

    match setting.field_type {
        SettingType::Boolean if !default.is_boolean() => {
            Err(unprocessable(format!("setting {name:?}: default must be true or false")))
        }
        SettingType::Number if !default.is_number() => {
            Err(unprocessable(format!("setting {name:?}: default must be a number")))
        }
        SettingType::Text if !default.is_string() => {
            Err(unprocessable(format!("setting {name:?}: default must be text")))
        }
        SettingType::Radio | SettingType::Dropdown => match default.as_str() {
            Some(code) if !manual_choice || codes.contains(code) => Ok(()),
            Some(code) => Err(unprocessable(format!("setting {name:?}: default {code:?} is not one of its options"))),
            None => Err(unprocessable(format!("setting {name:?}: default must be one option code"))),
        },
        SettingType::Checkbox => match default.as_array() {
            Some(items) => {
                for item in items {
                    match item.as_str() {
                        Some(code) if !manual_choice || codes.contains(code) => {}
                        _ => {
                            return Err(unprocessable(format!(
                                "setting {name:?}: default must be a list of its option codes"
                            )));
                        }
                    }
                }
                Ok(())
            }
            None => Err(unprocessable(format!("setting {name:?}: default must be a list of option codes"))),
        },
        _ => Ok(()),
    }
}

fn validate_setting(setting: &SettingInput) -> Result<(), ApplicationError> {
    let name = setting.name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_LEN {
        return Err(unprocessable(format!("a setting name must be 1-{MAX_NAME_LEN} characters")));
    }
    if key_from_name(name).is_empty() {
        return Err(unprocessable(format!("setting name {name:?} must contain a letter or digit")));
    }

    let mut codes = HashSet::new();

    if setting.field_type.is_choice() {
        let has_library = setting.library_id.is_some();
        let has_options = !setting.options.is_empty();
        if has_library && has_options {
            return Err(unprocessable(format!(
                "setting {name:?}: use a library OR your own options, not both (the library is then the only source)"
            )));
        }
        if !has_library && !has_options {
            return Err(unprocessable(format!(
                "setting {name:?}: a {} needs a library_id or at least one option",
                setting.field_type.as_str()
            )));
        }
        if setting.options.len() > MAX_OPTIONS_PER_SETTING {
            return Err(unprocessable(format!(
                "setting {name:?}: at most {MAX_OPTIONS_PER_SETTING} options"
            )));
        }
        for option in &setting.options {
            let label = option.label.trim();
            if label.is_empty() || label.chars().count() > MAX_LABEL_LEN {
                return Err(unprocessable(format!(
                    "setting {name:?}: an option label must be 1-{MAX_LABEL_LEN} characters"
                )));
            }
            let code = code_for_option(option);
            if !valid_code(&code) {
                return Err(unprocessable(format!(
                    "setting {name:?}: option {label:?} needs a code of letters, digits, '.', '_' or '-' (up to {MAX_CODE_LEN})"
                )));
            }
            if !codes.insert(code.clone()) {
                return Err(unprocessable(format!("setting {name:?}: two options share the code {code:?}")));
            }
        }
    } else if setting.library_id.is_some() || !setting.options.is_empty() {
        return Err(unprocessable(format!(
            "setting {name:?}: a {} takes no options or library",
            setting.field_type.as_str()
        )));
    }

    check_default(setting, &codes)
}

/// Checks everything that can be checked without the database or global-settings.
pub fn validate_changes(changes: &[ModuleChange]) -> Result<(), ApplicationError> {
    if changes.len() > MAX_MODULE_CHANGES {
        return Err(ApplicationError::validation_error(&format!("at most {MAX_MODULE_CHANGES} modules")));
    }

    let mut seen_modules = HashSet::new();
    for change in changes {
        if change.key.trim().is_empty() {
            return Err(unprocessable("a module entry needs a key".to_string()));
        }
        if !seen_modules.insert(change.key.as_str()) {
            return Err(unprocessable(format!("module {:?} is listed twice", change.key)));
        }

        let Some(settings) = &change.settings else { continue };
        if change.mode == Some(ModuleMode::Rule) {
            return Err(unprocessable(format!(
                "module {:?}: a rule module has no settings (mode is rule)",
                change.key
            )));
        }
        if settings.len() > MAX_SETTINGS_PER_MODULE {
            return Err(unprocessable(format!(
                "module {:?}: at most {MAX_SETTINGS_PER_MODULE} settings",
                change.key
            )));
        }

        let mut keys = HashSet::new();
        let mut ids = HashSet::new();
        for setting in settings {
            validate_setting(setting)?;
            if let Some(id) = setting.id
                && !ids.insert(id)
            {
                return Err(unprocessable(format!("module {:?}: setting id {id} is listed twice", change.key)));
            }
            let key = key_from_name(&setting.name);
            if !keys.insert(key.clone()) {
                return Err(unprocessable(format!(
                    "module {:?}: two settings are named alike ({:?})",
                    change.key, setting.name.trim()
                )));
            }
        }
    }
    Ok(())
}

/// The distinct global-settings libraries a set of changes refers to.
pub fn library_ids(changes: &[ModuleChange]) -> Vec<Uuid> {
    let mut ids: Vec<Uuid> = changes
        .iter()
        .filter_map(|change| change.settings.as_ref())
        .flatten()
        .filter_map(|setting| setting.library_id)
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn setting(name: &str, field_type: SettingType) -> SettingInput {
        SettingInput {
            id: None,
            name: name.to_string(),
            field_type,
            required: false,
            default_value: None,
            library_id: None,
            options: vec![],
        }
    }

    fn option(label: &str) -> OptionInput {
        OptionInput { code: None, label: label.to_string() }
    }

    fn module(settings: Vec<SettingInput>) -> Vec<ModuleChange> {
        vec![ModuleChange { key: "m".into(), enabled: None, mode: Some(ModuleMode::Settings), settings: Some(settings) }]
    }

    #[test]
    fn keys_and_codes_are_generated() {
        assert_eq!(key_from_name("  Max days!  "), "max_days");
        assert_eq!(key_from_name("Warranty enabled"), "warranty_enabled");
        assert_eq!(key_from_name("!!!"), "");
        assert_eq!(code_for_option(&option("Grade A+")), "grade_a");
        assert_eq!(code_for_option(&OptionInput { code: Some(" a-1 ".into()), label: "x".into() }), "a-1");
    }

    #[test]
    fn each_type_has_a_render_hint() {
        let rendered: Vec<&str> = SettingType::ALL.iter().map(|t| SettingType::parse(t).unwrap().render()).collect();
        assert_eq!(rendered, ["yes_no", "number_input", "text_input", "radio_group", "checkbox_group", "dropdown"]);
        assert!(SettingType::parse("slider").is_err());
    }

    #[test]
    fn plain_types_take_no_options() {
        let mut s = setting("Enabled", SettingType::Boolean);
        s.options = vec![option("Yes")];
        assert!(validate_changes(&module(vec![s])).is_err());

        let mut s = setting("Enabled", SettingType::Text);
        s.library_id = Some(Uuid::new_v4());
        assert!(validate_changes(&module(vec![s])).is_err());

        assert!(validate_changes(&module(vec![setting("Enabled", SettingType::Boolean)])).is_ok());
    }

    #[test]
    fn choice_types_need_exactly_one_source() {
        let neither = setting("Grade", SettingType::Radio);
        assert!(validate_changes(&module(vec![neither])).is_err());

        let mut both = setting("Grade", SettingType::Dropdown);
        both.library_id = Some(Uuid::new_v4());
        both.options = vec![option("A")];
        assert!(validate_changes(&module(vec![both])).is_err(), "library and manual options must not mix");

        let mut manual = setting("Grade", SettingType::Dropdown);
        manual.options = vec![option("A"), option("B")];
        assert!(validate_changes(&module(vec![manual])).is_ok());

        let mut library = setting("Grade", SettingType::Checkbox);
        library.library_id = Some(Uuid::new_v4());
        assert!(validate_changes(&module(vec![library])).is_ok());
    }

    #[test]
    fn duplicate_names_options_and_ids_are_rejected() {
        assert!(validate_changes(&module(vec![
            setting("Max days", SettingType::Number),
            setting("max  days", SettingType::Text)
        ]))
        .is_err());

        let mut dup = setting("Grade", SettingType::Radio);
        dup.options = vec![option("Grade A"), option("grade-a")];
        assert!(validate_changes(&module(vec![dup])).is_err());

        let id = Uuid::new_v4();
        let mut a = setting("A", SettingType::Text);
        a.id = Some(id);
        let mut b = setting("B", SettingType::Text);
        b.id = Some(id);
        assert!(validate_changes(&module(vec![a, b])).is_err());
    }

    #[test]
    fn defaults_must_match_the_type() {
        let mut s = setting("On", SettingType::Boolean);
        s.default_value = Some(json!("yes"));
        assert!(validate_changes(&module(vec![s])).is_err());

        let mut s = setting("On", SettingType::Boolean);
        s.default_value = Some(json!(true));
        assert!(validate_changes(&module(vec![s])).is_ok());

        let mut s = setting("Days", SettingType::Number);
        s.default_value = Some(json!(30));
        assert!(validate_changes(&module(vec![s])).is_ok());

        let mut s = setting("Grade", SettingType::Radio);
        s.options = vec![option("A"), option("B")];
        s.default_value = Some(json!("c"));
        assert!(validate_changes(&module(vec![s.clone()])).is_err(), "default must be one of the options");
        s.default_value = Some(json!("a"));
        assert!(validate_changes(&module(vec![s])).is_ok());

        let mut s = setting("Tags", SettingType::Checkbox);
        s.options = vec![option("A"), option("B")];
        s.default_value = Some(json!(["a", "b"]));
        assert!(validate_changes(&module(vec![s])).is_ok());
    }

    #[test]
    fn rule_modules_cannot_carry_settings_and_modules_are_unique() {
        let rule_with_settings = vec![ModuleChange {
            key: "m".into(), enabled: None, mode: Some(ModuleMode::Rule), settings: Some(vec![]),
        }];
        assert!(validate_changes(&rule_with_settings).is_err());

        let twice = vec![ModuleChange::plain("m".into()), ModuleChange::plain("m".into())];
        assert!(validate_changes(&twice).is_err());
        assert!(validate_changes(&[ModuleChange::plain(" ".into())]).is_err());
    }

    #[test]
    fn limits_are_enforced() {
        let many: Vec<SettingInput> = (0..=MAX_SETTINGS_PER_MODULE).map(|i| setting(&format!("S{i}"), SettingType::Text)).collect();
        assert!(validate_changes(&module(many)).is_err());

        let mut s = setting("Big", SettingType::Dropdown);
        s.options = (0..=MAX_OPTIONS_PER_SETTING).map(|i| option(&format!("Option {i}"))).collect();
        assert!(validate_changes(&module(vec![s])).is_err());
        assert!(validate_changes(&module(vec![setting(&"x".repeat(81), SettingType::Text)])).is_err());
    }

    #[test]
    fn collects_distinct_library_ids() {
        let lib = Uuid::new_v4();
        let mut a = setting("A", SettingType::Dropdown);
        a.library_id = Some(lib);
        let mut b = setting("B", SettingType::Radio);
        b.library_id = Some(lib);
        assert_eq!(library_ids(&module(vec![a, b, setting("C", SettingType::Text)])), vec![lib]);
    }
}
