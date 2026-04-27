# Agnostik Architecture

## Module Map

```
agnostik (Cyrius)
├── src/error.cyr      — AgnostikError (11 kinds), error codes (1001–1010), Result helpers
├── src/types.cyr      — AgentId, UserId (UUID v4), Version (SemVer), Capabilities,
│                        MessageType, SystemStatus, ComponentConfig
├── src/agent.cyr      — AgentConfig, AgentManifest, AgentStatus, AgentType, ResourceLimits,
│                        ResourceUsage, AgentRateLimit, AgentEvent, AgentInfo, AgentStats,
│                        RestartPolicy, HealthCheck, LifecycleHooks, StopReason,
│                        AgentPool, AgentCapabilities, SchedulingConstraints,
│                        Topic, Subscription, TopicMessage, AgentMessage
├── src/security.cyr   — SandboxConfig, Permission, NetworkAccess, NetworkPolicy,
│                        CgroupLimits, NamespaceConfig, IdMapping,
│                        LandlockRuleset, LandlockFsAccess, LandlockNetAccess,
│                        LinuxCapability (39 variants), CapabilitySet, SystemFeature,
│                        SeccompProfile, SeccompArch, SeccompArg, SeccompArgOp,
│                        MountPropagation, SandboxCapabilities, SeccompMode,
│                        Role, RolePermission, TokenPayload, AuthContext,
│                        SecurityContext, SecurityPolicy, PolicyEffect,
│                        EncryptedStorage, Rlimit, DeviceRule, NamespaceEntry
├── src/telemetry.cyr  — TraceContext (W3C), TraceId, SpanId, Span, SpanStatus,
│                        SpanKind, SpanEvent, SpanLink, Resource, InstrumentationScope,
│                        TelemetryConfig, CrashReport, EventType,
│                        MetricKind, MetricValue, MetricDataPoint, InstrumentDescriptor,
│                        AggregationTemporality, Exemplar, Baggage,
│                        LogSeverity, LogRecord,
│                        SpanCollector trait, MetricSink trait,
│                        TextMapCarrier trait, TextMapPropagator trait
├── src/audit.cyr      — AuditEntry, AuditSeverity, AuditResult, IntegrityFields,
│                        RetentionPolicy, AuditSink trait
├── src/llm.cyr        — LlmProvider (13), MessageRole, Message, ContentBlock (8 types),
│                        ToolDefinition, ToolCall, ToolResult, ToolChoice, SamplingParams,
│                        InferenceRequest, InferenceResponse, TokenUsage, FinishReason,
│                        ResponseFormat, ModelCapabilities, RateLimitInfo, StreamEvent,
│                        SafetyCategory, SafetyRating, BatchRequest, BatchResult,
│                        EmbeddingRequest, EmbeddingResponse, LogprobEntry
├── src/secrets.cyr    — Secret (zeroize on destroy), SecretMetadata, SecretKind,
│                        SecretStore trait
├── src/config.cyr     — EnvironmentProfile, AgnosConfig, EdgeResourceOverrides,
│                        ProfileDefinition, FleetConfig
├── src/classification.cyr — ClassificationLevel, PiiKind (16), ClassificationResult
├── src/validation.cyr — ValidationResult, ValidationWarning, ValidationSeverity,
│                        InjectionScores
└── src/hardware.cyr   — AcceleratorDevice, DeviceFamily, DeviceVendor (9),
                         DeviceHealth, MemoryType (12), AcceleratorFlags,
                         AcceleratorSummary (by_family, by_vendor)
```

## Library Dependencies

```
lib/syscalls.cyr   — Linux syscall wrappers (trimmed for types library)
lib/string.cyr     — C string operations (strlen, memcpy, memeq, memset)
lib/alloc.cyr      — Bump allocator (brk-based) + arena allocator
lib/fmt.cyr        — Integer/float formatting
lib/str.cyr        — Fat string type (data + length), str_builder (direct buffer)
lib/vec.cyr        — Dynamic array
lib/hashmap.cyr    — Hash map (FNV-1a, open addressing)
lib/tagged.cyr     — Tagged unions: Option, Result, Either
lib/fnptr.cyr      — Function pointer dispatch (inline asm)
lib/trait.cyr      — Trait objects (vtable + data fat pointers)
lib/io.cyr         — File I/O (open, read, write, close, getenv)
lib/json.cyr       — Minimal JSON parser and builder
lib/assert.cyr     — Test assertions (eq, neq, gt, lt, gte, lte, nonnull, streq)
lib/bench.cyr      — Benchmark harness (ns timing via clock_gettime)
lib/sakshi.cyr     — Tracing/error handling (stderr profile, packed i64 errors)
```

## Traits (vtable dispatch)

| Trait | Module | Purpose |
|-------|--------|---------|
| `SpanCollector` | telemetry | Export completed spans (export, flush, shutdown) |
| `MetricSink` | telemetry | Export metric data points (export, flush) |
| `TextMapPropagator` | telemetry | Inject/extract trace context from carriers |
| `TextMapCarrier` | telemetry | Get/set key-value pairs for propagation |
| `AuditSink` | audit | Append, verify chain/entry, query, seal |
| `SecretStore` | secrets | Get, put, delete, list, rotate, search |

## Serialization

9 struct types have `_to_json(ptr, sb)` functions for JSON serialization:
TokenUsage, AgentInfo, AgentStats, ResourceLimits, ResourceUsage,
InjectionScores, AcceleratorFlags, EdgeResourceOverrides, TelemetryConfig.

## Consumers

Every AGNOS component depends on agnostik for shared types. Canonical list maintained in [`../development/state.md`](../development/state.md); typical type usage:

- **daimon** — agent runtime: AgentId, AgentConfig, AgentStatus, TraceContext
- **hoosh** — LLM grounding service: InferenceRequest, TokenUsage, LlmProvider, Message, ToolCall
- **agnoshi** — shell: AgentEvent, AuthContext
- **aegis** — security policy engine: SecurityPolicy, LinuxCapability, CapabilitySet, AuditEntry, Role, TokenPayload
- **argonaut** — agent orchestrator: EnvironmentProfile, AgnosConfig, FleetConfig
- **sigil** — capability/auth issuer: TokenPayload, Role, AuthContext
- **ark** — packaging / distributable: AgentManifest, ComponentConfig
- **kavach** — sandbox enforcement: SandboxConfig, Permission, CgroupLimits, LandlockRuleset, NamespaceConfig, SeccompProfile
- **stiva** — telemetry pipeline: Span, MetricDataPoint, LogRecord
- **nein** — refusal / safety layer: ValidationResult, InjectionScores, ClassificationLevel, PiiKind
- **yukti** — device abstraction: AcceleratorDevice, AcceleratorFlags, AcceleratorSummary
- **All consumer apps** — AgentManifest, AgentEvent, ValidationResult
