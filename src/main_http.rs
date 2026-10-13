use ms_telemetry_rs::{TelemetryConfig, init_telemetry_with_fallback};
use global_settings::config::ApplicationConfig;
use global_settings::infrastructure::http_server::http_server;
use global_settings::infrastructure::shutdown::install_signal_handler;
use std::net::TcpListener;
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let application_cfg = ApplicationConfig::new().expect("Failed to load application config");
    let otel_cfg = TelemetryConfig::load_config("src/config/otel_config.toml")
        .expect("Failed to load otel config");

    let _guard = init_telemetry_with_fallback(&otel_cfg);

    info!("Application Config: {:?}", application_cfg);
    info!("Telemetry Config: {:?}", otel_cfg);

    let listner =
        TcpListener::bind("0.0.0.0:".to_string() + &application_cfg.http_port.to_string())
            .expect("Port binding failed");

    let shutdown = install_signal_handler();

    http_server(listner, shutdown).await?;

    Ok(())
}
