use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    GetNotificationsRequest, GetNotificationsResponse, NotificationTrigger as GrpcNotificationTrigger,
    Notifications as GrpcNotifications, ReplaceNotificationsRequest, ReplaceNotificationsResponse,
};
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::notifications::{GetNotificationsInput, NotificationsOutput, ReplaceNotificationsInput};
use crate::domain::aep::ResourceIdentifier;
use crate::domain::notifications::{CHANNELS, NotificationTrigger};

fn to_grpc_notifications(output: NotificationsOutput) -> GrpcNotifications {
    let page = output.notifications;
    GrpcNotifications {
        path: page.path(),
        version: page.version,
        channels: CHANNELS.iter().map(|c| c.to_string()).collect(),
        triggers: page
            .triggers
            .into_iter()
            .map(|t| GrpcNotificationTrigger {
                key: t.key,
                label: t.label,
                email: t.email,
                sms: t.sms,
                push: t.push,
                in_app: t.in_app,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

pub struct GetNotificationsGrpcMapper;

impl GetNotificationsGrpcMapper {
    pub fn input(request: GetNotificationsRequest) -> Result<GetNotificationsInput, ApplicationError> {
        Ok(GetNotificationsInput { program: ResourceIdentifier::parse(&request.program) })
    }

    pub fn output_success(output: NotificationsOutput) -> GetNotificationsResponse {
        GetNotificationsResponse { notifications: Some(to_grpc_notifications(output)).into(), ..Default::default() }
    }
}

pub struct ReplaceNotificationsGrpcMapper;

impl ReplaceNotificationsGrpcMapper {
    pub fn input(request: ReplaceNotificationsRequest, actor: String) -> Result<ReplaceNotificationsInput, ApplicationError> {
        Ok(ReplaceNotificationsInput {
            program: ResourceIdentifier::parse(&request.program),
            version: request.version,
            triggers: request
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

    pub fn output_success(output: NotificationsOutput) -> ReplaceNotificationsResponse {
        ReplaceNotificationsResponse { notifications: Some(to_grpc_notifications(output)).into(), ..Default::default() }
    }
}
