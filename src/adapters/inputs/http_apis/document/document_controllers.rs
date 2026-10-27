use crate::adapters::inputs::http_apis::document::document_mappers::{
    CompleteDocumentMapper, CreateDocumentFolderMapper, CreateDocumentMapper, DeleteDocumentFolderMapper,
    DeleteDocumentMapper, DownloadDocumentMapper, GetDocumentMapper, GetDocumentSummaryMapper, ListDocumentFoldersMapper,
    ListDocumentsMapper, ReviewDocumentMapper, ScanDocumentMapper, UpdateDocumentFolderMapper, UpdateDocumentMapper,
};
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::document::{
    CreateDocumentBodyPayload, CreateFolderBodyPayload, DocumentPathPayload, DocumentScanBodyPayload, FolderPathPayload,
    ListDocumentsQueryPayload, ProgramPathPayload, ReviewDocumentBodyPayload, UpdateDocumentBodyPayload,
    UpdateFolderBodyPayload,
};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_document_summary)
        .service(list_document_folders)
        .service(create_document_folder)
        .service(update_document_folder)
        .service(delete_document_folder)
        .service(list_documents)
        .service(create_document)
        .service(complete_document)
        .service(download_document)
        .service(review_document)
        .service(get_document)
        .service(update_document)
        .service(delete_document);
}

pub fn webhook_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(document_scan);
}

/// Runs one document use case: build the input, execute, map either error.
macro_rules! run {
    ($data:expr, $op:ident, $input:expr) => {
        $data
            .use_cases
            .document
            .$op
            .execute(Some($input.map_err(ErrorResponse::map_response_error)?))
            .await
            .map_err(ErrorResponse::map_response_error)?
    };
}

