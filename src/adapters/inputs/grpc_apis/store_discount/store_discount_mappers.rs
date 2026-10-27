use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchUpdateStoreDiscountsRequest, BatchUpdateStoreDiscountsResponse, DiscountChange as GrpcDiscountChange,
    DiscountConfig as GrpcDiscountConfig, DiscountSources, DomainDiscountLevel, GetStoreDiscountRequest,
    GetStoreDiscountResponse, ListDiscountLevelsRequest, ListDiscountLevelsResponse, ListStoreDiscountsRequest,
    ListStoreDiscountsResponse, ResolvedDiscount, StoreDiscount as GrpcStoreDiscount, UpdateDiscountLevelRequest,
    UpdateDiscountLevelResponse, UpdateStoreDiscountRequest, UpdateStoreDiscountResponse,
};
use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::store_discount::batch_update_store_discounts::{
    BatchUpdateStoreDiscountsInput, BatchUpdateStoreDiscountsOutput,
};
use crate::application::usecases::store_discount::get_store_discount::GetStoreDiscountInput;
use crate::application::usecases::store_discount::list_discount_levels::ListDiscountLevelsInput;
use crate::application::usecases::store_discount::list_store_discounts::{ListStoreDiscountsInput, ListStoreDiscountsOutput};
use crate::application::usecases::store_discount::update_discount_level::{UpdateDiscountLevelInput, UpdateDiscountLevelOutput};
use crate::application::usecases::store_discount::update_store_discount::UpdateStoreDiscountInput;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::store_discount::{
    DiscountChange, DiscountConfig, DiscountLevels, DiscountPatch, FIELDS, LevelTarget, Resolved, Sources, StoreDiscountRow,
};

fn config_to_grpc(config: DiscountConfig) -> GrpcDiscountConfig {
    GrpcDiscountConfig {
        enabled: config.enabled,
        r#type: config.discount_type,
        amount: config.amount,
        currency_code: config.currency_code,
        ..Default::default()
    }
}

fn resolved_to_grpc(resolved: Resolved) -> ResolvedDiscount {
    ResolvedDiscount {
        enabled: resolved.enabled,
        r#type: resolved.discount_type,
        amount: resolved.amount,
        currency_code: resolved.currency_code,
        ..Default::default()
    }
}

fn sources_to_grpc(sources: Sources) -> DiscountSources {
    DiscountSources {
        enabled: sources.enabled.as_str().to_string(),
        r#type: sources.discount_type.as_str().to_string(),
        amount: sources.amount.as_str().to_string(),
        currency_code: sources.currency_code.as_str().to_string(),
        ..Default::default()
    }
}

fn to_grpc_row(row: StoreDiscountRow) -> GrpcStoreDiscount {
    GrpcStoreDiscount {
        path: format!("programs/{}/store-discounts/{}", row.program_id, row.enrollment_id),
        id: row.enrollment_id.to_string(),
        store_id: row.store_id,
        store_name: row.store_name,
        website_id: row.website_id,
        website_name: row.website_name,
        enrollment_status: row.enrollment_status,
        resolved: Some(resolved_to_grpc(row.resolved)).into(),
        sources: Some(sources_to_grpc(row.sources)).into(),
        set_at: row.set_at.as_str().to_string(),
        store_override: Some(config_to_grpc(row.store_override)).into(),
        inherited: Some(resolved_to_grpc(row.inherited)).into(),
        ..Default::default()
    }
}

/// A set field is set, a field named in `revert_fields` is reverted, `revert_all` reverts everything.
fn change_from_grpc(change: Option<GrpcDiscountChange>) -> Result<DiscountChange, ApplicationError> {
    let change = change.ok_or_else(|| ApplicationError::validation_error("discount is required"))?;
    let sets_something = change.enabled.is_some() || change.r#type.is_some() || change.amount.is_some() || change.currency_code.is_some();
    if change.revert_all {
        if sets_something || !change.revert_fields.is_empty() {
            return Err(ApplicationError::validation_error("revert_all cannot be combined with other changes"));
        }
        return Ok(DiscountChange::RevertAll);
    }

    let mut patch = DiscountPatch {
        enabled: change.enabled.map(Some),
        discount_type: change.r#type.map(|t| Some(t.trim().to_string())),
        amount: change.amount.map(Some),
        currency_code: change.currency_code.map(|c| Some(c.trim().to_string())),
    };
    for field in &change.revert_fields {
        let name = field.trim();
        let already_set = match name {
            "enabled" => patch.enabled.replace(None).is_some(),
            "type" => patch.discount_type.replace(None).is_some(),
            "amount" => patch.amount.replace(None).is_some(),
            "currency_code" => patch.currency_code.replace(None).is_some(),
            _ => {
                return Err(ApplicationError::validation_error(&format!(
                    "revert_fields has an unknown field {name:?}; use: {}",
                    FIELDS.join(", ")
                )));
            }
        };
        if already_set {
            return Err(ApplicationError::validation_error(&format!("{name} is both set and reverted")));
        }
    }
    Ok(DiscountChange::Patch(patch))
}

