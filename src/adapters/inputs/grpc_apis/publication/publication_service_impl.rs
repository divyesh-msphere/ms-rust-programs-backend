use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    CreatePublicationRequest, CreatePublicationResponse, GetPublicationRequest, GetPublicationResponse,
    ListPublicationsRequest, ListPublicationsResponse,
};
use crate::adapters::inputs::grpc_apis::publication::publication_mappers::{
    CreatePublicationGrpcMapper, GetPublicationGrpcMapper, ListPublicationsGrpcMapper,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct PublicationServiceImpl {
    app_state: Arc<AppState>,
}

impl PublicationServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl PublicationServiceImpl {
    pub async fn create_publication(
        &self,
        actor: String,
        request: ServiceRequest<'_, CreatePublicationRequest>,
    ) -> ServiceResult<CreatePublicationResponse> {
        let input = CreatePublicationGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .publication
            .create_publication
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(CreatePublicationGrpcMapper::output_success(output))
    }

    pub async fn list_publications(
        &self,
        request: ServiceRequest<'_, ListPublicationsRequest>,
    ) -> ServiceResult<ListPublicationsResponse> {
        let input = ListPublicationsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .publication
            .list_publications
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListPublicationsGrpcMapper::output_success(output))
    }

    pub async fn get_publication(
        &self,
        request: ServiceRequest<'_, GetPublicationRequest>,
    ) -> ServiceResult<GetPublicationResponse> {
        let input = GetPublicationGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .publication
            .get_publication
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetPublicationGrpcMapper::output_success(output))
    }
}
