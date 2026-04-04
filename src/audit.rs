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

/// Whether the audited action succeeded or failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum AuditResult {
    #[default]
    Success,
    Failure,
    Denied,
}

/// An audit log entry with tamper-evident integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry ID.
    pub id: String,
    /// Correlation ID for tracing across services.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub agent_id: AgentId,
    pub action: String,
    pub severity: AuditSeverity,
    /// Whether the action succeeded or failed.
    #[serde(default)]
    pub result: AuditResult,
    pub details: serde_json::Value,
    /// User who triggered the action (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    /// Source IP address of the request (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ip: Option<String>,
    /// Resource that was the target of the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_resource: Option<String>,
    /// Duration of the audited operation in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Searchable tags for filtering.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Integrity chain fields.
    pub integrity: IntegrityFields,
}

/// Audit log retention policy (SOC2 CC7.2 compliance).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Minimum retention period in days (e.g., 90 for SOC2).
    pub min_retention_days: u32,
    /// Maximum retention period in days (for GDPR right-to-erasure).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retention_days: Option<u32>,
    /// Whether to archive entries after the retention period (vs. delete).
    #[serde(default)]
    pub archive_on_expiry: bool,
}

/// Trait for audit log destinations.
pub trait AuditSink: Send + Sync {
    /// Append an entry to the audit log.
    fn append(&self, entry: &AuditEntry) -> crate::Result<()>;

    /// Verify the integrity chain of all entries.
    fn verify_chain(&self) -> crate::Result<bool>;

    /// Verify a single entry against its claimed previous hash.
    fn verify_entry(&self, entry: &AuditEntry) -> crate::Result<bool> {
        let _ = entry;
        Err(crate::AgnostikError::NotSupported(
            "single-entry verification not implemented".into(),
        ))
    }

    /// Query entries by time range.
    fn query(
        &self,
        _from: chrono::DateTime<chrono::Utc>,
        _to: chrono::DateTime<chrono::Utc>,
    ) -> crate::Result<Vec<AuditEntry>> {
        Err(crate::AgnostikError::NotSupported(
            "query not implemented".into(),
        ))
    }

    /// Seal the current chain state (periodic integrity proof for compliance).
    fn seal(&self) -> crate::Result<String> {
        Err(crate::AgnostikError::NotSupported(
            "seal not implemented".into(),
        ))
    }
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
            result: AuditResult::Success,
            details: serde_json::json!({"path": "/tmp/test"}),
            user_id: Some(UserId::new()),
            source_ip: Some("10.0.0.1".into()),
            target_resource: Some("/tmp/test".into()),
            duration_ms: Some(42),
            tags: vec!["filesystem".into(), "read".into()],
            integrity: IntegrityFields::genesis("sig".into()),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: AuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, id);
        assert_eq!(back.id, "entry-001");
        assert_eq!(back.correlation_id.as_deref(), Some("corr-abc"));
        assert_eq!(back.result, AuditResult::Success);
        assert_eq!(back.source_ip.as_deref(), Some("10.0.0.1"));
        assert_eq!(back.target_resource.as_deref(), Some("/tmp/test"));
        assert_eq!(back.duration_ms, Some(42));
        assert_eq!(back.tags, vec!["filesystem", "read"]);
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
            result: AuditResult::default(),
            details: serde_json::Value::Null,
            user_id: None,
            source_ip: None,
            target_resource: None,
            duration_ms: None,
            tags: vec![],
            integrity: IntegrityFields {
                version: "1.0.0".parse().unwrap(),
                signature: "prev_sig".into(),
                previous_entry_hash: "prev_hash".into(),
            },
        };
        let json = serde_json::to_string(&e).unwrap();
        let _back: AuditEntry = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn audit_result_serde_roundtrip() {
        for variant in [
            AuditResult::Success,
            AuditResult::Failure,
            AuditResult::Denied,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: AuditResult = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn audit_entry_defaults_on_deserialize() {
        // New fields default correctly when deserializing old data
        let json = r#"{"id":"e1","timestamp":"2026-04-02T00:00:00Z","agent_id":"00000000-0000-0000-0000-000000000001","action":"test","severity":"Info","details":null,"integrity":{"version":"1.0.0","signature":"s","previous_entry_hash":"h"}}"#;
        let e: AuditEntry = serde_json::from_str(json).unwrap();
        assert_eq!(e.result, AuditResult::Success);
        assert!(e.source_ip.is_none());
        assert!(e.target_resource.is_none());
        assert!(e.duration_ms.is_none());
        assert!(e.tags.is_empty());
    }

    #[test]
    fn retention_policy_serde_roundtrip() {
        let rp = RetentionPolicy {
            min_retention_days: 90,
            max_retention_days: Some(365),
            archive_on_expiry: true,
        };
        let json = serde_json::to_string(&rp).unwrap();
        let back: RetentionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back.min_retention_days, 90);
        assert_eq!(back.max_retention_days, Some(365));
        assert!(back.archive_on_expiry);
    }
}
