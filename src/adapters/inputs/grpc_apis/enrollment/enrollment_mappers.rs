use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchCreateStoresRequest as BatchCreateEnrollmentsRequest, BatchCreateStoresResponse as BatchCreateEnrollmentsResponse,
    BuLocation, CreateStoreRequest as CreateEnrollmentRequest, CreateStoreResponse as CreateEnrollmentResponse,
    GetStoreRequest as GetEnrollmentRequest, GetStoreResponse as GetEnrollmentResponse,
    ListStoresRequest as ListEnrollmentsRequest, ListStoresResponse as ListEnrollmentsResponse, Store as GrpcEnrollment,
    BusinessUnit, StoreItemResult as EnrollmentItemResult, StoreWebsite, UpdateStoreRequest as UpdateEnrollmentRequest,
    UpdateStoreResponse as UpdateEnrollmentResponse,
};
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

fn required<T>(field: Option<T>, name: &str) -> Result<T, ApplicationError> {
    field.ok_or_else(|| ApplicationError::validation_error(&format!("{name} is required")))
}

fn to_grpc_enrollment(enrollment: Enrollment) -> GrpcEnrollment {
    GrpcEnrollment {
        path: format!("programs/{}/stores/{}", enrollment.program_id, enrollment.id),
        id: enrollment.id.to_string(),
        application_id: enrollment.application_id.map(|id| id.to_string()),
        bu_location: Some(BuLocation {
            code: enrollment.store_id,
            name: enrollment.store_name,
            bu: enrollment
                .bu_unit_code
                .map(|code| BusinessUnit { code, name: enrollment.bu_unit_name, ..Default::default() })
                .into(),
            ..Default::default()
        })
        .into(),
        website: Some(StoreWebsite {
            code: enrollment.website_id,
            name: enrollment.website_name,
            domain: enrollment.website_domain,
            ..Default::default()
        })
        .into(),
        customer_name: enrollment.customer_group,
        actions: allowed_transitions(&enrollment.status).into_iter().map(String::from).collect(),
        status: enrollment.status,
        enrolled_time: enrollment.enrolled_time.map(format_datetime),
        contracts_count: enrollment.contracts_count,
        last_contract_time: enrollment.last_contract_time.map(format_datetime),
        create_time: format_datetime(enrollment.create_time),
        update_time: format_datetime(enrollment.update_time),
        ..Default::default()
    }
}

pub struct ListEnrollmentsGrpcMapper;

impl ListEnrollmentsGrpcMapper {
    pub fn input(request: ListEnrollmentsRequest) -> Result<ListEnrollmentsInput, ApplicationError> {
        Ok(ListEnrollmentsInput {
            program: ResourceIdentifier::parse(&request.program),
            search: request.search,
            filter: {
                // The flat fields are filter terms too.
                let terms: Vec<String> = request
                    .filter
                    .into_iter()
                    .chain(request.website.map(|v| format!("website={v}")))
                    .chain(request.status.map(|v| format!("status={v}")))
                    .chain(request.bu_location.map(|v| format!("bu_location={v}")))
                    .filter(|term| !term.trim().is_empty())
                    .collect();
                if terms.is_empty() { None } else { Some(terms.join("&")) }
            },
            page_size: request.max_page_size,
            page_token: parse::optional_uuid("page_token", request.page_token.as_deref())?,
        })
    }

    pub fn output_success(output: ListEnrollmentsOutput) -> ListEnrollmentsResponse {
        ListEnrollmentsResponse {
            results: output.enrollments.into_iter().map(to_grpc_enrollment).collect(),
            next_page_token: output.next_page_token.map(|id| id.to_string()),
            total_size: output.total_size,
            ..Default::default()
        }
    }
}

pub struct GetEnrollmentGrpcMapper;

impl GetEnrollmentGrpcMapper {
    pub fn input(request: GetEnrollmentRequest) -> Result<GetEnrollmentInput, ApplicationError> {
        Ok(GetEnrollmentInput {
            program: ResourceIdentifier::parse(&request.program),
            enrollment: parse::uuid("store", &request.store)?,
        })
    }

    pub fn output_success(output: Enrollment) -> GetEnrollmentResponse {
        GetEnrollmentResponse { store: Some(to_grpc_enrollment(output)).into(), ..Default::default() }
    }
}

