// domain/document.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::errors::application_error::ApplicationError;

/// The module a program must have switched on to use Documents.
pub const MODULE_KEY: &str = "document_agreement.documents.documents_panel";

pub const UPLOADING: &str = "uploading";
pub const PENDING_SCAN: &str = "pending_scan";
pub const OK: &str = "ok";
pub const NEEDS_REVIEW: &str = "needs_review";
pub const QUARANTINED: &str = "quarantined";

/// The system folders every program has (they cannot be renamed or deleted).
pub const GENERAL_FOLDER: &str = "general";

/// How many files "Recent" shows.
pub const RECENT_LIMIT: i64 = 20;

const MAX_NAME_CHARS: usize = 255;
const MAX_FOLDER_NAME_CHARS: usize = 100;

/// What the file manager accepts. The limits come from the configuration.
#[derive(Debug, Clone)]
pub struct DocumentPolicy {
    pub max_file_bytes: u64,
    pub allowed_content_types: Vec<String>,
    pub max_files_per_program: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub program_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    /// `general`, `import_artifacts`, `program_documents`, `store_enrollment_documents`; `None` for a custom folder.
    pub key: Option<String>,
    pub system: bool,
    /// Files in this folder (a counter, kept in the same transaction as every upload, move and delete).
    pub document_count: i32,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// One file's metadata. The bytes are in object storage under `file_key`.
#[derive(Debug, Clone)]
pub struct Document {
    pub id: Uuid,
    pub program_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub file_key: String,
    pub status: String,
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub uploaded_by: Option<String>,
    pub checksum: Option<String>,
    pub scan_time: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
    pub reviewed_by: Option<String>,
    pub reviewed_time: Option<DateTime<Utc>>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// The header, the left panel and the footer of the page in one read.
#[derive(Debug, Clone)]
pub struct DocumentSummary {
    pub files: i32,
    pub bytes: i64,
    pub needs_review: i32,
    pub quarantined: i32,
    pub folders: Vec<Folder>,
}

impl DocumentSummary {
    /// "Recent" is the newest few, so it never counts more than that.
    pub fn recent(&self) -> i32 {
        self.files.min(RECENT_LIMIT as i32)
    }
}

#[derive(Debug, Clone)]
pub struct NewDocument {
    pub program_id: Uuid,
    /// `None` puts it in the General folder.
    pub folder_id: Option<Uuid>,
    pub name: String,
    pub content_type: String,
    pub size_bytes: i64,
    /// SHA-256 in hex; when given, the storage itself refuses an upload that differs.
    pub checksum: Option<String>,
    pub uploaded_by: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateDocument {
    pub name: Option<String>,
    /// Move to this folder.
    pub folder_id: Option<Uuid>,
    pub actor: String,
}

#[derive(Debug, Clone)]
pub struct NewFolder {
    pub program_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub actor: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateFolder {
    pub name: Option<String>,
    pub actor: String,
}

/// The page's left-panel locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Recent,
    NeedsReview,
    Quarantined,
}

impl Location {
    pub fn parse(raw: &str) -> Result<Self, ApplicationError> {
        match raw.trim() {
            "recent" => Ok(Location::Recent),
            "needs_review" => Ok(Location::NeedsReview),
            "quarantined" => Ok(Location::Quarantined),
            other => Err(ApplicationError::validation_error(&format!(
                "location {other:?} must be one of: recent, needs_review, quarantined"
            ))),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DocumentFilter {
    pub folder_id: Option<Uuid>,
    pub location: Option<Location>,
    /// A prefix of the file name (case-insensitive).
    pub name_prefix: Option<String>,
}

/// An admin's decision on a file that needs review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewDecision {
    Approve,
    Reject,
}

impl ReviewDecision {
    pub fn parse(raw: &str) -> Result<Self, ApplicationError> {
        match raw.trim() {
            "approve" => Ok(ReviewDecision::Approve),
            "reject" => Ok(ReviewDecision::Reject),
            other => Err(ApplicationError::unprocessable_entity(&format!("decision {other:?} must be \"approve\" or \"reject\""))),
        }
    }
}

/// The scanner's verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanResult {
    Clean,
    Suspicious,
    Infected,
}

impl ScanResult {
    pub fn parse(raw: &str) -> Result<Self, ApplicationError> {
        match raw.trim() {
            "clean" => Ok(ScanResult::Clean),
            "suspicious" => Ok(ScanResult::Suspicious),
            "infected" => Ok(ScanResult::Infected),
            other => Err(ApplicationError::unprocessable_entity(&format!(
                "result {other:?} must be \"clean\", \"suspicious\" or \"infected\""
            ))),
        }
    }

