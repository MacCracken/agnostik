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
    Restarting,
    Terminated,
    Failed,
}

/// Resource limits for an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory: u64,
    pub max_cpu_time: u64,
    pub max_file_descriptors: u32,
    pub max_processes: u32,
    /// Maximum disk usage in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_disk_bytes: Option<u64>,
    /// Maximum network bandwidth in bytes per second.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_bandwidth_bps: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 1024 * 1024 * 1024,
            max_cpu_time: 3600 * 1000,
            max_file_descriptors: 1024,
            max_processes: 64,
            max_disk_bytes: None,
            network_bandwidth_bps: None,
        }
    }
}

/// Lifecycle hook definition for agent startup/shutdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleHook {
    /// Command to execute.
    pub command: Vec<String>,
    /// Timeout for the hook in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u32>,
}

/// Lifecycle hooks for agent orchestration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LifecycleHooks {
    /// Runs before the agent process starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_start: Option<LifecycleHook>,
    /// Runs after the agent process starts successfully.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_start: Option<LifecycleHook>,
    /// Runs before sending the stop signal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_stop: Option<LifecycleHook>,
    /// Runs after the agent process exits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_stop: Option<LifecycleHook>,
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

/// Backoff configuration for agent restarts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartBackoff {
    /// Initial delay before first restart in milliseconds.
    #[serde(default = "default_restart_delay")]
    pub initial_delay_ms: u64,
    /// Multiplier applied after each restart (delay *= multiplier).
    #[serde(default = "default_restart_multiplier")]
    pub multiplier: f64,
    /// Maximum delay between restarts in milliseconds.
    #[serde(default = "default_restart_max_delay")]
    pub max_delay_ms: u64,
    /// Add random jitter to avoid thundering herd.
    #[serde(default)]
    pub jitter: bool,
}

fn default_restart_delay() -> u64 {
    1000
}
fn default_restart_multiplier() -> f64 {
    2.0
}
fn default_restart_max_delay() -> u64 {
    60_000
}

impl Default for RestartBackoff {
    fn default() -> Self {
        Self {
            initial_delay_ms: default_restart_delay(),
            multiplier: default_restart_multiplier(),
            max_delay_ms: default_restart_max_delay(),
            jitter: true,
        }
    }
}

/// Health check probe type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum ProbeType {
    /// Combined liveness + readiness (default for backward compatibility).
    #[default]
    LivenessReadiness,
    /// Liveness only — failure triggers restart.
    Liveness,
    /// Readiness only — failure removes from routing, does not restart.
    Readiness,
    /// Startup probe — checked only during startup, overrides liveness.
    Startup,
}

/// Health check configuration (liveness/readiness probes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Probe type (liveness, readiness, or combined).
    #[serde(default)]
    pub probe_type: ProbeType,
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
            probe_type: ProbeType::default(),
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<Permission>,
    pub metadata: serde_json::Value,
    /// Restart policy on failure.
    #[serde(default)]
    pub restart_policy: RestartPolicy,
    /// Maximum restart attempts (0 = unlimited, only applies when restart_policy != Never).
    #[serde(default)]
    pub max_restarts: u32,
    /// Health check configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_check: Option<HealthCheck>,
    /// Maximum seconds to wait for startup before marking failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startup_timeout_secs: Option<u32>,
    /// Graceful shutdown timeout in seconds (SIGTERM → SIGKILL delay).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shutdown_timeout_secs: Option<u32>,
    /// Lifecycle hooks (pre/post start/stop).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle_hooks: Option<LifecycleHooks>,
    /// Backoff configuration for restart delays.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart_backoff: Option<RestartBackoff>,
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
            lifecycle_hooks: None,
            restart_backoff: None,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requested_permissions: Vec<Permission>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub permission_rationale: HashMap<String, String>,
    #[serde(default)]
    pub resource_limits: ResourceLimits,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data_accessed: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<uuid::Uuid>,
    /// ID of message this is replying to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub justification: Option<String>,
}

/// The runtime's response to a resource request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGrant {
    pub agent_id: AgentId,
    pub granted_limits: ResourceLimits,
    pub approved: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Task / Execution identity
// ---------------------------------------------------------------------------

/// Unique identifier for a discrete unit of agent work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub uuid::Uuid);

