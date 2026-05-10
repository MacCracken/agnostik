# Agnostik Architecture

## Module Map

```
agnostik (Cyrius)
├── src/error.cyr      — AgnostikError (11 kinds), error codes (1001–1010), Result helpers
├── src/types.cyr      — AgentId, UserId (UUID v4), Version (SemVer), Capabilities,
│                        MessageType, SystemStatus, ComponentConfig.
│                        Hosts JSON parser primitives (_json_int / _json_str /
│                        _json_find_value) including \uXXXX Unicode + surrogate-pair
│                        UTF-8 decoding (v1.0.7).
├── src/proto.cyr      — Protobuf wire-format helpers (varint, tag, length-delim,
│                        fixed64, nested message). Foundation for OTLP encoders (v1.2.0).
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
│                        TextMapCarrier trait, TextMapPropagator trait.
│                        OTLP wire encoder: Span_to_otlp_proto (v1.2.0).
├── src/audit.cyr      — AuditEntry, AuditSeverity, AuditResult, IntegrityFields,
│                        RetentionPolicy, AuditSink trait
├── src/llm.cyr        — LlmProvider (18 variants — incl. v1.1.0 additions: Together,
│                        Fireworks, Bedrock, Vertex, Cohere), MessageRole, Message,
│                        ContentBlock (8 types), ToolDefinition, ToolCall, ToolResult,
│                        ToolChoice, SamplingParams, InferenceRequest, InferenceResponse,
│                        TokenUsage, FinishReason, ResponseFormat,
│                        ModelCapabilities (15 flags incl. v1.1.0 additions: video_input,
│                        caching, parallel_tool_calls), RateLimitInfo, StreamEvent,
│                        SafetyCategory, SafetyRating, BatchRequest, BatchResult,
│                        EmbeddingRequest, EmbeddingResponse, LogprobEntry
├── src/secrets.cyr    — Secret (zeroize on destroy), SecretMetadata, SecretKind,
│                        SecretStore trait
├── src/config.cyr     — EnvironmentProfile, AgnosConfig, EdgeResourceOverrides,
│                        ProfileDefinition, FleetConfig
├── src/classification.cyr — ClassificationLevel, PiiKind (19 variants — incl. v1.0.7
│                            additions: Genetic, BiometricTemplate, PreciseGeolocation),
│                            ClassificationResult
├── src/validation.cyr — ValidationResult, ValidationWarning, ValidationSeverity,
│                        InjectionScores (5 i8 fields — v1.1.1 sub-byte widths)
└── src/hardware.cyr   — AcceleratorDevice, DeviceFamily, DeviceVendor (9),
                         DeviceHealth, MemoryType (12),
                         AcceleratorFlags (9 i8 fields — v1.1.1 sub-byte widths),
                         AcceleratorSummary (by_family, by_vendor)
```

## Library Dependencies

Auto-resolved into `lib/` by `cyrius deps` from the `[deps] stdlib` block in
`cyrius.cyml`:

```
lib/syscalls.cyr   — Linux syscall wrappers (arch-dispatched x86_64 / aarch64)
lib/string.cyr     — C string operations (strlen, memcpy, memeq, memset)
lib/alloc.cyr      — Bump allocator (brk-based) + arena allocator
lib/fmt.cyr        — Integer/float formatting
lib/str.cyr        — Fat string type (data + length), str_builder
lib/vec.cyr        — Dynamic array
lib/hashmap.cyr    — Hash map (FNV-1a, open addressing)
lib/tagged.cyr     — Tagged unions (transitively pulls lib/result.cyr)
lib/result.cyr     — Result<T, E> (added to deps in v1.0.2)
lib/fnptr.cyr      — Function-pointer dispatch (inline asm)
lib/trait.cyr      — Trait objects (vtable + data fat pointers)
lib/assert.cyr     — Test assertions (transitive via lib/test.cyr)
lib/io.cyr         — File I/O (open, read, write, close, getenv)
lib/json.cyr       — Minimal JSON parser and builder
lib/chrono.cyr     — clock_now_ns() etc. (added to deps in v1.0.2)
```

Test files additionally include `lib/test.cyr` (table-driven `test_each` —
adopted in v1.0.5 for the F-005 / F-010 audit-regression clusters and in
v1.0.7 / v1.1.0 / v1.1.1 / v1.1.2 / v1.2.0 feature-coverage files).

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

**JSON** — 9 struct types ship JSON serializers. Per
[ADR-002](../adr/002-derive-serialize-7-of-9.md) the v1.1.0 cut split this:

| Struct | Camp | Reason |
|---|---|---|
| `ResourceLimits` | `#derive(Serialize)` | All-int fields |
| `ResourceUsage` | `#derive(Serialize)` | All-int fields |
| `AgentStats` | `#derive(Serialize)` | All-int fields |
| `EdgeResourceOverrides` | `#derive(Serialize)` | All-int fields |
| `InjectionScores` | `#derive(Serialize)` | All-i8 fields (v1.1.1 sub-byte) |
| `TokenUsage` | `#derive(Serialize)` | All-int fields |
| `AcceleratorFlags` | `#derive(Serialize)` | All-i8 fields (v1.1.1 sub-byte) |
| `AgentInfo` | hand-written | UUID stringification + enum-name lookup + null-Str |
| `TelemetryConfig` | hand-written | null-Str fallback for `export_endpoint` |

Both camps emit the same compact byte format
(`{"k":v,"k":v}` no spaces). Derive structs are accessed as
`<Struct>_from_json_str(str_data(j))`; hand-written structs as
`<Struct>_from_json(s: Str)`.

**Protobuf (OTLP)** — `Span_to_otlp_proto(ptr, sb)` ships in v1.2.0.
`LogRecord_to_otlp_proto` and `MetricDataPoint_to_otlp_proto` are pinned for
v1.2.2 along with Span repeated-field encoders (attributes / events / links).

## Consumers

Every AGNOS component depends on agnostik for shared types. Canonical list
maintained in [`../development/state.md`](../development/state.md); typical
type usage:

- **daimon** — agent runtime: AgentId, AgentConfig, AgentStatus, TraceContext
- **hoosh** — LLM grounding service: InferenceRequest, TokenUsage, LlmProvider, Message, ToolCall, ModelCapabilities
- **agnoshi** — shell: AgentEvent, AuthContext
- **aegis** — security policy engine: SecurityPolicy, LinuxCapability, CapabilitySet, AuditEntry, Role, TokenPayload
- **argonaut** — agent orchestrator: EnvironmentProfile, AgnosConfig, FleetConfig
- **sigil** — capability/auth issuer: TokenPayload, Role, AuthContext
- **ark** — packaging / distributable: AgentManifest, ComponentConfig
- **kavach** — sandbox enforcement: SandboxConfig, Permission, CgroupLimits, LandlockRuleset, NamespaceConfig, SeccompProfile
- **stiva** — telemetry pipeline: Span (+ `Span_to_otlp_proto` for OTLP wire export), MetricDataPoint, LogRecord
- **nein** — refusal / safety layer: ValidationResult, InjectionScores, ClassificationLevel, PiiKind
- **yukti** — device abstraction: AcceleratorDevice, AcceleratorFlags, AcceleratorSummary
- **All consumer apps** — AgentManifest, AgentEvent, ValidationResult
