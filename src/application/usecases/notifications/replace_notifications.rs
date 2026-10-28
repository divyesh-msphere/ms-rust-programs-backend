use super::{NotificationsOutput, ReplaceNotificationsInput};
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::notifications::{NotificationsRepository, validate_triggers};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

pub struct ReplaceNotificationsUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    notifications_repository: Arc<dyn NotificationsRepository>,
}

impl ReplaceNotificationsUseCase {
    pub fn new(program_repository: Arc<dyn ProgramRepository>, notifications_repository: Arc<dyn NotificationsRepository>) -> Self {
        Self { program_repository, notifications_repository }
    }
}

#[async_trait::async_trait]
impl UseCase<NotificationsOutput, ReplaceNotificationsInput> for ReplaceNotificationsUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<ReplaceNotificationsInput>) -> Result<NotificationsOutput, ApplicationError> {
        let input = input.ok_or_else(|| ApplicationError::validation_error("Input is required in ReplaceNotificationsUseCase"))?;

        let triggers = validate_triggers(input.triggers)?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("ReplaceNotificationsUseCase::get_program"))
            .await?;

        let notifications = self
            .notifications_repository
            .replace(program.id, input.version, triggers, &input.actor)
            .instrument(info_span!("ReplaceNotificationsUseCase::replace"))
            .await?;
        Ok(NotificationsOutput { notifications })
    }
}
