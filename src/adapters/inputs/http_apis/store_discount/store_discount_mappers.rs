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
    DiscountChange, DiscountConfig, DiscountLevels, DiscountPatch, Level, LevelTarget, Resolved, Sources, StoreDiscountRow,
};
use crate::dto::payloads::json::store_discount::{
    BatchUpdateStoreDiscountsBodyPayload, BatchUpdateStoreDiscountsPathPayload, DiscountPatchPayload, GetStoreDiscountPayload,
    ListDiscountLevelsPayload, ListStoreDiscountsPathPayload, ListStoreDiscountsQueryPayload, UpdateDiscountLevelBodyPayload,
    UpdateDomainLevelPathPayload, UpdateProgramLevelPathPayload, UpdateStoreDiscountBodyPayload,
    UpdateStoreDiscountPathPayload,
};
use crate::dto::presenters::json::store_discount::{
    BatchStoreDiscountsPresenter, DiscountConfigPresenter, DiscountLevelPresenter, DiscountLevelsPresenter,
    DiscountSourcesPresenter, DomainLevelPresenter, ListStoreDiscountsPresenter, ResolvedDiscountPresenter,
    StoreDiscountPresenter,
};

pub fn config_presenter(config: DiscountConfig) -> DiscountConfigPresenter {
    DiscountConfigPresenter {
        enabled: config.enabled,
        discount_type: config.discount_type,
        amount: config.amount,
        currency_code: config.currency_code,
    }
}

fn resolved_presenter(resolved: Resolved) -> ResolvedDiscountPresenter {
    ResolvedDiscountPresenter {
        enabled: resolved.enabled,
        discount_type: resolved.discount_type,
        amount: resolved.amount,
        currency_code: resolved.currency_code,
    }
}

fn sources_presenter(sources: Sources) -> DiscountSourcesPresenter {
    DiscountSourcesPresenter {
        enabled: sources.enabled.as_str(),
        discount_type: sources.discount_type.as_str(),
        amount: sources.amount.as_str(),
        currency_code: sources.currency_code.as_str(),
    }
}

pub fn to_presenter(row: StoreDiscountRow) -> StoreDiscountPresenter {
    StoreDiscountPresenter {
        path: format!("programs/{}/store-discounts/{}", row.program_id, row.enrollment_id),
        id: row.enrollment_id.to_string(),
        store_id: row.store_id,
        store_name: row.store_name,
        website_id: row.website_id,
        website_name: row.website_name,
        enrollment_status: row.enrollment_status,
        resolved: resolved_presenter(row.resolved),
        sources: sources_presenter(row.sources),
        set_at: Level::as_str(&row.set_at),
        store_override: config_presenter(row.store_override),
        inherited: resolved_presenter(row.inherited),
    }
}

/// `discount: {...}` changes the fields it names; `discount: null` reverts everything; a request
/// without `discount` is refused so a forgotten key cannot wipe a store's overrides.
pub fn change_from(discount: Option<Option<DiscountPatchPayload>>) -> Result<DiscountChange, ApplicationError> {
    match discount {
        None => Err(ApplicationError::validation_error(
            "discount is required (send null to revert everything to the inherited values)",
        )),
        Some(None) => Ok(DiscountChange::RevertAll),
        Some(Some(patch)) => Ok(DiscountChange::Patch(DiscountPatch {
            enabled: patch.enabled,
            discount_type: patch.discount_type.map(|t| t.map(|t| t.trim().to_string())),
            amount: patch.amount,
            currency_code: patch.currency_code.map(|c| c.map(|c| c.trim().to_string())),
        })),
    }
}

pub struct ListStoreDiscountsMapper;

