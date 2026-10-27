use crate::adapters::inputs::http_apis::shared::actor::actor_from;
use crate::adapters::inputs::http_apis::shared::error_response::ErrorResponse;
use crate::adapters::inputs::http_apis::store_discount::store_discount_mappers::{
    BatchUpdateStoreDiscountsMapper, GetStoreDiscountMapper, ListDiscountLevelsMapper, ListStoreDiscountsMapper,
    UpdateDiscountLevelMapper, UpdateStoreDiscountMapper,
};
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use crate::dto::payloads::json::store_discount::{
    BatchUpdateStoreDiscountsBodyPayload, BatchUpdateStoreDiscountsPathPayload, GetStoreDiscountPayload,
    ListDiscountLevelsPayload, ListStoreDiscountsPathPayload, ListStoreDiscountsQueryPayload,
    UpdateDiscountLevelBodyPayload, UpdateDomainLevelPathPayload, UpdateProgramLevelPathPayload,
    UpdateStoreDiscountBodyPayload, UpdateStoreDiscountPathPayload,
};
use actix_web::{HttpRequest, HttpResponse, get, patch, post, web};
use ms_telemetry_rs::tracing::{info, instrument};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_store_discounts)
        .service(batch_update_store_discounts)
        .service(get_store_discount)
        .service(update_store_discount)
        .service(list_discount_levels)
        .service(update_program_level)
        .service(update_domain_level);
}

/// The Store Discounts page: each enrolled store's effective discount and where it came from,
/// newest first, with the "All websites" filter and a store-name search.
#[instrument(skip(data))]
#[get("/{program}/store-discounts")]
pub async fn list_store_discounts(
    data: web::Data<AppState>,
    path: web::Path<ListStoreDiscountsPathPayload>,
    query: web::Query<ListStoreDiscountsQueryPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListStoreDiscountsController called");

    let usecase_input = ListStoreDiscountsMapper::input(path.into_inner(), query.into_inner())
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .store_discount
        .list_store_discounts
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListStoreDiscountsMapper::output_success(result)))
}

/// "Mass edit": the same change for every ticked store, in one request.
#[instrument(skip(data, req, body))]
#[post("/{program}/store-discounts:batchUpdate")]
pub async fn batch_update_store_discounts(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<BatchUpdateStoreDiscountsPathPayload>,
    body: web::Json<BatchUpdateStoreDiscountsBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("BatchUpdateStoreDiscountsController called");

    let usecase_input = BatchUpdateStoreDiscountsMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .store_discount
        .batch_update_store_discounts
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(BatchUpdateStoreDiscountsMapper::output_success(result)))
}

/// One store: what the edit panel opens with.
#[instrument(skip(data))]
#[get("/{program}/store-discounts/{enrollment}")]
pub async fn get_store_discount(
    data: web::Data<AppState>,
    path: web::Path<GetStoreDiscountPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("GetStoreDiscountController called");

    let usecase_input = GetStoreDiscountMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .store_discount
        .get_store_discount
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetStoreDiscountMapper::output_success(result)))
}

/// The edit panel's Save (fields it names), a field's revert (that field as `null`) and "Revert all to
/// inherited" (`discount: null`).
#[instrument(skip(data, req, body))]
#[patch("/{program}/store-discounts/{enrollment}")]
pub async fn update_store_discount(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<UpdateStoreDiscountPathPayload>,
    body: web::Json<UpdateStoreDiscountBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("UpdateStoreDiscountController called");

    let usecase_input = UpdateStoreDiscountMapper::input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .store_discount
        .update_store_discount
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(UpdateStoreDiscountMapper::output_success(result)))
}

/// The program default and every website that sets something of its own.
#[instrument(skip(data))]
#[get("/{program}/store-discount-levels")]
pub async fn list_discount_levels(
    data: web::Data<AppState>,
    path: web::Path<ListDiscountLevelsPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("ListDiscountLevelsController called");

    let usecase_input = ListDiscountLevelsMapper::input(path.into_inner()).map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .store_discount
        .list_discount_levels
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(ListDiscountLevelsMapper::output_success(result)))
}

#[instrument(skip(data, req, body))]
#[patch("/{program}/store-discount-levels/program")]
pub async fn update_program_level(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<UpdateProgramLevelPathPayload>,
    body: web::Json<UpdateDiscountLevelBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("UpdateProgramLevelController called");

    let usecase_input = UpdateDiscountLevelMapper::program_input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .store_discount
        .update_discount_level
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(UpdateDiscountLevelMapper::output_success(result)))
}

#[instrument(skip(data, req, body))]
#[patch("/{program}/store-discount-levels/domains/{website_id}")]
pub async fn update_domain_level(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<UpdateDomainLevelPathPayload>,
    body: web::Json<UpdateDiscountLevelBodyPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    info!("UpdateDomainLevelController called");

    let usecase_input = UpdateDiscountLevelMapper::domain_input(path.into_inner(), body.into_inner(), actor_from(&req))
        .map_err(ErrorResponse::map_response_error)?;
    let result = data
        .use_cases
        .store_discount
        .update_discount_level
        .execute(Some(usecase_input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(UpdateDiscountLevelMapper::output_success(result)))
}
