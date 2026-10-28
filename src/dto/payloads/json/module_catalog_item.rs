use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GetModuleCatalogItemPayload {
    /// The module catalog item's ID, or its key (`catalog_pricing`).
    pub module_catalog_item: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListModuleCatalogPayload {
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
