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

/// Kind of secret stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum SecretKind {
    #[default]
    Opaque,
    ApiKey,
    Password,
    Certificate,
    SshKey,
    Token,
}

/// Secret metadata (not the value itself).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_policy: Option<String>,
    /// Kind of secret.
    #[serde(default)]
    pub kind: SecretKind,
    /// Searchable tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Owner (user or service that created this secret).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// When the secret was last accessed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_accessed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// When the secret was last rotated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_rotated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Trait for pluggable secret storage backends.
pub trait SecretStore: Send + Sync {
    /// Retrieve a secret by name.
    fn get(&self, name: &str) -> crate::Result<Secret>;

    /// Store a secret.
    fn put(&self, name: &str, value: Secret) -> crate::Result<()>;

    /// Delete a secret by name.
    fn delete(&self, name: &str) -> crate::Result<()>;

    /// List metadata for all stored secrets.
    fn list_metadata(&self) -> crate::Result<Vec<SecretMetadata>>;

    /// Rotate a secret: generate a new value and return it.
    fn rotate(&self, _name: &str) -> crate::Result<Secret> {
        Err(crate::AgnostikError::NotSupported(
            "SecretStore::rotate".into(),
        ))
    }

    /// Search secrets by tag.
    fn search_by_tag(&self, tag: &str) -> crate::Result<Vec<SecretMetadata>> {
        let _ = tag;
        Err(crate::AgnostikError::NotSupported(
            "SecretStore::search_by_tag".into(),
        ))
    }
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
            kind: SecretKind::ApiKey,
            tags: vec!["production".into(), "external".into()],
            owner: Some("platform-team".into()),
            last_accessed_at: None,
            last_rotated_at: None,
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: SecretMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "api-key");
        assert_eq!(back.kind, SecretKind::ApiKey);
        assert_eq!(back.tags, vec!["production", "external"]);
        assert_eq!(back.owner.as_deref(), Some("platform-team"));
    }

    #[test]
    fn secret_kind_serde_roundtrip() {
        for variant in [
            SecretKind::Opaque,
            SecretKind::ApiKey,
            SecretKind::Password,
            SecretKind::Certificate,
            SecretKind::SshKey,
            SecretKind::Token,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: SecretKind = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }
}