pub struct CreateEnrollmentGrpcMapper;

impl CreateEnrollmentGrpcMapper {
    pub fn input(request: CreateEnrollmentRequest, actor: String) -> Result<CreateEnrollmentInput, ApplicationError> {
        let bu_location = required(request.bu_location.into_option(), "bu_location")?;
        let website = required(request.website.into_option(), "website")?;
        let bu = bu_location.bu.into_option();
        Ok(CreateEnrollmentInput {
            program: ResourceIdentifier::parse(&request.program),
            id: parse::optional_uuid("id", request.id.as_deref())?,
            store_id: bu_location.code,
            store_name: bu_location.name.clone(),
            website_id: website.code,
            website_name: website.name,
            website_domain: website.domain,
            bu_unit_code: bu.as_ref().map(|b| b.code.clone()),
            bu_unit_name: bu.and_then(|b| b.name),
            customer_group: request.customer_name,
            status: request.status.map(|s| s.trim().to_string()),
            actor,
        })
    }

    pub fn output_success(output: Enrollment) -> CreateEnrollmentResponse {
        CreateEnrollmentResponse { store: Some(to_grpc_enrollment(output)).into(), ..Default::default() }
    }
}

pub struct UpdateEnrollmentGrpcMapper;

impl UpdateEnrollmentGrpcMapper {
    pub fn input(request: UpdateEnrollmentRequest, actor: String) -> Result<UpdateEnrollmentInput, ApplicationError> {
        Ok(UpdateEnrollmentInput {
            program: ResourceIdentifier::parse(&request.program),
            enrollment: parse::uuid("store", &request.store)?,
            status: request.status.map(|s| s.trim().to_string()),
            customer_group: request.customer_name,
            // A `bu` with an empty code clears the business unit.
            store_name: request.bu_location.as_option().and_then(|l| l.name.clone()),
            bu: request
                .bu_location
                .into_option()
                .and_then(|l| l.bu.into_option())
                .map(|b| if b.code.trim().is_empty() { None } else { Some(BuUnit { code: b.code, name: b.name }) }),
            actor,
        })
    }

    pub fn output_success(output: Enrollment) -> UpdateEnrollmentResponse {
        UpdateEnrollmentResponse { store: Some(to_grpc_enrollment(output)).into(), ..Default::default() }
    }
}

pub struct BatchCreateEnrollmentsGrpcMapper;

impl BatchCreateEnrollmentsGrpcMapper {
    pub fn input(request: BatchCreateEnrollmentsRequest, actor: String) -> Result<BatchCreateEnrollmentsInput, ApplicationError> {
        let items = request
            .items
            .into_iter()
            .map(|item| {
                let bu_location = required(item.bu_location.into_option(), "items.bu_location")?;
                let website = required(item.website.into_option(), "items.website")?;
                let bu = bu_location.bu.into_option();
                Ok(BatchItem {
                    store_id: bu_location.code,
                    store_name: bu_location.name.clone(),
                    website_id: website.code,
                    website_name: website.name,
                    website_domain: website.domain,
                    bu_unit_code: bu.as_ref().map(|b| b.code.clone()),
                    bu_unit_name: bu.and_then(|b| b.name),
                    customer_group: item.customer_name,
                    status: item.status.map(|s| s.trim().to_string()),
                })
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?;
        Ok(BatchCreateEnrollmentsInput { program: ResourceIdentifier::parse(&request.program), items, actor })
    }

    pub fn output_success(output: Vec<BatchResult>) -> BatchCreateEnrollmentsResponse {
        let results: Vec<EnrollmentItemResult> = output
            .into_iter()
            .map(|r| {
                let (outcome, id, error) = match r.outcome {
                    BatchOutcome::Created(id) => ("created", Some(id.to_string()), None),
                    BatchOutcome::AlreadyEnrolled => ("already_enrolled", None, None),
                    BatchOutcome::Failed(message) => ("failed", None, Some(message)),
                };
                EnrollmentItemResult {
                    bu_location_code: r.store_id,
                    website_code: r.website_id,
                    outcome: outcome.to_string(),
                    id,
                    error,
                    ..Default::default()
                }
            })
            .collect();
        BatchCreateEnrollmentsResponse {
            created: results.iter().filter(|r| r.outcome == "created").count() as i32,
            results,
            ..Default::default()
        }
    }
}
