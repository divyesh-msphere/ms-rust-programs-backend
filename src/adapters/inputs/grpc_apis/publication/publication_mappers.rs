use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    CreatePublicationRequest, CreatePublicationResponse, GetPublicationRequest, GetPublicationResponse,
    ListPublicationsRequest, ListPublicationsResponse, Publication as GrpcPublication,
};
use crate::adapters::inputs::grpc_apis::shared::json_struct::json_to_buffa_value;
use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::publication::create_publication::CreatePublicationInput;
use crate::application::usecases::publication::get_publication::GetPublicationInput;
use crate::application::usecases::publication::list_publications::{ListPublicationsInput, ListPublicationsOutput};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::publication::Publication;

fn to_grpc_publication(publication: Publication) -> GrpcPublication {
    GrpcPublication {
        path: format!("programs/{}/publications/{}", publication.program_id, publication.id),
        id: publication.id.to_string(),
        config_version: publication.config_version,
        config_hash: publication.config_hash,
        summary: Some(json_to_buffa_value(publication.summary)).into(),
        bundle: publication.bundle.map(json_to_buffa_value).into(),
        created_by: publication.created_by,
        create_time: format_datetime(publication.create_time),
        ..Default::default()
    }
}

pub struct CreatePublicationGrpcMapper;

impl CreatePublicationGrpcMapper {
    pub fn input(request: CreatePublicationRequest, actor: String) -> Result<CreatePublicationInput, ApplicationError> {
        Ok(CreatePublicationInput { program: ResourceIdentifier::parse(&request.program), actor })
    }

    /// The answer to a publish carries the counts, not the whole bundle (read it with GetPublication).
    pub fn output_success(mut output: Publication) -> CreatePublicationResponse {
        output.bundle = None;
        CreatePublicationResponse { publication: Some(to_grpc_publication(output)).into(), ..Default::default() }
    }
}

pub struct ListPublicationsGrpcMapper;

impl ListPublicationsGrpcMapper {
    pub fn input(request: ListPublicationsRequest) -> Result<ListPublicationsInput, ApplicationError> {
        Ok(ListPublicationsInput {
            program: ResourceIdentifier::parse(&request.program),
            page_size: request.max_page_size,
            page_token: parse::optional_uuid("page_token", request.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListPublicationsOutput) -> ListPublicationsResponse {
        ListPublicationsResponse {
            results: output.publications.into_iter().map(to_grpc_publication).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            ..Default::default()
        }
    }
}

pub struct GetPublicationGrpcMapper;

impl GetPublicationGrpcMapper {
    pub fn input(request: GetPublicationRequest) -> Result<GetPublicationInput, ApplicationError> {
        Ok(GetPublicationInput {
            program: ResourceIdentifier::parse(&request.program),
            publication: parse::uuid("publication", &request.publication)?,
        })
    }

    pub fn output_success(output: Publication) -> GetPublicationResponse {
        GetPublicationResponse { publication: Some(to_grpc_publication(output)).into(), ..Default::default() }
    }
}