impl TaskId {
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a single execution attempt of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutionId(pub uuid::Uuid);

impl ExecutionId {
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for ExecutionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ExecutionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Retry policy
// ---------------------------------------------------------------------------

/// Retry policy with exponential backoff for failed operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Initial delay between retries in milliseconds.
    #[serde(default = "default_initial_delay")]
    pub initial_delay_ms: u64,
    /// Backoff multiplier (delay *= multiplier after each retry).
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,
    /// Maximum delay between retries in milliseconds.
    #[serde(default = "default_max_delay")]
    pub max_delay_ms: u64,
    /// Whether to add random jitter to the delay.
    #[serde(default)]
    pub jitter: bool,
}

fn default_initial_delay() -> u64 {
    1000
}
fn default_backoff_multiplier() -> f64 {
    2.0
}
fn default_max_delay() -> u64 {
    30_000
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: default_initial_delay(),
            backoff_multiplier: default_backoff_multiplier(),
            max_delay_ms: default_max_delay(),
            jitter: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Circuit breaker
// ---------------------------------------------------------------------------

/// Circuit breaker state for downstream service protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum CircuitBreakerState {
    /// Normal operation — requests pass through.
    #[default]
    Closed,
    /// Failures exceeded threshold — requests are rejected.
    Open,
    /// Testing recovery — limited requests pass through.
    HalfOpen,
}

// ---------------------------------------------------------------------------
// Agent pool / group
// ---------------------------------------------------------------------------

/// Configuration for a pool of identical agents (replica set).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPool {
    /// Pool name.
    pub name: String,
    /// Agent config template for pool members.
    pub agent_config: AgentConfig,
    /// Desired number of running replicas.
    #[serde(default = "default_replicas")]
    pub replicas: u32,
    /// Minimum replicas (for autoscaling).
    #[serde(default = "default_replicas")]
    pub min_replicas: u32,
    /// Maximum replicas (for autoscaling).
    #[serde(default = "default_max_replicas")]
    pub max_replicas: u32,
}

fn default_replicas() -> u32 {
    1
}
fn default_max_replicas() -> u32 {
    10
}

// ---------------------------------------------------------------------------
// Agent capabilities advertisement
// ---------------------------------------------------------------------------

/// Capabilities that an agent advertises for matchmaking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Skills or capabilities this agent provides (e.g., "code-review", "translation").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    /// Input types this agent accepts (e.g., "text", "image", "code").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_types: Vec<String>,
    /// Output types this agent produces.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_types: Vec<String>,
    /// Maximum concurrent tasks this agent can handle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_concurrent_tasks: Option<u32>,
}

// ---------------------------------------------------------------------------
// Scheduling / Placement
// ---------------------------------------------------------------------------

/// Scheduling constraints for agent placement.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulingConstraints {
    /// Node labels the agent prefers to run on (soft preference).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub node_affinity: Vec<LabelSelector>,
    /// Node labels the agent must NOT run on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub node_anti_affinity: Vec<LabelSelector>,
    /// Agent names that should be co-located.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pod_affinity: Vec<String>,
    /// Agent names that should NOT be co-located.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pod_anti_affinity: Vec<String>,
    /// Priority class (higher = more important, can preempt lower-priority agents).
    #[serde(default)]
    pub priority: i32,
    /// Whether this agent can preempt lower-priority agents for resources.
    #[serde(default)]
    pub preemptible: bool,
}

/// A label selector for node matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelector {
    /// Label key (e.g., "gpu", "zone", "arch").
    pub key: String,
    /// Match operator.
    pub operator: LabelOperator,
    /// Values to match against.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
}

/// Label match operator for scheduling selectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LabelOperator {
    /// Label value must equal one of the specified values.
    In,
    /// Label value must not equal any of the specified values.
    NotIn,
    /// Label must exist (values ignored).
    Exists,
    /// Label must not exist (values ignored).
    DoesNotExist,
}

// ---------------------------------------------------------------------------
// Inter-agent pub/sub
// ---------------------------------------------------------------------------

/// A topic/channel for pub/sub agent communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    /// Topic name (e.g., "task.completed", "alert.security").
    pub name: String,
    /// Optional topic description.
    #[serde(default)]
    pub description: String,
    /// Maximum message TTL in seconds (None = no expiry).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_ttl_secs: Option<u64>,
}

/// A subscription to a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Subscriber agent ID.
    pub agent_id: AgentId,
    /// Topic name to subscribe to.
    pub topic: String,
    /// Optional filter expression (e.g., "severity >= 'error'").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

