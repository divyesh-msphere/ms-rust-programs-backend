use crate::Dependencies;
use crate::adapters::inputs::grpc_apis::shared::app_state::GrpcAppState;
use std::sync::Arc;
use tonic::service::Routes;
use tonic::transport::Server;
use tracing::info;
use tokio::sync::watch;

pub async fn grpc_server(mut shutdown: watch::Receiver<bool>) -> Result<(), Box<dyn std::error::Error>> {
    let deps = Dependencies::load().await;
    let addr = format!("0.0.0.0:{}", deps.application_config.grpc_port).parse()?;
    info!("gRPC server is running on addr: {}", addr);

    let _grpc_app_state = Arc::new(GrpcAppState {
        application_config: deps.application_config,
        repositories: deps.repositories,
        use_cases: deps.use_cases,
    });

    // Register generated services here: routes.add_service(XServiceServer::new(XServiceImpl::new(state)))
    let routes = Routes::default();
    Server::builder()
        .add_routes(routes)
        .serve_with_shutdown(addr, async move {
            let _ = shutdown.wait_for(|&s| s).await; // resolves on Ctrl+C/SIGTERM
        })
        .await?;

    info!("gRPC server stopped gracefully");

    Ok(())
}
