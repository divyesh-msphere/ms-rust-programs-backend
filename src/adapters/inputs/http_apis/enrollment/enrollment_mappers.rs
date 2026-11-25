use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::enrollment::batch_create_enrollments::{BatchCreateEnrollmentsInput, BatchItem, BatchResult};
use crate::application::usecases::enrollment::create_enrollment::CreateEnrollmentInput;
use crate::application::usecases::enrollment::get_enrollment::GetEnrollmentInput;
use crate::application::usecases::enrollment::list_enrollments::{ListEnrollmentsInput, ListEnrollmentsOutput};
use crate::application::usecases::enrollment::update_enrollment::UpdateEnrollmentInput;
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::enrollment::{BatchOutcome, BuUnit, Enrollment, allowed_transitions};
use crate::dto::payloads::json::enrollment::{
    BatchCreateEnrollmentsBodyPayload, BatchCreateEnrollmentsPathPayload, CreateEnrollmentBodyPayload,
    CreateEnrollmentPathPayload, CreateEnrollmentQueryPayload, GetEnrollmentPayload, ListEnrollmentsPathPayload,
    ListEnrollmentsQueryPayload, UpdateEnrollmentBodyPayload, UpdateEnrollmentPathPayload,
};
use crate::dto::presenters::json::enrollment::{
    BatchCreateEnrollmentsPresenter, BatchItemResultPresenter, BuLocationPresenter, BuPresenter, EnrollmentPresenter, ListEnrollmentsPresenter,
    WebsitePresenter,
};

pub fn to_presenter(enrollment: Enrollment) -> EnrollmentPresenter {
    EnrollmentPresenter {
        path: format!("programs/{}/stores/{}", enrollment.program_id, enrollment.id),
        id: enrollment.id.to_string(),
        application_id: enrollment.application_id.map(|id| id.to_string()),
        bu_location: BuLocationPresenter {
            code: enrollment.store_id,
            name: enrollment.store_name,
            bu: enrollment.bu_unit_code.map(|code| BuPresenter { code, name: enrollment.bu_unit_name }),
        },
        website: WebsitePresenter {
            code: enrollment.website_id,
            name: enrollment.website_name,
            domain: enrollment.website_domain,
        },
        customer_name: enrollment.customer_group,
        actions: allowed_transitions(&enrollment.status),
        status: enrollment.status,
        enrolled_time: enrollment.enrolled_time.map(format_datetime),
        contracts_count: enrollment.contracts_count,
        last_contract_time: enrollment.last_contract_time.map(format_datetime),
        create_time: format_datetime(enrollment.create_time),
        update_time: format_datetime(enrollment.update_time),
    }
}

pub struct ListEnrollmentsMapper;

