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
    /// Finds modules whose name contains this text (any depth, case-insensitive). `module=text`
    /// is accepted too. With `parent_id`, only modules beneath it. Ignored with `full_tree=true`.
    pub search: Option<String>,
    /// Comma-separated module keys or ids (`[a,b]` also works): returns exactly these modules, flat and
    /// with nothing beneath them, in display order. `parent_id` is ignored; `search` narrows within them. Wins over `full_tree`.
    pub keys: Option<String>,
    /// With `checked`: the program (id or slug) whose ticked modules to return.
    pub program: Option<String>,
    /// `true` returns only the modules the `program` has ticked (selected and switched on), flat and with
    /// nothing beneath them; `false` returns the ones it has not. Needs `program`; `program` alone means
    /// `true`. Wins over `full_tree` and `parent_id`.
    pub checked: Option<bool>,
    /// When true, ignores `max_page_size` / `page_token` and returns the whole
    /// tree (or, with `parent_id`, everything beneath that module) nested
    /// under `children`, in one response.
    pub full_tree: Option<bool>,
}
