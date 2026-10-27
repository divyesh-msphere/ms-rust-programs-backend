use crate::application::errors::application_error::ApplicationError;
use ms_telemetry_rs::tracing::error;

/// Logs a database error and turns it into the internal error every repository returns.
pub(crate) fn db_error(err: sqlx::Error) -> ApplicationError {
    error!("DB error: {:?}", err);
    ApplicationError::internal_error(Some(err))
}
