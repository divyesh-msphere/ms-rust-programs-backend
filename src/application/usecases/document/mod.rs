use crate::domain::document::{DocumentPolicy, DocumentRepository};
use crate::domain::object_storage::ObjectStorage;
use crate::domain::program::ProgramRepository;
use std::sync::Arc;

pub mod get_document_summary;
pub mod list_document_folders;
pub mod create_document_folder;
pub mod update_document_folder;
pub mod delete_document_folder;
pub mod list_documents;
pub mod create_document;
pub mod complete_document;
pub mod get_document;
pub mod update_document;
pub mod delete_document;
pub mod download_document;
pub mod review_document;
pub mod scan_document;

use get_document_summary::GetDocumentSummaryUseCase;
use list_document_folders::ListDocumentFoldersUseCase;
use create_document_folder::CreateDocumentFolderUseCase;
use update_document_folder::UpdateDocumentFolderUseCase;
use delete_document_folder::DeleteDocumentFolderUseCase;
use list_documents::ListDocumentsUseCase;
use create_document::CreateDocumentUseCase;
use complete_document::CompleteDocumentUseCase;
use get_document::GetDocumentUseCase;
use update_document::UpdateDocumentUseCase;
use delete_document::DeleteDocumentUseCase;
use download_document::DownloadDocumentUseCase;
use review_document::ReviewDocumentUseCase;
use scan_document::ScanDocumentUseCase;

pub struct DocumentUseCases {
    pub get_document_summary: GetDocumentSummaryUseCase,
    pub list_document_folders: ListDocumentFoldersUseCase,
    pub create_document_folder: CreateDocumentFolderUseCase,
    pub update_document_folder: UpdateDocumentFolderUseCase,
    pub delete_document_folder: DeleteDocumentFolderUseCase,
    pub list_documents: ListDocumentsUseCase,
    pub create_document: CreateDocumentUseCase,
    pub complete_document: CompleteDocumentUseCase,
    pub get_document: GetDocumentUseCase,
    pub update_document: UpdateDocumentUseCase,
    pub delete_document: DeleteDocumentUseCase,
    pub download_document: DownloadDocumentUseCase,
    pub review_document: ReviewDocumentUseCase,
    pub scan_document: ScanDocumentUseCase,
}

impl DocumentUseCases {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        document_repository: Arc<dyn DocumentRepository>,
        object_storage: Arc<dyn ObjectStorage>,
        policy: DocumentPolicy,
    ) -> Self {
        Self {
            get_document_summary: GetDocumentSummaryUseCase::new(program_repository.clone(), document_repository.clone()),
            list_document_folders: ListDocumentFoldersUseCase::new(program_repository.clone(), document_repository.clone()),
            create_document_folder: CreateDocumentFolderUseCase::new(program_repository.clone(), document_repository.clone()),
            update_document_folder: UpdateDocumentFolderUseCase::new(program_repository.clone(), document_repository.clone()),
            delete_document_folder: DeleteDocumentFolderUseCase::new(program_repository.clone(), document_repository.clone()),
            list_documents: ListDocumentsUseCase::new(program_repository.clone(), document_repository.clone()),
            create_document: CreateDocumentUseCase::new(program_repository.clone(), document_repository.clone(), object_storage.clone(), policy.clone()),
            complete_document: CompleteDocumentUseCase::new(program_repository.clone(), document_repository.clone(), object_storage.clone()),
            get_document: GetDocumentUseCase::new(program_repository.clone(), document_repository.clone()),
            update_document: UpdateDocumentUseCase::new(program_repository.clone(), document_repository.clone()),
            delete_document: DeleteDocumentUseCase::new(program_repository.clone(), document_repository.clone()),
            download_document: DownloadDocumentUseCase::new(program_repository.clone(), document_repository.clone(), object_storage.clone()),
            review_document: ReviewDocumentUseCase::new(program_repository.clone(), document_repository.clone()),
            scan_document: ScanDocumentUseCase::new(document_repository),
        }
    }
}
