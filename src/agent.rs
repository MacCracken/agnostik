use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::security::{Permission, SandboxConfig};
pub use crate::types::{AgentId, UserId, Version};

/// Agent type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum AgentType {
    System,
    #[default]
    User,
    Service,
}

/// Agent lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_used: u64,
    pub cpu_time_used: u64,
    pub file_descriptors_used: u32,
    pub processes_used: u32,
}

/// Restart policy for failed agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum RestartPolicy {
    /// Never restart.
    #[default]
    Never,
    /// Always restart, regardless of exit status.
    Always,
    /// Restart only on non-zero exit (failure).
    OnFailure,
}

/// Health check configuration (liveness/readiness probes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Interval between health checks in seconds.
    #[serde(default = "default_health_interval")]
    pub interval_secs: u32,
    /// Timeout for each health check in seconds.
    #[serde(default = "default_health_timeout")]
    pub timeout_secs: u32,
    /// Number of consecutive failures before marking unhealthy.
    #[serde(default = "default_health_retries")]
    pub retries: u32,
    /// Seconds to wait before first health check after start.
    #[serde(default)]
    pub initial_delay_secs: u32,
}

fn default_health_interval() -> u32 {
    30
}
fn default_health_timeout() -> u32 {
    5
}
fn default_health_retries() -> u32 {
    3
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self {
            interval_secs: default_health_interval(),
            timeout_secs: default_health_timeout(),
            retries: default_health_retries(),
            initial_delay_secs: 0,
        }
    }
}

/// Agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: AgentType,
    pub resource_limits: ResourceLimits,
    #[serde(default)]
    pub sandbox: SandboxConfig,
    #[serde(default)]
    pub permissions: Vec<Permission>,
    pub metadata: serde_json::Value,
    /// Restart policy on failure.
    #[serde(default)]
    pub restart_policy: RestartPolicy,
    /// Maximum restart attempts (0 = unlimited, only applies when restart_policy != Never).
    #[serde(default)]
    pub max_restarts: u32,
    /// Health check configuration.
    #[serde(default)]
    pub health_check: Option<HealthCheck>,
    /// Maximum seconds to wait for startup before marking failed.
    #[serde(default)]
    pub startup_timeout_secs: Option<u32>,
    /// Graceful shutdown timeout in seconds (SIGTERM → SIGKILL delay).
    #[serde(default)]
    pub shutdown_timeout_secs: Option<u32>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            agent_type: AgentType::User,
            resource_limits: ResourceLimits::default(),
            sandbox: SandboxConfig::default(),
            permissions: Vec::new(),
            metadata: serde_json::Value::Null,
            restart_policy: RestartPolicy::default(),
            max_restarts: 0,
            health_check: None,
            startup_timeout_secs: None,
            shutdown_timeout_secs: None,
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
    pub version: Version,
    #[serde(default)]
    pub homepage: String,
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
    pub agent_id: AgentId,
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// Agent runtime info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: AgentId,
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

// ---------------------------------------------------------------------------
// Agent-to-agent messaging
// ---------------------------------------------------------------------------

/// A typed message envelope for agent-to-agent communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Unique message ID.
    pub id: uuid::Uuid,
    pub sender: AgentId,
    pub receiver: AgentId,
    /// Correlation ID for request/response pairing.
    #[serde(default)]
    pub correlation_id: Option<uuid::Uuid>,
    /// ID of message this is replying to.
    #[serde(default)]
    pub reply_to: Option<uuid::Uuid>,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// Agent dependencies & resource negotiation
// ---------------------------------------------------------------------------

/// Declaration that an agent depends on another agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDependency {
    /// The agent that is required.
    pub required_agent: String,
    /// Minimum version required (SemVer).
    #[serde(default)]
    pub min_version: Option<Version>,
    /// Whether the dependency is mandatory or optional.
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

/// A resource request from an agent to the runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    pub agent_id: AgentId,
    pub requested_limits: ResourceLimits,
    #[serde(default)]
    pub justification: Option<String>,
}

