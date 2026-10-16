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
    /// When set, `POST /webhooks/global-settings` requires this value in the `X-Webhook-Secret`
    /// header (configure the same header on the global-settings webhook). Empty = no check.
    #[serde(default)]
    pub webhook_secret: String,
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
            webhook_secret: String::new(),
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
