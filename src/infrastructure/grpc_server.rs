use crate::Dependencies;
use crate::adapters::inputs::shared::app_state::AppState;
use connectrpc::{Router, Server};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::info;

// Server reflection (`grpc.reflection.v1` + `v1alpha`, for `grpcurl` /
// `buf curl` / Postman / `grpcui`) needs the compiled `FileDescriptorSet`.
// `buf generate` doesn't produce one — generate it with a plain `buf build`
// and check it in next to the generated Rust code:
//
//   buf build "ssh://git@github.com/MobileSentrix/ms-protos.git#subdir=proto,branch=<proto-branch>" \
//     --path programs/v1 -o src/adapters/generated/grpc_services/programs.binpb
//
// Re-run it whenever the proto schema changes, same as `buf generate`. Then:
//
//   const PROGRAMS_DESCRIPTOR_SET: &[u8] =
//       include_bytes!("../adapters/generated/grpc_services/programs.binpb");
//   let reflector = connectrpc_reflection::Reflector::from_descriptor_set_bytes(PROGRAMS_DESCRIPTOR_SET)
//       .expect("embedded FileDescriptorSet failed to decode");
//   let router = connectrpc_reflection::install(router, reflector);

/// Standalone gRPC server: mirrors `http_server.rs`'s shape (own
/// `Dependencies::load()`, own graceful-shutdown wiring via the same
/// `watch::Receiver<bool>` convention).
pub async fn grpc_server(mut shutdown: watch::Receiver<bool>) -> Result<(), Box<dyn std::error::Error>> {
    let deps = Dependencies::load().await;
    let addr = format!("0.0.0.0:{}", deps.application_config.grpc_port);
    info!("gRPC server is running on addr: {}", addr);

    let _app_state = Arc::new(AppState {
        application_config: deps.application_config,
        repositories: deps.repositories,
        use_cases: deps.use_cases,
    });

    // Register the generated service here once `buf generate` has run:
    //   let router = Router::new().add_service(Arc::new(ProgramsServiceImpl::new(app_state)));
    let router = Router::new();

    let bound = Server::bind(&addr).await.map_err(|e| e.to_string())?;
    bound
        .serve_with_graceful_shutdown(router, async move {
            let _ = shutdown.wait_for(|&s| s).await;
        })
        .await
        .map_err(|e| e.to_string())?;

    info!("gRPC server stopped gracefully");
    Ok(())
}