/// The runtime's response to a resource request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGrant {
    pub agent_id: AgentId,
    pub granted_limits: ResourceLimits,
    pub approved: bool,
    #[serde(default)]
    pub reason: Option<String>,
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

    #[test]
    fn agent_type_serde_roundtrip() {
        for variant in [AgentType::System, AgentType::User, AgentType::Service] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: AgentType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn agent_status_serde_roundtrip() {
        for variant in [
            AgentStatus::Pending,
            AgentStatus::Starting,
            AgentStatus::Running,
            AgentStatus::Paused,
            AgentStatus::Stopping,
            AgentStatus::Stopped,
            AgentStatus::Failed,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: AgentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn resource_limits_serde_roundtrip() {
        let l = ResourceLimits::default();
        let json = serde_json::to_string(&l).unwrap();
        let back: ResourceLimits = serde_json::from_str(&json).unwrap();
        assert_eq!(l, back);
    }

    #[test]
    fn resource_usage_serde_roundtrip() {
        let u = ResourceUsage {
            memory_used: 512,
            cpu_time_used: 1000,
            file_descriptors_used: 10,
            processes_used: 2,
        };
        let json = serde_json::to_string(&u).unwrap();
        let back: ResourceUsage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.memory_used, 512);
        assert_eq!(back.cpu_time_used, 1000);
    }

    #[test]
    fn agent_rate_limit_serde_roundtrip() {
        let r = AgentRateLimit {
            max_tokens_per_hour: 10000,
            max_requests_per_minute: 60,
            max_concurrent_requests: 5,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: AgentRateLimit = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_tokens_per_hour, 10000);
        assert_eq!(back.max_requests_per_minute, 60);
    }

    #[test]
    fn agent_config_serde_roundtrip() {
        let c = AgentConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, c.name);
        assert_eq!(back.agent_type, c.agent_type);
        assert_eq!(back.restart_policy, RestartPolicy::Never);
    }

    #[test]
    fn restart_policy_serde_roundtrip() {
        for variant in [
            RestartPolicy::Never,
            RestartPolicy::Always,
            RestartPolicy::OnFailure,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: RestartPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn health_check_default() {
        let h = HealthCheck::default();
        assert_eq!(h.interval_secs, 30);
        assert_eq!(h.timeout_secs, 5);
        assert_eq!(h.retries, 3);
        assert_eq!(h.initial_delay_secs, 0);
    }

    #[test]
    fn health_check_serde_roundtrip() {
        let h = HealthCheck {
            interval_secs: 10,
            timeout_secs: 2,
            retries: 5,
            initial_delay_secs: 15,
        };
        let json = serde_json::to_string(&h).unwrap();
        let back: HealthCheck = serde_json::from_str(&json).unwrap();
        assert_eq!(back.interval_secs, 10);
        assert_eq!(back.retries, 5);
        assert_eq!(back.initial_delay_secs, 15);
    }

    #[test]
    fn agent_config_with_lifecycle() {
        let c = AgentConfig {
            restart_policy: RestartPolicy::OnFailure,
            max_restarts: 5,
            health_check: Some(HealthCheck::default()),
            startup_timeout_secs: Some(60),
            shutdown_timeout_secs: Some(30),
            ..AgentConfig::default()
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.restart_policy, RestartPolicy::OnFailure);
        assert_eq!(back.max_restarts, 5);
        assert!(back.health_check.is_some());
        assert_eq!(back.startup_timeout_secs, Some(60));
        assert_eq!(back.shutdown_timeout_secs, Some(30));
    }

    #[test]
    fn agent_manifest_serde_roundtrip() {
        let m = AgentManifest {
            name: "test-agent".into(),
            description: "A test agent".into(),
            ..AgentManifest::default()
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: AgentManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test-agent");
        assert_eq!(back.description, "A test agent");
    }

    #[test]
    fn agent_event_serde_roundtrip() {
        let id = AgentId::new();
        let e = AgentEvent {
            agent_id: id,
            event_type: "started".into(),
            payload: serde_json::json!({"status": "ok"}),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, id);
    }

    #[test]
    fn agent_info_serde_roundtrip() {
        let id = AgentId::new();
        let i = AgentInfo {
            id,
            name: "test".into(),
            agent_type: AgentType::User,
            status: AgentStatus::Running,
        };
        let json = serde_json::to_string(&i).unwrap();
        let back: AgentInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, id);
        assert_eq!(back.status, AgentStatus::Running);
    }

    #[test]
    fn agent_stats_serde_roundtrip() {
        let s = AgentStats {
            total_requests: 100,
            total_tokens: 5000,
            uptime_seconds: 3600,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: AgentStats = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total_requests, 100);
        assert_eq!(back.uptime_seconds, 3600);
    }

    #[test]
    fn stop_reason_serde_roundtrip() {
        for variant in [
            StopReason::UserRequested,
            StopReason::ResourceExhausted,
            StopReason::Error("oops".into()),
            StopReason::Timeout,
            StopReason::SystemShutdown,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let _back: StopReason = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn agent_message_serde_roundtrip() {
        let m = AgentMessage {
            id: uuid::Uuid::new_v4(),
            sender: AgentId::new(),
            receiver: AgentId::new(),
            correlation_id: Some(uuid::Uuid::new_v4()),
            reply_to: None,
            payload: serde_json::json!({"action": "delegate", "task": "search"}),
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: AgentMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, m.id);
        assert_eq!(back.sender, m.sender);
        assert_eq!(back.correlation_id, m.correlation_id);
    }

    #[test]
    fn agent_dependency_serde_roundtrip() {
        let d = AgentDependency {
            required_agent: "search-agent".into(),
            min_version: Some("1.0.0".parse().unwrap()),
            required: true,
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: AgentDependency = serde_json::from_str(&json).unwrap();
        assert_eq!(back.required_agent, "search-agent");
        assert!(back.required);
        assert_eq!(back.min_version.unwrap().major, 1);
    }

    #[test]
    fn resource_request_serde_roundtrip() {
        let rr = ResourceRequest {
            agent_id: AgentId::new(),
            requested_limits: ResourceLimits::default(),
            justification: Some("needs more memory for embeddings".into()),
        };
        let json = serde_json::to_string(&rr).unwrap();
        let back: ResourceRequest = serde_json::from_str(&json).unwrap();
        assert!(back.justification.is_some());
    }

    #[test]
    fn resource_grant_serde_roundtrip() {
        let rg = ResourceGrant {
            agent_id: AgentId::new(),
            granted_limits: ResourceLimits::default(),
            approved: true,
            reason: None,
        };
        let json = serde_json::to_string(&rg).unwrap();
        let back: ResourceGrant = serde_json::from_str(&json).unwrap();
        assert!(back.approved);
    }
}
