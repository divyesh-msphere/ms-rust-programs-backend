use tracing::error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationErrorKind {
    InvalidArgument,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    Unauthenticated,
    FailedPrecondition,
    /// The change lost a race: the data moved on since the caller read it (for example a stale
    /// `version`). Reload and retry. HTTP 409, gRPC `Aborted`.
    Conflict,
    ResourceExhausted,
    /// A well-formed request whose content violates a resource-format rule
    /// (e.g. AEP-122 `code` pattern) — distinct from `InvalidArgument`
    /// (400) so this class of validation can map to 422 instead.
    UnprocessableEntity,
    Internal,
}

#[derive(Debug, Clone)]
pub struct ApplicationError {
    kind: ApplicationErrorKind,
    message: String,
    metadata: Option<serde_json::Value>,
}

impl std::error::Error for ApplicationError {}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl Default for ApplicationError {
    fn default() -> Self {
        Self {
            kind: ApplicationErrorKind::Internal,
            message: "An unknown error occurred".to_string(),
            metadata: None,
        }
    }
}

impl ApplicationError {
    pub fn kind(&self) -> ApplicationErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }

    pub fn unauthenticated(msg: &str) -> Self {
        Self::into::<std::io::Error>(None, ApplicationErrorKind::Unauthenticated, Some(msg), None)
    }

    pub fn validation_error(msg: &str) -> Self {
        Self::into::<std::io::Error>(
            None,
            ApplicationErrorKind::InvalidArgument,
            Some(format!("Validation error: {}", msg)),
            None,
        )
    }

    pub fn unprocessable_entity(msg: &str) -> Self {
        Self::into::<std::io::Error>(
            None,
            ApplicationErrorKind::UnprocessableEntity,
            Some(msg.to_string()),
            None,
        )
    }

    pub fn not_found<T: ToString>(resource: &str, id: T) -> Self {
        Self::into::<std::io::Error>(
            None,
            ApplicationErrorKind::NotFound,
            Some(format!("{} with ID {} not found", resource, id.to_string())),
            None,
        )
    }

    pub fn not_found_with_message(msg: &str) -> Self {
        Self::into::<std::io::Error>(
            None,
            ApplicationErrorKind::NotFound,
            Some(msg.to_string()),
            None,
        )
    }

    pub fn already_exists(msg: &str) -> Self {
        Self::into::<std::io::Error>(
            None,
            ApplicationErrorKind::AlreadyExists,
            Some(msg.to_string()),
            None,
        )
    }

    pub fn conflict(msg: &str) -> Self {
        Self::into::<std::io::Error>(
            None,
            ApplicationErrorKind::Conflict,
            Some(msg.to_string()),
            None,
        )
    }

    pub fn failed_precondition(msg: &str) -> Self {
        Self::into::<std::io::Error>(
            None,
            ApplicationErrorKind::FailedPrecondition,
            Some(msg.to_string()),
            None,
        )
    }

    pub fn internal_error<E: std::error::Error + 'static>(err: Option<E>) -> Self {
        Self::into(
            err,
            ApplicationErrorKind::Internal,
            Some("Internal Server Error".to_string()),
            None,
        )
    }

    pub fn internal_error_with_message<E: std::error::Error + 'static>(
        err: Option<E>,
        msg: &str,
    ) -> Self {
        error!("Internal message: {msg}");
        Self::into(
            err,
            ApplicationErrorKind::Internal,
            Some("Internal Server Error".to_string()),
            None,
        )
    }

    #[track_caller]
    pub fn into<E: std::error::Error + 'static>(
        err: Option<E>,
        kind: ApplicationErrorKind,
        msg: Option<impl Into<String>>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        let location = std::panic::Location::caller();

        let message = match (&err, msg) {
            (Some(error), Some(m)) => {
                let m = m.into();
                error!(
                    "Error at {}:{} - {:?}, message: {}",
                    location.file(),
                    location.line(),
                    error,
                    m
                );
                m
            }
            (Some(error), None) => {
                error!(
                    "Error at {}:{} - {:?}",
                    location.file(),
                    location.line(),
                    error
                );
                error.to_string()
            }
            (None, Some(m)) => {
                let m = m.into();
                error!("Error at {}:{} - {}", location.file(), location.line(), m);
                m
            }
            (None, None) => {
                error!("Unknown error at {}:{}", location.file(), location.line());
                "Unknown error".to_string()
            }
        };

        Self {
            kind,
            message,
            metadata,
        }
    }
}
