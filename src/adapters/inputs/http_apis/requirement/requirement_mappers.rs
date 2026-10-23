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
use crate::dto::payloads::json::requirement::{
    BatchUpdateRequirementsBodyPayload, BatchUpdateRequirementsPathPayload, CreateRequirementBodyPayload,
    CreateRequirementPathPayload, CreateRequirementQueryPayload, DeleteRequirementPayload, GetRequirementPayload,
    ListRequirementsPayload, UpdateRequirementBodyPayload, UpdateRequirementPathPayload,
};
use crate::dto::presenters::json::requirement::{RequirementPresenter, RequirementsPresenter};
use serde_json::json;

pub fn to_presenter(requirement: Requirement) -> RequirementPresenter {
    RequirementPresenter {
        path: format!("programs/{}/requirements/{}", requirement.program_id, requirement.id),
        id: requirement.id.to_string(),
        requirement_type: requirement.requirement_type,
        name: requirement.name,
        config: requirement.config,
        position: requirement.position,
        create_time: format_datetime(requirement.create_time),
        update_time: format_datetime(requirement.update_time),
    }
}

pub struct ListRequirementsMapper;

impl ListRequirementsMapper {
    pub fn input(path: ListRequirementsPayload) -> Result<ListRequirementsInput, ApplicationError> {
        Ok(ListRequirementsInput { program: ResourceIdentifier::parse(&path.program) })
    }

    pub fn output_success(output: Vec<Requirement>) -> RequirementsPresenter {
        RequirementsPresenter { results: output.into_iter().map(to_presenter).collect() }
    }
}

pub struct GetRequirementMapper;

impl GetRequirementMapper {
    pub fn input(path: GetRequirementPayload) -> Result<GetRequirementInput, ApplicationError> {
        Ok(GetRequirementInput {
            program: ResourceIdentifier::parse(&path.program),
            requirement: parse::uuid("requirement", &path.requirement)?,
        })
    }

    pub fn output_success(output: Requirement) -> RequirementPresenter {
        to_presenter(output)
    }
}

pub struct CreateRequirementMapper;

impl CreateRequirementMapper {
    pub fn input(
        path: CreateRequirementPathPayload,
        query: CreateRequirementQueryPayload,
        body: CreateRequirementBodyPayload,
        actor: String,
    ) -> Result<CreateRequirementInput, ApplicationError> {
        Ok(CreateRequirementInput {
            program: ResourceIdentifier::parse(&path.program),
            id: parse::optional_uuid("id", query.id.as_deref())?,
            requirement_type: body.requirement_type.trim().to_string(),
            name: body.name,
            config: body.config.unwrap_or_else(|| json!({})),
            position: body.position,
            actor,
        })
    }

    pub fn output_success(output: Requirement) -> RequirementPresenter {
        to_presenter(output)
    }
}

pub struct UpdateRequirementMapper;

impl UpdateRequirementMapper {
    pub fn input(
        path: UpdateRequirementPathPayload,
        body: UpdateRequirementBodyPayload,
        actor: String,
    ) -> Result<UpdateRequirementInput, ApplicationError> {
        Ok(UpdateRequirementInput {
            program: ResourceIdentifier::parse(&path.program),
            requirement: parse::uuid("requirement", &path.requirement)?,
            name: body.name,
            config: body.config,
            position: body.position,
            actor,
        })
    }

    pub fn output_success(output: Requirement) -> RequirementPresenter {
        to_presenter(output)
    }
}

pub struct DeleteRequirementMapper;

impl DeleteRequirementMapper {
    pub fn input(path: DeleteRequirementPayload, actor: String) -> Result<DeleteRequirementInput, ApplicationError> {
        Ok(DeleteRequirementInput {
            program: ResourceIdentifier::parse(&path.program),
            requirement: parse::uuid("requirement", &path.requirement)?,
            actor,
        })
    }
}

pub struct BatchUpdateRequirementsMapper;

impl BatchUpdateRequirementsMapper {
    pub fn input(
        path: BatchUpdateRequirementsPathPayload,
        body: BatchUpdateRequirementsBodyPayload,
        actor: String,
    ) -> Result<BatchUpdateRequirementsInput, ApplicationError> {
        let positions = body
            .requirements
            .into_iter()
            .map(|item| Ok((parse::uuid("requirements.id", &item.id)?, item.position)))
            .collect::<Result<Vec<_>, ApplicationError>>()?;
        Ok(BatchUpdateRequirementsInput { program: ResourceIdentifier::parse(&path.program), positions, actor })
    }

    pub fn output_success(output: Vec<Requirement>) -> RequirementsPresenter {
        RequirementsPresenter { results: output.into_iter().map(to_presenter).collect() }
    }
}