impl ListStoreDiscountsMapper {
    pub fn input(
        path: ListStoreDiscountsPathPayload,
        query: ListStoreDiscountsQueryPayload,
    ) -> Result<ListStoreDiscountsInput, ApplicationError> {
        Ok(ListStoreDiscountsInput {
            program: ResourceIdentifier::parse(&path.program),
            website_id: query.website_id,
            store: query.store,
            page_size: query.max_page_size,
            page_token: parse::optional_uuid("page_token", query.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListStoreDiscountsOutput) -> ListStoreDiscountsPresenter {
        ListStoreDiscountsPresenter {
            results: output.rows.into_iter().map(to_presenter).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            total_size: output.total_size,
        }
    }
}

pub struct GetStoreDiscountMapper;

impl GetStoreDiscountMapper {
    pub fn input(path: GetStoreDiscountPayload) -> Result<GetStoreDiscountInput, ApplicationError> {
        Ok(GetStoreDiscountInput {
            program: ResourceIdentifier::parse(&path.program),
            enrollment: parse::uuid("enrollment", &path.enrollment)?,
        })
    }

    pub fn output_success(output: StoreDiscountRow) -> StoreDiscountPresenter {
        to_presenter(output)
    }
}

pub struct UpdateStoreDiscountMapper;

impl UpdateStoreDiscountMapper {
    pub fn input(
        path: UpdateStoreDiscountPathPayload,
        body: UpdateStoreDiscountBodyPayload,
        actor: String,
    ) -> Result<UpdateStoreDiscountInput, ApplicationError> {
        Ok(UpdateStoreDiscountInput {
            program: ResourceIdentifier::parse(&path.program),
            enrollment: parse::uuid("enrollment", &path.enrollment)?,
            change: change_from(body.discount)?,
            actor,
        })
    }

    pub fn output_success(output: StoreDiscountRow) -> StoreDiscountPresenter {
        to_presenter(output)
    }
}

pub struct BatchUpdateStoreDiscountsMapper;

impl BatchUpdateStoreDiscountsMapper {
    pub fn input(
        path: BatchUpdateStoreDiscountsPathPayload,
        body: BatchUpdateStoreDiscountsBodyPayload,
        actor: String,
    ) -> Result<BatchUpdateStoreDiscountsInput, ApplicationError> {
        Ok(BatchUpdateStoreDiscountsInput {
            program: ResourceIdentifier::parse(&path.program),
            ids: body.ids.iter().map(|id| parse::uuid("ids", id)).collect::<Result<Vec<_>, _>>()?,
            change: change_from(body.discount)?,
            actor,
        })
    }

    pub fn output_success(output: BatchUpdateStoreDiscountsOutput) -> BatchStoreDiscountsPresenter {
        BatchStoreDiscountsPresenter { updated: output.updated, results: output.rows.into_iter().map(to_presenter).collect() }
    }
}

pub struct ListDiscountLevelsMapper;

impl ListDiscountLevelsMapper {
    pub fn input(path: ListDiscountLevelsPayload) -> Result<ListDiscountLevelsInput, ApplicationError> {
        Ok(ListDiscountLevelsInput { program: ResourceIdentifier::parse(&path.program) })
    }

    pub fn output_success(output: DiscountLevels) -> DiscountLevelsPresenter {
        DiscountLevelsPresenter {
            program: config_presenter(output.program),
            domains: output
                .domains
                .into_iter()
                .map(|(website_id, config)| DomainLevelPresenter { website_id, discount: config_presenter(config) })
                .collect(),
        }
    }
}

pub struct UpdateDiscountLevelMapper;

impl UpdateDiscountLevelMapper {
    pub fn program_input(
        path: UpdateProgramLevelPathPayload,
        body: UpdateDiscountLevelBodyPayload,
        actor: String,
    ) -> Result<UpdateDiscountLevelInput, ApplicationError> {
        Ok(UpdateDiscountLevelInput {
            program: ResourceIdentifier::parse(&path.program),
            target: LevelTarget::Program,
            change: change_from(body.discount)?,
            actor,
        })
    }

    pub fn domain_input(
        path: UpdateDomainLevelPathPayload,
        body: UpdateDiscountLevelBodyPayload,
        actor: String,
    ) -> Result<UpdateDiscountLevelInput, ApplicationError> {
        Ok(UpdateDiscountLevelInput {
            program: ResourceIdentifier::parse(&path.program),
            target: LevelTarget::Domain(path.website_id.trim().to_string()),
            change: change_from(body.discount)?,
            actor,
        })
    }

    pub fn output_success(output: UpdateDiscountLevelOutput) -> DiscountLevelPresenter {
        let (level, website_id) = match output.target {
            LevelTarget::Program => ("program", None),
            LevelTarget::Domain(website) => ("domain", Some(website)),
        };
        DiscountLevelPresenter { level, website_id, discount: config_presenter(output.discount) }
    }
}
