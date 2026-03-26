# Agnostik Architecture

## Module Map

```
agnostik
├── error.rs           — AgnostikError (11 variants), From<io::Error>, From<serde_json::Error>
├── types.rs           — AgentId, UserId, Version (FromStr), Capabilities, MessageType, SystemStatus
├── agent.rs           — AgentConfig, AgentManifest, AgentStatus, AgentType, ResourceLimits,
│                        ResourceUsage, AgentRateLimit, AgentEvent, AgentInfo, AgentStats, StopReason
├── security.rs        — SandboxConfig, Permission, NetworkAccess, NetworkPolicy,
│                        CgroupLimits, NamespaceConfig, IdMapping,
│                        LandlockRuleset, LandlockFsAccess, LandlockNetAccess,
│                        LinuxCapability, CapabilitySet, SystemFeature,
│                        SandboxCapabilities, SeccompMode,
│                        Role, RolePermission, TokenPayload, AuthContext,
│                        SecurityContext, SecurityPolicy, PolicyEffect
├── telemetry.rs       — TraceContext (W3C), TraceId, SpanId, Span, SpanStatus,
│                        TelemetryConfig, CrashReport, EventType,
│                        MetricKind, MetricValue, MetricDataPoint, InstrumentDescriptor,
│                        SpanCollector trait, MetricSink trait
├── audit.rs           — AuditEntry, AuditSeverity, IntegrityFields, AuditSink trait
├── llm.rs             — LlmProvider, MessageRole, Message, ContentBlock,
│                        ToolDefinition, ToolCall, ToolResult, SamplingParams,
│                        InferenceRequest, InferenceResponse, TokenUsage, FinishReason,
│                        StreamEvent
├── secrets.rs         — Secret (zeroize, redacted Debug), SecretMetadata
├── config.rs          — EnvironmentProfile, AgnosConfig
├── classification.rs  — ClassificationLevel, PiiKind, ClassificationResult
├── validation.rs      — ValidationResult, ValidationWarning, ValidationSeverity
├── hardware.rs        — AcceleratorDevice, DeviceFamily, DeviceVendor, AcceleratorSummary
└── logging.rs         — try_init() tracing subscriber setup
```

## Feature Dependencies

```
default = [agent, security, telemetry]

agent ──→ security
telemetry ──→ chrono
audit ──→ sha2, hex, chrono
secrets ──→ zeroize, chrono
logging ──→ tracing-subscriber
classification, validation, hardware ──→ (no extra deps)
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
| `AuditSink` | audit | Append and verify audit chain entries |

## Consumers

Every AGNOS component depends on agnostik for shared types:
- **daimon** — AgentId, AgentConfig, AgentStatus, TraceContext
- **hoosh** — InferenceRequest, TokenUsage, LlmProvider, Message, ToolCall
- **aegis** — SecurityPolicy, LinuxCapability, CapabilitySet, AuditEntry, Role, TokenPayload
- **argonaut** — EnvironmentProfile, AgnosConfig
- **kavach** — SandboxConfig, Permission, CgroupLimits, LandlockRuleset, NamespaceConfig, SandboxCapabilities
- **secureyeoman** — ClassificationLevel, ValidationResult, AcceleratorDevice, Role, TokenPayload, AuditEntry
- **All consumer apps** — AgentManifest, AgentEvent
