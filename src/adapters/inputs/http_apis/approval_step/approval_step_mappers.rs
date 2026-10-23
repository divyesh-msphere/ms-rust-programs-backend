use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::approval_step::batch_update_approval_steps::BatchUpdateApprovalStepsInput;
use crate::application::usecases::approval_step::create_approval_step::CreateApprovalStepInput;
use crate::application::usecases::approval_step::delete_approval_step::DeleteApprovalStepInput;
use crate::application::usecases::approval_step::get_approval_step::GetApprovalStepInput;
use crate::application::usecases::approval_step::list_approval_steps::ListApprovalStepsInput;
use crate::application::usecases::approval_step::update_approval_step::UpdateApprovalStepInput;
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::approval_step::ApprovalStep;
use crate::dto::payloads::json::approval_step::{
    BatchUpdateApprovalStepsBodyPayload, BatchUpdateApprovalStepsPathPayload, CreateApprovalStepBodyPayload,
    CreateApprovalStepPathPayload, CreateApprovalStepQueryPayload, DeleteApprovalStepPayload, GetApprovalStepPayload,
    ListApprovalStepsPayload, UpdateApprovalStepBodyPayload, UpdateApprovalStepPathPayload,
};
use crate::dto::presenters::json::approval_step::{ApprovalStepPresenter, ApprovalStepsPresenter};

pub fn to_presenter(approval_step: ApprovalStep) -> ApprovalStepPresenter {
    ApprovalStepPresenter {
        path: format!("programs/{}/approval-steps/{}", approval_step.program_id, approval_step.id),
        id: approval_step.id.to_string(),
        name: approval_step.name,
        approver_role: approval_step.approver_role,
        position: approval_step.position,
        create_time: format_datetime(approval_step.create_time),
        update_time: format_datetime(approval_step.update_time),
    }
}

pub struct ListApprovalStepsMapper;

impl ListApprovalStepsMapper {
    pub fn input(path: ListApprovalStepsPayload) -> Result<ListApprovalStepsInput, ApplicationError> {
        Ok(ListApprovalStepsInput { program: ResourceIdentifier::parse(&path.program) })
    }

    pub fn output_success(output: Vec<ApprovalStep>) -> ApprovalStepsPresenter {
        ApprovalStepsPresenter { results: output.into_iter().map(to_presenter).collect() }
    }
}

pub struct GetApprovalStepMapper;

impl GetApprovalStepMapper {
    pub fn input(path: GetApprovalStepPayload) -> Result<GetApprovalStepInput, ApplicationError> {
        Ok(GetApprovalStepInput {
            program: ResourceIdentifier::parse(&path.program),
            approval_step: parse::uuid("approval_step", &path.approval_step)?,
        })
    }

    pub fn output_success(output: ApprovalStep) -> ApprovalStepPresenter {
        to_presenter(output)
    }
}

pub struct CreateApprovalStepMapper;

impl CreateApprovalStepMapper {
    pub fn input(
        path: CreateApprovalStepPathPayload,
        query: CreateApprovalStepQueryPayload,
        body: CreateApprovalStepBodyPayload,
        actor: String,
    ) -> Result<CreateApprovalStepInput, ApplicationError> {
        Ok(CreateApprovalStepInput {
            program: ResourceIdentifier::parse(&path.program),
            id: parse::optional_uuid("id", query.id.as_deref())?,
            name: body.name,
            approver_role: body.approver_role,
            position: body.position,
            actor,
        })
    }

    pub fn output_success(output: ApprovalStep) -> ApprovalStepPresenter {
        to_presenter(output)
    }
}

pub struct UpdateApprovalStepMapper;

impl UpdateApprovalStepMapper {
    pub fn input(
        path: UpdateApprovalStepPathPayload,
        body: UpdateApprovalStepBodyPayload,
        actor: String,
    ) -> Result<UpdateApprovalStepInput, ApplicationError> {
        Ok(UpdateApprovalStepInput {
            program: ResourceIdentifier::parse(&path.program),
            approval_step: parse::uuid("approval_step", &path.approval_step)?,
            name: body.name,
            approver_role: body.approver_role,
            position: body.position,
            actor,
        })
    }

    pub fn output_success(output: ApprovalStep) -> ApprovalStepPresenter {
        to_presenter(output)
    }
}

pub struct DeleteApprovalStepMapper;

impl DeleteApprovalStepMapper {
    pub fn input(path: DeleteApprovalStepPayload, actor: String) -> Result<DeleteApprovalStepInput, ApplicationError> {
        Ok(DeleteApprovalStepInput {
            program: ResourceIdentifier::parse(&path.program),
            approval_step: parse::uuid("approval_step", &path.approval_step)?,
            actor,
        })
    }
}

pub struct BatchUpdateApprovalStepsMapper;

impl BatchUpdateApprovalStepsMapper {
    pub fn input(
        path: BatchUpdateApprovalStepsPathPayload,
        body: BatchUpdateApprovalStepsBodyPayload,
        actor: String,
    ) -> Result<BatchUpdateApprovalStepsInput, ApplicationError> {
        let positions = body
            .approval_steps
            .into_iter()
            .map(|item| Ok((parse::uuid("approval_steps.id", &item.id)?, item.position)))
            .collect::<Result<Vec<_>, ApplicationError>>()?;
        Ok(BatchUpdateApprovalStepsInput { program: ResourceIdentifier::parse(&path.program), positions, actor })
    }

    pub fn output_success(output: Vec<ApprovalStep>) -> ApprovalStepsPresenter {
        ApprovalStepsPresenter { results: output.into_iter().map(to_presenter).collect() }
    }
}
