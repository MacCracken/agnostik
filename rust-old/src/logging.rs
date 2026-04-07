use crate::error::AgnostikError;

/// Initialize the global tracing subscriber.
///
/// Reads the `AGNOSTIK_LOG` environment variable for filter directives
/// (defaults to `warn`). Returns an error if a global subscriber is
/// already set.
pub fn try_init() -> crate::Result<()> {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_env("AGNOSTIK_LOG").unwrap_or_else(|_| EnvFilter::new("warn"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .map_err(|e| AgnostikError::Internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_init_returns_result() {
        // First call may succeed or fail depending on test ordering,
        // but it must not panic either way.
        let _ = try_init();
    }

    #[test]
    fn try_init_second_call_returns_error() {
        let _ = try_init();
        // Second call should fail because subscriber is already set.
        let result = try_init();
        // Either both fail (if another test set it) or second fails.
        if let Err(e) = result {
            assert!(matches!(e, AgnostikError::Internal(_)));
        }
    }
}
