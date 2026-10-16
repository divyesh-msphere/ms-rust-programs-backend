use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ListProgramModulesPathPayload {
    /// The program's ID or its slug.
    pub program: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListProgramModulesQueryPayload {
    /// Only modules the program has switched on (what the frontend draws).
    /// Defaults to true.
    pub enabled_only: Option<bool>,
    /// Fill library-backed settings with their library's options (default true).
    pub include_options: Option<bool>,
}
