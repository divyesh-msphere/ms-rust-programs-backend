use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    CreateSignoffRequest, CreateSignoffResponse, GetSignoffRequest, GetSignoffResponse, ListSignoffsRequest,
    ListSignoffsResponse, Signoff as GrpcSignoff,
};
use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::signoff::create_signoff::CreateSignoffInput;
use crate::application::usecases::signoff::get_signoff::GetSignoffInput;
use crate::application::usecases::signoff::list_signoffs::ListSignoffsInput;
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::signoff::Signoff;

fn to_grpc_signoff(signoff: Signoff) -> GrpcSignoff {
    GrpcSignoff {
        path: format!("programs/{}/applications/{}/signoffs/{}", signoff.program_id, signoff.application_id, signoff.id),
        id: signoff.id.to_string(),
        application_id: signoff.application_id.to_string(),
        approval_step_id: signoff.approval_step_id.to_string(),
        approval_step_name: signoff.approval_step_name,
        decision: signoff.decision,
        comment: signoff.comment,
        actor: signoff.actor,
        create_time: format_datetime(signoff.create_time),
        ..Default::default()
    }
}

pub struct ListSignoffsGrpcMapper;

impl ListSignoffsGrpcMapper {
    pub fn input(request: ListSignoffsRequest) -> Result<ListSignoffsInput, ApplicationError> {
        Ok(ListSignoffsInput {
            program: ResourceIdentifier::parse(&request.program),
            application: parse::uuid("application", &request.application)?,
        })
    }

    pub fn output_success(output: Vec<Signoff>) -> ListSignoffsResponse {
        ListSignoffsResponse { results: output.into_iter().map(to_grpc_signoff).collect(), ..Default::default() }
    }
}

pub struct GetSignoffGrpcMapper;

impl GetSignoffGrpcMapper {
    pub fn input(request: GetSignoffRequest) -> Result<GetSignoffInput, ApplicationError> {
        Ok(GetSignoffInput {
            program: ResourceIdentifier::parse(&request.program),
            application: parse::uuid("application", &request.application)?,
            signoff: parse::uuid("signoff", &request.signoff)?,
        })
    }

    pub fn output_success(output: Signoff) -> GetSignoffResponse {
        GetSignoffResponse { signoff: Some(to_grpc_signoff(output)).into(), ..Default::default() }
    }
}

pub struct CreateSignoffGrpcMapper;

impl CreateSignoffGrpcMapper {
    pub fn input(request: CreateSignoffRequest, actor: String) -> Result<CreateSignoffInput, ApplicationError> {
        Ok(CreateSignoffInput {
            program: ResourceIdentifier::parse(&request.program),
            application: parse::uuid("application", &request.application)?,
            approval_step_id: parse::optional_uuid("approval_step_id", request.approval_step_id.as_deref())?,
            decision: request.decision.trim().to_string(),
            comment: request.comment,
            actor,
        })
    }

    pub fn output_success(output: Signoff) -> CreateSignoffResponse {
        CreateSignoffResponse { signoff: Some(to_grpc_signoff(output)).into(), ..Default::default() }
    }
}
