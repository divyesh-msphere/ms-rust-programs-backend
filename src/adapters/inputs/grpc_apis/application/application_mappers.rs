use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    Application as GrpcApplication, CreateApplicationRequest, CreateApplicationResponse, GetApplicationRequest,
    GetApplicationResponse, ListApplicationsRequest, ListApplicationsResponse, SubmitApplicationRequest,
    SubmitApplicationResponse, WithdrawApplicationRequest, WithdrawApplicationResponse,
};
use crate::adapters::inputs::grpc_apis::shared::json_struct::{buffa_value_to_json, json_to_buffa_value};
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
use serde_json::json;

fn to_grpc_application(application: Application) -> GrpcApplication {
    GrpcApplication {
        path: format!("programs/{}/applications/{}", application.program_id, application.id),
        id: application.id.to_string(),
        store_id: application.store_id,
        store_name: application.store_name,
        website_id: application.website_id,
        website_name: application.website_name,
        actions: allowed_actions(&application.status, application.current_step_id.is_some()).into_iter().map(String::from).collect(),
        status: application.status,
        current_step_id: application.current_step_id.map(|id| id.to_string()),
        current_step_name: application.current_step_name,
        answers: application.answers.map(json_to_buffa_value).into(),
        requirements_snapshot: application.requirements_snapshot.map(json_to_buffa_value).into(),
        submitted_time: application.submitted_time.map(format_datetime),
        decided_time: application.decided_time.map(format_datetime),
        create_time: format_datetime(application.create_time),
        update_time: format_datetime(application.update_time),
        ..Default::default()
    }
}

pub struct ListApplicationsGrpcMapper;

impl ListApplicationsGrpcMapper {
    pub fn input(request: ListApplicationsRequest) -> Result<ListApplicationsInput, ApplicationError> {
        Ok(ListApplicationsInput {
            program: ResourceIdentifier::parse(&request.program),
            status: request.status,
            page_size: request.max_page_size,
            page_token: parse::optional_uuid("page_token", request.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListApplicationsOutput) -> ListApplicationsResponse {
        ListApplicationsResponse {
            results: output.applications.into_iter().map(to_grpc_application).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            ..Default::default()
        }
    }
}

pub struct GetApplicationGrpcMapper;

impl GetApplicationGrpcMapper {
    pub fn input(request: GetApplicationRequest) -> Result<GetApplicationInput, ApplicationError> {
        Ok(GetApplicationInput {
            program: ResourceIdentifier::parse(&request.program),
            application: parse::uuid("application", &request.application)?,
        })
    }

    pub fn output_success(output: Application) -> GetApplicationResponse {
        GetApplicationResponse { application: Some(to_grpc_application(output)).into(), ..Default::default() }
    }
}

pub struct CreateApplicationGrpcMapper;

impl CreateApplicationGrpcMapper {
    pub fn input(request: CreateApplicationRequest, actor: String) -> Result<CreateApplicationInput, ApplicationError> {
        Ok(CreateApplicationInput {
            program: ResourceIdentifier::parse(&request.program),
            id: parse::optional_uuid("id", request.id.as_deref())?,
            store_id: request.store_id,
            store_name: request.store_name,
            website_id: request.website_id,
            website_name: request.website_name,
            answers: request.answers.into_option().map(buffa_value_to_json).unwrap_or_else(|| json!({})),
            actor,
        })
    }

    pub fn output_success(output: Application) -> CreateApplicationResponse {
        CreateApplicationResponse { application: Some(to_grpc_application(output)).into(), ..Default::default() }
    }
}

pub struct SubmitApplicationGrpcMapper;

impl SubmitApplicationGrpcMapper {
    pub fn input(request: SubmitApplicationRequest, actor: String) -> Result<SubmitApplicationInput, ApplicationError> {
        Ok(SubmitApplicationInput {
            program: ResourceIdentifier::parse(&request.program),
            application: parse::uuid("application", &request.application)?,
            actor,
        })
    }

    pub fn output_success(output: Application) -> SubmitApplicationResponse {
        SubmitApplicationResponse { application: Some(to_grpc_application(output)).into(), ..Default::default() }
    }
}

pub struct WithdrawApplicationGrpcMapper;

impl WithdrawApplicationGrpcMapper {
    pub fn input(request: WithdrawApplicationRequest, actor: String) -> Result<WithdrawApplicationInput, ApplicationError> {
        Ok(WithdrawApplicationInput {
            program: ResourceIdentifier::parse(&request.program),
            application: parse::uuid("application", &request.application)?,
            actor,
        })
    }

    pub fn output_success(output: Application) -> WithdrawApplicationResponse {
        WithdrawApplicationResponse { application: Some(to_grpc_application(output)).into(), ..Default::default() }
    }
}
