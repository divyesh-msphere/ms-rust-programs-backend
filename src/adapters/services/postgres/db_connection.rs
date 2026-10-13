use crate::config::PostgresConfig;
use sqlx::pool::{PoolConnection, PoolOptions};
use sqlx::postgres::PgConnectOptions;
use sqlx::{PgPool, Postgres};
use ms_telemetry_rs::tracing::info;

#[derive(Clone)]
#[allow(warnings)]
pub struct PostgresDbConnection {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
    max_connections: u32,
    pool: PgPool,
}

impl PostgresDbConnection {
    pub fn new(cfg: &PostgresConfig) -> Self {
        info!(
            "Connecting to Postgres Host: {}, Port: {}",
            cfg.host, cfg.port
        );

        let connection_opts = PgConnectOptions::new()
            .host(&cfg.host)
            .port(cfg.port)
            .username(&cfg.username)
            .password(&cfg.password)
            .database(&cfg.database);

        let pool = PoolOptions::new()
            .min_connections(1)
            .max_connections(cfg.max_connections)
            .connect_lazy_with(connection_opts);

        Self {
            host: cfg.host.clone(),
            port: cfg.port,
            username: cfg.username.clone(),
            password: cfg.password.clone(),
            database: cfg.database.clone(),
            max_connections: cfg.max_connections,
            pool,
        }
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn get_connection(&self) -> Result<PoolConnection<Postgres>, sqlx::Error> {
        self.pool.acquire().await
    }

    pub async fn close_connection(
        &self,
        conn: PoolConnection<Postgres>,
    ) -> Result<(), sqlx::Error> {
        conn.close().await
    }
}
