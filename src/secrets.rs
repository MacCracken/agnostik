use serde::{Deserialize, Serialize};

/// A secret value that is zeroized on drop.
///
/// `Debug` output is redacted to prevent accidental leaks in logs or panics.
/// `Serialize`/`Deserialize` are intentionally omitted — use [`Secret::expose`]
/// explicitly when the raw value is needed, and [`SecretMetadata`] for
/// serializable metadata about the secret.
#[derive(Clone)]
pub struct Secret {
    value: String,
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secret")
            .field("value", &"[REDACTED]")
            .finish()
    }
}

impl Secret {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    #[must_use]
    pub fn expose(&self) -> &str {
        &self.value
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.value.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        zeroize::Zeroize::zeroize(&mut self.value);
    }
}

/// Secret metadata (not the value itself).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rotation_policy: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_expose() {
        let s = Secret::new("hunter2");
        assert_eq!(s.expose(), "hunter2");
    }

    #[test]
    fn secret_len() {
        let s = Secret::new("test");
        assert_eq!(s.len(), 4);
        assert!(!s.is_empty());
    }

    #[test]
    fn empty_secret() {
        let s = Secret::new("");
        assert!(s.is_empty());
    }

    #[test]
    fn secret_debug_redacted() {
        let s = Secret::new("super-secret-value");
        let debug = format!("{s:?}");
        assert!(debug.contains("REDACTED"));
        assert!(!debug.contains("super-secret-value"));
    }

    #[test]
    fn secret_metadata_serde() {
        let m = SecretMetadata {
            name: "api-key".into(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            rotation_policy: Some("90d".into()),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: SecretMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "api-key");
    }
}
