use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::signoff::create_signoff::CreateSignoffInput;
use crate::application::usecases::signoff::get_signoff::GetSignoffInput;
use crate::application::usecases::signoff::list_signoffs::ListSignoffsInput;
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::signoff::Signoff;
use crate::dto::payloads::json::signoff::{
    CreateSignoffBodyPayload, CreateSignoffPathPayload, GetSignoffPayload, ListSignoffsPayload,
};
use crate::dto::presenters::json::signoff::{SignoffPresenter, SignoffsPresenter};

pub fn to_presenter(signoff: Signoff) -> SignoffPresenter {
    SignoffPresenter {
        path: format!("programs/{}/applications/{}/signoffs/{}", signoff.program_id, signoff.application_id, signoff.id),
        id: signoff.id.to_string(),
        application_id: signoff.application_id.to_string(),
        approval_step_id: signoff.approval_step_id.to_string(),
        approval_step_name: signoff.approval_step_name,
        decision: signoff.decision,
        comment: signoff.comment,
        actor: signoff.actor,
        create_time: format_datetime(signoff.create_time),
    }
}

pub struct ListSignoffsMapper;

impl ListSignoffsMapper {
    pub fn input(path: ListSignoffsPayload) -> Result<ListSignoffsInput, ApplicationError> {
        Ok(ListSignoffsInput {
            program: ResourceIdentifier::parse(&path.program),
            application: parse::uuid("application", &path.application)?,
        })
    }

    pub fn output_success(output: Vec<Signoff>) -> SignoffsPresenter {
        SignoffsPresenter { results: output.into_iter().map(to_presenter).collect() }
    }
}

pub struct GetSignoffMapper;

impl GetSignoffMapper {
    pub fn input(path: GetSignoffPayload) -> Result<GetSignoffInput, ApplicationError> {
        Ok(GetSignoffInput {
            program: ResourceIdentifier::parse(&path.program),
            application: parse::uuid("application", &path.application)?,
            signoff: parse::uuid("signoff", &path.signoff)?,
        })
    }

    pub fn output_success(output: Signoff) -> SignoffPresenter {
        to_presenter(output)
    }
}

pub struct CreateSignoffMapper;

impl CreateSignoffMapper {
    pub fn input(
        path: CreateSignoffPathPayload,
        body: CreateSignoffBodyPayload,
        actor: String,
    ) -> Result<CreateSignoffInput, ApplicationError> {
        Ok(CreateSignoffInput {
            program: ResourceIdentifier::parse(&path.program),
            application: parse::uuid("application", &path.application)?,
            approval_step_id: parse::optional_uuid("approval_step_id", body.approval_step_id.as_deref())?,
            decision: body.decision.trim().to_string(),
            comment: body.comment,
            actor,
        })
    }

    pub fn output_success(output: Signoff) -> SignoffPresenter {
        to_presenter(output)
    }
}
