# Changelog

## [0.1.0] - 2026-03-25

Initial extraction from agnos-common as standalone shared types crate.

### Modules
- **error** — AgnostikError with 9 variants, retriable classification
- **types** — Version, Capabilities, MessageType, SystemStatus, ComponentConfig
- **agent** — AgentId, UserId, AgentConfig, AgentManifest, AgentStatus, ResourceLimits, ResourceUsage, AgentRateLimit, StopReason
- **security** — SandboxConfig, Permission, NetworkAccess, NetworkPolicy, FsAccess, SeccompAction, SecurityContext, SecurityPolicy, Capability
- **telemetry** — TraceContext (W3C), TraceId, SpanId, Span, SpanStatus, TelemetryConfig, CrashReport, EventType
- **audit** — AuditEntry, AuditSeverity
- **llm** — LlmProvider, InferenceRequest, InferenceResponse, TokenUsage, FinishReason
- **secrets** — Secret (zeroize-backed), SecretMetadata
- **config** — EnvironmentProfile, AgnosConfig
