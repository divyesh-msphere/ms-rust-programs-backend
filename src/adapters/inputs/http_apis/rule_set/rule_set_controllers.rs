use crate::adapters::inputs::http_apis::rule_set::rule_set_mappers::{
    CreateRuleVersionMapper, ExecuteRuleMapper, GetRuleSetMapper, GetRuleVersionMapper, ListRuleSetsMapper, ListRuleVersionsMapper,
};
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::rule_set::{
    CreateRuleVersionBodyPayload, CreateRuleVersionPathPayload, ExecuteRuleBodyPayload, ExecuteRulePathPayload, GetRuleSetPayload, GetRuleVersionPayload,
    ListRuleSetsPayload, ListRuleVersionsPathPayload, ListRuleVersionsQueryPayload,
};
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_rule_sets)
        .service(get_rule_set)
        .service(create_rule_version)
        .service(list_rule_versions)
        .service(get_rule_version)
        .service(execute_rule);
}

/// The program's rule leaves with their publish state (`empty`, `never`, `drifted`, `in_sync`).
#[instrument(skip(data))]
#[get("/{program}/rule-sets")]
pub async fn list_rule_sets(
    data: web::Data<AppState>,
    path: web::Path<ListRuleSetsPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListRuleSetsController called");

    let usecase_input = ListRuleSetsMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .rule_set
        .list_rule_sets
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListRuleSetsMapper::output_success(result)))
}

/// One rule leaf with its newest version in full: what the rule builder opens with.
#[instrument(skip(data))]
#[get("/{program}/rule-sets/{rule_set}")]
pub async fn get_rule_set(
    data: web::Data<AppState>,
    path: web::Path<GetRuleSetPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetRuleSetController called");

    let usecase_input = GetRuleSetMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .rule_set
        .get_rule_set
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetRuleSetMapper::output_success(result)))
}

/// Saves the rule builder's draft as a new immutable version (nothing new is made when nothing changed).
#[instrument(skip(data, req, body))]
#[post("/{program}/rule-sets/{rule_set}/versions")]
pub async fn create_rule_version(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<CreateRuleVersionPathPayload>,
    body: web::Json<CreateRuleVersionBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("CreateRuleVersionController called");

    let usecase_input = CreateRuleVersionMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .rule_set
        .create_rule_version
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(CreateRuleVersionMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{program}/rule-sets/{rule_set}/versions")]
pub async fn list_rule_versions(
    data: web::Data<AppState>,
    path: web::Path<ListRuleVersionsPathPayload>,
    query: web::Query<ListRuleVersionsQueryPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListRuleVersionsController called");

    let usecase_input = ListRuleVersionsMapper::input(path.into_inner(), query.into_inner())
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .rule_set
        .list_rule_versions
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListRuleVersionsMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{program}/rule-sets/{rule_set}/versions/{version}")]
pub async fn get_rule_version(
    data: web::Data<AppState>,
    path: web::Path<GetRuleVersionPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetRuleVersionController called");

    let usecase_input = GetRuleVersionMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .rule_set
        .get_rule_version
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetRuleVersionMapper::output_success(result)))
}

/// Runs a rule on the caller's input and returns what it outputs. Read-only: no version is made or bumped.
#[instrument(skip(data, body))]
#[post("/{program}/rule-sets/{rule_set}:execute")]
pub async fn execute_rule(
    data: web::Data<AppState>,
    path: web::Path<ExecuteRulePathPayload>,
    body: web::Json<ExecuteRuleBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ExecuteRuleController called");

    let usecase_input =
        ExecuteRuleMapper::input(path.into_inner(), body.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .rule_set
        .execute_rule
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ExecuteRuleMapper::output_success(result)))
}
