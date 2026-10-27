use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    CompleteDocumentRequest, CompleteDocumentResponse, CreateDocumentFolderRequest, CreateDocumentFolderResponse,
    CreateDocumentRequest, CreateDocumentResponse, DeleteDocumentFolderRequest, DeleteDocumentRequest, Document as GrpcDocument,
    DocumentFolder as GrpcFolder, DocumentLocations, DownloadDocumentRequest, DownloadDocumentResponse, GetDocumentRequest,
    GetDocumentResponse, GetDocumentSummaryRequest, GetDocumentSummaryResponse, ListDocumentFoldersRequest,
    ListDocumentFoldersResponse, ListDocumentsRequest, ListDocumentsResponse, ReviewDocumentRequest, ReviewDocumentResponse,
    StorageRequest, UpdateDocumentFolderRequest, UpdateDocumentFolderResponse, UpdateDocumentRequest, UpdateDocumentResponse,
};
use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::document::complete_document::CompleteDocumentInput;
use crate::application::usecases::document::create_document::{CreateDocumentInput, CreateDocumentOutput};
use crate::application::usecases::document::create_document_folder::CreateDocumentFolderInput;
use crate::application::usecases::document::delete_document::DeleteDocumentInput;
use crate::application::usecases::document::delete_document_folder::DeleteDocumentFolderInput;
use crate::application::usecases::document::download_document::{DownloadDocumentInput, DownloadDocumentOutput};
use crate::application::usecases::document::get_document::GetDocumentInput;
use crate::application::usecases::document::get_document_summary::GetDocumentSummaryInput;
use crate::application::usecases::document::list_document_folders::ListDocumentFoldersInput;
use crate::application::usecases::document::list_documents::{ListDocumentsInput, ListDocumentsOutput};
use crate::application::usecases::document::review_document::ReviewDocumentInput;
use crate::application::usecases::document::update_document::UpdateDocumentInput;
use crate::application::usecases::document::update_document_folder::UpdateDocumentFolderInput;
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{Document, DocumentSummary, Folder};
use crate::domain::object_storage::PresignedRequest;

fn to_grpc_folder(folder: Folder) -> GrpcFolder {
    GrpcFolder {
        path: format!("programs/{}/document-folders/{}", folder.program_id, folder.id),
        id: folder.id.to_string(),
        parent_id: folder.parent_id.map(|id| id.to_string()),
        name: folder.name,
        key: folder.key,
        system: folder.system,
        document_count: folder.document_count,
        create_time: format_datetime(folder.create_time),
        update_time: format_datetime(folder.update_time),
        ..Default::default()
    }
}

fn to_grpc_document(document: Document) -> GrpcDocument {
    GrpcDocument {
        path: format!("programs/{}/documents/{}", document.program_id, document.id),
        id: document.id.to_string(),
        name: document.name,
        folder_id: document.folder_id.map(|id| id.to_string()),
        content_type: document.content_type,
        size_bytes: document.size_bytes,
        status: document.status,
        source_type: document.source_type,
        source_id: document.source_id,
        uploaded_by: document.uploaded_by,
        checksum: document.checksum,
        scan_time: document.scan_time.map(format_datetime),
        review_note: document.review_note,
        reviewed_by: document.reviewed_by,
        reviewed_time: document.reviewed_time.map(format_datetime),
        create_time: format_datetime(document.create_time),
        update_time: format_datetime(document.update_time),
        ..Default::default()
    }
}

fn to_grpc_request(request: PresignedRequest) -> StorageRequest {
    StorageRequest {
        url: request.url,
        method: request.method,
        headers: request.headers.into_iter().collect(),
        expires_time: format_datetime(request.expires_time),
        ..Default::default()
    }
}

fn program(raw: &str) -> ResourceIdentifier {
    ResourceIdentifier::parse(raw)
}

pub struct GetDocumentSummaryGrpcMapper;

