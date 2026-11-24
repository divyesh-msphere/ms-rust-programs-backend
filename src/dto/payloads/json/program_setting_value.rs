use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct ListSettingValuesPathPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListSettingValuesQueryPayload {
    /// Only one module's values (its key, e.g. `catalog_pricing.products_rules.warranty`).
    pub module: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateSettingValuesPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateSettingValuesBodyPayload {
    /// The program's `config_version` as the caller last read it. A stale one is rejected with 409
    /// so two admins cannot overwrite each other; omit it to skip the check.
    pub version: Option<i32>,
    /// The changed values, keyed by setting id. A `null` entry (or an entry with neither field)
    /// clears that setting's value.
    pub values: HashMap<String, Option<SettingValuePayload>>,
}

/// One setting's new value: `value` for a boolean / number / text setting, `option_ids` for a
/// radio / checkbox / dropdown setting.
#[derive(Debug, Deserialize, Clone)]
pub struct SettingValuePayload {
    pub value: Option<Value>,
    pub option_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateModuleSettingsBodyPayload {
    /// The program's `config_version` as the caller last read it (409 when stale); omit to skip the check.
    pub version: Option<i32>,
    /// Settings of one or more modules, saved together in ONE transaction.
    pub modules: Vec<ModuleSettingsPayload>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModuleSettingsPayload {
    /// The module's key, e.g. `catalog_pricing.pricing.discount_rules`.
    pub key: String,
    pub settings: Vec<ModuleSettingPayload>,
}

/// One setting: named by `id` or by its `key` in the module (exactly one), with `value` for a
/// boolean / number / text / percentage setting or `option_ids` for a choice. Neither clears it.
#[derive(Debug, Deserialize, Clone)]
pub struct ModuleSettingPayload {
    pub id: Option<String>,
    pub key: Option<String>,
    pub value: Option<Value>,
    pub option_ids: Option<Vec<String>>,
}
