use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::notifications::{GetNotificationsInput, NotificationsOutput, ReplaceNotificationsInput};
use crate::domain::aep::ResourceIdentifier;
use crate::domain::notifications::{CHANNELS, NotificationTrigger};
use crate::dto::payloads::json::notifications::{NotificationsPathPayload, ReplaceNotificationsBodyPayload};
use crate::dto::presenters::json::notifications::{NotificationTriggerPresenter, NotificationsPresenter};

fn to_presenter(output: NotificationsOutput) -> NotificationsPresenter {
    let page = output.notifications;
    NotificationsPresenter {
        path: page.path(),
        version: page.version,
        channels: CHANNELS.to_vec(),
        triggers: page
            .triggers
            .into_iter()
            .map(|t| NotificationTriggerPresenter {
                key: t.key,
                label: t.label,
                email: t.email,
                sms: t.sms,
                push: t.push,
                in_app: t.in_app,
            })
            .collect(),
    }
}

pub struct GetNotificationsMapper;

impl GetNotificationsMapper {
    pub fn input(path: NotificationsPathPayload) -> Result<GetNotificationsInput, ApplicationError> {
        Ok(GetNotificationsInput { program: ResourceIdentifier::parse(&path.program) })
    }

    pub fn output_success(output: NotificationsOutput) -> NotificationsPresenter {
        to_presenter(output)
    }
}

pub struct ReplaceNotificationsMapper;

impl ReplaceNotificationsMapper {
    pub fn input(
        path: NotificationsPathPayload,
        body: ReplaceNotificationsBodyPayload,
        actor: String,
    ) -> Result<ReplaceNotificationsInput, ApplicationError> {
        Ok(ReplaceNotificationsInput {
            program: ResourceIdentifier::parse(&path.program),
            version: body.version,
            triggers: body
                .triggers
                .into_iter()
                .map(|t| NotificationTrigger {
                    key: t.key,
                    label: t.label,
                    email: t.email,
                    sms: t.sms,
                    push: t.push,
                    in_app: t.in_app,
                })
                .collect(),
            actor,
        })
    }

    pub fn output_success(output: NotificationsOutput) -> NotificationsPresenter {
        to_presenter(output)
    }
}
