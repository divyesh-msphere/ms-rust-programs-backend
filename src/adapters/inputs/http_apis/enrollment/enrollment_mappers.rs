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
use crate::dto::payloads::json::enrollment::{
    BatchCreateEnrollmentsBodyPayload, BatchCreateEnrollmentsPathPayload, CreateEnrollmentBodyPayload,
    CreateEnrollmentPathPayload, CreateEnrollmentQueryPayload, GetEnrollmentPayload, ListEnrollmentsPathPayload,
    ListEnrollmentsQueryPayload, UpdateEnrollmentBodyPayload, UpdateEnrollmentPathPayload,
};
use crate::dto::presenters::json::enrollment::{
    BatchCreateEnrollmentsPresenter, BatchItemResultPresenter, EnrollmentPresenter, ListEnrollmentsPresenter,
};

pub fn to_presenter(enrollment: Enrollment) -> EnrollmentPresenter {
    EnrollmentPresenter {
        path: format!("programs/{}/enrollments/{}", enrollment.program_id, enrollment.id),
        id: enrollment.id.to_string(),
        application_id: enrollment.application_id.map(|id| id.to_string()),
        store_id: enrollment.store_id,
        store_name: enrollment.store_name,
        store_resolved: enrollment.store_resolved,
        website_id: enrollment.website_id,
        website_name: enrollment.website_name,
        customer_group: enrollment.customer_group,
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
    pub fn input(
        path: ListEnrollmentsPathPayload,
        query: ListEnrollmentsQueryPayload,
    ) -> Result<ListEnrollmentsInput, ApplicationError> {
        Ok(ListEnrollmentsInput {
            program: ResourceIdentifier::parse(&path.program),
            website_id: query.website_id,
            status: query.status,
            store: query.store,
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
            enrollment: parse::uuid("enrollment", &path.enrollment)?,
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
            store_id: body.store_id,
            store_name: body.store_name,
            website_id: body.website_id,
            website_name: body.website_name,
            customer_group: body.customer_group,
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
            enrollment: parse::uuid("enrollment", &path.enrollment)?,
            status: body.status.map(|s| s.trim().to_string()),
            customer_group: body.customer_group,
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

    pub fn output_success(output: Vec<BatchResult>) -> BatchCreateEnrollmentsPresenter {
        let results: Vec<BatchItemResultPresenter> = output
            .into_iter()
            .map(|r| {
                let (outcome, enrollment_id, error) = match r.outcome {
                    BatchOutcome::Created(id) => ("created", Some(id.to_string()), None),
                    BatchOutcome::AlreadyEnrolled => ("already_enrolled", None, None),
                    BatchOutcome::Failed(message) => ("failed", None, Some(message)),
                };
                BatchItemResultPresenter { store_id: r.store_id, website_id: r.website_id, outcome, enrollment_id, error }
            })
            .collect();
        BatchCreateEnrollmentsPresenter { created: results.iter().filter(|r| r.outcome == "created").count(), results }
    }
}
