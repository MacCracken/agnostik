use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[cfg(feature = "security")]
use crate::security::{SandboxConfig, Permission, NetworkAccess, FsAccess, FilesystemRule};

/// Unique agent identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    #[must_use]
    pub fn new() -> Self { Self(Uuid::new_v4()) }

    #[must_use]
    pub fn from_uuid(id: Uuid) -> Self { Self(id) }
}

impl Default for AgentId {
    fn default() -> Self { Self::new() }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique user identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    #[must_use]
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl Default for UserId {
    fn default() -> Self { Self::new() }
}

/// Agent type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum AgentType {
    System,
    #[default]
    User,
    Service,
}

/// Agent lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AgentStatus {
    Pending,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed,
}

/// Resource limits for an agent.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory: u64,
    pub max_cpu_time: u64,
    pub max_file_descriptors: u32,
    pub max_processes: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 1024 * 1024 * 1024,
            max_cpu_time: 3600 * 1000,
            max_file_descriptors: 1024,
            max_processes: 64,
        }
    }
}

/// Current resource usage snapshot.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_used: u64,
    pub cpu_time_used: u64,
    pub file_descriptors_used: u32,
    pub processes_used: u32,
}

/// Agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: AgentType,
    pub resource_limits: ResourceLimits,
    #[cfg(feature = "security")]
    pub sandbox: SandboxConfig,
    #[cfg(feature = "security")]
    pub permissions: Vec<Permission>,
    pub metadata: serde_json::Value,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            agent_type: AgentType::User,
            resource_limits: ResourceLimits::default(),
            #[cfg(feature = "security")]
            sandbox: SandboxConfig::default(),
            #[cfg(feature = "security")]
            permissions: Vec::new(),
            metadata: serde_json::Value::Null,
        }
    }
}

/// Per-agent rate limit for LLM inference.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentRateLimit {
    #[serde(default)]
    pub max_tokens_per_hour: u64,
    #[serde(default)]
    pub max_requests_per_minute: u32,
    #[serde(default)]
    pub max_concurrent_requests: u32,
}

/// Declarative agent manifest for consent display.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentManifest {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub homepage: String,
    #[cfg(feature = "security")]
    #[serde(default)]
    pub requested_permissions: Vec<Permission>,
    #[serde(default)]
    pub permission_rationale: HashMap<String, String>,
    #[serde(default)]
    pub resource_limits: ResourceLimits,
    #[serde(default)]
    pub data_accessed: Vec<String>,
    #[serde(default)]
    pub external_services: Vec<String>,
}

/// Agent event for pub/sub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub agent_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// Agent runtime info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
}

/// Agent statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub uptime_seconds: u64,
}

/// Reason an agent was stopped.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StopReason {
    UserRequested,
    ResourceExhausted,
    Error(String),
    Timeout,
    SystemShutdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_id_unique() {
        let a = AgentId::new();
        let b = AgentId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn agent_id_display() {
        let id = AgentId::new();
        let s = format!("{id}");
        assert_eq!(s.len(), 36);
    }

    #[test]
    fn agent_id_serde_roundtrip() {
        let id = AgentId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: AgentId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn agent_type_default() {
        assert_eq!(AgentType::default(), AgentType::User);
    }

    #[test]
    fn resource_limits_default() {
        let l = ResourceLimits::default();
        assert_eq!(l.max_memory, 1024 * 1024 * 1024);
        assert_eq!(l.max_file_descriptors, 1024);
    }

    #[test]
    fn resource_usage_default_zero() {
        let u = ResourceUsage::default();
        assert_eq!(u.memory_used, 0);
    }

    #[test]
    fn rate_limit_default() {
        let r = AgentRateLimit::default();
        assert_eq!(r.max_tokens_per_hour, 0);
    }

    #[test]
    fn stop_reason_variants() {
        let _ = StopReason::UserRequested;
        let _ = StopReason::Timeout;
        let _ = StopReason::Error("oops".into());
    }

    #[test]
    fn user_id_unique() {
        let a = UserId::new();
        let b = UserId::new();
        assert_ne!(a, b);
    }
}
