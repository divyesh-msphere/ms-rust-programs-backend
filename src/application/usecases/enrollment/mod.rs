use crate::domain::enrollment::EnrollmentRepository;
use crate::domain::program::ProgramRepository;
use std::sync::Arc;

pub mod batch_create_enrollments;
pub mod create_enrollment;
pub mod get_enrollment;
pub mod list_enrollments;
pub mod update_enrollment;

use batch_create_enrollments::BatchCreateEnrollmentsUseCase;
use create_enrollment::CreateEnrollmentUseCase;
use get_enrollment::GetEnrollmentUseCase;
use list_enrollments::ListEnrollmentsUseCase;
use update_enrollment::UpdateEnrollmentUseCase;

pub struct EnrollmentUseCases {
    pub list_enrollments: ListEnrollmentsUseCase,
    pub get_enrollment: GetEnrollmentUseCase,
    pub create_enrollment: CreateEnrollmentUseCase,
    pub batch_create_enrollments: BatchCreateEnrollmentsUseCase,
    pub update_enrollment: UpdateEnrollmentUseCase,
}

impl EnrollmentUseCases {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, enrollment_repository: Arc<dyn EnrollmentRepository>) -> Self {
        Self {
            list_enrollments: ListEnrollmentsUseCase::new(program_repository.clone(), enrollment_repository.clone()),
            get_enrollment: GetEnrollmentUseCase::new(program_repository.clone(), enrollment_repository.clone()),
            create_enrollment: CreateEnrollmentUseCase::new(program_repository.clone(), enrollment_repository.clone()),
            batch_create_enrollments: BatchCreateEnrollmentsUseCase::new(program_repository.clone(), enrollment_repository.clone()),
            update_enrollment: UpdateEnrollmentUseCase::new(program_repository, enrollment_repository),
        }
    }
}
