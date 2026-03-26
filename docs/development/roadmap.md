# Agnostik Roadmap

## Status
**v0.1.0+** — 12 modules, 180 tests, 99%+ coverage. All quality gates passing.

## Completed

### P(-1) Scaffold Hardening
- Type-safe AgentId everywhere (replaced stringly-typed IDs)
- Typed timestamps via `chrono::DateTime<Utc>` (replaced String timestamps)
- W3C TraceContext compliance (trace_flags, trace_state)
- FromStr impls for AgentId, Version, TraceId, SpanId
- Error source chaining (From<io::Error>, From<serde_json::Error>)
- Consistent derives (Hash, Eq, PartialEq) across all types
- Secret Debug redaction, logging panic fix, feature contamination fix
- 99.53% test coverage

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

## Backlog

### Environment Profiles
- Edge-specific resource limits
- Fleet-wide configuration distribution
- Profile inheritance (edge inherits from production, overrides specific fields)

### Agent Manifest v2
- Resource negotiation (agent requests, runtime approves/modifies)
- Dependency declaration (agent A requires agent B)

### LLM Expansion
- Embedding types (EmbeddingRequest/Response for RAG pipelines)

### Agent Communication
- Agent-to-agent message envelope (sender/receiver AgentId, correlation_id)

### Trait Interfaces
- SecretStore trait (pluggable secret backends)

## v1.0.0 Criteria
- API frozen — no breaking changes
- All types Serialize + Deserialize with roundtrip tests ✅
- All enums #[non_exhaustive] ✅
- Zero unwrap/panic ✅
- 90%+ coverage ✅ (99.53%)
- 3+ downstream consumers in production
