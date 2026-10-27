// domain/library_catalog.rs
//
// Where a library-backed setting reads its options. The use cases depend on this trait, not
// on HTTP, so global-settings can be swapped or faked in tests.
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LibraryOption {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct Library {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    /// `single` or `multi`.
    pub selection_mode: String,
    pub options: Vec<LibraryOption>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    /// global-settings has no such library.
    NotFound,
    /// global-settings could not be reached or answered badly.
    Unavailable(String),
}

#[async_trait]
pub trait LibraryCatalog: Send + Sync {
    /// False when global-settings integration is switched off: library ids are then stored
    /// as given and no options are resolved.
    fn enabled(&self) -> bool;

    /// A library with its options. Served from memory when it can be (see the adapter); a
    /// library that is gone is `NotFound`, an unreachable service `Unavailable`.
    async fn get(&self, library_id: Uuid) -> Result<Arc<Library>, CatalogError>;
}
