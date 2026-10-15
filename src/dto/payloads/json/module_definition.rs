use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GetModuleDefinitionPayload {
    /// The module definition's ID, or its key (`catalog_pricing`).
    pub module_definition: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListModuleDefinitionsPayload {
    pub max_page_size: Option<i64>,
    pub page_token: Option<String>,
    /// List the direct children of this module (ID). Omitted returns the
    /// root-level (main) modules.
    pub parent_id: Option<String>,
    /// When true, ignores `max_page_size` / `page_token` and returns the whole
    /// tree (or, with `parent_id`, everything beneath that module) nested
    /// under `children`, in one response.
    pub full_tree: Option<bool>,
}
