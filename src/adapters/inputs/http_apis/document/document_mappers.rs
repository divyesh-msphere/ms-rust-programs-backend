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
use crate::application::usecases::document::scan_document::ScanDocumentInput;
use crate::application::usecases::document::update_document::UpdateDocumentInput;
use crate::application::usecases::document::update_document_folder::UpdateDocumentFolderInput;
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::document::{Document, DocumentSummary, Folder};
use crate::domain::object_storage::PresignedRequest;
use crate::dto::payloads::json::document::{
    CreateDocumentBodyPayload, CreateFolderBodyPayload, DocumentPathPayload, DocumentScanBodyPayload, FolderPathPayload,
    ListDocumentsQueryPayload, ProgramPathPayload, ReviewDocumentBodyPayload, UpdateDocumentBodyPayload,
    UpdateFolderBodyPayload,
};
use crate::dto::presenters::json::document::{
    CreateDocumentPresenter, DocumentPresenter, DocumentSummaryPresenter, DocumentsPresenter, DownloadDocumentPresenter,
    FolderPresenter, FoldersPresenter, LocationsPresenter, StorageRequestPresenter,
};

pub fn folder_presenter(folder: Folder) -> FolderPresenter {
    FolderPresenter {
        path: format!("programs/{}/document-folders/{}", folder.program_id, folder.id),
        id: folder.id.to_string(),
        parent_id: folder.parent_id.map(|id| id.to_string()),
        name: folder.name,
        key: folder.key,
        system: folder.system,
        document_count: folder.document_count,
        create_time: format_datetime(folder.create_time),
        update_time: format_datetime(folder.update_time),
    }
}

pub fn document_presenter(document: Document) -> DocumentPresenter {
    DocumentPresenter {
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
    }
}

fn storage_request_presenter(request: PresignedRequest) -> StorageRequestPresenter {
    StorageRequestPresenter {
        url: request.url,
        method: request.method,
        headers: request.headers.into_iter().collect(),
        expires_time: format_datetime(request.expires_time),
    }
}

fn program(path: &str) -> ResourceIdentifier {
    ResourceIdentifier::parse(path)
}

pub struct GetDocumentSummaryMapper;

impl GetDocumentSummaryMapper {
    pub fn input(path: ProgramPathPayload) -> Result<GetDocumentSummaryInput, ApplicationError> {
        Ok(GetDocumentSummaryInput { program: program(&path.program) })
    }

    pub fn output_success(output: DocumentSummary) -> DocumentSummaryPresenter {
        DocumentSummaryPresenter {
            files: output.files,
            bytes: output.bytes,
            locations: LocationsPresenter {
                all: output.files,
                recent: output.recent(),
                needs_review: output.needs_review,
                quarantined: output.quarantined,
            },
            folders: output.folders.into_iter().map(folder_presenter).collect(),
        }
    }
}

pub struct ListDocumentFoldersMapper;

impl ListDocumentFoldersMapper {
    pub fn input(path: ProgramPathPayload) -> Result<ListDocumentFoldersInput, ApplicationError> {
        Ok(ListDocumentFoldersInput { program: program(&path.program) })
    }

    pub fn output_success(output: Vec<Folder>) -> FoldersPresenter {
        FoldersPresenter { results: output.into_iter().map(folder_presenter).collect() }
    }
}

pub struct CreateDocumentFolderMapper;

impl CreateDocumentFolderMapper {
    pub fn input(path: ProgramPathPayload, body: CreateFolderBodyPayload, actor: String) -> Result<CreateDocumentFolderInput, ApplicationError> {
        Ok(CreateDocumentFolderInput {
            program: program(&path.program),
            parent_id: parse::optional_uuid("parent_id", body.parent_id.as_deref())?,
            name: body.name,
            actor,
        })
    }

    pub fn output_success(output: Folder) -> FolderPresenter {
        folder_presenter(output)
    }
}

pub struct UpdateDocumentFolderMapper;

impl UpdateDocumentFolderMapper {
    pub fn input(path: FolderPathPayload, body: UpdateFolderBodyPayload, actor: String) -> Result<UpdateDocumentFolderInput, ApplicationError> {
        Ok(UpdateDocumentFolderInput {
            program: program(&path.program),
            folder: parse::uuid("folder", &path.folder)?,
            name: body.name,
            actor,
        })
    }

    pub fn output_success(output: Folder) -> FolderPresenter {
        folder_presenter(output)
    }
}

pub struct DeleteDocumentFolderMapper;

impl DeleteDocumentFolderMapper {
    pub fn input(path: FolderPathPayload, actor: String) -> Result<DeleteDocumentFolderInput, ApplicationError> {
        Ok(DeleteDocumentFolderInput { program: program(&path.program), folder: parse::uuid("folder", &path.folder)?, actor })
    }
}

