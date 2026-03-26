use serde::{Deserialize, Serialize};

/// A secret value that is zeroized on drop.
#[derive(Debug, Clone)]
pub struct Secret {
    value: String,
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

#[cfg(feature = "secrets")]
impl Drop for Secret {
    fn drop(&mut self) {
        zeroize::Zeroize::zeroize(&mut self.value);
    }
}

/// Secret metadata (not the value itself).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub name: String,
    pub created_at: String,
    pub expires_at: Option<String>,
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
    fn secret_metadata_serde() {
        let m = SecretMetadata {
            name: "api-key".into(),
            created_at: "2026-03-25".into(),
            expires_at: None,
            rotation_policy: Some("90d".into()),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: SecretMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "api-key");
    }
}
