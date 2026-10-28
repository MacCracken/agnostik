# Agnostik Architecture

## Module Map

```
agnostik (Cyrius)
├── src/error.cyr      — AgnostikError (11 kinds), error codes (1001–1010), Result helpers
├── src/types.cyr      — AgentId, UserId (UUID v4), Version (SemVer), Capabilities,
│                        MessageType, SystemStatus, ComponentConfig.
│                        Hosts JSON parser primitives (_stik_json_int / _stik_json_str /
│                        _stik_json_find_value) including \uXXXX Unicode + surrogate-pair
│                        UTF-8 decoding (v1.0.7).
├── src/proto.cyr      — Protobuf wire-format writer (varint, tag, length-delim,
│                        fixed64, nested message): a private buffer that writes
│                        nested messages in place and patches their length
│                        (v1.10.0; ADR-004). Foundation for the OTLP encoders.
├── src/agent.cyr      — AgentConfig, AgentManifest, AgentStatus, AgentType, ResourceLimits,
│                        ResourceUsage, AgentRateLimit, AgentEvent, AgentInfo, AgentStats,
│                        RestartPolicy, HealthCheck, LifecycleHooks, StopReason,
│                        AgentPool, AgentCapabilities, SchedulingConstraints,
│                        Topic, Subscription, TopicMessage, AgentMessage
├── src/security.cyr   — SandboxConfig, Permission, NetworkAccess, NetworkPolicy,
│                        CgroupLimits, NamespaceConfig, IdMapping,
│                        LandlockRuleset, LandlockFsAccess, LandlockNetAccess,
│                        LinuxCapability (41 variants), CapabilitySet, SystemFeature,
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
│                        AnyValue (typed attribute values, v1.10.0).
│                        OTLP wire encoders: Span_to_otlp_proto (v1.2.0; attributes,
│                        events and links v1.10.0), LogRecord_to_otlp_proto,
│                        MetricDataPoint_to_otlp_proto, metric_to_otlp_proto,
│                        AnyValue_to_otlp_proto (v1.10.0); Resource /
│                        InstrumentationScope encoders, histograms, exemplars
│                        and the otlp_*_request envelopes (v1.12.0).
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

**JSON**: 9 struct types ship JSON serializers, all hand-written. From v1.1.0 to 1.12.2 seven of them used
`#derive(Serialize)` ([ADR-002](../adr/002-derive-serialize-7-of-9.md)). Since 1.12.3
([ADR-008](../adr/008-value-struct-serde-is-hand-written.md)) those seven are hand-written with the same names,
arity and bytes, because the generated decoders read past truncated input and wrapped out-of-range numbers.

| Struct | Shape | Notes |
|---|---|---|
| `ResourceLimits` | all-int | the three-fn value-struct pattern |
| `ResourceUsage` | all-int | the three-fn value-struct pattern |
| `AgentStats` | all-int | the three-fn value-struct pattern |
| `EdgeResourceOverrides` | all-int | the three-fn value-struct pattern |
| `InjectionScores` | all-i8 (v1.1.1 sub-byte) | scores saturate to 0..100 on every path |
| `TokenUsage` | all-int | the three-fn value-struct pattern |
| `AcceleratorFlags` | all-i8 (v1.1.1 sub-byte) | flags normalise to 0/1 on every path |
| `AgentInfo` | custom | UUID stringification (`null` for an id of 0) + enum names, parsed back by name (legacy `agent_type_id` / `status_id` ints accepted as a fallback) + null-Str |
| `TelemetryConfig` | custom | null-Str fallback for `export_endpoint` |

All emit the same compact byte format (`{"k":v,"k":v}` no spaces).
- **The seven value structs** each have three fns:
  - `<Struct>_to_json(ptr, sb)`;
  - `<Struct>_from_json_str(cstr)`: one bounded pass up to the NUL;
  - `<Struct>_from_json(pairs)`: from `bayan_json_parse`.

  The two decoders read numbers with F-008's overflow rule.
- **`AgentInfo` and `TelemetryConfig`** are read as `<Struct>_from_json(s: Str)`, through `_stik_json_find_value`.
  It matches top-level members only, and a repeated member gives its last value (since 1.12.3).

**Protobuf (OTLP)** — `Span_to_otlp_proto(ptr, sb)` shipped in v1.2.0; v1.10.0
adds its attributes, events and links, `LogRecord_to_otlp_proto`,
`MetricDataPoint_to_otlp_proto` (a `NumberDataPoint`), `metric_to_otlp_proto`
(a `Metric` with its gauge or sum, from an `InstrumentDescriptor`) and
`AnyValue_to_otlp_proto`. Attribute map values are `AnyValue` boxes
(`any_string`, `any_int`, …). Output is canonical (ascending fields, proto3
defaults omitted, keys sorted) and tested byte for byte against protoc's
encoding of the official schema. See
[ADR-004](../adr/004-otlp-attributes-are-anyvalue-boxes-encoded-canonically.md).
v1.12.0 adds `Resource_to_otlp_proto`, `InstrumentationScope_to_otlp_proto`,
histogram data points (`histogram_value_new` / `metric_histogram`), exemplars
and start times on data points, the span status message, and the
`otlp_traces_request` / `otlp_logs_request` / `otlp_metrics_request` builders
for the collector's Export*ServiceRequest bodies. Every `string` field is valid
UTF-8 ([ADR-006](../adr/006-otlp-strings-are-valid-utf8.md)).

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
