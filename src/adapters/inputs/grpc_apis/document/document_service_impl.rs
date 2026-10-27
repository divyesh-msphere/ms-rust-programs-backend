use crate::adapters::generated::grpc_services::buffa::programs::v1::{GetDocumentSummaryRequest, ListDocumentFoldersRequest, CreateDocumentFolderRequest, UpdateDocumentFolderRequest, DeleteDocumentFolderRequest, ListDocumentsRequest, CreateDocumentRequest, CompleteDocumentRequest, GetDocumentRequest, UpdateDocumentRequest, DeleteDocumentRequest, DownloadDocumentRequest, ReviewDocumentRequest, GetDocumentSummaryResponse, ListDocumentFoldersResponse, CreateDocumentFolderResponse, UpdateDocumentFolderResponse, ListDocumentsResponse, CreateDocumentResponse, CompleteDocumentResponse, GetDocumentResponse, UpdateDocumentResponse, DownloadDocumentResponse, ReviewDocumentResponse};
use crate::adapters::inputs::grpc_apis::document::document_mappers::{GetDocumentSummaryGrpcMapper, ListDocumentFoldersGrpcMapper, CreateDocumentFolderGrpcMapper, UpdateDocumentFolderGrpcMapper, DeleteDocumentFolderGrpcMapper, ListDocumentsGrpcMapper, CreateDocumentGrpcMapper, CompleteDocumentGrpcMapper, GetDocumentGrpcMapper, UpdateDocumentGrpcMapper, DeleteDocumentGrpcMapper, DownloadDocumentGrpcMapper, ReviewDocumentGrpcMapper};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use buffa_types::google::protobuf::Empty;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct DocumentServiceImpl {
    app_state: Arc<AppState>,
}

impl DocumentServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl DocumentServiceImpl {
    pub async fn get_document_summary(
        &self,
        request: ServiceRequest<'_, GetDocumentSummaryRequest>,
    ) -> ServiceResult<GetDocumentSummaryResponse> {
        let input = GetDocumentSummaryGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .document
            .get_document_summary
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetDocumentSummaryGrpcMapper::output_success(output))
    }

    pub async fn list_document_folders(
        &self,
        request: ServiceRequest<'_, ListDocumentFoldersRequest>,
    ) -> ServiceResult<ListDocumentFoldersResponse> {
        let input = ListDocumentFoldersGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .document
            .list_document_folders
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListDocumentFoldersGrpcMapper::output_success(output))
    }

    pub async fn create_document_folder(
        &self,
        actor: String,
        request: ServiceRequest<'_, CreateDocumentFolderRequest>,
    ) -> ServiceResult<CreateDocumentFolderResponse> {
        let input = CreateDocumentFolderGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .document
            .create_document_folder
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(CreateDocumentFolderGrpcMapper::output_success(output))
    }

    pub async fn update_document_folder(
        &self,
        actor: String,
        request: ServiceRequest<'_, UpdateDocumentFolderRequest>,
    ) -> ServiceResult<UpdateDocumentFolderResponse> {
        let input = UpdateDocumentFolderGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .document
            .update_document_folder
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(UpdateDocumentFolderGrpcMapper::output_success(output))
    }

    pub async fn delete_document_folder(
        &self,
        actor: String,
        request: ServiceRequest<'_, DeleteDocumentFolderRequest>,
    ) -> ServiceResult<Empty> {
        let input = DeleteDocumentFolderGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        self
            .app_state
            .use_cases
            .document
            .delete_document_folder
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(Empty::default())
    }

    pub async fn list_documents(
        &self,
        request: ServiceRequest<'_, ListDocumentsRequest>,
    ) -> ServiceResult<ListDocumentsResponse> {
        let input = ListDocumentsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .document
            .list_documents
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListDocumentsGrpcMapper::output_success(output))
    }

    pub async fn create_document(
        &self,
        actor: String,
        request: ServiceRequest<'_, CreateDocumentRequest>,
    ) -> ServiceResult<CreateDocumentResponse> {
        let input = CreateDocumentGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .document
            .create_document
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(CreateDocumentGrpcMapper::output_success(output))
    }

    pub async fn complete_document(
        &self,
        actor: String,
        request: ServiceRequest<'_, CompleteDocumentRequest>,
    ) -> ServiceResult<CompleteDocumentResponse> {
        let input = CompleteDocumentGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .document
            .complete_document
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(CompleteDocumentGrpcMapper::output_success(output))
    }

    pub async fn get_document(
        &self,
        request: ServiceRequest<'_, GetDocumentRequest>,
    ) -> ServiceResult<GetDocumentResponse> {
        let input = GetDocumentGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .document
            .get_document
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetDocumentGrpcMapper::output_success(output))
    }

    pub async fn update_document(
        &self,
        actor: String,
        request: ServiceRequest<'_, UpdateDocumentRequest>,
    ) -> ServiceResult<UpdateDocumentResponse> {
        let input = UpdateDocumentGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .document
            .update_document
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(UpdateDocumentGrpcMapper::output_success(output))
    }

    pub async fn delete_document(
        &self,
        actor: String,
        request: ServiceRequest<'_, DeleteDocumentRequest>,
    ) -> ServiceResult<Empty> {
        let input = DeleteDocumentGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        self
            .app_state
            .use_cases
            .document
            .delete_document
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(Empty::default())
    }

    pub async fn download_document(
        &self,
        request: ServiceRequest<'_, DownloadDocumentRequest>,
    ) -> ServiceResult<DownloadDocumentResponse> {
        let input = DownloadDocumentGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .document
            .download_document
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(DownloadDocumentGrpcMapper::output_success(output))
    }

    pub async fn review_document(
        &self,
        actor: String,
        request: ServiceRequest<'_, ReviewDocumentRequest>,
    ) -> ServiceResult<ReviewDocumentResponse> {
        let input = ReviewDocumentGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .document
            .review_document
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ReviewDocumentGrpcMapper::output_success(output))
    }
}
