use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationConfig {
    pub app_name: String,
    pub http_port: u16,
    pub grpc_port: u16,
    pub postgres: PostgresConfig,
    /// One browser origin allowed to call the HTTP API from another origin (for example
    /// Swagger UI's "Try it out": `http://localhost:8082`). Empty (the default) turns CORS
    /// off entirely, so a deployed service behind the gateway never answers cross-origin
    /// calls. Set with `APP__CORS_ALLOWED_ORIGIN`.
    #[serde(default)]
    pub cors_allowed_origin: String,
    /// The global-settings service: where library-backed settings read their options.
    #[serde(default)]
    pub global_settings: GlobalSettingsConfig,
    /// Object storage for the Documents module (S3 or any S3-compatible server such as MinIO).
    #[serde(default)]
    pub storage: StorageConfig,
    /// Parts of the product that are built but switched off.
    #[serde(default)]
    pub features: FeaturesConfig,
}

/// Switches for finished modules that are parked for now. Everything stays in the code and the
/// database; a switch only decides whether the module is served. See docs/parked-enrollment-review.md.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct FeaturesConfig {
    /// Requirements, Approval steps, Applications and Signoffs, with their two pages in the module
    /// tree (Requirements, Applications & Approvals). `false` (the default) serves none of it.
    /// Set with `APP__FEATURES__ENROLLMENT_REVIEW=true`.
    #[serde(default)]
    pub enrollment_review: bool,
}

/// Where document bytes live. The service never carries file bytes: it hands the browser a presigned
/// URL and reads back only the object's size and checksum.
#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    /// The S3 API address the SERVICE calls (to check an upload), e.g. `http://localhost:9000`.
    #[serde(default = "default_storage_endpoint")]
    pub endpoint: String,
    /// The address put into presigned URLs, which the BROWSER calls. Empty = the same as `endpoint`.
    #[serde(default)]
    pub public_endpoint: String,
    #[serde(default = "default_storage_bucket")]
    pub bucket: String,
    #[serde(default = "default_storage_region")]
    pub region: String,
    #[serde(default)]
    pub access_key: String,
    #[serde(default)]
    pub secret_key: String,
    /// How long a presigned upload or download URL works.
    #[serde(default = "default_storage_url_expiry_seconds")]
    pub url_expiry_seconds: u64,
    /// Per-call budget for checking an uploaded object.
    #[serde(default = "default_storage_timeout_ms")]
    pub timeout_ms: u64,
    /// The largest file accepted.
    #[serde(default = "default_storage_max_file_bytes")]
    pub max_file_bytes: u64,
    /// The most live files one program may hold.
    #[serde(default = "default_storage_max_files_per_program")]
    pub max_files_per_program: i64,
    /// The content types accepted.
    #[serde(default = "default_storage_allowed_content_types")]
    pub allowed_content_types: Vec<String>,
    /// When set, `POST /webhooks/document-scan` requires this value in the `X-Webhook-Secret` header.
    /// Empty = no check.
    #[serde(default)]
    pub scan_webhook_secret: String,
}

fn default_storage_endpoint() -> String { "http://localhost:9000".to_string() }
fn default_storage_bucket() -> String { "programs-documents".to_string() }
fn default_storage_region() -> String { "us-east-1".to_string() }
fn default_storage_url_expiry_seconds() -> u64 { 600 }
fn default_storage_timeout_ms() -> u64 { 3000 }
fn default_storage_max_file_bytes() -> u64 { 25 * 1024 * 1024 }
fn default_storage_max_files_per_program() -> i64 { 10_000 }
fn default_storage_allowed_content_types() -> Vec<String> {
    [
        "application/pdf",
        "image/png",
        "image/jpeg",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "text/csv",
        "text/plain",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            endpoint: default_storage_endpoint(),
            public_endpoint: String::new(),
            bucket: default_storage_bucket(),
            region: default_storage_region(),
            access_key: String::new(),
            secret_key: String::new(),
            url_expiry_seconds: default_storage_url_expiry_seconds(),
            timeout_ms: default_storage_timeout_ms(),
            max_file_bytes: default_storage_max_file_bytes(),
            max_files_per_program: default_storage_max_files_per_program(),
            allowed_content_types: default_storage_allowed_content_types(),
            scan_webhook_secret: String::new(),
        }
    }
}

/// Calls to global-settings are server to server, so they use the service headers
/// (`X-Client-ID` / `X-Client-Secret`), never the browser's session.
#[derive(Debug, Deserialize, Clone)]
pub struct GlobalSettingsConfig {
    /// Base URL of the global-settings API, e.g. `http://localhost:8000`.
    #[serde(default = "default_gs_base_url")]
    pub base_url: String,
    /// `false` skips every call: a `library_id` is stored as given (not validated) and no
    /// options are resolved. For environments where global-settings is not reachable.
    #[serde(default = "default_gs_enabled")]
    pub enabled: bool,
    /// Per-call budget. A library read sits on the page-load path, so it is short.
    #[serde(default = "default_gs_timeout_ms")]
    pub timeout_ms: u64,
    /// How long a fetched library (with its options) is reused before it is read again.
    #[serde(default = "default_gs_cache_seconds")]
    pub cache_seconds: u64,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
}

fn default_gs_base_url() -> String { "http://localhost:8000".to_string() }
fn default_gs_enabled() -> bool { true }
fn default_gs_timeout_ms() -> u64 { 1500 }
fn default_gs_cache_seconds() -> u64 { 30 }

impl Default for GlobalSettingsConfig {
    fn default() -> Self {
        Self {
            base_url: default_gs_base_url(),
            enabled: default_gs_enabled(),
            timeout_ms: default_gs_timeout_ms(),
            cache_seconds: default_gs_cache_seconds(),
            client_id: String::new(),
            client_secret: String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
}

impl ApplicationConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let cfg = Config::builder()
            .add_source(File::with_name("src/config/config.toml").required(true))
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        cfg.try_deserialize()
    }
}