pub struct ListDocumentsMapper;

impl ListDocumentsMapper {
    pub fn input(path: ProgramPathPayload, query: ListDocumentsQueryPayload) -> Result<ListDocumentsInput, ApplicationError> {
        Ok(ListDocumentsInput {
            program: program(&path.program),
            folder_id: parse::optional_uuid("folder_id", query.folder_id.as_deref())?,
            location: query.location,
            q: query.q,
            page_size: query.max_page_size,
            page_token: parse::optional_uuid("page_token", query.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListDocumentsOutput) -> DocumentsPresenter {
        DocumentsPresenter {
            results: output.documents.into_iter().map(document_presenter).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
        }
    }
}

pub struct CreateDocumentMapper;

impl CreateDocumentMapper {
    pub fn input(path: ProgramPathPayload, body: CreateDocumentBodyPayload, actor: String) -> Result<CreateDocumentInput, ApplicationError> {
        Ok(CreateDocumentInput {
            program: program(&path.program),
            folder_id: parse::optional_uuid("folder_id", body.folder_id.as_deref())?,
            name: body.name,
            content_type: body.content_type,
            size_bytes: body.size_bytes,
            checksum: body.checksum.map(|c| c.trim().to_string()).filter(|c| !c.is_empty()),
            actor,
        })
    }

    pub fn output_success(output: CreateDocumentOutput) -> CreateDocumentPresenter {
        CreateDocumentPresenter { document: document_presenter(output.document), upload: storage_request_presenter(output.upload) }
    }
}

pub struct CompleteDocumentMapper;

impl CompleteDocumentMapper {
    pub fn input(path: DocumentPathPayload, actor: String) -> Result<CompleteDocumentInput, ApplicationError> {
        Ok(CompleteDocumentInput { program: program(&path.program), document: parse::uuid("document", &path.document)?, actor })
    }

    pub fn output_success(output: Document) -> DocumentPresenter {
        document_presenter(output)
    }
}

pub struct GetDocumentMapper;

impl GetDocumentMapper {
    pub fn input(path: DocumentPathPayload) -> Result<GetDocumentInput, ApplicationError> {
        Ok(GetDocumentInput { program: program(&path.program), document: parse::uuid("document", &path.document)? })
    }

    pub fn output_success(output: Document) -> DocumentPresenter {
        document_presenter(output)
    }
}

pub struct UpdateDocumentMapper;

impl UpdateDocumentMapper {
    pub fn input(path: DocumentPathPayload, body: UpdateDocumentBodyPayload, actor: String) -> Result<UpdateDocumentInput, ApplicationError> {
        Ok(UpdateDocumentInput {
            program: program(&path.program),
            document: parse::uuid("document", &path.document)?,
            name: body.name,
            folder_id: parse::optional_uuid("folder_id", body.folder_id.as_deref())?,
            actor,
        })
    }

    pub fn output_success(output: Document) -> DocumentPresenter {
        document_presenter(output)
    }
}

pub struct DeleteDocumentMapper;

impl DeleteDocumentMapper {
    pub fn input(path: DocumentPathPayload, actor: String) -> Result<DeleteDocumentInput, ApplicationError> {
        Ok(DeleteDocumentInput { program: program(&path.program), document: parse::uuid("document", &path.document)?, actor })
    }
}

pub struct DownloadDocumentMapper;

impl DownloadDocumentMapper {
    pub fn input(path: DocumentPathPayload) -> Result<DownloadDocumentInput, ApplicationError> {
        Ok(DownloadDocumentInput { program: program(&path.program), document: parse::uuid("document", &path.document)? })
    }

    pub fn output_success(output: DownloadDocumentOutput) -> DownloadDocumentPresenter {
        DownloadDocumentPresenter { document: document_presenter(output.document), download: storage_request_presenter(output.request) }
    }
}

pub struct ReviewDocumentMapper;

impl ReviewDocumentMapper {
    pub fn input(path: DocumentPathPayload, body: ReviewDocumentBodyPayload, actor: String) -> Result<ReviewDocumentInput, ApplicationError> {
        Ok(ReviewDocumentInput {
            program: program(&path.program),
            document: parse::uuid("document", &path.document)?,
            decision: body.decision,
            note: body.note,
            actor,
        })
    }

    pub fn output_success(output: Document) -> DocumentPresenter {
        document_presenter(output)
    }
}

pub struct ScanDocumentMapper;

impl ScanDocumentMapper {
    pub fn input(body: DocumentScanBodyPayload) -> Result<ScanDocumentInput, ApplicationError> {
        Ok(ScanDocumentInput {
            document: parse::uuid("document_id", &body.document_id)?,
            result: body.result,
            scanner: body.scanner.unwrap_or_default(),
        })
    }
}
