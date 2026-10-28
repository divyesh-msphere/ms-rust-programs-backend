use super::{GetNotificationsInput, NotificationsOutput};
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::notifications::NotificationsRepository;
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

pub struct GetNotificationsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    notifications_repository: Arc<dyn NotificationsRepository>,
}

impl GetNotificationsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, notifications_repository: Arc<dyn NotificationsRepository>) -> Self {
        Self { program_repository, notifications_repository }
    }
}

#[async_trait::async_trait]
impl UseCase<NotificationsOutput, GetNotificationsInput> for GetNotificationsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetNotificationsInput>) -> Result<NotificationsOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in GetNotificationsUseCase"))?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetNotificationsUseCase::get_program"))
            .await?;

        let notifications = self
            .notifications_repository
            .get(program.id)
            .instrument(info_span!("GetNotificationsUseCase::get"))
            .await?;
        Ok(NotificationsOutput { notifications })
    }
}