/// A message published to a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicMessage {
    /// Unique message ID.
    pub id: uuid::Uuid,
    /// Topic this message was published to.
    pub topic: String,
    /// Publisher agent ID.
    pub publisher: AgentId,
    /// Message payload.
    pub payload: serde_json::Value,
    /// Message timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Message priority (higher = more urgent).
    #[serde(default)]
    pub priority: u8,
    /// Message TTL in seconds (overrides topic default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_secs: Option<u64>,
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
            AgentStatus::Restarting,
            AgentStatus::Terminated,
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
            probe_type: ProbeType::Liveness,
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
    fn lifecycle_hooks_serde_roundtrip() {
        let hooks = LifecycleHooks {
            pre_start: Some(LifecycleHook {
                command: vec!["/usr/bin/init".into(), "--setup".into()],
                timeout_secs: Some(10),
            }),
            post_start: None,
            pre_stop: Some(LifecycleHook {
                command: vec!["/usr/bin/cleanup".into()],
                timeout_secs: Some(5),
            }),
            post_stop: None,
        };
        let json = serde_json::to_string(&hooks).unwrap();
        let back: LifecycleHooks = serde_json::from_str(&json).unwrap();
        assert!(back.pre_start.is_some());
        assert!(back.post_start.is_none());
        assert_eq!(back.pre_start.unwrap().command[0], "/usr/bin/init");
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

    #[test]
    fn task_id_unique() {
        assert_ne!(TaskId::new(), TaskId::new());
    }

    #[test]
    fn task_id_serde_roundtrip() {
        let id = TaskId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: TaskId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn execution_id_serde_roundtrip() {
        let id = ExecutionId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: ExecutionId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn retry_policy_default() {
        let rp = RetryPolicy::default();
        assert_eq!(rp.max_retries, 3);
        assert_eq!(rp.initial_delay_ms, 1000);
        assert!((rp.backoff_multiplier - 2.0).abs() < f64::EPSILON);
        assert!(rp.jitter);
    }

    #[test]
    fn retry_policy_serde_roundtrip() {
        let rp = RetryPolicy {
            max_retries: 5,
            initial_delay_ms: 500,
            backoff_multiplier: 1.5,
            max_delay_ms: 60_000,
            jitter: false,
        };
        let json = serde_json::to_string(&rp).unwrap();
        let back: RetryPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_retries, 5);
        assert!(!back.jitter);
    }

    #[test]
    fn circuit_breaker_state_serde_roundtrip() {
        for variant in [
            CircuitBreakerState::Closed,
            CircuitBreakerState::Open,
            CircuitBreakerState::HalfOpen,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: CircuitBreakerState = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn probe_type_serde_roundtrip() {
        for variant in [
            ProbeType::LivenessReadiness,
            ProbeType::Liveness,
            ProbeType::Readiness,
            ProbeType::Startup,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ProbeType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn agent_capabilities_serde_roundtrip() {
        let ac = AgentCapabilities {
            skills: vec!["code-review".into(), "translation".into()],
            input_types: vec!["text".into()],
            output_types: vec!["text".into()],
            max_concurrent_tasks: Some(5),
        };
        let json = serde_json::to_string(&ac).unwrap();
        let back: AgentCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(back.skills.len(), 2);
        assert_eq!(back.max_concurrent_tasks, Some(5));
    }

    #[test]
    fn agent_pool_serde_roundtrip() {
        let pool = AgentPool {
            name: "workers".into(),
            agent_config: AgentConfig::default(),
            replicas: 3,
            min_replicas: 1,
            max_replicas: 10,
        };
        let json = serde_json::to_string(&pool).unwrap();
        let back: AgentPool = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "workers");
        assert_eq!(back.replicas, 3);
    }

    #[test]
    fn scheduling_constraints_serde_roundtrip() {
        let sc = SchedulingConstraints {
            node_affinity: vec![LabelSelector {
                key: "gpu".into(),
                operator: LabelOperator::Exists,
                values: vec![],
            }],
            node_anti_affinity: vec![],
            pod_affinity: vec!["helper-agent".into()],
            pod_anti_affinity: vec![],
            priority: 100,
            preemptible: false,
        };
        let json = serde_json::to_string(&sc).unwrap();
        let back: SchedulingConstraints = serde_json::from_str(&json).unwrap();
        assert_eq!(back.node_affinity.len(), 1);
        assert_eq!(back.priority, 100);
    }

    #[test]
    fn label_operator_serde_roundtrip() {
        for variant in [
            LabelOperator::In,
            LabelOperator::NotIn,
            LabelOperator::Exists,
            LabelOperator::DoesNotExist,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: LabelOperator = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn topic_serde_roundtrip() {
        let t = Topic {
            name: "task.completed".into(),
            description: "Emitted when a task finishes".into(),
            message_ttl_secs: Some(3600),
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: Topic = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "task.completed");
        assert_eq!(back.message_ttl_secs, Some(3600));
    }

    #[test]
    fn topic_message_serde_roundtrip() {
        let tm = TopicMessage {
            id: uuid::Uuid::new_v4(),
            topic: "alerts".into(),
            publisher: AgentId::new(),
            payload: serde_json::json!({"level": "critical"}),
            timestamp: chrono::Utc::now(),
            priority: 10,
            ttl_secs: Some(60),
        };
        let json = serde_json::to_string(&tm).unwrap();
        let back: TopicMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.topic, "alerts");
        assert_eq!(back.priority, 10);
    }

    #[test]
    fn subscription_serde_roundtrip() {
        let s = Subscription {
            agent_id: AgentId::new(),
            topic: "alerts".into(),
            filter: Some("severity >= 'error'".into()),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Subscription = serde_json::from_str(&json).unwrap();
        assert_eq!(back.topic, "alerts");
        assert!(back.filter.is_some());
    }
}
