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
}
