use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct ListFieldValuesPathPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListFieldValuesQueryPayload {
    /// Only one module's values (its key, e.g. `catalog_pricing.products_rules.warranty`).
    pub module: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateFieldValuesPathPayload {
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateFieldValuesBodyPayload {
    /// The program's `config_version` as the caller last read it. A stale one is rejected with 409
    /// so two admins cannot overwrite each other; omit it to skip the check.
    pub version: Option<i32>,
    /// The changed values, keyed by setting id. A `null` entry (or an entry with neither field)
    /// clears that setting's value.
    pub values: HashMap<String, Option<FieldValuePayload>>,
}

/// One setting's new value: `value` for a boolean / number / text setting, `option_ids` for a
/// radio / checkbox / dropdown setting.
#[derive(Debug, Deserialize, Clone)]
pub struct FieldValuePayload {
    pub value: Option<Value>,
    pub option_ids: Option<Vec<String>>,
}
