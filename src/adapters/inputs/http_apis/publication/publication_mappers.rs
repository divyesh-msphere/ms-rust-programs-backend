use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::publication::create_publication::CreatePublicationInput;
use crate::application::usecases::publication::get_publication::GetPublicationInput;
use crate::application::usecases::publication::list_publications::{ListPublicationsInput, ListPublicationsOutput};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::publication::Publication;
use crate::dto::payloads::json::publication::{
    CreatePublicationPathPayload, GetPublicationPayload, ListPublicationsPathPayload, ListPublicationsQueryPayload,
};
use crate::dto::presenters::json::publication::{ListPublicationsPresenter, PublicationPresenter};

pub fn to_presenter(publication: Publication) -> PublicationPresenter {
    PublicationPresenter {
        path: format!("programs/{}/publications/{}", publication.program_id, publication.id),
        id: publication.id.to_string(),
        config_version: publication.config_version,
        config_hash: publication.config_hash,
        summary: publication.summary,
        bundle: publication.bundle,
        created_by: publication.created_by,
        create_time: format_datetime(publication.create_time),
    }
}

pub struct CreatePublicationMapper;

impl CreatePublicationMapper {
    pub fn input(path: CreatePublicationPathPayload, actor: String) -> Result<CreatePublicationInput, ApplicationError> {
        Ok(CreatePublicationInput { program: ResourceIdentifier::parse(&path.program), actor })
    }

    /// The answer to a publish carries the counts, not the whole bundle (read it with GET).
    pub fn output_success(mut output: Publication) -> PublicationPresenter {
        output.bundle = None;
        to_presenter(output)
    }
}

pub struct ListPublicationsMapper;

impl ListPublicationsMapper {
    pub fn input(path: ListPublicationsPathPayload, query: ListPublicationsQueryPayload) -> Result<ListPublicationsInput, ApplicationError> {
        Ok(ListPublicationsInput {
            program: ResourceIdentifier::parse(&path.program),
            page_size: query.max_page_size,
            page_token: parse::optional_uuid("page_token", query.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListPublicationsOutput) -> ListPublicationsPresenter {
        ListPublicationsPresenter {
            results: output.publications.into_iter().map(to_presenter).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
        }
    }
}

pub struct GetPublicationMapper;

impl GetPublicationMapper {
    pub fn input(path: GetPublicationPayload) -> Result<GetPublicationInput, ApplicationError> {
        Ok(GetPublicationInput {
            program: ResourceIdentifier::parse(&path.program),
            publication: parse::uuid("publication", &path.publication)?,
        })
    }

    pub fn output_success(output: Publication) -> PublicationPresenter {
        to_presenter(output)
    }
}
