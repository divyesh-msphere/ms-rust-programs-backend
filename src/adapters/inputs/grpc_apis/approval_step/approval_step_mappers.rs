use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchUpdateApprovalStepsRequest, BatchUpdateApprovalStepsResponse, CreateApprovalStepRequest,
    CreateApprovalStepResponse, DeleteApprovalStepRequest, GetApprovalStepRequest, GetApprovalStepResponse,
    ListApprovalStepsRequest, ListApprovalStepsResponse, ApprovalStep as GrpcApprovalStep, UpdateApprovalStepRequest,
    UpdateApprovalStepResponse,
};
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

fn to_grpc_approval_step(approval_step: ApprovalStep) -> GrpcApprovalStep {
    GrpcApprovalStep {
        path: format!("programs/{}/approval-steps/{}", approval_step.program_id, approval_step.id),
        id: approval_step.id.to_string(),
        name: approval_step.name,
        approver_role: approval_step.approver_role,
        position: approval_step.position,
        create_time: format_datetime(approval_step.create_time),
        update_time: format_datetime(approval_step.update_time),
        ..Default::default()
    }
}

pub struct ListApprovalStepsGrpcMapper;

impl ListApprovalStepsGrpcMapper {
    pub fn input(request: ListApprovalStepsRequest) -> Result<ListApprovalStepsInput, ApplicationError> {
        Ok(ListApprovalStepsInput { program: ResourceIdentifier::parse(&request.program) })
    }

    pub fn output_success(output: Vec<ApprovalStep>) -> ListApprovalStepsResponse {
        ListApprovalStepsResponse { results: output.into_iter().map(to_grpc_approval_step).collect(), ..Default::default() }
    }
}

pub struct GetApprovalStepGrpcMapper;

impl GetApprovalStepGrpcMapper {
    pub fn input(request: GetApprovalStepRequest) -> Result<GetApprovalStepInput, ApplicationError> {
        Ok(GetApprovalStepInput {
            program: ResourceIdentifier::parse(&request.program),
            approval_step: parse::uuid("approval_step", &request.approval_step)?,
        })
    }

    pub fn output_success(output: ApprovalStep) -> GetApprovalStepResponse {
        GetApprovalStepResponse { approval_step: Some(to_grpc_approval_step(output)).into(), ..Default::default() }
    }
}

pub struct CreateApprovalStepGrpcMapper;

impl CreateApprovalStepGrpcMapper {
    pub fn input(request: CreateApprovalStepRequest, actor: String) -> Result<CreateApprovalStepInput, ApplicationError> {
        Ok(CreateApprovalStepInput {
            program: ResourceIdentifier::parse(&request.program),
            id: parse::optional_uuid("id", request.id.as_deref())?,
            name: request.name,
            approver_role: request.approver_role,
            position: request.position,
            actor,
        })
    }

    pub fn output_success(output: ApprovalStep) -> CreateApprovalStepResponse {
        CreateApprovalStepResponse { approval_step: Some(to_grpc_approval_step(output)).into(), ..Default::default() }
    }
}

pub struct UpdateApprovalStepGrpcMapper;

impl UpdateApprovalStepGrpcMapper {
    pub fn input(request: UpdateApprovalStepRequest, actor: String) -> Result<UpdateApprovalStepInput, ApplicationError> {
        Ok(UpdateApprovalStepInput {
            program: ResourceIdentifier::parse(&request.program),
            approval_step: parse::uuid("approval_step", &request.approval_step)?,
            name: request.name,
            approver_role: request.approver_role,
            position: request.position,
            actor,
        })
    }

    pub fn output_success(output: ApprovalStep) -> UpdateApprovalStepResponse {
        UpdateApprovalStepResponse { approval_step: Some(to_grpc_approval_step(output)).into(), ..Default::default() }
    }
}

pub struct DeleteApprovalStepGrpcMapper;

impl DeleteApprovalStepGrpcMapper {
    pub fn input(request: DeleteApprovalStepRequest, actor: String) -> Result<DeleteApprovalStepInput, ApplicationError> {
        Ok(DeleteApprovalStepInput {
            program: ResourceIdentifier::parse(&request.program),
            approval_step: parse::uuid("approval_step", &request.approval_step)?,
            actor,
        })
    }
}

pub struct BatchUpdateApprovalStepsGrpcMapper;

impl BatchUpdateApprovalStepsGrpcMapper {
    pub fn input(request: BatchUpdateApprovalStepsRequest, actor: String) -> Result<BatchUpdateApprovalStepsInput, ApplicationError> {
        let positions = request
            .approval_steps
            .into_iter()
            .map(|item| Ok((parse::uuid("approval_steps.id", &item.id)?, item.position)))
            .collect::<Result<Vec<_>, ApplicationError>>()?;
        Ok(BatchUpdateApprovalStepsInput { program: ResourceIdentifier::parse(&request.program), positions, actor })
    }

    pub fn output_success(output: Vec<ApprovalStep>) -> BatchUpdateApprovalStepsResponse {
        BatchUpdateApprovalStepsResponse { results: output.into_iter().map(to_grpc_approval_step).collect(), ..Default::default() }
    }
}
