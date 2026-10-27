use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Clone)]
pub struct FolderPresenter {
    pub path: String,
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    /// `general`, `import_artifacts`, `program_documents`, `store_enrollment_documents`; `null` for a custom folder.
    pub key: Option<String>,
    /// A system folder cannot be renamed or deleted.
    pub system: bool,
    pub document_count: i32,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct FoldersPresenter {
    pub results: Vec<FolderPresenter>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DocumentPresenter {
    pub path: String,
    pub id: String,
    pub name: String,
    pub folder_id: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    /// `uploading`, `pending_scan`, `ok`, `needs_review` or `quarantined`.
    pub status: String,
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub uploaded_by: Option<String>,
    pub checksum: Option<String>,
    pub scan_time: Option<String>,
    pub review_note: Option<String>,
    pub reviewed_by: Option<String>,
    pub reviewed_time: Option<String>,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct DocumentsPresenter {
    pub results: Vec<DocumentPresenter>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct LocationsPresenter {
    pub all: i32,
    pub recent: i32,
    pub needs_review: i32,
    pub quarantined: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct DocumentSummaryPresenter {
    /// "N files".
    pub files: i32,
    /// "X KB" (the Storage line in the footer).
    pub bytes: i64,
    pub locations: LocationsPresenter,
    pub folders: Vec<FolderPresenter>,
}

/// A request the BROWSER makes straight to the storage.
#[derive(Debug, Serialize, Clone)]
pub struct StorageRequestPresenter {
    pub url: String,
    pub method: String,
    /// Send these headers exactly as given: they are part of the signature.
    pub headers: BTreeMap<String, String>,
    pub expires_time: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CreateDocumentPresenter {
    /// The new file, still `uploading`.
    pub document: DocumentPresenter,
    pub upload: StorageRequestPresenter,
}

#[derive(Debug, Serialize, Clone)]
pub struct DownloadDocumentPresenter {
    pub document: DocumentPresenter,
    pub download: StorageRequestPresenter,
}
