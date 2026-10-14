use ms_telemetry_rs::{TelemetryConfig, init_telemetry_with_fallback};
use programs::config::ApplicationConfig;
use programs::infrastructure::grpc_server::grpc_server;
use programs::infrastructure::shutdown::install_signal_handler;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let application_config = ApplicationConfig::new().expect("Failed to load application config");

    let otel_cfg =
        TelemetryConfig::load_config("src/config/otel_config.toml").expect("Failed to load otel_config");

    let _guard = init_telemetry_with_fallback(&otel_cfg);
    info!("Application Config: {:?}", application_config);
    info!("Telemetry Config: {:?}", otel_cfg);

    let shutdown = install_signal_handler();

    grpc_server(shutdown).await?;

    Ok(())
}
