// domain/program_field_value.rs
//
// The VALUES admins enter into the settings they defined. One value per setting (program
// level for now). A boolean / number / text setting stores a JSON `value`; a radio / checkbox /
// dropdown setting stores the ids of the chosen options. Everything that can be checked
// without the database or global-settings is checked here.
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashSet;
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;
use crate::domain::program_setting::{OptionsSource, SettingType};

pub const MAX_VALUES_PER_REQUEST: usize = 200;
pub const MAX_TEXT_LEN: usize = 10_000;

/// What a value is checked against: the setting as defined.
#[derive(Debug, Clone)]
pub struct SettingDefinition {
    pub id: Uuid,
    pub module_key: String,
    pub key: String,
    pub name: String,
    pub field_type: SettingType,
    pub required: bool,
    pub options_source: Option<OptionsSource>,
    pub library_id: Option<Uuid>,
    /// The admin's own ACTIVE options (empty for a library or a plain setting).
    pub manual_option_ids: HashSet<Uuid>,
}

/// What the caller sent for one setting.
#[derive(Debug, Clone, Default)]
pub struct ValueInput {
    pub value: Option<Value>,
    pub option_ids: Option<Vec<Uuid>>,
}

/// A validated change, ready to write.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueChange {
    Set { setting_id: Uuid, value: Option<Value>, option_ids: Vec<Uuid>, library_id: Option<Uuid> },
    Clear { setting_id: Uuid },
}

