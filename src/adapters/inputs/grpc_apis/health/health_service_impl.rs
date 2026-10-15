use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    ListHealthCheckRequest, ListHealthCheckResponse,
};
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::utils::datetime::format_datetime;
use chrono::Utc;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct HealthServiceImpl {
    app_state: Arc<AppState>,
}

impl HealthServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl HealthServiceImpl {
    pub async fn list_health_check(
        &self,
        _request: ServiceRequest<'_, ListHealthCheckRequest>,
    ) -> ServiceResult<ListHealthCheckResponse> {
        Response::ok(ListHealthCheckResponse {
            status: "ok".to_string(),
            service: self.app_state.application_config.app_name.clone(),
            check_time: format_datetime(Utc::now()),
            ..Default::default()
        })
    }
}