    /// Nothing becomes `ok` until it was scanned.
    pub fn status(self) -> &'static str {
        match self {
            ScanResult::Clean => OK,
            ScanResult::Suspicious => NEEDS_REVIEW,
            ScanResult::Infected => QUARANTINED,
        }
    }
}

/// Only a file that needs review can be reviewed: approve makes it `ok`, reject quarantines it.
pub fn reviewed_status(current: &str, decision: ReviewDecision) -> Result<&'static str, ApplicationError> {
    if current != NEEDS_REVIEW {
        return Err(ApplicationError::conflict(&format!("only a file that needs review can be reviewed (this one is {current})")));
    }
    Ok(match decision {
        ReviewDecision::Approve => OK,
        ReviewDecision::Reject => QUARANTINED,
    })
}

/// A file that has completed its upload is counted in the page's totals.
pub fn is_counted(status: &str) -> bool {
    status != UPLOADING
}

/// A quarantined file cannot be downloaded, and one still uploading is not a file yet.
pub fn can_download(status: &str) -> Result<(), ApplicationError> {
    match status {
        QUARANTINED => Err(ApplicationError::conflict("a quarantined file cannot be downloaded")),
        UPLOADING => Err(ApplicationError::conflict("the upload of this file is not complete")),
        _ => Ok(()),
    }
}

/// The name is only a label (the storage key is generated by the server), but it is cleaned so it can never
/// carry a path, a control character or leave the file nameless.
pub fn clean_file_name(raw: &str) -> Result<String, ApplicationError> {
    let cleaned: String = raw
        .chars()
        .map(|c| if c == '/' || c == '\\' || c.is_control() { '_' } else { c })
        .collect::<String>()
        .trim()
        .to_string();
    if cleaned.is_empty() || cleaned.chars().all(|c| c == '.' || c == '_') || cleaned.chars().count() > MAX_NAME_CHARS {
        return Err(ApplicationError::unprocessable_entity(&format!("name must be 1-{MAX_NAME_CHARS} characters and not just dots")));
    }
    Ok(cleaned)
}

pub fn clean_folder_name(raw: &str) -> Result<String, ApplicationError> {
    let cleaned = raw.trim().to_string();
    if cleaned.is_empty()
        || cleaned.chars().count() > MAX_FOLDER_NAME_CHARS
        || cleaned.chars().any(|c| c == '/' || c == '\\' || c.is_control())
    {
        return Err(ApplicationError::unprocessable_entity(&format!(
            "folder name must be 1-{MAX_FOLDER_NAME_CHARS} characters without slashes"
        )));
    }
    Ok(cleaned)
}

impl DocumentPolicy {
    /// Size and type are refused before any row is made. 413 and 415 are the HTTP answers.
    pub fn validate_upload(&self, content_type: &str, size_bytes: i64, checksum: &Option<String>) -> Result<(), ApplicationError> {
        if size_bytes <= 0 {
            return Err(ApplicationError::unprocessable_entity("size_bytes must be more than 0"));
        }
        if size_bytes as u64 > self.max_file_bytes {
            return Err(ApplicationError::payload_too_large(&format!(
                "a file can be at most {} bytes (this one is {size_bytes})",
                self.max_file_bytes
            )));
        }
        let kind = content_type.trim().to_lowercase();
        if !self.allowed_content_types.iter().any(|allowed| allowed.eq_ignore_ascii_case(&kind)) {
            return Err(ApplicationError::unsupported_media_type(&format!(
                "content type {content_type:?} is not allowed; allowed: {}",
                self.allowed_content_types.join(", ")
            )));
        }
        if let Some(sum) = checksum
            && !(sum.len() == 64 && sum.chars().all(|c| c.is_ascii_hexdigit()))
        {
            return Err(ApplicationError::unprocessable_entity("checksum must be a SHA-256 in hex (64 characters)"));
        }
        Ok(())
    }
}

#[async_trait]
pub trait DocumentRepository: Send + Sync {
    /// The header, the left panel and the footer in ONE read: the counters row and the folders.
    async fn summary(&self, program_id: Uuid) -> Result<DocumentSummary, ApplicationError>;

