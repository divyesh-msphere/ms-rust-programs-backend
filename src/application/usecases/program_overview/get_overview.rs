use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::interface::UseCase;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program::{Program, ProgramRepository};
use crate::domain::program_overview::{Overview, ProgramOverviewRepository};
use std::sync::Arc;
use tracing::{Instrument, info_span, instrument};

/// How many entries "Recent activity" shows.
const RECENT_ACTIVITY: i64 = 10;

pub struct GetOverviewUseCase {
    program_repository: Arc<dyn ProgramRepository>,
    overview_repository: Arc<dyn ProgramOverviewRepository>,
}

impl GetOverviewUseCase {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        overview_repository: Arc<dyn ProgramOverviewRepository>,
    ) -> Self {
        Self { program_repository, overview_repository }
    }
}

#[derive(Debug)]
pub struct GetOverviewInput {
    pub program: ResourceIdentifier,
}

#[derive(Debug)]
pub struct GetOverviewOutput {
    pub program: Program,
    pub overview: Overview,
    /// The program changed after it was last published (or has changed since it was created and
    /// was never published): the page shows "unpublished changes" / Republish.
    pub unpublished_changes: bool,
}

/// A program that was never published has "unpublished changes" only once something was changed
/// (its `config_version` starts at 1); a published one, when it moved on since.
pub fn has_unpublished_changes(config_version: i32, published_config_version: Option<i32>) -> bool {
    match published_config_version {
        None => config_version > 1,
        Some(published) => published < config_version,
    }
}

#[async_trait::async_trait]
impl UseCase<GetOverviewOutput, GetOverviewInput> for GetOverviewUseCase {
    #[instrument(skip(self))]
    async fn execute(&self, input: Option<GetOverviewInput>) -> Result<GetOverviewOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::validation_error("Input is required in GetOverviewUseCase")
        })?;

        let program = self
            .program_repository
            .get(&input.program, false)
            .instrument(info_span!("GetOverviewUseCase::get_program"))
            .await?;

        let overview = self
            .overview_repository
            .get(program.id, RECENT_ACTIVITY)
            .instrument(info_span!("GetOverviewUseCase::get"))
            .await?;

        let unpublished_changes = has_unpublished_changes(program.config_version, overview.published_config_version);
        Ok(GetOverviewOutput { program, overview, unpublished_changes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpublished_changes_rules() {
        assert!(!has_unpublished_changes(1, None), "a brand-new program has nothing to publish yet");
        assert!(has_unpublished_changes(2, None), "changed since creation, never published");
        assert!(!has_unpublished_changes(5, Some(5)), "published at the current version");
        assert!(has_unpublished_changes(6, Some(5)), "changed since the last publication");
    }
}
