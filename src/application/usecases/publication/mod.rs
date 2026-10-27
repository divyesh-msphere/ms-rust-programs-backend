use crate::domain::program::ProgramRepository;
use crate::domain::publication::PublicationRepository;
use std::sync::Arc;

pub mod create_publication;
pub mod get_publication;
pub mod list_publications;

use create_publication::CreatePublicationUseCase;
use get_publication::GetPublicationUseCase;
use list_publications::ListPublicationsUseCase;

pub struct PublicationUseCases {
    pub list_publications: ListPublicationsUseCase,
    pub get_publication: GetPublicationUseCase,
    pub create_publication: CreatePublicationUseCase,
}

impl PublicationUseCases {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, publication_repository: Arc<dyn PublicationRepository>) -> Self {
        Self {
            list_publications: ListPublicationsUseCase::new(program_repository.clone(), publication_repository.clone()),
            get_publication: GetPublicationUseCase::new(program_repository.clone(), publication_repository.clone()),
            create_publication: CreatePublicationUseCase::new(program_repository, publication_repository),
        }
    }
}
