use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct ListRuleSetsPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetRuleSetPayload {
    pub program: String,
    /// The rule leaf's module key (for example `catalog_pricing.products_rules.warranty`) or the id of its rule set.
    pub rule_set: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateRuleVersionPathPayload {
    pub program: String,
    pub rule_set: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateRuleVersionBodyPayload {
    /// The rule model the builder draws (a JSON object, up to 1 MB).
    pub definition: Value,
    /// The generated decision graph, when the rule engine produced one (a JSON object, up to 1 MB).
    pub compiled: Option<Value>,
    /// Informational. Defaults from the module's rule family.
    pub rule_type: Option<String>,
    /// Whether the rule engine found the definition complete. Defaults to "no validation errors were sent".
    pub valid: Option<bool>,
    /// The rule engine's complaints (a list, up to 100). A version with errors cannot be `valid`.
    pub validation_errors: Option<Value>,
    /// The input the rule reads: a list of `{name, type, required, description?}` (up to 100 fields).
    /// Execute checks the caller's input against it; Publish refuses a rule that has none.
    pub input_schema: Option<Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExecuteRulePathPayload {
    pub program: String,
    /// The rule leaf's module key (for example `catalog_pricing.products_rules.pre_enrollment`) or the id of its rule set.
    pub rule_set: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExecuteRuleBodyPayload {
    /// The rule family the caller expects (for example `pre_enrollment`), checked against the module.
    pub r#type: Option<String>,
    /// `published` (the default), `draft` or a version number, as a string or a number.
    pub version: Option<Value>,
    /// What the rule reads: a JSON object. Required.
    pub input: Value,
    /// Also return the nodes that were visited.
    pub trace: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListRuleVersionsPathPayload {
    pub program: String,
    pub rule_set: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListRuleVersionsQueryPayload {
    pub max_page_size: Option<i64>,
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetRuleVersionPayload {
    pub program: String,
    pub rule_set: String,
    /// The version number (1, 2, 3, ...).
    pub version: String,
}
