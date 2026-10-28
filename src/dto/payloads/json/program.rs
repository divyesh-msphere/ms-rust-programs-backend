use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// Distinguishes "field absent" (`None`) from "field present and null"
/// (`Some(None)`), so PATCH can clear a nullable field.
fn present_or_null<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateProgramBodyPayload {
    pub name: String,
    /// Lowercase letters, digits and single hyphens. Immutable after create.
    pub slug: String,
    pub description: Option<String>,
    /// `user`, `organization` or `location` (default).
    pub level: Option<String>,
    /// The modules the admin picked: a plain key (`enrollment.stores.enrolled_stores_control_panel`) or an
    /// object with the key plus what the admin made of the leaf (`mode`, `settings`). Ancestors, the
    /// descendants of a picked group and Overview are added automatically.
    pub modules: Option<Vec<ModulePayload>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateProgramQueryPayload {
    /// Optional client-specified ID (AEP-133); server generates one if omitted.
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetProgramPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListProgramsPayload {
    pub max_page_size: Option<i64>,
    pub page_token: Option<String>,
    pub show_deleted: Option<bool>,
    /// Only archived programs; the default lists the working catalog.
    pub show_archived: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProgramPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct UpdateProgramBodyPayload {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "present_or_null")]
    pub description: Option<Option<String>>,
    pub level: Option<String>,
    pub active: Option<bool>,
    /// Module changes: a new key adds the module; `mode` sets what a leaf is; `settings` REPLACES that
    /// module's settings; `enabled: false` switches it off. Absent touches no module.
    pub modules: Option<Vec<ModulePayload>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeleteProgramPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UndeleteProgramPayload {
    pub program: String,
}

/// A module entry: a plain key, or an object. Written by hand so a malformed entry says what is
/// wrong ("unknown field `setting`") instead of a generic "did not match any variant".
#[derive(Debug, Clone)]
pub enum ModulePayload {
    Key(String),
    Config(ModuleConfigPayload),
}

impl<'de> Deserialize<'de> for ModulePayload {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::String(key) => Ok(ModulePayload::Key(key)),
            object @ Value::Object(_) => serde_json::from_value(object).map(ModulePayload::Config).map_err(D::Error::custom),
            _ => Err(D::Error::custom("a module must be a key (text) or an object with a `key`")),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ModuleConfigPayload {
    pub key: String,
    /// `false` switches the module off (its data is kept).
    pub enabled: Option<bool>,
    /// What the admin made this leaf: `rule` or `settings`.
    pub mode: Option<String>,
    /// For `mode: settings`. Present REPLACES the module's settings (items with an `id` are kept and
    /// updated, items without one are created, the rest are removed); absent leaves them alone.
    pub settings: Option<Vec<SettingPayload>>,
}

/// One setting. The read-only fields a manifest returns (`key`, `render`, `position`, ...) are
/// accepted and ignored, so a setting can be sent straight back after editing it.
#[derive(Debug, Deserialize, Clone)]
pub struct SettingPayload {
    /// Present for an existing setting.
    pub id: Option<String>,
    pub name: String,
    /// `boolean`, `number`, `text`, `radio`, `checkbox` or `dropdown`.
    #[serde(rename = "type")]
    pub field_type: String,
    pub required: Option<bool>,
    /// The starting value (a boolean, number, text, option code, or list of option codes).
    #[serde(default, rename = "default")]
    pub default_value: Option<Value>,
    /// A global-settings library: then it is the ONLY source of the options.
    pub library_id: Option<String>,
    /// The admin's own options (when there is no library).
    pub options: Option<Vec<OptionPayload>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OptionPayload {
    /// The option's identity; omitted means "derive it from the label".
    pub code: Option<String>,
    pub label: String,
}
