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
