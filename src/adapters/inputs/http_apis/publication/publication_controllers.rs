use crate::adapters::inputs::http_apis::publication::publication_mappers::{
    CreatePublicationMapper, GetPublicationMapper, ListPublicationsMapper,
};
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::publication::{
    CreatePublicationPathPayload, GetPublicationPayload, ListPublicationsPathPayload, ListPublicationsQueryPayload,
};
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_publications).service(create_publication).service(get_publication);
}

/// Newest first; rows carry the counts, not the bundle.
#[instrument(skip(data))]
#[get("/{program}/publications")]
pub async fn list_publications(
    data: web::Data<AppState>,
    path: web::Path<ListPublicationsPathPayload>,
    query: web::Query<ListPublicationsQueryPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListPublicationsController called");

    let usecase_input = ListPublicationsMapper::input(path.into_inner(), query.into_inner())
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .publication
        .list_publications
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListPublicationsMapper::output_success(result)))
}

/// Publish: freezes the program's configuration (and promotes the rule drafts) into one bundle.
#[instrument(skip(data, req))]
#[post("/{program}/publications")]
pub async fn create_publication(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<CreatePublicationPathPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("CreatePublicationController called");

    let usecase_input = CreatePublicationMapper::input(path.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .publication
        .create_publication
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(CreatePublicationMapper::output_success(result)))
}

/// One publication with its whole bundle.
#[instrument(skip(data))]
#[get("/{program}/publications/{publication}")]
pub async fn get_publication(
    data: web::Data<AppState>,
    path: web::Path<GetPublicationPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetPublicationController called");

    let usecase_input = GetPublicationMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .publication
        .get_publication
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetPublicationMapper::output_success(result)))
}
