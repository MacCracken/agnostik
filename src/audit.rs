use crate::types::{AgentId, UserId, Version};
use serde::{Deserialize, Serialize};

/// Audit severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Integrity fields for tamper-evident audit chain (HMAC-SHA256 linked).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityFields {
    /// Integrity format version.
    pub version: Version,
    /// HMAC-SHA256 signature of this entry (hex-encoded, 64 chars).
    pub signature: String,
    /// SHA-256 hash of the previous entry (hex-encoded, 64 chars).
    pub previous_entry_hash: String,
}

/// Genesis hash for the first entry in an audit chain.
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

impl IntegrityFields {
    /// Create genesis integrity fields (first entry in chain).
    #[must_use]
    pub fn genesis(signature: String) -> Self {
        Self {
            version: Version {
                major: 1,
                minor: 0,
                patch: 0,
                prerelease: None,
                build: None,
            },
            signature,
            previous_entry_hash: GENESIS_HASH.into(),
        }
    }

    /// Whether this is a genesis (first) entry.
    #[must_use]
    pub fn is_genesis(&self) -> bool {
        self.previous_entry_hash == GENESIS_HASH
    }
}

/// An audit log entry with tamper-evident integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry ID.
    pub id: String,
    /// Correlation ID for tracing across services.
    #[serde(default)]
    pub correlation_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub agent_id: AgentId,
    pub action: String,
    pub severity: AuditSeverity,
    pub details: serde_json::Value,
    /// User who triggered the action (if applicable).
    #[serde(default)]
    pub user_id: Option<UserId>,
    /// Integrity chain fields.
    pub integrity: IntegrityFields,
}

/// Trait for audit log destinations.
pub trait AuditSink: Send + Sync {
    /// Append an entry to the audit log.
    fn append(&self, entry: &AuditEntry) -> crate::Result<()>;

    /// Verify the integrity chain of all entries.
    fn verify_chain(&self) -> crate::Result<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(AuditSeverity::Critical > AuditSeverity::Info);
        assert!(AuditSeverity::Debug < AuditSeverity::Warning);
    }

    #[test]
    fn audit_severity_serde_roundtrip() {
        for variant in [
            AuditSeverity::Debug,
            AuditSeverity::Info,
            AuditSeverity::Warning,
            AuditSeverity::Error,
            AuditSeverity::Critical,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: AuditSeverity = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn integrity_fields_genesis() {
        let i = IntegrityFields::genesis("abc123def456".into());
        assert!(i.is_genesis());
        assert_eq!(i.version.major, 1);
        assert_eq!(i.version.minor, 0);
        assert_eq!(i.version.patch, 0);
        assert_eq!(i.previous_entry_hash, GENESIS_HASH);
    }

    #[test]
    fn integrity_fields_non_genesis() {
        let i = IntegrityFields {
            version: "1.0.0".parse().unwrap(),
            signature: "sig".into(),
            previous_entry_hash: "abcd1234".into(),
        };
        assert!(!i.is_genesis());
    }

    #[test]
    fn integrity_fields_serde_roundtrip() {
        let i = IntegrityFields::genesis("sig123".into());
        let json = serde_json::to_string(&i).unwrap();
        let back: IntegrityFields = serde_json::from_str(&json).unwrap();
        assert_eq!(i, back);
    }

    #[test]
    fn audit_entry_serde_roundtrip() {
        let id = AgentId::new();
        let e = AuditEntry {
            id: "entry-001".into(),
            correlation_id: Some("corr-abc".into()),
            timestamp: chrono::Utc::now(),
            agent_id: id,
            action: "file_read".into(),
            severity: AuditSeverity::Info,
            details: serde_json::json!({"path": "/tmp/test"}),
            user_id: Some(UserId::new()),
            integrity: IntegrityFields::genesis("sig".into()),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: AuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, id);
        assert_eq!(back.id, "entry-001");
        assert_eq!(back.correlation_id.as_deref(), Some("corr-abc"));
        assert!(back.integrity.is_genesis());
    }

    #[test]
    fn audit_entry_minimal() {
        let e = AuditEntry {
            id: "entry-002".into(),
            correlation_id: None,
            timestamp: chrono::Utc::now(),
            agent_id: AgentId::new(),
            action: "login".into(),
            severity: AuditSeverity::Info,
            details: serde_json::Value::Null,
            user_id: None,
            integrity: IntegrityFields {
                version: "1.0.0".parse().unwrap(),
                signature: "prev_sig".into(),
                previous_entry_hash: "prev_hash".into(),
            },
        };
        let json = serde_json::to_string(&e).unwrap();
        let _back: AuditEntry = serde_json::from_str(&json).unwrap();
    }
}
