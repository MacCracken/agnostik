# Agnostik Architecture

## Module Map

```
agnostik
├── error.rs      — AgnostikError (9 variants), retriable classification
├── types.rs      — Version, Capabilities, MessageType, SystemStatus, ComponentConfig
├── agent.rs      — AgentId, UserId, AgentConfig, AgentManifest, AgentStatus, ResourceLimits
├── security.rs   — SandboxConfig, Permission, NetworkAccess, NetworkPolicy, Capability
├── telemetry.rs  — TraceContext, TraceId, SpanId, Span, EventType, TelemetryConfig
├── audit.rs      — AuditEntry, AuditSeverity
├── llm.rs        — LlmProvider, InferenceRequest/Response, TokenUsage
├── secrets.rs    — Secret (zeroize), SecretMetadata
└── config.rs     — EnvironmentProfile, AgnosConfig
```

## Consumers

Every AGNOS component depends on agnostik for shared types:
- **daimon** — AgentId, AgentConfig, AgentStatus, TraceContext
- **hoosh** — InferenceRequest, TokenUsage, LlmProvider
- **aegis** — SecurityPolicy, Capability, AuditEntry
- **argonaut** — EnvironmentProfile, AgnosConfig
- **kavach** — SandboxConfig, Permission, NetworkAccess
- **All consumer apps** — AgentManifest, AgentEvent