impl GetDocumentSummaryGrpcMapper {
    pub fn input(request: GetDocumentSummaryRequest) -> Result<GetDocumentSummaryInput, ApplicationError> {
        Ok(GetDocumentSummaryInput { program: program(&request.program) })
    }

    pub fn output_success(output: DocumentSummary) -> GetDocumentSummaryResponse {
        GetDocumentSummaryResponse {
            files: output.files,
            bytes: output.bytes,
            locations: Some(DocumentLocations {
                all: output.files,
                recent: output.recent(),
                needs_review: output.needs_review,
                quarantined: output.quarantined,
                ..Default::default()
            })
            .into(),
            folders: output.folders.into_iter().map(to_grpc_folder).collect(),
            ..Default::default()
        }
    }
}

pub struct ListDocumentFoldersGrpcMapper;

impl ListDocumentFoldersGrpcMapper {
    pub fn input(request: ListDocumentFoldersRequest) -> Result<ListDocumentFoldersInput, ApplicationError> {
        Ok(ListDocumentFoldersInput { program: program(&request.program) })
    }

    pub fn output_success(output: Vec<Folder>) -> ListDocumentFoldersResponse {
        ListDocumentFoldersResponse { results: output.into_iter().map(to_grpc_folder).collect(), ..Default::default() }
    }
}

pub struct CreateDocumentFolderGrpcMapper;

impl CreateDocumentFolderGrpcMapper {
    pub fn input(request: CreateDocumentFolderRequest, actor: String) -> Result<CreateDocumentFolderInput, ApplicationError> {
        Ok(CreateDocumentFolderInput {
            program: program(&request.program),
            parent_id: parse::optional_uuid("parent_id", request.parent_id.as_deref())?,
            name: request.name,
            actor,
        })
    }

    pub fn output_success(output: Folder) -> CreateDocumentFolderResponse {
        CreateDocumentFolderResponse { folder: Some(to_grpc_folder(output)).into(), ..Default::default() }
    }
}

pub struct UpdateDocumentFolderGrpcMapper;

impl UpdateDocumentFolderGrpcMapper {
    pub fn input(request: UpdateDocumentFolderRequest, actor: String) -> Result<UpdateDocumentFolderInput, ApplicationError> {
        Ok(UpdateDocumentFolderInput {
            program: program(&request.program),
            folder: parse::uuid("folder", &request.folder)?,
            name: request.name,
            actor,
        })
    }

    pub fn output_success(output: Folder) -> UpdateDocumentFolderResponse {
        UpdateDocumentFolderResponse { folder: Some(to_grpc_folder(output)).into(), ..Default::default() }
    }
}

pub struct DeleteDocumentFolderGrpcMapper;

impl DeleteDocumentFolderGrpcMapper {
    pub fn input(request: DeleteDocumentFolderRequest, actor: String) -> Result<DeleteDocumentFolderInput, ApplicationError> {
        Ok(DeleteDocumentFolderInput { program: program(&request.program), folder: parse::uuid("folder", &request.folder)?, actor })
    }
}

pub struct ListDocumentsGrpcMapper;