pub struct ListStoreDiscountsGrpcMapper;

impl ListStoreDiscountsGrpcMapper {
    pub fn input(request: ListStoreDiscountsRequest) -> Result<ListStoreDiscountsInput, ApplicationError> {
        Ok(ListStoreDiscountsInput {
            program: ResourceIdentifier::parse(&request.program),
            website_id: request.website_id,
            store: request.store,
            page_size: request.max_page_size,
            page_token: parse::optional_uuid("page_token", request.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListStoreDiscountsOutput) -> ListStoreDiscountsResponse {
        ListStoreDiscountsResponse {
            results: output.rows.into_iter().map(to_grpc_row).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            total_size: output.total_size,
            ..Default::default()
        }
    }
}

pub struct GetStoreDiscountGrpcMapper;

impl GetStoreDiscountGrpcMapper {
    pub fn input(request: GetStoreDiscountRequest) -> Result<GetStoreDiscountInput, ApplicationError> {
        Ok(GetStoreDiscountInput {
            program: ResourceIdentifier::parse(&request.program),
            enrollment: parse::uuid("enrollment", &request.enrollment)?,
        })
    }

    pub fn output_success(output: StoreDiscountRow) -> GetStoreDiscountResponse {
        GetStoreDiscountResponse { store_discount: Some(to_grpc_row(output)).into(), ..Default::default() }
    }
}

pub struct UpdateStoreDiscountGrpcMapper;

impl UpdateStoreDiscountGrpcMapper {
    pub fn input(request: UpdateStoreDiscountRequest, actor: String) -> Result<UpdateStoreDiscountInput, ApplicationError> {
        Ok(UpdateStoreDiscountInput {
            program: ResourceIdentifier::parse(&request.program),
            enrollment: parse::uuid("enrollment", &request.enrollment)?,
            change: change_from_grpc(request.discount.into_option())?,
            actor,
        })
    }

    pub fn output_success(output: StoreDiscountRow) -> UpdateStoreDiscountResponse {
        UpdateStoreDiscountResponse { store_discount: Some(to_grpc_row(output)).into(), ..Default::default() }
    }
}

pub struct BatchUpdateStoreDiscountsGrpcMapper;

impl BatchUpdateStoreDiscountsGrpcMapper {
    pub fn input(request: BatchUpdateStoreDiscountsRequest, actor: String) -> Result<BatchUpdateStoreDiscountsInput, ApplicationError> {
        Ok(BatchUpdateStoreDiscountsInput {
            program: ResourceIdentifier::parse(&request.program),
            ids: request.ids.iter().map(|id| parse::uuid("ids", id)).collect::<Result<Vec<_>, _>>()?,
            change: change_from_grpc(request.discount.into_option())?,
            actor,
        })
    }

    pub fn output_success(output: BatchUpdateStoreDiscountsOutput) -> BatchUpdateStoreDiscountsResponse {
        BatchUpdateStoreDiscountsResponse {
            updated: i32::try_from(output.updated).unwrap_or(i32::MAX),
            results: output.rows.into_iter().map(to_grpc_row).collect(),
            ..Default::default()
        }
    }
}

pub struct ListDiscountLevelsGrpcMapper;

impl ListDiscountLevelsGrpcMapper {
    pub fn input(request: ListDiscountLevelsRequest) -> Result<ListDiscountLevelsInput, ApplicationError> {
        Ok(ListDiscountLevelsInput { program: ResourceIdentifier::parse(&request.program) })
    }

    pub fn output_success(output: DiscountLevels) -> ListDiscountLevelsResponse {
        ListDiscountLevelsResponse {
            program: Some(config_to_grpc(output.program)).into(),
            domains: output
                .domains
                .into_iter()
                .map(|(website_id, config)| DomainDiscountLevel {
                    website_id,
                    discount: Some(config_to_grpc(config)).into(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }
}

pub struct UpdateDiscountLevelGrpcMapper;

impl UpdateDiscountLevelGrpcMapper {
    pub fn input(request: UpdateDiscountLevelRequest, actor: String) -> Result<UpdateDiscountLevelInput, ApplicationError> {
        Ok(UpdateDiscountLevelInput {
            program: ResourceIdentifier::parse(&request.program),
            target: match request.website_id {
                Some(website) => LevelTarget::Domain(website.trim().to_string()),
                None => LevelTarget::Program,
            },
            change: change_from_grpc(request.discount.into_option())?,
            actor,
        })
    }

    pub fn output_success(output: UpdateDiscountLevelOutput) -> UpdateDiscountLevelResponse {
        let (level, website_id) = match output.target {
            LevelTarget::Program => ("program", None),
            LevelTarget::Domain(website) => ("domain", Some(website)),
        };
        UpdateDiscountLevelResponse {
            level: level.to_string(),
            website_id,
            discount: Some(config_to_grpc(output.discount)).into(),
            ..Default::default()
        }
    }
}
