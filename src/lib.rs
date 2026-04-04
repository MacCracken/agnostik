//! # Agnostik
//!
//! **Agnostik** (agnostic) — shared types, error handling, and domain primitives
//! for the AGNOS ecosystem.
//!
//! Extracted from `agnos-common` as a standalone crate. Provides the core type
//! vocabulary that all AGNOS components share: agent identity, sandbox config,
//! security policies, telemetry, LLM types, and error handling.
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `agent` | yes | Agent identity, configuration, lifecycle (implies `security`) |
//! | `security` | yes | Sandbox, RBAC, cgroup, namespace, landlock, capability types |
//! | `telemetry` | yes | W3C tracing, metrics, SpanCollector/MetricSink traits |
//! | `audit` | no | Tamper-evident audit chain with HMAC integrity |
//! | `llm` | no | Conversation, tool calling, streaming, inference types |
//! | `secrets` | no | Zeroize-backed secret storage |
//! | `config` | no | Environment profile and component config |
//! | `classification` | no | Data classification and PII detection types |
//! | `validation` | no | Input validation and sanitization types |
//! | `hardware` | no | Hardware accelerator detection types |

pub mod error;

#[cfg(feature = "agent")]
pub mod agent;

#[cfg(feature = "security")]
pub mod security;

#[cfg(feature = "telemetry")]
pub mod telemetry;

#[cfg(feature = "audit")]
pub mod audit;

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "secrets")]
pub mod secrets;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "classification")]
pub mod classification;

#[cfg(feature = "validation")]
pub mod validation;

#[cfg(feature = "hardware")]
pub mod hardware;

pub mod types;

#[cfg(feature = "logging")]
pub mod logging;

// Core re-exports (always available)
pub use error::{AgnostikError, Result};
pub use types::*;

// Feature-gated re-exports
#[cfg(feature = "agent")]
pub use agent::{
    AgentCapabilities, AgentConfig, AgentDependency, AgentEvent, AgentInfo, AgentManifest,
    AgentMessage, AgentPool, AgentRateLimit, AgentStats, AgentStatus, AgentType,
    CircuitBreakerState, ExecutionId, HealthCheck, LabelOperator, LabelSelector, LifecycleHook,
    LifecycleHooks, ProbeType, ResourceGrant, ResourceLimits, ResourceRequest, ResourceUsage,
    RestartBackoff, RestartPolicy, RetryPolicy, SchedulingConstraints, StopReason, Subscription,
    TaskId, Topic, TopicMessage,
};

#[cfg(feature = "security")]
pub use security::{
    AuthContext, CapabilitySet, CgroupLimits, ConditionOperator, DeviceRule, DeviceType,
    EncryptedStorageConfig, FilesystemRule, FsAccess, IdMapping, LandlockFsAccess, LandlockFsRule,
    LandlockNetAccess, LandlockNetRule, LandlockRuleset, LandlockScope, LinuxCapability,
    MountPropagation, NamespaceConfig, NamespaceEntry, NetworkAccess, NetworkPolicy, Permission,
    PermissionCondition, PolicyEffect, Rlimit, RlimitType, Role, RolePermission,
    SandboxCapabilities, SandboxConfig, SeccompAction, SeccompArch, SeccompArg, SeccompArgOp,
    SeccompMode, SeccompProfile, SeccompRule, SecurityContext, SecurityPolicy, SystemFeature,
    TokenPayload,
};

#[cfg(feature = "telemetry")]
pub use telemetry::{
    AggregationTemporality, Baggage, BaggageEntry, EventType, Exemplar, InstrumentDescriptor,
    InstrumentationScope, LogRecord, LogSeverity, MetricDataPoint, MetricKind, MetricSink,
    MetricValue, Resource, Span, SpanCollector, SpanEvent, SpanId, SpanKind, SpanLink, SpanStatus,
    TRACE_FLAG_SAMPLED, TelemetryConfig, TextMapCarrier, TextMapPropagator, TraceContext, TraceId,
};

#[cfg(feature = "audit")]
pub use audit::{
    AuditEntry, AuditResult, AuditSeverity, AuditSink, GENESIS_HASH, IntegrityFields,
    RetentionPolicy,
};

#[cfg(feature = "llm")]
pub use llm::{
    BatchRequest, BatchResult, BatchStatus, CacheControl, ContentBlock, EmbeddingRequest,
    EmbeddingResponse, FinishReason, InferenceRequest, InferenceResponse, LlmProvider,
    LogprobEntry, Message, MessageRole, ModelCapabilities, RateLimitInfo, ResponseFormat,
    SafetyCategory, SafetyProbability, SafetyRating, SamplingParams, StreamEvent, TokenLogprob,
    TokenUsage, ToolCall, ToolChoice, ToolDefinition, ToolResult,
};

#[cfg(feature = "secrets")]
pub use secrets::{Secret, SecretKind, SecretMetadata, SecretStore};

#[cfg(feature = "config")]
pub use config::{
    AgnosConfig, EdgeResourceOverrides, EnvironmentProfile, FleetConfig, ProfileDefinition,
};

#[cfg(feature = "classification")]
pub use classification::{ClassificationLevel, ClassificationResult, PiiKind};

#[cfg(feature = "validation")]
pub use validation::{InjectionScores, ValidationResult, ValidationSeverity, ValidationWarning};

#[cfg(feature = "hardware")]
pub use hardware::{
    AcceleratorDevice, AcceleratorFlags, AcceleratorSummary, DeviceFamily, DeviceHealth,
    DeviceVendor, MemoryType,
};
