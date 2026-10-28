use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModuleCatalogItemPresenter {
    pub path: String,
    pub id: String,
    pub key: String,
    pub name: String,
    pub kind: String,
    pub parent_id: Option<String>,
    pub ancestry: String,
    pub position: i32,
    pub ui_component: Option<String>,
    pub rule_family: Option<String>,
    pub universal: bool,
    pub requires_publish: bool,
    pub permission_key: Option<String>,
    pub source: String,
    pub handler_key: Option<String>,
    pub api_base: Option<String>,
    pub selectable: bool,
    pub allow_components: bool,
    pub default_mode: Option<String>,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ListModuleCatalogPresenter {
    pub results: Vec<ModuleCatalogItemPresenter>,
    pub next_page_token: Option<String>,
    pub total_size: i64,
}

/// A module with its descendants nested under `children` (`full_tree=true`).
/// Carries every `ModuleCatalogItemPresenter` field, flattened, so a tree node
/// reads exactly like a list item plus its subtree.
#[derive(Debug, Serialize, Clone)]
pub struct ModuleCatalogTreeNodePresenter {
    #[serde(flatten)]
    pub module_catalog_item: ModuleCatalogItemPresenter,
    pub children: Vec<ModuleCatalogTreeNodePresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ModuleCatalogTreePresenter {
    pub results: Vec<ModuleCatalogTreeNodePresenter>,
}
