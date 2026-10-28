// domain/module_catalog_item.rs
use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;
use crate::domain::aep::ResourceIdentifier;

/// A node of the program module tree. The tree nests to any depth: a `group`
/// holds further modules, a `leaf` is a screen. Seeded, not created through
/// the API, so there is no `NewModuleCatalogItem` and no write method on the
/// repository.
///
/// `Serialize`/`Deserialize` are for a future in-memory/Redis registry cache,
/// not an API response shape — HTTP/gRPC have their own presenters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCatalogItem {
    pub id: Uuid,                        // OUTPUT_ONLY
    pub key: String,                     // IMMUTABLE — stable dotted/underscored identifier
    pub name: String,
    pub kind: String,                    // "group" | "leaf"
    pub parent_id: Option<Uuid>,
    pub ancestry: String,                // ltree path, e.g. "catalog_pricing.products_rules"
    pub position: i32,
    pub ui_component: Option<String>,    // how a leaf renders: rule_playground | field_form | custom:<name>
    pub rule_family: Option<String>,     // GoRules family for rule leaves
    pub universal: bool,                 // every program gets it
    pub requires_publish: bool,          // changes only take effect after a publication
    pub permission_key: Option<String>,
    pub source: String,                  // "premade" | "container" | "default"
    pub handler_key: Option<String>,     // code that handles a premade / default module
    pub api_base: Option<String>,        // the API the frontend calls for it
    pub selectable: bool,                // false for Overview (always on)
    pub allow_components: bool,          // the admin may add settings / rules here
    pub default_mode: Option<String>,    // "rule" | "settings": the mode a program gets when it selects the page
    pub create_time: DateTime<Utc>,      // OUTPUT_ONLY
    pub update_time: DateTime<Utc>,      // OUTPUT_ONLY
}

#[async_trait]
pub trait ModuleCatalogItemRepository: Send + Sync {
    /// Accepts the id or the key, same as `GET /libraries/{library}`.
    async fn get(&self, identifier: ResourceIdentifier) -> Result<ModuleCatalogItem, ApplicationError>;

    /// Direct children of `parent_id`, or the root-level modules when it is
    /// `None`, in display order (`position`, then `id`). The returned `i64` is
    /// `total_size`, the count matching this same filter.
    async fn list(
        &self,
        parent_id: Option<Uuid>,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<ModuleCatalogItem>, Option<Uuid>, i64), ApplicationError>;

    /// Every module below `root` (or every module when `None`), flat and in
    /// display order, in ONE query — the caller nests them with `build_tree`.
    /// `NotFound` when `root` is given but does not exist.
    async fn list_descendants(&self, root: Option<Uuid>) -> Result<Vec<ModuleCatalogItem>, ApplicationError>;
}

/// A module with its descendants nested under it, for the full-tree view.
#[derive(Debug, Clone)]
pub struct ModuleCatalogItemNode {
    pub module_catalog_item: ModuleCatalogItem,
    pub children: Vec<ModuleCatalogItemNode>,
}

/// Nests a flat list into a tree. `rows` MUST already be in display order
/// (`position`, then `id`): each node's children keep that order. `root` is
/// the parent whose direct children become the top level — `None` for the
/// whole tree, or a module id for the subtree beneath it. Rows that hang off a
/// parent not present in `rows` are not reachable from `root` and are dropped.
pub fn build_tree(rows: Vec<ModuleCatalogItem>, root: Option<Uuid>) -> Vec<ModuleCatalogItemNode> {
    let mut by_parent: HashMap<Option<Uuid>, Vec<ModuleCatalogItem>> = HashMap::new();
    for row in rows {
        by_parent.entry(row.parent_id).or_default().push(row);
    }

    fn nest(
        parent: Option<Uuid>,
        by_parent: &mut HashMap<Option<Uuid>, Vec<ModuleCatalogItem>>,
    ) -> Vec<ModuleCatalogItemNode> {
        by_parent
            .remove(&parent)
            .unwrap_or_default()
            .into_iter()
            .map(|module_catalog_item| {
                let children = nest(Some(module_catalog_item.id), by_parent);
                ModuleCatalogItemNode { module_catalog_item, children }
            })
            .collect()
    }

    nest(root, &mut by_parent)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(key: &str, id: u128, parent: Option<u128>, position: i32) -> ModuleCatalogItem {
        ModuleCatalogItem {
            id: Uuid::from_u128(id),
            key: key.to_string(),
            name: key.to_string(),
            kind: "group".to_string(),
            parent_id: parent.map(Uuid::from_u128),
            ancestry: key.to_string(),
            position,
            ui_component: None,
            rule_family: None,
            universal: false,
            requires_publish: false,
            permission_key: None,
            source: "container".to_string(),
            handler_key: None,
            api_base: None,
            selectable: true,
            allow_components: false,
            default_mode: None,
            create_time: chrono::Utc::now(),
            update_time: chrono::Utc::now(),
        }
    }

    fn keys(nodes: &[ModuleCatalogItemNode]) -> Vec<&str> {
        nodes.iter().map(|n| n.module_catalog_item.key.as_str()).collect()
    }

    #[test]
    fn nests_children_under_parents_in_order() {
        let rows = vec![
            module("a", 1, None, 1),
            module("b", 2, None, 2),
            module("a1", 3, Some(1), 1),
            module("a2", 4, Some(1), 2),
            module("a1x", 5, Some(3), 1),
        ];
        let tree = build_tree(rows, None);

        assert_eq!(keys(&tree), ["a", "b"]);
        assert_eq!(keys(&tree[0].children), ["a1", "a2"]);
        assert_eq!(keys(&tree[0].children[0].children), ["a1x"]);
        assert!(tree[1].children.is_empty());
    }

    #[test]
    fn subtree_starts_at_the_given_parent() {
        let rows = vec![module("a1", 3, Some(1), 1), module("a1x", 5, Some(3), 1)];
        let tree = build_tree(rows, Some(Uuid::from_u128(1)));

        assert_eq!(keys(&tree), ["a1"]);
        assert_eq!(keys(&tree[0].children), ["a1x"]);
    }

    #[test]
    fn unreachable_rows_are_dropped() {
        let tree = build_tree(vec![module("orphan", 9, Some(404), 1)], None);
        assert!(tree.is_empty());
    }
}
