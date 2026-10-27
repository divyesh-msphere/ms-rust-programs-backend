use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchCreateEnrollmentsRequest, BatchCreateEnrollmentsResponse, CreateEnrollmentRequest, CreateEnrollmentResponse,
    Enrollment as GrpcEnrollment, EnrollmentItemResult, GetEnrollmentRequest, GetEnrollmentResponse,
    ListEnrollmentsRequest, ListEnrollmentsResponse, UpdateEnrollmentRequest, UpdateEnrollmentResponse,
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
use crate::domain::enrollment::{BatchOutcome, Enrollment, allowed_transitions};

fn to_grpc_enrollment(enrollment: Enrollment) -> GrpcEnrollment {
    GrpcEnrollment {
        path: format!("programs/{}/enrollments/{}", enrollment.program_id, enrollment.id),
        id: enrollment.id.to_string(),
        application_id: enrollment.application_id.map(|id| id.to_string()),
        store_id: enrollment.store_id,
        store_name: enrollment.store_name,
        store_resolved: enrollment.store_resolved,
        website_id: enrollment.website_id,
        website_name: enrollment.website_name,
        customer_group: enrollment.customer_group,
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
            website_id: request.website_id,
            status: request.status,
            store: request.store,
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
            enrollment: parse::uuid("enrollment", &request.enrollment)?,
        })
    }

    pub fn output_success(output: Enrollment) -> GetEnrollmentResponse {
        GetEnrollmentResponse { enrollment: Some(to_grpc_enrollment(output)).into(), ..Default::default() }
    }
}

pub struct CreateEnrollmentGrpcMapper;

impl CreateEnrollmentGrpcMapper {
    pub fn input(request: CreateEnrollmentRequest, actor: String) -> Result<CreateEnrollmentInput, ApplicationError> {
        Ok(CreateEnrollmentInput {
            program: ResourceIdentifier::parse(&request.program),
            id: parse::optional_uuid("id", request.id.as_deref())?,
            store_id: request.store_id,
            store_name: request.store_name,
            website_id: request.website_id,
            website_name: request.website_name,
            customer_group: request.customer_group,
            status: request.status.map(|s| s.trim().to_string()),
            actor,
        })
    }

    pub fn output_success(output: Enrollment) -> CreateEnrollmentResponse {
        CreateEnrollmentResponse { enrollment: Some(to_grpc_enrollment(output)).into(), ..Default::default() }
    }
}

pub struct UpdateEnrollmentGrpcMapper;

impl UpdateEnrollmentGrpcMapper {
    pub fn input(request: UpdateEnrollmentRequest, actor: String) -> Result<UpdateEnrollmentInput, ApplicationError> {
        Ok(UpdateEnrollmentInput {
            program: ResourceIdentifier::parse(&request.program),
            enrollment: parse::uuid("enrollment", &request.enrollment)?,
            status: request.status.map(|s| s.trim().to_string()),
            customer_group: request.customer_group,
            actor,
        })
    }

    pub fn output_success(output: Enrollment) -> UpdateEnrollmentResponse {
        UpdateEnrollmentResponse { enrollment: Some(to_grpc_enrollment(output)).into(), ..Default::default() }
    }
}

pub struct BatchCreateEnrollmentsGrpcMapper;

impl BatchCreateEnrollmentsGrpcMapper {
    pub fn input(request: BatchCreateEnrollmentsRequest, actor: String) -> Result<BatchCreateEnrollmentsInput, ApplicationError> {
        Ok(BatchCreateEnrollmentsInput {
            program: ResourceIdentifier::parse(&request.program),
            items: request
                .items
                .into_iter()
                .map(|item| BatchItem {
                    store_id: item.store_id,
                    store_name: item.store_name,
                    website_id: item.website_id,
                    website_name: item.website_name,
                    customer_group: item.customer_group,
                    status: item.status.map(|s| s.trim().to_string()),
                })
                .collect(),
            actor,
        })
    }

    pub fn output_success(output: Vec<BatchResult>) -> BatchCreateEnrollmentsResponse {
        let results: Vec<EnrollmentItemResult> = output
            .into_iter()
            .map(|r| {
                let (outcome, enrollment_id, error) = match r.outcome {
                    BatchOutcome::Created(id) => ("created", Some(id.to_string()), None),
                    BatchOutcome::AlreadyEnrolled => ("already_enrolled", None, None),
                    BatchOutcome::Failed(message) => ("failed", None, Some(message)),
                };
                EnrollmentItemResult {
                    store_id: r.store_id,
                    website_id: r.website_id,
                    outcome: outcome.to_string(),
                    enrollment_id,
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
