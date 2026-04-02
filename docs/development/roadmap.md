# Agnostik Roadmap

## Status
**v0.1.0+** — 12 modules, 216 tests. API frozen. All quality gates passing.

## Completed

### P(-1) Scaffold Hardening
- Type-safe AgentId everywhere (replaced stringly-typed IDs)
- Typed timestamps via `chrono::DateTime<Utc>` (replaced String timestamps)
- W3C TraceContext compliance (trace_flags, trace_state)
- FromStr impls for AgentId, Version, TraceId, SpanId
- Error source chaining (From<io::Error>, From<serde_json::Error>)
- Consistent derives (Hash, Eq, PartialEq) across all types
- Secret Debug redaction, logging panic fix, feature contamination fix
- Version serde as string (SemVer convention)
- UserId Display/FromStr for consistency with AgentId
- PartialEq/Eq on Capabilities
- Typed versions: AgentManifest.version, AgentDependency.min_version, IntegrityFields.version → Version
- Typed IDs: AuditEntry.user_id → Option<UserId>
- Required timestamps: AgentMessage.timestamp (no longer Optional)
- Span.started_at uses chrono::DateTime<Utc> (was start_ms: u64)
- Crate-root re-exports for all feature-gated module types

### LLM Module v2
- Structured conversation types (MessageRole, Message, ContentBlock)
- Tool/function calling (ToolDefinition, ToolCall, ToolResult)
- Streaming types (StreamEvent variants)
- Sampling parameters (top_p, top_k, penalties, stop sequences, seed)

### Telemetry v2
- SpanCollector trait for pluggable backends
- MetricSink trait for metric export
- Metric types (Counter, UpDownCounter, Gauge, Histogram)
- MetricValue, MetricDataPoint, InstrumentDescriptor

### Security Expansion
- Cgroup v2 types (CgroupLimits)
- Namespace configuration (NamespaceConfig, IdMapping)
- Landlock v3 types (LandlockRuleset, LandlockFsAccess, LandlockNetAccess)
- Linux capabilities (LinuxCapability, CapabilitySet)
- SystemFeature rename (Capability alias preserved)
- RBAC types (Role, RolePermission, TokenPayload, AuthContext)
- SandboxCapabilities detection (SeccompMode, landlock ABI, cgroup v2, namespaces)

### Cross-Pollination from SecureYeoman
- Classification module (ClassificationLevel, PiiKind, ClassificationResult)
- Validation module (ValidationResult, ValidationWarning, injection scoring)
- Hardware module (AcceleratorDevice, DeviceFamily, DeviceVendor, AcceleratorSummary)
- Audit integrity chain (IntegrityFields, HMAC-SHA256, AuditSink trait)

## Migration Plan

1. agnos-common currently owns these types in the monolith
2. agnostik extracts them as standalone crate
3. agnos-common becomes thin re-export over agnostik (same pattern as agnos-sys → agnosys)
4. Consumers gradually migrate `agnos_common::*` → `agnostik::*`
5. agnos-common deprecated once all consumers migrated

### Completed Backlog
- Environment Profiles: EdgeResourceOverrides, ProfileDefinition, FleetConfig (config module)
- Agent Manifest v2: ResourceRequest/ResourceGrant (negotiation), AgentDependency (dependency declaration)
- LLM Embeddings: EmbeddingRequest/EmbeddingResponse (llm module)
- Agent Communication: AgentMessage envelope with correlation_id, reply_to (agent module)
- SecretStore trait (secrets module)

## Backlog

_(No outstanding backlog items — all planned features implemented)_

## v1.0.0 Criteria
- API frozen — no breaking changes ✅
- Consistent error types — all FromStr impls return AgnostikError ✅
- Descriptive error messages with expected format hints ✅
- All types Serialize + Deserialize with roundtrip tests ✅
- All enums #[non_exhaustive] ✅
- Zero unwrap/panic ✅
- 90%+ coverage ✅ (99.53%)
- 3+ downstream consumers in production
