// domain/object_storage.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::application::errors::application_error::ApplicationError;

/// A request the BROWSER makes straight to the storage: the service only signs it.
#[derive(Debug, Clone)]
pub struct PresignedRequest {
    pub url: String,
    pub method: String,
    /// Headers the caller must send exactly as given: they are part of the signature, so a different
    /// size, type or checksum is rejected by the storage itself.
    pub headers: Vec<(String, String)>,
    pub expires_time: DateTime<Utc>,
}

/// What the storage reports about an uploaded object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectInfo {
    pub size_bytes: i64,
    pub content_type: Option<String>,
    /// SHA-256 as hex, when the storage reports one.
    pub checksum_sha256: Option<String>,
}

/// Object storage for document bytes (S3 or an S3-compatible server). The service never carries the bytes.
#[async_trait]
pub trait ObjectStorage: Send + Sync {
    /// A short-lived URL for ONE exact object key. The declared size, type and (when given) SHA-256 are
    /// signed in, so the storage refuses anything else.
    fn presign_upload(
        &self,
        key: &str,
        content_type: &str,
        size_bytes: i64,
        checksum_sha256_hex: Option<&str>,
    ) -> Result<PresignedRequest, ApplicationError>;

    /// A short-lived URL that downloads the object as an attachment named `file_name`.
    fn presign_download(&self, key: &str, file_name: &str, content_type: &str) -> Result<PresignedRequest, ApplicationError>;

    /// The object's size, type and checksum, or `None` when it is not there (not uploaded yet).
    async fn head(&self, key: &str) -> Result<Option<ObjectInfo>, ApplicationError>;
}
