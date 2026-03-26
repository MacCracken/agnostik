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
    pub timestamp: String,
    pub agent_id: String,
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
        let e = AuditEntry {
            timestamp: "2026-03-25T00:00:00Z".into(),
            agent_id: "agent-001".into(),
            action: "file_read".into(),
            severity: AuditSeverity::Info,
            details: serde_json::json!({"path": "/tmp/test"}),
            hash: "abc123".into(),
            previous_hash: "000000".into(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: AuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, "agent-001");
    }
}
