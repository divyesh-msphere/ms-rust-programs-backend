use crate::adapters::inputs::http_apis::program::program_mappers::{
    CreateProgramMapper, DeleteProgramMapper, GetProgramMapper, ListProgramsMapper, UndeleteProgramMapper,
    UpdateProgramMapper,
};
use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::program::{
    CreateProgramBodyPayload, CreateProgramQueryPayload, DeleteProgramPayload, GetProgramPayload,
    ListProgramsPayload, UndeleteProgramPayload, UpdateProgramBodyPayload, UpdateProgramPathPayload,
};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_programs)
        .service(create_program)
        .service(get_program)
        .service(update_program)
        .service(delete_program);
}

/// Custom methods sit outside the scope, same as `:recover` in the reference.
pub fn root_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(undelete_program);
}

#[instrument(skip(data))]
#[get("")]
pub async fn list_programs(
    data: web::Data<AppState>,
    query: web::Query<ListProgramsPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListProgramsController called");

    let usecase_input = ListProgramsMapper::input(query.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .program
        .list_programs
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListProgramsMapper::output_success(result)))
}

#[instrument(skip(data, req))]
#[post("")]
pub async fn create_program(
    req: HttpRequest,
    data: web::Data<AppState>,
    body: web::Json<CreateProgramBodyPayload>,
    query: web::Query<CreateProgramQueryPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("CreateProgramController called");

    let usecase_input = CreateProgramMapper::input(body.into_inner(), query.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .program
        .create_program
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(CreateProgramMapper::output_success(result)))
}

#[instrument(skip(data))]
#[get("/{program}")]
pub async fn get_program(
    data: web::Data<AppState>,
    path: web::Path<GetProgramPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetProgramController called");

    let usecase_input = GetProgramMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .program
        .get_program
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetProgramMapper::output_success(result)))
}

#[instrument(skip(data, req))]
#[patch("/{program}")]
pub async fn update_program(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<UpdateProgramPathPayload>,
    body: web::Json<UpdateProgramBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("UpdateProgramController called");

    let usecase_input = UpdateProgramMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .program
        .update_program
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(UpdateProgramMapper::output_success(result)))
}

#[instrument(skip(data, req))]
#[delete("/{program}")]
pub async fn delete_program(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<DeleteProgramPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("DeleteProgramController called");

    let usecase_input = DeleteProgramMapper::input(path.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    data.use_cases
        .program
        .delete_program
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::NoContent().finish())
}

#[instrument(skip(data, req))]
#[post("/programs/{program}:undelete")]
pub async fn undelete_program(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<UndeleteProgramPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("UndeleteProgramController called");

    let usecase_input = UndeleteProgramMapper::input(path.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .program
        .undelete_program
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(UndeleteProgramMapper::output_success(result)))
}
