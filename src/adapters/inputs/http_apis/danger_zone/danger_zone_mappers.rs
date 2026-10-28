use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::danger_zone::{DangerZoneInput, DangerZoneOutput};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::danger_zone::DangerZone;
use crate::dto::payloads::json::danger_zone::DangerZonePayload;
use crate::dto::presenters::json::danger_zone::{DangerZonePresenter, DeleteBlockersPresenter};

fn to_presenter(page: DangerZone) -> DangerZonePresenter {
    DangerZonePresenter {
        path: page.path(),
        archived: page.archived(),
        archive_time: page.archive_time.map(format_datetime),
        can_archive: page.can_archive(),
        can_unarchive: page.can_unarchive(),
        can_delete: page.can_delete(),
        delete_blocked_reason: page.blockers.reason(),
        delete_blockers: DeleteBlockersPresenter {
            enrollments: page.blockers.enrollments,
            applications: page.blockers.applications,
            publications: page.blockers.publications,
            documents: page.blockers.documents,
        },
    }
}

/// Get, archive and unarchive share one shape: the page after the call.
pub struct DangerZoneMapper;

impl DangerZoneMapper {
    pub fn input(payload: DangerZonePayload, actor: String) -> Result<DangerZoneInput, ApplicationError> {
        Ok(DangerZoneInput { program: ResourceIdentifier::parse(&payload.program), actor })
    }

    pub fn output_success(output: DangerZoneOutput) -> DangerZonePresenter {
        to_presenter(output.danger_zone)
    }
}
