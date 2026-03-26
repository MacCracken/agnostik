# Changelog

## [Unreleased]

### Added

#### LLM Module Expansion
- **MessageRole**, **Message**, **ContentBlock** — structured multi-turn conversation types replacing bare `prompt: String`
- **ToolDefinition**, **ToolCall**, **ToolResult** — function/tool calling types
- **SamplingParams** — top_p, top_k, frequency_penalty, presence_penalty, stop_sequences, seed
- **StreamEvent** — Delta, ToolCallDelta, Usage, Done, Error variants for streaming responses
- **FinishReason::ToolUse** variant for tool-calling flows
- `InferenceRequest` now supports `messages`, `tools`, and `sampling` fields
- `InferenceResponse` now uses `Vec<ContentBlock>` and `tool_calls`

#### Telemetry v2
- **MetricKind** (Counter, UpDownCounter, Gauge, Histogram), **MetricValue**, **MetricDataPoint**, **InstrumentDescriptor** — OTel-aligned metric types
- **SpanCollector** trait — pluggable span export backend (export, flush, shutdown)
- **MetricSink** trait — pluggable metric export backend
- **SpanId::Display** — hex-formatted display impl
- `TraceContext.trace_flags` (u8) and `trace_state` (String) for W3C Trace Context compliance
- `TraceContext::is_sampled()` and `TRACE_FLAG_SAMPLED` constant
- Child spans now propagate trace_flags and trace_state

#### Security Module Expansion
- **CgroupLimits** — memory_max/high, cpu_max/period/weight, pids_max (cgroups v2)
- **NamespaceConfig** — pid, net, mount, user, uts, ipc, cgroup namespace flags
- **IdMapping** — UID/GID mapping for user namespaces
- **LandlockFsAccess** (15 variants), **LandlockFsRule**, **LandlockNetAccess**, **LandlockNetRule**, **LandlockRuleset** — fine-grained Landlock v3 types
- **LinuxCapability** (19 POSIX caps), **CapabilitySet** (effective, permitted, inheritable, bounding, ambient)
- **SystemFeature** — renamed from `Capability` to resolve naming collision with Linux capabilities (`Capability` preserved as type alias)

#### Core Improvements
- **AgentId**, **UserId** moved to always-available `types` module (no longer feature-gated)
- **FromStr** impls for AgentId (UUID), Version (SemVer), TraceId (hex), SpanId (hex)
- **From\<Uuid\>** for AgentId and UserId
- **From\<std::io::Error\>** for AgnostikError (new `Io` variant)
- **From\<serde_json::Error\>** for AgnostikError (into `Serialization` variant)
- **Hash** added to AgentType, AgentStatus, LlmProvider, FinishReason, SystemFeature
- **Eq** added to ResourceLimits, ResourceUsage, TokenUsage

### Fixed
- `Version::default()` was hardcoded to `2026.3.25` — now correctly defaults to `0.0.0`
- `AgentConfig` serde shape changed based on `security` feature flag — `agent` feature now depends on `security`, fields always present
- `AgentManifest.requested_permissions` was conditionally compiled — now always present
- `Secret` derived `Debug` which leaked values in logs — now uses custom redacted Debug impl
- `Secret::Drop` had redundant `#[cfg(feature = "secrets")]` — removed
- `logging::init()` panicked if subscriber already set — replaced with `try_init()` returning `Result`
- Unused imports in `agent.rs` (FilesystemRule, FsAccess, NetworkAccess)
- Derivable `Default` impls flagged by clippy (Version, EnvironmentProfile)
- Redundant closures in benchmarks
- SPDX license identifier `GPL-3.0` → `GPL-3.0-only`

### Changed
- **Breaking**: `AgentEvent.agent_id`, `AgentInfo.id`, `SecurityContext.agent_id`, `AuditEntry.agent_id`, `CrashReport.agent_id` changed from `String` to `AgentId`
- **Breaking**: `AuditEntry.timestamp`, `CrashReport.timestamp`, `SecretMetadata.created_at`, `SecretMetadata.expires_at` changed from `String` to `chrono::DateTime<Utc>`
- **Breaking**: `InferenceResponse.text` replaced with `content: Vec<ContentBlock>`
- **Breaking**: `InferenceRequest.temperature` moved into `SamplingParams`
- **Breaking**: `logging::init()` renamed to `logging::try_init()` with `Result` return
- `agent` feature now implies `security` feature
- `audit` and `secrets` features now depend on `chrono`
- `Secret` Serialize/Deserialize omission documented in doc comments

### Performance
- Serde benchmarks added: AgentId (33/59 ns ser/de), TraceContext (168/325 ns), SandboxConfig (262/331 ns)
- No regressions in existing benchmarks

### Testing
- 147 tests total (143 unit + 4 integration), up from 49
- 99.53% line coverage (8 files at 100%, all files above 94%)
- Serde roundtrip tests for all public types

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
