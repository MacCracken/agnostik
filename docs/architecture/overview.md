# Agnostik Architecture

## Module Map

```
agnostik
├── error.rs      — AgnostikError (11 variants), From<io::Error>, From<serde_json::Error>
├── types.rs      — AgentId, UserId, Version (FromStr), Capabilities, MessageType, SystemStatus
├── agent.rs      — AgentConfig, AgentManifest, AgentStatus, AgentType, ResourceLimits,
│                   ResourceUsage, AgentRateLimit, AgentEvent, AgentInfo, AgentStats, StopReason
├── security.rs   — SandboxConfig, Permission, NetworkAccess, NetworkPolicy,
│                   CgroupLimits, NamespaceConfig, IdMapping,
│                   LandlockRuleset, LandlockFsAccess, LandlockNetAccess,
│                   LinuxCapability, CapabilitySet, SystemFeature,
│                   SecurityContext, SecurityPolicy, PolicyEffect
├── telemetry.rs  — TraceContext (W3C), TraceId, SpanId, Span, SpanStatus,
│                   TelemetryConfig, CrashReport, EventType,
│                   MetricKind, MetricValue, MetricDataPoint, InstrumentDescriptor,
│                   SpanCollector trait, MetricSink trait
├── audit.rs      — AuditEntry, AuditSeverity
├── llm.rs        — LlmProvider, MessageRole, Message, ContentBlock,
│                   ToolDefinition, ToolCall, ToolResult, SamplingParams,
│                   InferenceRequest, InferenceResponse, TokenUsage, FinishReason,
│                   StreamEvent
├── secrets.rs    — Secret (zeroize, redacted Debug), SecretMetadata
├── config.rs     — EnvironmentProfile, AgnosConfig
└── logging.rs    — try_init() tracing subscriber setup
```

## Feature Dependencies

```
default = [agent, security, telemetry]

agent ──→ security
telemetry ──→ chrono
audit ──→ sha2, hex, chrono
secrets ──→ zeroize, chrono
logging ──→ tracing-subscriber
full = [all features]
```

## Core Types (always available)

`AgentId`, `UserId`, `Version`, `Capabilities`, `MessageType`, `SystemStatus`,
`ComponentConfig`, `AgnostikError`, `Result` — available without any feature flags.

## Traits

| Trait | Module | Purpose |
|-------|--------|---------|
| `SpanCollector` | telemetry | Export completed spans to a backend |
| `MetricSink` | telemetry | Export metric data points to a backend |

## Consumers

Every AGNOS component depends on agnostik for shared types:
- **daimon** — AgentId, AgentConfig, AgentStatus, TraceContext
- **hoosh** — InferenceRequest, TokenUsage, LlmProvider, Message, ToolCall
- **aegis** — SecurityPolicy, LinuxCapability, CapabilitySet, AuditEntry
- **argonaut** — EnvironmentProfile, AgnosConfig
- **kavach** — SandboxConfig, Permission, NetworkAccess, CgroupLimits, LandlockRuleset, NamespaceConfig
- **All consumer apps** — AgentManifest, AgentEvent
