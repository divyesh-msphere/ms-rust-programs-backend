use crate::Dependencies;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::adapters::inputs::http_apis::shared::json_input_error_handler::json_error_handler;
use crate::adapters::inputs::http_apis::shared::query_input_error_handler::query_error_handler;
use crate::adapters::inputs::http_apis::shared::route_not_found::not_found_handler;
use crate::adapters::inputs::http_apis::shared::routes::routes;
use actix_cors::Cors;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Condition;
use actix_web::{App, Error, HttpServer, middleware, web};
use tracing::{info, Span};
use std::net::TcpListener;
use tokio::sync::watch;
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};

/// Skips access-log spans for `/health` so uptime probes don't flood logs;
/// every other route logs as normal via `DefaultRootSpanBuilder`.
struct HealthSkippingRootSpanBuilder;

impl RootSpanBuilder for HealthSkippingRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        if request.path().starts_with("/health") {
            Span::none()
        } else {
            DefaultRootSpanBuilder::on_request_start(request)
        }
    }

    fn on_request_end<B: actix_web::body::MessageBody>(
        span: Span,
        outcome: &Result<ServiceResponse<B>, Error>,
    ) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

pub async fn http_server(
    listner: TcpListener,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), std::io::Error> {
    let deps = Dependencies::load().await;
    let port = listner.local_addr().unwrap().port();
    info!("Starting HTTP server on port {}", port);

    let server = HttpServer::new(move || {
        let data = web::Data::new(AppState {
            application_config: deps.application_config.clone(),
            repositories: deps.repositories.clone(),
            use_cases: deps.use_cases.clone(),
            library_catalog: deps.library_catalog.clone(),
        });

        // Opt-in CORS for browser tools such as Swagger UI's "Try it out". Off unless an
        // origin is configured (`cors_allowed_origin`), so nothing changes in a deployment.
        let cors_origin = deps.application_config.cors_allowed_origin.trim().to_string();
        let cors = Cors::default()
            .allowed_origin(&cors_origin)
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(data.clone())
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
            .app_data(web::QueryConfig::default().error_handler(query_error_handler))
            .configure(routes)
            .default_service(web::route().to(not_found_handler))
            .wrap(TracingLogger::<HealthSkippingRootSpanBuilder>::new())
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            // Outermost, so a preflight (OPTIONS) is answered before routing.
            .wrap(Condition::new(!cors_origin.is_empty(), cors))
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
