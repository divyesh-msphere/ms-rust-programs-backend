use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    GetNotificationsRequest, GetNotificationsResponse, ReplaceNotificationsRequest, ReplaceNotificationsResponse,
};
use crate::adapters::inputs::grpc_apis::notifications::notifications_mappers::{
    GetNotificationsGrpcMapper, ReplaceNotificationsGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct NotificationsServiceImpl {
    app_state: Arc<AppState>,
}

impl NotificationsServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    pub async fn get_notifications(
        &self,
        request: ServiceRequest<'_, GetNotificationsRequest>,
    ) -> ServiceResult<GetNotificationsResponse> {
        let input = GetNotificationsGrpcMapper::input(request.to_owned_message()).map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .notifications
            .get_notifications
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;
        Response::ok(GetNotificationsGrpcMapper::output_success(output))
    }

    pub async fn replace_notifications(
        &self,
        actor: String,
        request: ServiceRequest<'_, ReplaceNotificationsRequest>,
    ) -> ServiceResult<ReplaceNotificationsResponse> {
        let input = ReplaceNotificationsGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .notifications
            .replace_notifications
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;
        Response::ok(ReplaceNotificationsGrpcMapper::output_success(output))
    }
}