impl ValueChange {
    pub fn setting_id(&self) -> Uuid {
        match self {
            ValueChange::Set { setting_id, .. } | ValueChange::Clear { setting_id } => *setting_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoredValue {
    pub setting_id: Uuid,
    pub module_key: String,
    pub key: String,
    pub name: String,
    pub field_type: SettingType,
    pub value: Option<Value>,
    pub option_ids: Vec<Uuid>,
    pub update_time: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SaveOutcome {
    /// The program's `config_version` after the save.
    pub version: i32,
    /// How many values actually changed (a value sent unchanged is not a change).
    pub changed: usize,
}

/// Checks one value against its setting. `valid_options` is the set of option ids that may be
/// chosen: the setting's own active options, or the library's active options (the caller reads
/// the library); it is only consulted for radio / checkbox / dropdown.
///
/// Returns the change to make, or a message naming the setting and what is wrong.
pub fn validate_value(
    definition: &SettingDefinition,
    input: &ValueInput,
    valid_options: Option<&HashSet<Uuid>>,
) -> Result<ValueChange, String> {
    let name = &definition.name;
    let id = definition.id;
    let clear = |reason_if_required: &str| -> Result<ValueChange, String> {
        if definition.required {
            Err(format!("{name:?} is required: {reason_if_required}"))
        } else {
            Ok(ValueChange::Clear { setting_id: id })
        }
    };

    if definition.field_type.is_choice() {
        if input.value.as_ref().is_some_and(|v| !v.is_null()) {
            return Err(format!("{name:?} is a {}: send option_ids, not value", definition.field_type.as_str()));
        }
        let mut ids = input.option_ids.clone().unwrap_or_default();
        let mut seen = HashSet::new();
        ids.retain(|id| seen.insert(*id));

        if ids.is_empty() {
            return clear("choose at least one option");
        }
        if definition.field_type != SettingType::Checkbox && ids.len() > 1 {
            return Err(format!("{name:?} takes one option, not {}", ids.len()));
        }
        let allowed = valid_options.ok_or_else(|| format!("{name:?}: its options could not be read"))?;
        if let Some(bad) = ids.iter().find(|id| !allowed.contains(id)) {
            return Err(format!("{name:?}: option {bad} is not one of its active options"));
        }
        return Ok(ValueChange::Set { setting_id: id, value: None, option_ids: ids, library_id: definition.library_id });
    }

    if input.option_ids.as_ref().is_some_and(|ids| !ids.is_empty()) {
        return Err(format!("{name:?} is a {}: send value, not option_ids", definition.field_type.as_str()));
    }
    let Some(value) = input.value.clone().filter(|v| !v.is_null()) else {
        return clear("send a value");
    };

    let valid = match definition.field_type {
        SettingType::Boolean => value.is_boolean().then_some(()).ok_or("must be true or false"),
        SettingType::Number => value.is_number().then_some(()).ok_or("must be a number"),
        SettingType::Text => match value.as_str() {
            Some(text) if text.chars().count() <= MAX_TEXT_LEN => Ok(()),
            Some(_) => Err("is too long"),
            None => Err("must be text"),
        },
        _ => Ok(()),
    };
    valid.map_err(|why| format!("{name:?} {why}"))?;

    Ok(ValueChange::Set { setting_id: id, value: Some(value), option_ids: Vec::new(), library_id: None })
}

/// The saved options that are no longer offered.
pub fn missing_options(saved: &[Uuid], offered: &[Uuid]) -> Vec<Uuid> {
    let offered: HashSet<&Uuid> = offered.iter().collect();
    saved.iter().filter(|id| !offered.contains(id)).copied().collect()
}

pub fn too_many(count: usize) -> Result<(), ApplicationError> {
    if count > MAX_VALUES_PER_REQUEST {
        Err(ApplicationError::validation_error(&format!("at most {MAX_VALUES_PER_REQUEST} values per request")))
    } else {
        Ok(())
    }
}

#[async_trait]
pub trait ProgramFieldValueRepository: Send + Sync {
    /// The definitions of these settings, when they are live settings of this program.
    async fn definitions(&self, program_id: Uuid, setting_ids: &[Uuid]) -> Result<Vec<SettingDefinition>, ApplicationError>;

    /// The program's `config_version` and its saved values (optionally one module's).
    async fn list(&self, program_id: Uuid, module_key: Option<&str>) -> Result<(i32, Vec<StoredValue>), ApplicationError>;

    /// ONE transaction: lock the program, check `expected_version` (a stale one is a conflict),
    /// write only what differs, bump `config_version`, and record the audit entry and outbox event.
    /// Nothing is written, and the version is not bumped, when nothing differs.
    async fn save(
        &self,
        program_id: Uuid,
        expected_version: Option<i32>,
        changes: Vec<ValueChange>,
        actor: &str,
    ) -> Result<SaveOutcome, ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn def(field_type: SettingType, required: bool) -> SettingDefinition {
        SettingDefinition {
            id: Uuid::new_v4(),
            module_key: "m".into(),
            key: "k".into(),
            name: "Setting".into(),
            field_type,
            required,
            options_source: field_type.is_choice().then_some(OptionsSource::Manual),
            library_id: None,
            manual_option_ids: HashSet::new(),
        }
    }

    fn value(v: Value) -> ValueInput {
        ValueInput { value: Some(v), option_ids: None }
    }

    fn options(ids: &[Uuid]) -> ValueInput {
        ValueInput { value: None, option_ids: Some(ids.to_vec()) }
    }

    #[test]
    fn plain_values_must_match_their_type() {
        let d = def(SettingType::Boolean, false);
        assert!(validate_value(&d, &value(json!(true)), None).is_ok());
        assert!(validate_value(&d, &value(json!("true")), None).is_err());
        assert!(validate_value(&d, &value(json!(1)), None).is_err());

        let d = def(SettingType::Number, false);
        assert!(validate_value(&d, &value(json!(90)), None).is_ok());
        assert!(validate_value(&d, &value(json!(12.5)), None).is_ok());
        assert!(validate_value(&d, &value(json!("90")), None).is_err());

        let d = def(SettingType::Text, false);
        assert!(validate_value(&d, &value(json!("hello")), None).is_ok());
        assert!(validate_value(&d, &value(json!(5)), None).is_err());
        assert!(validate_value(&d, &value(json!("x".repeat(MAX_TEXT_LEN + 1))), None).is_err());
    }

    #[test]
    fn plain_values_take_no_option_ids_and_choices_take_no_value() {
        let d = def(SettingType::Text, false);
        let input = ValueInput { value: Some(json!("x")), option_ids: Some(vec![Uuid::new_v4()]) };
        assert!(validate_value(&d, &input, None).is_err());

        let d = def(SettingType::Radio, false);
        let a = Uuid::new_v4();
        let allowed: HashSet<Uuid> = [a].into_iter().collect();
        let input = ValueInput { value: Some(json!("a")), option_ids: Some(vec![a]) };
        assert!(validate_value(&d, &input, Some(&allowed)).is_err());
    }

    #[test]
    fn nothing_or_null_clears_unless_required() {
        for t in [SettingType::Boolean, SettingType::Text, SettingType::Radio, SettingType::Checkbox] {
            let d = def(t, false);
            assert!(matches!(validate_value(&d, &ValueInput::default(), None), Ok(ValueChange::Clear { .. })), "{t:?}");
            assert!(matches!(validate_value(&d, &value(Value::Null), None), Ok(ValueChange::Clear { .. })), "{t:?}");
            let d = def(t, true);
            assert!(validate_value(&d, &ValueInput::default(), None).is_err(), "{t:?} is required");
        }
        let d = def(SettingType::Checkbox, true);
        assert!(validate_value(&d, &options(&[]), None).is_err(), "an empty list does not satisfy a required checkbox");
    }

    #[test]
    fn choices_must_be_offered_options() {
        let (a, b, stranger) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let allowed: HashSet<Uuid> = [a, b].into_iter().collect();

        let radio = def(SettingType::Radio, false);
        assert_eq!(
            validate_value(&radio, &options(&[a]), Some(&allowed)),
            Ok(ValueChange::Set { setting_id: radio.id, value: None, option_ids: vec![a], library_id: None })
        );
        assert!(validate_value(&radio, &options(&[stranger]), Some(&allowed)).is_err());
        assert!(validate_value(&radio, &options(&[a, b]), Some(&allowed)).is_err(), "radio takes one");
        assert!(validate_value(&def(SettingType::Dropdown, false), &options(&[a, b]), Some(&allowed)).is_err());

        let checkbox = def(SettingType::Checkbox, false);
        assert!(validate_value(&checkbox, &options(&[a, b]), Some(&allowed)).is_ok());
        assert!(validate_value(&checkbox, &options(&[a, stranger]), Some(&allowed)).is_err());
        let deduped = validate_value(&checkbox, &options(&[a, a, b]), Some(&allowed)).unwrap();
        assert_eq!(deduped, ValueChange::Set { setting_id: checkbox.id, value: None, option_ids: vec![a, b], library_id: None });

        assert!(validate_value(&radio, &options(&[a]), None).is_err(), "options that could not be read cannot be checked");
    }

    #[test]
    fn a_library_choice_remembers_its_library() {
        let lib = Uuid::new_v4();
        let a = Uuid::new_v4();
        let mut d = def(SettingType::Dropdown, false);
        d.options_source = Some(OptionsSource::Library);
        d.library_id = Some(lib);
        let allowed: HashSet<Uuid> = [a].into_iter().collect();
        assert_eq!(
            validate_value(&d, &options(&[a]), Some(&allowed)),
            Ok(ValueChange::Set { setting_id: d.id, value: None, option_ids: vec![a], library_id: Some(lib) })
        );
    }

    #[test]
    fn finds_saved_options_that_are_no_longer_offered() {
        let (a, b, c) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(missing_options(&[a, b, c], &[a, c]), vec![b]);
        assert!(missing_options(&[a], &[a, b]).is_empty());
        assert!(missing_options(&[], &[a]).is_empty());
    }

    #[test]
    fn caps_the_batch() {
        assert!(too_many(MAX_VALUES_PER_REQUEST).is_ok());
        assert!(too_many(MAX_VALUES_PER_REQUEST + 1).is_err());
    }
}
