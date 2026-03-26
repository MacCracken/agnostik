use thiserror::Error;

/// Core error type for the AGNOS ecosystem.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AgnostikError {
    #[error("agent not found: {0}")]
    AgentNotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("configuration error: {0}")]
    ConfigError(String),

    #[error("timeout")]
    Timeout,

    #[error("resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("not supported: {0}")]
    NotSupported(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<serde_json::Error> for AgnostikError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

impl AgnostikError {
    /// Whether this error is retriable (transient failures).
    #[must_use]
    pub fn is_retriable(&self) -> bool {
        matches!(self, Self::Timeout | Self::ResourceExhausted(_))
    }
}

/// Convenience result type.
pub type Result<T> = std::result::Result<T, AgnostikError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let e = AgnostikError::AgentNotFound("agent-001".into());
        assert!(e.to_string().contains("agent-001"));
    }

    #[test]
    fn timeout_is_retriable() {
        assert!(AgnostikError::Timeout.is_retriable());
    }

    #[test]
    fn permission_denied_not_retriable() {
        assert!(!AgnostikError::PermissionDenied("test".into()).is_retriable());
    }

    #[test]
    fn resource_exhausted_is_retriable() {
        assert!(AgnostikError::ResourceExhausted("memory".into()).is_retriable());
    }

    #[test]
    fn config_error_not_retriable() {
        assert!(!AgnostikError::ConfigError("bad".into()).is_retriable());
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let e: AgnostikError = io_err.into();
        assert!(matches!(e, AgnostikError::Io(_)));
        assert!(e.to_string().contains("gone"));
    }

    #[test]
    fn from_serde_error() {
        let serde_err = serde_json::from_str::<String>("not json").unwrap_err();
        let e: AgnostikError = serde_err.into();
        assert!(matches!(e, AgnostikError::Serialization(_)));
    }

    #[test]
    fn io_not_retriable() {
        let io_err = std::io::Error::other("fail");
        let e: AgnostikError = io_err.into();
        assert!(!e.is_retriable());
    }
}
