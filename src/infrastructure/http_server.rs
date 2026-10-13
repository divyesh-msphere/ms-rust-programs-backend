use crate::Dependencies;
use crate::adapters::inputs::http_apis::shared::app_state::AppState;
use crate::adapters::inputs::http_apis::shared::json_input_error_handler::json_error_handler;
use crate::adapters::inputs::http_apis::shared::route_not_found::not_found_handler;
use crate::adapters::inputs::http_apis::shared::routes::routes;
use actix_web::{App, HttpServer, middleware, web};
use ms_telemetry_rs::tracing::info;
use std::net::TcpListener;
use tokio::sync::watch;
use tracing_actix_web::TracingLogger;

pub async fn http_server(
    listner: TcpListener,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), std::io::Error> {
    let deps = Dependencies::load().await;
    let port = listner.local_addr().unwrap().port();

    info!("Server is ready and running on port {}", port);

    let server = HttpServer::new(move || {
        let data = web::Data::new(AppState {
            application_config: deps.application_config.clone(),
            repositories: deps.repositories.clone(),
            use_cases: deps.use_cases.clone(),
        });

        App::new()
            .app_data(data.clone())
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
            .configure(routes)
            .default_service(web::route().to(not_found_handler))
            .wrap(TracingLogger::default())
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
    })
    .listen(listner)?
    .disable_signals()
    .shutdown_timeout(100)
    .workers(1)
    .run();

    let handle = server.handle();

    actix_web::rt::spawn(async move {
        let _ = shutdown.wait_for(|&s| s).await;
        handle.stop(true).await;
    });

    server.await
}
