use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchUpdateStoreDiscountsRequest, BatchUpdateStoreDiscountsResponse, GetStoreDiscountRequest, GetStoreDiscountResponse,
    ListDiscountLevelsRequest, ListDiscountLevelsResponse, ListStoreDiscountsRequest, ListStoreDiscountsResponse,
    UpdateDiscountLevelRequest, UpdateDiscountLevelResponse, UpdateStoreDiscountRequest, UpdateStoreDiscountResponse,
};
use crate::adapters::inputs::grpc_apis::shared::error_response::GrpcErrorResponse;
use crate::adapters::inputs::grpc_apis::store_discount::store_discount_mappers::{
    BatchUpdateStoreDiscountsGrpcMapper, GetStoreDiscountGrpcMapper, ListDiscountLevelsGrpcMapper,
    ListStoreDiscountsGrpcMapper, UpdateDiscountLevelGrpcMapper, UpdateStoreDiscountGrpcMapper,
};
use crate::adapters::inputs::shared::app_state::AppState;
use crate::application::usecases::interface::UseCase;
use connectrpc::{Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub struct StoreDiscountServiceImpl {
    app_state: Arc<AppState>,
}

impl StoreDiscountServiceImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

impl StoreDiscountServiceImpl {
    pub async fn list_store_discounts(
        &self,
        request: ServiceRequest<'_, ListStoreDiscountsRequest>,
    ) -> ServiceResult<ListStoreDiscountsResponse> {
        let input = ListStoreDiscountsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .store_discount
            .list_store_discounts
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListStoreDiscountsGrpcMapper::output_success(output))
    }

    pub async fn get_store_discount(
        &self,
        request: ServiceRequest<'_, GetStoreDiscountRequest>,
    ) -> ServiceResult<GetStoreDiscountResponse> {
        let input = GetStoreDiscountGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .store_discount
            .get_store_discount
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(GetStoreDiscountGrpcMapper::output_success(output))
    }

    pub async fn update_store_discount(
        &self,
        actor: String,
        request: ServiceRequest<'_, UpdateStoreDiscountRequest>,
    ) -> ServiceResult<UpdateStoreDiscountResponse> {
        let input = UpdateStoreDiscountGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .store_discount
            .update_store_discount
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(UpdateStoreDiscountGrpcMapper::output_success(output))
    }

    pub async fn batch_update_store_discounts(
        &self,
        actor: String,
        request: ServiceRequest<'_, BatchUpdateStoreDiscountsRequest>,
    ) -> ServiceResult<BatchUpdateStoreDiscountsResponse> {
        let input = BatchUpdateStoreDiscountsGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .store_discount
            .batch_update_store_discounts
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(BatchUpdateStoreDiscountsGrpcMapper::output_success(output))
    }

    pub async fn list_discount_levels(
        &self,
        request: ServiceRequest<'_, ListDiscountLevelsRequest>,
    ) -> ServiceResult<ListDiscountLevelsResponse> {
        let input = ListDiscountLevelsGrpcMapper::input(request.to_owned_message())
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .store_discount
            .list_discount_levels
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(ListDiscountLevelsGrpcMapper::output_success(output))
    }

    pub async fn update_discount_level(
        &self,
        actor: String,
        request: ServiceRequest<'_, UpdateDiscountLevelRequest>,
    ) -> ServiceResult<UpdateDiscountLevelResponse> {
        let input = UpdateDiscountLevelGrpcMapper::input(request.to_owned_message(), actor)
            .map_err(GrpcErrorResponse::map_response_error)?;
        let output = self
            .app_state
            .use_cases
            .store_discount
            .update_discount_level
            .execute(Some(input))
            .await
            .map_err(GrpcErrorResponse::map_response_error)?;

        Response::ok(UpdateDiscountLevelGrpcMapper::output_success(output))
    }
}
