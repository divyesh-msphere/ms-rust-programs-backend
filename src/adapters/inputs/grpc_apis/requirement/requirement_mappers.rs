use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchUpdateRequirementsRequest, BatchUpdateRequirementsResponse, CreateRequirementRequest,
    CreateRequirementResponse, DeleteRequirementRequest, GetRequirementRequest, GetRequirementResponse,
    ListRequirementsRequest, ListRequirementsResponse, Requirement as GrpcRequirement, UpdateRequirementRequest,
    UpdateRequirementResponse,
};
use crate::adapters::inputs::grpc_apis::shared::json_struct::{buffa_value_to_json, json_to_buffa_value};
use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::requirement::batch_update_requirements::BatchUpdateRequirementsInput;
use crate::application::usecases::requirement::create_requirement::CreateRequirementInput;
use crate::application::usecases::requirement::delete_requirement::DeleteRequirementInput;
use crate::application::usecases::requirement::get_requirement::GetRequirementInput;
use crate::application::usecases::requirement::list_requirements::ListRequirementsInput;
use crate::application::usecases::requirement::update_requirement::UpdateRequirementInput;
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::requirement::Requirement;
use serde_json::json;

fn to_grpc_requirement(requirement: Requirement) -> GrpcRequirement {
    GrpcRequirement {
        path: format!("programs/{}/requirements/{}", requirement.program_id, requirement.id),
        id: requirement.id.to_string(),
        r#type: requirement.requirement_type,
        name: requirement.name,
        config: Some(json_to_buffa_value(requirement.config)).into(),
        position: requirement.position,
        create_time: format_datetime(requirement.create_time),
        update_time: format_datetime(requirement.update_time),
        ..Default::default()
    }
}

pub struct ListRequirementsGrpcMapper;

impl ListRequirementsGrpcMapper {
    pub fn input(request: ListRequirementsRequest) -> Result<ListRequirementsInput, ApplicationError> {
        Ok(ListRequirementsInput { program: ResourceIdentifier::parse(&request.program) })
    }

    pub fn output_success(output: Vec<Requirement>) -> ListRequirementsResponse {
        ListRequirementsResponse { results: output.into_iter().map(to_grpc_requirement).collect(), ..Default::default() }
    }
}

pub struct GetRequirementGrpcMapper;

impl GetRequirementGrpcMapper {
    pub fn input(request: GetRequirementRequest) -> Result<GetRequirementInput, ApplicationError> {
        Ok(GetRequirementInput {
            program: ResourceIdentifier::parse(&request.program),
            requirement: parse::uuid("requirement", &request.requirement)?,
        })
    }

    pub fn output_success(output: Requirement) -> GetRequirementResponse {
        GetRequirementResponse { requirement: Some(to_grpc_requirement(output)).into(), ..Default::default() }
    }
}

pub struct CreateRequirementGrpcMapper;

impl CreateRequirementGrpcMapper {
    pub fn input(request: CreateRequirementRequest, actor: String) -> Result<CreateRequirementInput, ApplicationError> {
        Ok(CreateRequirementInput {
            program: ResourceIdentifier::parse(&request.program),
            id: parse::optional_uuid("id", request.id.as_deref())?,
            requirement_type: request.r#type.trim().to_string(),
            name: request.name,
            config: request.config.into_option().map(buffa_value_to_json).unwrap_or_else(|| json!({})),
            position: request.position,
            actor,
        })
    }

    pub fn output_success(output: Requirement) -> CreateRequirementResponse {
        CreateRequirementResponse { requirement: Some(to_grpc_requirement(output)).into(), ..Default::default() }
    }
}

pub struct UpdateRequirementGrpcMapper;

impl UpdateRequirementGrpcMapper {
    pub fn input(request: UpdateRequirementRequest, actor: String) -> Result<UpdateRequirementInput, ApplicationError> {
        Ok(UpdateRequirementInput {
            program: ResourceIdentifier::parse(&request.program),
            requirement: parse::uuid("requirement", &request.requirement)?,
            name: request.name,
            config: request.config.into_option().map(buffa_value_to_json),
            position: request.position,
            actor,
        })
    }

    pub fn output_success(output: Requirement) -> UpdateRequirementResponse {
        UpdateRequirementResponse { requirement: Some(to_grpc_requirement(output)).into(), ..Default::default() }
    }
}

pub struct DeleteRequirementGrpcMapper;

impl DeleteRequirementGrpcMapper {
    pub fn input(request: DeleteRequirementRequest, actor: String) -> Result<DeleteRequirementInput, ApplicationError> {
        Ok(DeleteRequirementInput {
            program: ResourceIdentifier::parse(&request.program),
            requirement: parse::uuid("requirement", &request.requirement)?,
            actor,
        })
    }
}

pub struct BatchUpdateRequirementsGrpcMapper;

impl BatchUpdateRequirementsGrpcMapper {
    pub fn input(request: BatchUpdateRequirementsRequest, actor: String) -> Result<BatchUpdateRequirementsInput, ApplicationError> {
        let positions = request
            .requirements
            .into_iter()
            .map(|item| Ok((parse::uuid("requirements.id", &item.id)?, item.position)))
            .collect::<Result<Vec<_>, ApplicationError>>()?;
        Ok(BatchUpdateRequirementsInput { program: ResourceIdentifier::parse(&request.program), positions, actor })
    }

    pub fn output_success(output: Vec<Requirement>) -> BatchUpdateRequirementsResponse {
        BatchUpdateRequirementsResponse { results: output.into_iter().map(to_grpc_requirement).collect(), ..Default::default() }
    }
}