impl ListEnrollmentsMapper {
    /// `pairs` is the whole query string. Besides `filter=`, a parameter named like a filter key
    /// (`website`, `status`, `bu_location`, `customer_name`, with or without `.operator` / `!`) is one more
    /// filter term, so `?status=active&website=w1` and `?filter=status=active%26website=w1` are the same.
    pub fn input(
        path: ListEnrollmentsPathPayload,
        query: ListEnrollmentsQueryPayload,
        pairs: Vec<(String, String)>,
    ) -> Result<ListEnrollmentsInput, ApplicationError> {
        let is_filter_key = |key: &str| {
            let head = key.split('.').next().unwrap_or("").trim_end_matches('!').to_lowercase();
            matches!(head.as_str(), "website" | "status" | "state" | "bu_location" | "customer_name")
        };
        let terms: Vec<String> = query
            .filter
            .into_iter()
            .chain(pairs.into_iter().filter(|(key, _)| is_filter_key(key)).map(|(key, value)| format!("{key}={value}")))
            .filter(|term| !term.trim().is_empty())
            .collect();
        Ok(ListEnrollmentsInput {
            program: ResourceIdentifier::parse(&path.program),
            search: query.search,
            filter: if terms.is_empty() { None } else { Some(terms.join("&")) },
            page_size: query.max_page_size,
            page_token: parse::optional_uuid("page_token", query.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListEnrollmentsOutput) -> ListEnrollmentsPresenter {
        ListEnrollmentsPresenter {
            results: output.enrollments.into_iter().map(to_presenter).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            total_size: output.total_size,
        }
    }
}

pub struct GetEnrollmentMapper;

impl GetEnrollmentMapper {
    pub fn input(path: GetEnrollmentPayload) -> Result<GetEnrollmentInput, ApplicationError> {
        Ok(GetEnrollmentInput {
            program: ResourceIdentifier::parse(&path.program),
            enrollment: parse::uuid("store", &path.store)?,
        })
    }

    pub fn output_success(output: Enrollment) -> EnrollmentPresenter {
        to_presenter(output)
    }
}

pub struct CreateEnrollmentMapper;

impl CreateEnrollmentMapper {
    pub fn input(
        path: CreateEnrollmentPathPayload,
        query: CreateEnrollmentQueryPayload,
        body: CreateEnrollmentBodyPayload,
        actor: String,
    ) -> Result<CreateEnrollmentInput, ApplicationError> {
        Ok(CreateEnrollmentInput {
            program: ResourceIdentifier::parse(&path.program),
            id: parse::optional_uuid("id", query.id.as_deref())?,
            store_id: body.bu_location.code,
            store_name: body.bu_location.name.clone(),
            website_id: body.website.code,
            website_name: body.website.name,
            website_domain: body.website.domain,
            bu_unit_code: body.bu_location.bu.as_ref().map(|b| b.code.clone()),
            bu_unit_name: body.bu_location.bu.and_then(|b| b.name),
            customer_group: body.customer_name,
            status: body.status.map(|s| s.trim().to_string()),
            actor,
        })
    }

    pub fn output_success(output: Enrollment) -> EnrollmentPresenter {
        to_presenter(output)
    }
}

pub struct UpdateEnrollmentMapper;

impl UpdateEnrollmentMapper {
    pub fn input(
        path: UpdateEnrollmentPathPayload,
        body: UpdateEnrollmentBodyPayload,
        actor: String,
    ) -> Result<UpdateEnrollmentInput, ApplicationError> {
        Ok(UpdateEnrollmentInput {
            program: ResourceIdentifier::parse(&path.program),
            enrollment: parse::uuid("store", &path.store)?,
            status: body.status.map(|s| s.trim().to_string()),
            customer_group: body.customer_name,
            store_name: body.bu_location.as_ref().and_then(|l| l.name.clone()),
            bu: body.bu_location.and_then(|l| l.bu).map(|bu| bu.map(|b| BuUnit { code: b.code, name: b.name })),
            actor,
        })
    }

    pub fn output_success(output: Enrollment) -> EnrollmentPresenter {
        to_presenter(output)
    }
}

pub struct BatchCreateEnrollmentsMapper;

impl BatchCreateEnrollmentsMapper {
    pub fn input(
        path: BatchCreateEnrollmentsPathPayload,
        body: BatchCreateEnrollmentsBodyPayload,
        actor: String,
    ) -> Result<BatchCreateEnrollmentsInput, ApplicationError> {
        Ok(BatchCreateEnrollmentsInput {
            program: ResourceIdentifier::parse(&path.program),
            items: body
                .items
                .into_iter()
                .map(|item| BatchItem {
                    store_id: item.bu_location.code,
                    store_name: item.bu_location.name.clone(),
                    website_id: item.website.code,
                    website_name: item.website.name,
                    website_domain: item.website.domain,
                    bu_unit_code: item.bu_location.bu.as_ref().map(|b| b.code.clone()),
                    bu_unit_name: item.bu_location.bu.and_then(|b| b.name),
                    customer_group: item.customer_name,
                    status: item.status.map(|s| s.trim().to_string()),
                })
                .collect(),
            actor,
        })
    }

    pub fn output_success(output: Vec<BatchResult>) -> BatchCreateEnrollmentsPresenter {
        let results: Vec<BatchItemResultPresenter> = output
            .into_iter()
            .map(|r| {
                let (outcome, id, error) = match r.outcome {
                    BatchOutcome::Created(id) => ("created", Some(id.to_string()), None),
                    BatchOutcome::AlreadyEnrolled => ("already_enrolled", None, None),
                    BatchOutcome::Failed(message) => ("failed", None, Some(message)),
                };
                BatchItemResultPresenter { bu_location_code: r.store_id, website_code: r.website_id, outcome, id, error }
            })
            .collect();
        BatchCreateEnrollmentsPresenter { created: results.iter().filter(|r| r.outcome == "created").count(), results }
    }
}
