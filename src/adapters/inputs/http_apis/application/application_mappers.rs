use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::application::create_application::CreateApplicationInput;
use crate::application::usecases::application::get_application::GetApplicationInput;
use crate::application::usecases::application::list_applications::{ListApplicationsInput, ListApplicationsOutput};
use crate::application::usecases::application::submit_application::SubmitApplicationInput;
use crate::application::usecases::application::withdraw_application::WithdrawApplicationInput;
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::application::{Application, allowed_actions};
use crate::dto::payloads::json::application::{
    CreateApplicationBodyPayload, CreateApplicationPathPayload, CreateApplicationQueryPayload, GetApplicationPayload,
    ListApplicationsPathPayload, ListApplicationsQueryPayload, SubmitApplicationPayload, WithdrawApplicationPayload,
};
use crate::dto::presenters::json::application::{ApplicationPresenter, ListApplicationsPresenter};
use serde_json::json;

pub fn to_presenter(application: Application) -> ApplicationPresenter {
    ApplicationPresenter {
        path: format!("programs/{}/applications/{}", application.program_id, application.id),
        id: application.id.to_string(),
        store_id: application.store_id,
        store_name: application.store_name,
        website_id: application.website_id,
        website_name: application.website_name,
        actions: allowed_actions(&application.status),
        status: application.status,
        current_step_id: application.current_step_id.map(|id| id.to_string()),
        current_step_name: application.current_step_name,
        answers: application.answers,
        requirements_snapshot: application.requirements_snapshot,
        submitted_time: application.submitted_time.map(format_datetime),
        decided_time: application.decided_time.map(format_datetime),
        create_time: format_datetime(application.create_time),
        update_time: format_datetime(application.update_time),
    }
}

pub struct ListApplicationsMapper;

impl ListApplicationsMapper {
    pub fn input(
        path: ListApplicationsPathPayload,
        query: ListApplicationsQueryPayload,
    ) -> Result<ListApplicationsInput, ApplicationError> {
        Ok(ListApplicationsInput {
            program: ResourceIdentifier::parse(&path.program),
            status: query.status,
            page_size: query.max_page_size,
            page_token: parse::optional_uuid("page_token", query.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListApplicationsOutput) -> ListApplicationsPresenter {
        ListApplicationsPresenter {
            results: output.applications.into_iter().map(to_presenter).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
        }
    }
}

pub struct GetApplicationMapper;

impl GetApplicationMapper {
    pub fn input(path: GetApplicationPayload) -> Result<GetApplicationInput, ApplicationError> {
        Ok(GetApplicationInput {
            program: ResourceIdentifier::parse(&path.program),
            application: parse::uuid("application", &path.application)?,
        })
    }

    pub fn output_success(output: Application) -> ApplicationPresenter {
        to_presenter(output)
    }
}

pub struct CreateApplicationMapper;

impl CreateApplicationMapper {
    pub fn input(
        path: CreateApplicationPathPayload,
        query: CreateApplicationQueryPayload,
        body: CreateApplicationBodyPayload,
        actor: String,
    ) -> Result<CreateApplicationInput, ApplicationError> {
        Ok(CreateApplicationInput {
            program: ResourceIdentifier::parse(&path.program),
            id: parse::optional_uuid("id", query.id.as_deref())?,
            store_id: body.store_id,
            store_name: body.store_name,
            website_id: body.website_id,
            website_name: body.website_name,
            answers: body.answers.unwrap_or_else(|| json!({})),
            actor,
        })
    }

    pub fn output_success(output: Application) -> ApplicationPresenter {
        to_presenter(output)
    }
}

pub struct SubmitApplicationMapper;

impl SubmitApplicationMapper {
    pub fn input(path: SubmitApplicationPayload, actor: String) -> Result<SubmitApplicationInput, ApplicationError> {
        Ok(SubmitApplicationInput {
            program: ResourceIdentifier::parse(&path.program),
            application: parse::uuid("application", &path.application)?,
            actor,
        })
    }

    pub fn output_success(output: Application) -> ApplicationPresenter {
        to_presenter(output)
    }
}

pub struct WithdrawApplicationMapper;

impl WithdrawApplicationMapper {
    pub fn input(path: WithdrawApplicationPayload, actor: String) -> Result<WithdrawApplicationInput, ApplicationError> {
        Ok(WithdrawApplicationInput {
            program: ResourceIdentifier::parse(&path.program),
            application: parse::uuid("application", &path.application)?,
            actor,
        })
    }

    pub fn output_success(output: Application) -> ApplicationPresenter {
        to_presenter(output)
    }
}
