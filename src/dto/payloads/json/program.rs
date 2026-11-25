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
    /// The merchant-facing text (`description` is the internal one).
    pub about: Option<String>,
    /// Up to 5 short selling points shown next to the summary.
    pub highlights: Option<Vec<String>>,
    /// The program's domains, e.g. `["domain1", "domain2"]`.
    pub domain: Option<Vec<String>>,
    pub branding: Option<BrandingPayload>,
    /// Omitted parts stay null (term, grace, reminder) or false (auto_renew).
    pub renewal: Option<RenewalPayload>,
    /// Whether the program renews at all (false when omitted).
    pub renewal_enabled: Option<bool>,
    /// `user`, `organization` or `location` (default).
    pub level: Option<String>,
    /// The modules the admin picked: a plain key (`enrollment.stores.enrolled_stores_control_panel`) or an
    /// object with the key plus what the admin made of the leaf (`mode`, `settings`). Ancestors, the
    /// descendants of a picked group and Overview are added automatically.
    pub modules: Option<Vec<ModulePayload>>,
}

/// The badge. On update a sent part replaces that part (an empty text clears it).
#[derive(Debug, Deserialize, Clone, Default)]
pub struct BrandingPayload {
    /// `#RRGGBB`, the badge background.
    pub brand_color: Option<String>,
    /// `#RRGGBB`, the text on the badge.
    pub text_color: Option<String>,
    /// An icon name, e.g. `credit_card`.
    pub logo_icon: Option<String>,
}

/// The renewal defaults. On update a sent part replaces that part.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct RenewalPayload {
    pub term_months: Option<i32>,
    pub grace_period_days: Option<i32>,
    /// Days before the term ends.
    pub reminder_days: Option<i32>,
    pub auto_renew: Option<bool>,
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
    /// The search box: `program=abc` (the name or slug contains it).
    pub search: Option<String>,
    /// Operators and lists, e.g. `program.not_contains=abc,domain=[w1,w2],status=active`. Text
    /// operators: contains (default), not_contains, equals, not_equals, starts_with, ends_with,
    /// is_empty, is_not_empty; keys: program, description, domain, status, level.
    pub filter: Option<String>,
    /// `name`, `name asc` or `name desc` (case-insensitive); omitted keeps creation order.
    pub order_by: Option<String>,
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
    /// `null` clears the merchant-facing text.
    #[serde(default, deserialize_with = "present_or_null")]
    pub about: Option<Option<String>>,
    /// Replaces the whole list.
    pub highlights: Option<Vec<String>>,
    /// Replaces the whole domain list (an empty list clears it).
    pub domain: Option<Vec<String>>,
    pub branding: Option<BrandingPayload>,
    pub renewal: Option<RenewalPayload>,
    pub renewal_enabled: Option<bool>,
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
    /// The icon drawn beside this module (a name such as `tag`, `shield`). An empty text clears it, and
    /// the catalog module's icon shows again. Absent leaves it alone.
    pub icon: Option<String>,
    /// The rule's Display switch, for a rule module: `{"display": false}` hides the rule from the program's
    /// pages. Absent leaves it alone.
    pub rule: Option<RuleConfigPayload>,
    /// For `mode: settings`. Present REPLACES the module's settings (items with an `id` are kept and
    /// updated, items without one are created, the rest are removed); absent leaves them alone.
    pub settings: Option<Vec<SettingPayload>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RuleConfigPayload {
    pub display: Option<bool>,
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
    /// The Display switch (default true): false hides the setting from the form.
    pub display: Option<bool>,
    /// More than one option may be picked. Only a `dropdown` is free to choose (default false); a
    /// checkbox is always true, a radio and custom_option_single always false.
    pub multiple: Option<bool>,
    /// The starting value (a boolean, number, text, option code, or list of option codes).
    #[serde(default, rename = "default")]
    pub default_value: Option<Value>,
    /// A global-settings library: then it is the ONLY source of the options.
    pub library_id: Option<String>,
    /// The admin's own options (when there is no library).
    pub options: Option<Vec<OptionPayload>>,
    /// The custom choices of a radio / checkbox / dropdown, the way the form collects them: a text
    /// `"Weekly, Monthly"` (comma-separated), or a list of labels `["Weekly","Monthly"]`, or a list of
    /// `{label, code?}`. The same as `options`; send one of the two, not both, and not with a `library_id`.
    pub custom_options: Option<Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OptionPayload {
    /// The option's identity; omitted means "derive it from the label".
    pub code: Option<String>,
    pub label: String,
}