    async fn list_folders(&self, program_id: Uuid) -> Result<Vec<Folder>, ApplicationError>;
    async fn create_folder(&self, input: NewFolder) -> Result<Folder, ApplicationError>;
    /// A system folder cannot be renamed (`Conflict`).
    async fn update_folder(&self, program_id: Uuid, id: Uuid, input: UpdateFolder) -> Result<Folder, ApplicationError>;
    /// A system folder, or one with files or sub-folders, cannot be deleted (`Conflict`).
    async fn delete_folder(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<(), ApplicationError>;

    /// Newest first, keyset on the id. Files still uploading are in no list.
    async fn list(
        &self,
        program_id: Uuid,
        filter: &DocumentFilter,
        page_size: i64,
        page_token: Option<Uuid>,
    ) -> Result<(Vec<Document>, Option<Uuid>), ApplicationError>;

    /// The row first, hidden as `uploading`: nothing is counted until the upload completes.
    async fn create(&self, input: NewDocument, max_files: i64) -> Result<Document, ApplicationError>;

    /// A file that is still uploading is returned too, so the caller can follow its upload.
    async fn get(&self, program_id: Uuid, id: Uuid) -> Result<Document, ApplicationError>;

    /// `uploading` -> `pending_scan`, counted in the totals, with the audit entry and the outbox event the
    /// scanner listens to, in one transaction. A file already completed is returned as it is.
    async fn complete(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<Document, ApplicationError>;

    /// Rename and / or move. Moving keeps the folder counters right in the same transaction.
    async fn update(&self, program_id: Uuid, id: Uuid, input: UpdateDocument) -> Result<Document, ApplicationError>;

    /// Soft delete; the totals and the folder counter follow.
    async fn delete(&self, program_id: Uuid, id: Uuid, actor: &str) -> Result<(), ApplicationError>;

    async fn review(
        &self,
        program_id: Uuid,
        id: Uuid,
        decision: ReviewDecision,
        note: Option<String>,
        actor: &str,
    ) -> Result<Document, ApplicationError>;

    /// The scanner's verdict on a file that is `pending_scan` (the webhook has no program, only the document).
    /// A file that is not `pending_scan` is returned as it is, so a repeated callback is harmless.
    async fn apply_scan(&self, id: Uuid, result: ScanResult, scanner: &str) -> Result<Document, ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> DocumentPolicy {
        DocumentPolicy {
            max_file_bytes: 1000,
            allowed_content_types: vec!["application/pdf".into(), "image/png".into()],
            max_files_per_program: 5,
        }
    }

    #[test]
    fn file_names_are_cleaned_into_labels() {
        assert_eq!(clean_file_name("  Contract 2026.pdf ").unwrap(), "Contract 2026.pdf");
        assert_eq!(clean_file_name("../../etc/passwd").unwrap(), ".._.._etc_passwd");
        assert_eq!(clean_file_name("a\\b\u{0}c").unwrap(), "a_b_c");
        assert!(clean_file_name("   ").is_err());
        assert!(clean_file_name("..").is_err());
        assert!(clean_file_name("///").is_err());
        assert!(clean_file_name(&"n".repeat(256)).is_err());
        assert!(clean_file_name(&"n".repeat(255)).is_ok());
    }

    #[test]
    fn folder_names_have_no_slashes() {
        assert_eq!(clean_folder_name(" Legal ").unwrap(), "Legal");
        assert!(clean_folder_name("a/b").is_err());
        assert!(clean_folder_name("").is_err());
        assert!(clean_folder_name(&"f".repeat(101)).is_err());
    }

    #[test]
    fn uploads_are_checked_for_size_type_and_checksum() {
        let p = policy();
        assert!(p.validate_upload("application/pdf", 1000, &None).is_ok());
        assert!(p.validate_upload("IMAGE/PNG", 10, &None).is_ok(), "type is case-insensitive");
        assert!(p.validate_upload("application/pdf", 0, &None).is_err());
        let too_big = p.validate_upload("application/pdf", 1001, &None).unwrap_err();
        assert_eq!(too_big.kind(), crate::application::errors::application_error::ApplicationErrorKind::PayloadTooLarge);
        let bad_type = p.validate_upload("application/zip", 10, &None).unwrap_err();
        assert_eq!(bad_type.kind(), crate::application::errors::application_error::ApplicationErrorKind::UnsupportedMediaType);
        assert!(p.validate_upload("application/pdf", 10, &Some("abc".into())).is_err());
        assert!(p.validate_upload("application/pdf", 10, &Some("g".repeat(64))).is_err());
        assert!(p.validate_upload("application/pdf", 10, &Some("a".repeat(64))).is_ok());
    }

    #[test]
    fn a_scan_verdict_sets_the_status_and_nothing_is_ok_unscanned() {
        assert_eq!(ScanResult::parse("clean").unwrap().status(), OK);
        assert_eq!(ScanResult::parse("suspicious").unwrap().status(), NEEDS_REVIEW);
        assert_eq!(ScanResult::parse("infected").unwrap().status(), QUARANTINED);
        assert!(ScanResult::parse("fine").is_err());
    }

    #[test]
    fn only_a_file_that_needs_review_can_be_reviewed() {
        assert_eq!(reviewed_status(NEEDS_REVIEW, ReviewDecision::Approve).unwrap(), OK);
        assert_eq!(reviewed_status(NEEDS_REVIEW, ReviewDecision::Reject).unwrap(), QUARANTINED);
        for s in [UPLOADING, PENDING_SCAN, OK, QUARANTINED] {
            assert!(reviewed_status(s, ReviewDecision::Approve).is_err(), "{s}");
        }
        assert!(ReviewDecision::parse("maybe").is_err());
    }

    #[test]
    fn downloads_are_blocked_for_quarantined_and_unfinished_files() {
        assert!(can_download(OK).is_ok());
        assert!(can_download(PENDING_SCAN).is_ok());
        assert!(can_download(NEEDS_REVIEW).is_ok());
        assert!(can_download(QUARANTINED).is_err());
        assert!(can_download(UPLOADING).is_err());
    }

    #[test]
    fn totals_count_every_file_that_finished_uploading() {
        assert!(!is_counted(UPLOADING));
        for s in [PENDING_SCAN, OK, NEEDS_REVIEW, QUARANTINED] {
            assert!(is_counted(s));
        }
    }

    #[test]
    fn locations_parse() {
        assert_eq!(Location::parse("recent").unwrap(), Location::Recent);
        assert_eq!(Location::parse("needs_review").unwrap(), Location::NeedsReview);
        assert!(Location::parse("all").is_err());
    }

    #[test]
    fn recent_never_counts_more_than_it_shows() {
        let s = |files| DocumentSummary { files, bytes: 0, needs_review: 0, quarantined: 0, folders: vec![] };
        assert_eq!(s(3).recent(), 3);
        assert_eq!(s(500).recent(), 20);
    }
}
