use crate::domain::aep::ResourceIdentifier;
use crate::domain::notifications::{NotificationTrigger, Notifications, NotificationsRepository};
use crate::domain::program::ProgramRepository;
use std::sync::Arc;

pub mod get_notifications;
pub mod replace_notifications;

use get_notifications::GetNotificationsUseCase;
use replace_notifications::ReplaceNotificationsUseCase;

#[derive(Debug)]
pub struct GetNotificationsInput {
    pub program: ResourceIdentifier,
}

#[derive(Debug)]
pub struct ReplaceNotificationsInput {
    pub program: ResourceIdentifier,
    /// The `version` the caller last read; `None` skips the check.
    pub version: Option<i32>,
    pub triggers: Vec<NotificationTrigger>,
    pub actor: String,
}

#[derive(Debug)]
pub struct NotificationsOutput {
    pub notifications: Notifications,
}

pub struct NotificationsUseCases {
    pub get_notifications: GetNotificationsUseCase,
    pub replace_notifications: ReplaceNotificationsUseCase,
}

impl NotificationsUseCases {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        notifications_repository: Arc<dyn NotificationsRepository>,
    ) -> Self {
        Self {
            get_notifications: GetNotificationsUseCase::new(program_repository.clone(), notifications_repository.clone()),
            replace_notifications: ReplaceNotificationsUseCase::new(program_repository, notifications_repository),
        }
    }
}
