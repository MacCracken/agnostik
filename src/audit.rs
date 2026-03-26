use crate::types::AgentId;
use serde::{Deserialize, Serialize};

/// Audit severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// An audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub agent_id: AgentId,
    pub action: String,
    pub severity: AuditSeverity,
    pub details: serde_json::Value,
    pub hash: String,
    pub previous_hash: String,
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
    fn audit_entry_serde() {
        let id = AgentId::new();
        let e = AuditEntry {
            timestamp: chrono::Utc::now(),
            agent_id: id,
            action: "file_read".into(),
            severity: AuditSeverity::Info,
            details: serde_json::json!({"path": "/tmp/test"}),
            hash: "abc123".into(),
            previous_hash: "000000".into(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: AuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, id);
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
}
