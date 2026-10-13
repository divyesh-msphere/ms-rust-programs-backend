use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationConfig {
    pub app_name: String,
    pub http_port: u16,
    pub grpc_port: u16,
    pub postgres: PostgresConfig,
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