impl ListDocumentsGrpcMapper {
    pub fn input(request: ListDocumentsRequest) -> Result<ListDocumentsInput, ApplicationError> {
        Ok(ListDocumentsInput {
            program: program(&request.program),
            folder_id: parse::optional_uuid("folder_id", request.folder_id.as_deref())?,
            location: request.location,
            q: request.q,
            page_size: request.max_page_size,
            page_token: parse::optional_uuid("page_token", request.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListDocumentsOutput) -> ListDocumentsResponse {
        ListDocumentsResponse {
            results: output.documents.into_iter().map(to_grpc_document).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            ..Default::default()
        }
    }
}

pub struct CreateDocumentGrpcMapper;

impl CreateDocumentGrpcMapper {
    pub fn input(request: CreateDocumentRequest, actor: String) -> Result<CreateDocumentInput, ApplicationError> {
        Ok(CreateDocumentInput {
            program: program(&request.program),
            folder_id: parse::optional_uuid("folder_id", request.folder_id.as_deref())?,
            name: request.name,
            content_type: request.content_type,
            size_bytes: request.size_bytes,
            checksum: request.checksum.map(|c| c.trim().to_string()).filter(|c| !c.is_empty()),
            actor,
        })
    }

    pub fn output_success(output: CreateDocumentOutput) -> CreateDocumentResponse {
        CreateDocumentResponse {
            document: Some(to_grpc_document(output.document)).into(),
            upload: Some(to_grpc_request(output.upload)).into(),
            ..Default::default()
        }
    }
}

pub struct CompleteDocumentGrpcMapper;

impl CompleteDocumentGrpcMapper {
    pub fn input(request: CompleteDocumentRequest, actor: String) -> Result<CompleteDocumentInput, ApplicationError> {
        Ok(CompleteDocumentInput { program: program(&request.program), document: parse::uuid("document", &request.document)?, actor })
    }

    pub fn output_success(output: Document) -> CompleteDocumentResponse {
        CompleteDocumentResponse { document: Some(to_grpc_document(output)).into(), ..Default::default() }
    }
}

pub struct GetDocumentGrpcMapper;

impl GetDocumentGrpcMapper {
    pub fn input(request: GetDocumentRequest) -> Result<GetDocumentInput, ApplicationError> {
        Ok(GetDocumentInput { program: program(&request.program), document: parse::uuid("document", &request.document)? })
    }

    pub fn output_success(output: Document) -> GetDocumentResponse {
        GetDocumentResponse { document: Some(to_grpc_document(output)).into(), ..Default::default() }
    }
}

pub struct UpdateDocumentGrpcMapper;

impl UpdateDocumentGrpcMapper {
    pub fn input(request: UpdateDocumentRequest, actor: String) -> Result<UpdateDocumentInput, ApplicationError> {
        Ok(UpdateDocumentInput {
            program: program(&request.program),
            document: parse::uuid("document", &request.document)?,
            name: request.name,
            folder_id: parse::optional_uuid("folder_id", request.folder_id.as_deref())?,
            actor,
        })
    }

    pub fn output_success(output: Document) -> UpdateDocumentResponse {
        UpdateDocumentResponse { document: Some(to_grpc_document(output)).into(), ..Default::default() }
    }
}

pub struct DeleteDocumentGrpcMapper;

impl DeleteDocumentGrpcMapper {
    pub fn input(request: DeleteDocumentRequest, actor: String) -> Result<DeleteDocumentInput, ApplicationError> {
        Ok(DeleteDocumentInput { program: program(&request.program), document: parse::uuid("document", &request.document)?, actor })
    }
}

pub struct DownloadDocumentGrpcMapper;

impl DownloadDocumentGrpcMapper {
    pub fn input(request: DownloadDocumentRequest) -> Result<DownloadDocumentInput, ApplicationError> {
        Ok(DownloadDocumentInput { program: program(&request.program), document: parse::uuid("document", &request.document)? })
    }

    pub fn output_success(output: DownloadDocumentOutput) -> DownloadDocumentResponse {
        DownloadDocumentResponse {
            document: Some(to_grpc_document(output.document)).into(),
            download: Some(to_grpc_request(output.request)).into(),
            ..Default::default()
        }
    }
}

pub struct ReviewDocumentGrpcMapper;

impl ReviewDocumentGrpcMapper {
    pub fn input(request: ReviewDocumentRequest, actor: String) -> Result<ReviewDocumentInput, ApplicationError> {
        Ok(ReviewDocumentInput {
            program: program(&request.program),
            document: parse::uuid("document", &request.document)?,
            decision: request.decision,
            note: request.note,
            actor,
        })
    }

    pub fn output_success(output: Document) -> ReviewDocumentResponse {
        ReviewDocumentResponse { document: Some(to_grpc_document(output)).into(), ..Default::default() }
    }
}