/// The header, the left panel and the footer of the page in one call.
#[instrument(skip(data))]
#[get("/{program}/documents:summary")]
pub async fn get_document_summary(data: web::Data<AppState>, path: web::Path<ProgramPathPayload>) -> Result<HttpResponse, ErrorResponse> {
    info!("GetDocumentSummaryController called");
    let result = run!(data, get_document_summary, GetDocumentSummaryMapper::input(path.into_inner()));
    Ok(HttpResponse::Ok().json(GetDocumentSummaryMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{program}/document-folders")]
pub async fn list_document_folders(data: web::Data<AppState>, path: web::Path<ProgramPathPayload>) -> Result<HttpResponse, ErrorResponse> {
    info!("ListDocumentFoldersController called");
    let result = run!(data, list_document_folders, ListDocumentFoldersMapper::input(path.into_inner()));
    Ok(HttpResponse::Ok().json(ListDocumentFoldersMapper::output_success(result)))
}

#[instrument(skip(data, req, body))]
#[post("/{program}/document-folders")]
pub async fn create_document_folder(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<ProgramPathPayload>,
    body: web::Json<CreateFolderBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("CreateDocumentFolderController called");
    let result = run!(data, create_document_folder, CreateDocumentFolderMapper::input(path.into_inner(), body.into_inner(), actor_from(&req)));
    Ok(HttpResponse::Ok().json(CreateDocumentFolderMapper::output_success(result)))
}

#[instrument(skip(data, req, body))]
#[patch("/{program}/document-folders/{folder}")]
pub async fn update_document_folder(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<FolderPathPayload>,
    body: web::Json<UpdateFolderBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("UpdateDocumentFolderController called");
    let result = run!(data, update_document_folder, UpdateDocumentFolderMapper::input(path.into_inner(), body.into_inner(), actor_from(&req)));
    Ok(HttpResponse::Ok().json(UpdateDocumentFolderMapper::output_success(result)))
}

#[instrument(skip(data, req))]
#[delete("/{program}/document-folders/{folder}")]
pub async fn delete_document_folder(req: HttpRequest, data: web::Data<AppState>, path: web::Path<FolderPathPayload>) -> Result<HttpResponse, ErrorResponse> {
    info!("DeleteDocumentFolderController called");
    run!(data, delete_document_folder, DeleteDocumentFolderMapper::input(path.into_inner(), actor_from(&req)));
    Ok(HttpResponse::NoContent().finish())
}

/// The file list: All files, Recent (`location=recent`), Needs review, Quarantined, one folder, a name search.
#[instrument(skip(data))]
#[get("/{program}/documents")]
pub async fn list_documents(
    data: web::Data<AppState>,
    path: web::Path<ProgramPathPayload>,
    query: web::Query<ListDocumentsQueryPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListDocumentsController called");
    let result = run!(data, list_documents, ListDocumentsMapper::input(path.into_inner(), query.into_inner()));
    Ok(HttpResponse::Ok().json(ListDocumentsMapper::output_success(result)))
}

/// Step 1 of an upload: the row (hidden, `uploading`) and the presigned URL the browser sends the file to.
#[instrument(skip(data, req, body))]
#[post("/{program}/documents")]
pub async fn create_document(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<ProgramPathPayload>,
    body: web::Json<CreateDocumentBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("CreateDocumentController called");
    let result = run!(data, create_document, CreateDocumentMapper::input(path.into_inner(), body.into_inner(), actor_from(&req)));
    Ok(HttpResponse::Ok().json(CreateDocumentMapper::output_success(result)))
}

/// Step 3 of an upload: the server asks the storage what arrived and checks size, type and checksum.
#[instrument(skip(data, req))]
#[post("/{program}/documents/{document}:complete")]
pub async fn complete_document(req: HttpRequest, data: web::Data<AppState>, path: web::Path<DocumentPathPayload>) -> Result<HttpResponse, ErrorResponse> {
    info!("CompleteDocumentController called");
    let result = run!(data, complete_document, CompleteDocumentMapper::input(path.into_inner(), actor_from(&req)));
    Ok(HttpResponse::Ok().json(CompleteDocumentMapper::output_success(result)))
}

/// A short-lived link to download the file (not allowed while it is quarantined).
#[instrument(skip(data))]
#[post("/{program}/documents/{document}:download")]
pub async fn download_document(data: web::Data<AppState>, path: web::Path<DocumentPathPayload>) -> Result<HttpResponse, ErrorResponse> {
    info!("DownloadDocumentController called");
    let result = run!(data, download_document, DownloadDocumentMapper::input(path.into_inner()));
    Ok(HttpResponse::Ok().json(DownloadDocumentMapper::output_success(result)))
}

/// An admin's decision on a file that needs review.
#[instrument(skip(data, req, body))]
#[post("/{program}/documents/{document}:review")]
pub async fn review_document(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<DocumentPathPayload>,
    body: web::Json<ReviewDocumentBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ReviewDocumentController called");
    let result = run!(data, review_document, ReviewDocumentMapper::input(path.into_inner(), body.into_inner(), actor_from(&req)));
    Ok(HttpResponse::Ok().json(ReviewDocumentMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{program}/documents/{document}")]
pub async fn get_document(data: web::Data<AppState>, path: web::Path<DocumentPathPayload>) -> Result<HttpResponse, ErrorResponse> {
    info!("GetDocumentController called");
    let result = run!(data, get_document, GetDocumentMapper::input(path.into_inner()));
    Ok(HttpResponse::Ok().json(GetDocumentMapper::output_success(result)))
}

/// Rename and / or move to another folder.
#[instrument(skip(data, req, body))]
#[patch("/{program}/documents/{document}")]
pub async fn update_document(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<DocumentPathPayload>,
    body: web::Json<UpdateDocumentBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("UpdateDocumentController called");
    let result = run!(data, update_document, UpdateDocumentMapper::input(path.into_inner(), body.into_inner(), actor_from(&req)));
    Ok(HttpResponse::Ok().json(UpdateDocumentMapper::output_success(result)))
}

#[instrument(skip(data, req))]
#[delete("/{program}/documents/{document}")]
pub async fn delete_document(req: HttpRequest, data: web::Data<AppState>, path: web::Path<DocumentPathPayload>) -> Result<HttpResponse, ErrorResponse> {
    info!("DeleteDocumentController called");
    run!(data, delete_document, DeleteDocumentMapper::input(path.into_inner(), actor_from(&req)));
    Ok(HttpResponse::NoContent().finish())
}

/// The scanner's callback: `{document_id, result: clean|suspicious|infected, scanner}`. Answers 204. A file
/// that is not waiting for a scan is left as it is, so a repeated callback is harmless.
#[instrument(skip(data, req, body))]
#[post("/document-scan")]
pub async fn document_scan(
    req: HttpRequest,
    data: web::Data<AppState>,
    body: web::Json<DocumentScanBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("DocumentScanWebhook called");
    let secret = &data.application_config.storage.scan_webhook_secret;
    if !secret.is_empty() {
        let given = req.headers().get("x-webhook-secret").and_then(|v| v.to_str().ok()).unwrap_or_default();
        if given != secret {
            return Err(ErrorResponse::map_response_error(ApplicationError::unauthenticated("invalid or missing X-Webhook-Secret")));
        }
    }
    run!(data, scan_document, ScanDocumentMapper::input(body.into_inner()));
    Ok(HttpResponse::NoContent().finish())
}
