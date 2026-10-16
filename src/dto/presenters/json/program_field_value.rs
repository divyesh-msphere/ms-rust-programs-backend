use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct FieldValuePresenter {
    pub setting_id: String,
    pub module_key: String,
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    /// The saved value of a boolean / number / text setting.
    pub value: Option<serde_json::Value>,
    /// The saved choice of a radio / checkbox / dropdown setting.
    pub option_ids: Vec<String>,
    pub update_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct FieldValuesPresenter {
    /// The program's `config_version`: send it back with a save to detect a concurrent change.
    pub version: i32,
    pub results: Vec<FieldValuePresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BatchUpdateFieldValuesPresenter {
    pub version: i32,
    /// How many values actually changed (a value sent unchanged is not a change).
    pub changed: usize,
    pub results: Vec<FieldValuePresenter>,
}
