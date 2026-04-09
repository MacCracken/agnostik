# Changelog

## [0.96.0] - 2026-04-09

### Removed
- **rust-old/** — deleted 1.2 GB of Rust source, Cargo.lock, build artifacts, and criterion data. Rust→Cyrius port verified complete across all 12 modules. Rust benchmark reference numbers preserved in `benchmark-rustvcyrius.md`.

### Added
- **benchmark-rustvcyrius.md** — head-to-head Rust Criterion vs Cyrius bench comparison with analysis (Cyrius wins on agent_id_new 1.4x, trace_context_child 1.3x; Rust wins on sandbox_config_default 2.3x)

### Changed
- **CI workflows** — rewritten to use `cyrius build`/`cyrius test`/`cyrius bench` instead of raw `cat | cc2` pipes. Fixes `build/` directory pre-existence failures.
- **Release workflow** — uses `cyrius test` for gate instead of manual `.tcyr` loop
- **Source formatting** — all 6 files flagged by `cyrfmt` fixed (agent, config, hardware, llm, telemetry, validation)

## [0.95.0] - 2026-04-09

### Fixed
- **`version_to_str` buffer overflow** — increased buffer from 64 to 128 bytes with bounds checking on prerelease/build `memcpy` (was heap corruption on long prerelease strings)
- **`version_from_str` uninitialized fields** — major/minor/patch zeroed on alloc (was garbage on parse failure)
- **`secret_destroy` incomplete zeroize** — now zeros both the secret buffer AND the struct pointer/length fields
- **`_json_has` passed C string to `str_contains`** — fixed to pass Str (was always failing after str.cyr API update)
- **`accelerator_device_new` memory_type sentinel** — uses `MEM_UNKNOWN` enum instead of raw `-1`
- **`TelemetryConfig_to_json` null endpoint** — renders as `null` instead of `""` (ambiguous with empty string)
- **`AgentInfo_to_json` null name** — guarded with null check, renders as `null`

### Added
- **14 `_name()` functions** for consumer-facing enums (all use clean `elif` chains):
  - types: `message_type_name`, `system_status_name`
  - llm: `message_role_name`, `finish_reason_name`
  - telemetry: `span_status_name`, `span_kind_name`
  - hardware: `device_family_name`, `device_health_name`, `memory_type_name`
  - classification: `pii_kind_name` (16 PII variants)
  - validation: `validation_severity_name`
  - secrets: `secret_kind_name`
- **12 missing accessors/setters**: `scarg_value_two`, 6 AcceleratorFlags setters (metal, oneapi, tpu, sycl, openvino, directml), 2 TokenUsage cache setters, 3 content block factories (`content_document`, `content_audio`, `content_citation`)
- **sakshi.cyr** — vendored slim tracing/error profile (zero-alloc, stderr output, packed i64 errors)
- **Regression test** — `tests/string_shift_bug.tcyr` for compiler bug #30

### Changed
- **Stdlib synced to vidya 2.0** — alloc (arena allocator), assert (6 new helpers), fmt (f64 formatting), io (getenv), str (Str-based contains/ends_with, direct-buffer string builder), hashmap (refactored internals), process (pipefd fix), json (io.cyr dep), syscalls (threading/mmap/futex enums)
- **Str_ method wrappers removed** — reclaimed 16 function slots (unused by agnostik, consumers use `str_*` directly)
- **Syscalls trimmed** — removed admin/epoll/timer/signal/identity wrappers (not needed by a types library)
- **All existing `_name()` functions** converted from separate `if` blocks to `elif` chains
- **Cyrius compiler target**: v1.11.4 → v2.6.4
- **Test/bench file format**: `.tcyr`/`.bcyr` for `cyrius test`/`cyrius bench` auto-discovery
- **Build config**: `cyrb.toml` → `cyrius.toml`

### Testing
- 226 assertions (up from 198), 0 failures
- 15 benchmarks, no regressions
- Regression test for compiler bug #30 (str_data buffer overflow)

### Performance
- agent_id_new: 35ns, trace_context_child: 41ns, sandbox_config_default: 62ns
- version_to_str: 151ns (up from 124ns — bounds checking cost, acceptable)
- accelerator_device_full: 153ns, token_usage_update: 36ns

### Breaking
- `str_contains` and `str_ends_with` now take Str arguments instead of C strings. Callers must wrap C strings with `str_from()`.

## [0.91.0] - 2026-04-07

### Fixed
- **`#derive(Serialize)` no-op stubs** — compiler generates empty `_to_json` functions; added manual implementations for all 9 serializable structs (TokenUsage, AgentInfo, AgentStats, ResourceLimits, ResourceUsage, InjectionScores, AcceleratorFlags, EdgeResourceOverrides, TelemetryConfig)
- **SandboxConfig default** — `NET_NONE` → `NET_LOCALHOST_ONLY` (Rust parity)
- **`trace_id_from_str` rejected uppercase hex** — now accepts A-F (consistent with `agent_id_from_str`)
- **Stale version references** — CI and bench header updated from Cyrius v1.9.4 to v1.11.4
- **`file_read_all` undefined warning** — added `io.cyr` to test include chain
- **Unused `json.cyr` in bench** — removed (freed function slots, eliminated `file_read_all` warning)

### Added
- `span_id_from_str` — hex string parsing with roundtrip support
- `tctx_from_traceparent` — W3C traceparent header parse (reverse of `tctx_to_traceparent`)
- `CacheControl` enum (`CACHE_EPHEMERAL`) for Anthropic prompt caching
- `AcceleratorDevice` accessors: `temperature`, `driver_version`, `compute_capability`, `power_watts`, `memory_bandwidth_gbps`, `memory_type` (+ setters)
- `InferenceRequest` fields: `service_tier`, `metadata`, `reasoning_effort` (+ accessors)
- 84 new test assertions covering: version serde/prerelease/errors, error codes/names, sandbox defaults, RBAC roles, permissions, cgroup limits, trace context propagation, traceparent validation, log severity ordering/names, log records, crash reports, metric data points, agent dependency/manifest/pool/messages/topics, classification ordering, secret zeroize, env profiles, hardware extended fields, LLM new fields, audit integrity chain

### Changed
- **Cyrius compiler target**: v1.9.4 → v1.11.4
- **CLAUDE.md**: full rewrite for Cyrius tooling (build commands, conventions, compiler notes)
- **docs/architecture/overview.md**: rewritten from `.rs` module map to `.cyr`
- **docs/development/roadmap.md**: updated for current state, v1.0.0 criteria, backlog

### Testing
- 198 tests (up from 58 passing / 107 total), 0 failures
- 15 benchmarks, no regressions

### Performance
- agent_id_new: 35ns, trace_context_child: 43ns, sandbox_config_default: 62ns
- agent_id_to_str: 212ns, version_to_str: 123ns, token_usage_update: 36ns
- accelerator_device_full: 148ns, inference_request_full: 488ns

## [0.95.0] - 2026-04-07

### Changed
- **Ported from Rust to Cyrius** — complete rewrite from 7,121 lines of Rust to 2,624 lines of Cyrius. Zero external dependencies. 107 KB library binary (was 8.7 MB .rlib).

### Added
- 123 constructors, 57 enums, 785 functions across 12 modules
- 6 traits via vtable dispatch: SpanCollector, MetricSink, TextMapPropagator, TextMapCarrier, AuditSink, SecretStore
- `#derive(Serialize)` on 9 struct types — auto-generated `_to_json` functions
- xorshift64 PRNG replacing `/dev/urandom` syscalls — agent_id_new 28ns (was 3,000ns pre-PRNG, Rust 45ns)
- Lazy initialization for vec/map fields — sandbox_config_default 61ns (was 1,000ns, Rust 40ns)
- Hex lookup table for UUID formatting — agent_id_to_str 215ns (was 308ns)
- Direct buffer version_to_str — 122ns (was 477ns with str_builder)
- 3-tier benchmark suite: 15 benchmarks matching Rust Criterion baseline
- `cyrb.toml` with `[lib]` section and `[[bench]]` for dep consumption
- `src/lib.cyr` library entry point for downstream consumers
- CI/CD workflows updated from Cargo to Cyrius (cyrb check, fmt, lint)

### Performance
- Cyrius beats Rust on 6 of 9 comparable benchmarks
- agent_id_new: 28ns vs Rust 45ns (1.6x faster)
- trace_context_child: 40ns vs Rust 53ns (1.3x faster)
- accelerator_device_full: 148ns vs Rust 711ns (4.8x faster)
- token_usage_update: 38ns

### Testing
- 107 tests (58 functional + 49 serde serialization)
- 15 benchmarks across 3 tiers (core, format, integration)

## [0.90.0] - 2026-04-02

### Added

#### Telemetry — OTel Alignment
- **Resource** — service identity struct (service_name, service_version, service_instance_id, attributes) for OTel signal attribution
- **SpanKind** — Internal/Server/Client/Producer/Consumer (OTel span kind, added to `Span`)
- **SpanEvent** — timestamped annotations on spans (OTel span events, added to `Span.events`)
- **SpanLink** — cross-trace span relationships (OTel span links, added to `Span.links`)
- `TraceContext::to_traceparent()` / `from_traceparent()` — W3C `traceparent` header format
- **AggregationTemporality** — Cumulative/Delta for metric data points (OTel-aligned)
- `MetricDataPoint.temporality`, `is_monotonic` — metric temporality and monotonicity fields
- `SpanStatus::Unset` variant (OTel default status)

#### Security — OCI Runtime Spec Alignment
- **SeccompProfile** — complete seccomp filter profile with `default_action`, `architectures`, `flags`, and `syscalls`
- **SeccompArch** — 17 target architectures (x86, x86_64, aarch64, riscv64, etc.)
- **SeccompArg** + **SeccompArgOp** — syscall argument-level filtering with 7 comparison operators
- `SeccompAction::Kill`, `KillProcess`, `Errno(u32)`, `Trace(u32)`, `Log` variants
- `SandboxConfig.apparmor_profile` — explicit AppArmor profile field
- `SandboxConfig.selinux_label` — explicit SELinux process label field
- `SandboxConfig.seccomp` — full `SeccompProfile` field
- **MountPropagation** — Private/Shared/Slave/Unbindable for filesystem mount rules
- `FilesystemRule.readonly`, `noexec`, `nosuid`, `nodev`, `propagation` — mount option fields

#### Security — Linux Capabilities
- **LinuxCapability** expanded from 19 to 39 variants (full Linux kernel capability set including CapBpf, CapPerfmon, CapCheckpointRestore, etc.)

#### Agent — Lifecycle Management
- **RestartPolicy** — Never/Always/OnFailure for failed agent restart control
- **HealthCheck** — liveness/readiness probe configuration (interval, timeout, retries, initial delay)
- `AgentConfig.restart_policy`, `max_restarts`, `health_check`, `startup_timeout_secs`, `shutdown_timeout_secs`

#### LLM — Multimodal + Structured Output
- **ContentBlock::Image** — base64/URL image inputs with media type
- **ContentBlock::Document** — base64/URL document inputs (PDF, etc.)
- **ContentBlock::Thinking** — model reasoning/extended thinking blocks
- **ToolChoice** — Auto/None/Required/Tool(name) for tool selection control
- **ResponseFormat** — Text/JsonObject/JsonSchema for structured generation
- `TokenUsage.cache_creation_input_tokens`, `cache_read_input_tokens` — prompt caching fields
- `InferenceRequest.system` — top-level system prompt (Anthropic API pattern)
- `InferenceRequest.logprobs`, `top_logprobs` — log probability output control
- `InferenceResponse.id` — provider-assigned response ID

#### Audit — Forensics & Compliance
- **AuditResult** — Success/Failure/Denied outcome for audited actions
- `AuditEntry.result`, `source_ip`, `target_resource`, `duration_ms`, `tags` — enriched audit fields

#### Classification — Extended PII
- **PiiKind** expanded: FullName, StreetAddress, BankAccountNumber, TaxId, NationalId, MedicalRecordNumber, BiometricData
- `ClassificationResult.confidence` — classification confidence score (0.0–1.0)

#### Hardware — Device Diagnostics
- **DeviceHealth** — Ok/Degraded/Failed/Unknown health status per device
- **MemoryType** — Gddr5/Gddr6/Gddr6x/Hbm2/Hbm2e/Hbm3/Lpddr4/5/5x/Ddr4/5 memory technology
- `AcceleratorDevice.power_watts`, `memory_bandwidth_gbps`, `memory_type`, `health`
- `AcceleratorFlags.sycl_available`, `openvino_available`, `directml_available`
- `AcceleratorSummary::by_vendor()` — filter devices by vendor

#### Config — Environment Profiles
- `EnvironmentProfile::Testing`, `Canary` — CI/CD and gradual rollout profiles

#### Telemetry — Logging & Exemplars
- **LogSeverity** — Trace/Debug/Info/Warn/Error/Fatal (OTel severity levels)
- **LogRecord** — structured log type with severity, body, attributes, trace correlation, and resource
- **Exemplar** — links a metric data point to a specific trace (OTel exemplar)
- `MetricValue::Histogram.min`, `max` — OTel histogram min/max fields

#### Agent — Lifecycle Hooks & Resources
- **LifecycleHooks** — pre_start/post_start/pre_stop/post_stop with command + timeout
- `AgentConfig.lifecycle_hooks` — optional lifecycle hook configuration
- `ResourceLimits.max_disk_bytes`, `network_bandwidth_bps` — disk and network resource limits

#### Security — OCI Process Fields
- `SecurityContext.run_as_user`, `run_as_group` — UID/GID for sandboxed processes
- `SecurityContext.readonly_root_filesystem` — immutable root filesystem
- `SandboxConfig.masked_paths`, `readonly_paths` — OCI maskedPaths/readonlyPaths
- `NamespaceConfig.time` — time namespace (kernel 5.6+)

#### Validation — Injection Breakdown
- **InjectionScores** — per-category scores: sql, xss, command, path_traversal, prompt_injection

#### Error — Numeric Codes
- `AgnostikError::code()` — numeric error code (1001–1010) for API versioning and client routing

#### LLM — Provider Routing
- **ModelCapabilities** — model metadata for routing: context window, supported features, pricing
- **RateLimitInfo** — provider rate limit state: limits, remaining, reset timer

### Changed
- **Breaking**: `SpanStatus` changed from `Copy` enum `{Ok, Error, Cancelled}` to `{Unset, Ok, Error { message }}` (OTel-aligned, Error now carries optional message)
- **Breaking**: `SeccompRule.syscall: String` replaced with `names: Vec<String>` + `args: Vec<SeccompArg>`
- **Breaking**: `SeccompAction::Deny` removed — use `Kill`, `KillProcess`, or `Errno(1)` instead
- **Breaking**: `SandboxConfig.mac_profile` split into `apparmor_profile` + `selinux_label`
- **Breaking**: `SandboxConfig.seccomp_rules` replaced with `seccomp: Option<SeccompProfile>`
- **Breaking**: `AgentStatus` — added `Restarting` and `Terminated` variants (consumers must update match arms)
- **Breaking**: `SecretMetadata` — added `kind`, `tags`, `owner`, `last_accessed_at`, `last_rotated_at` fields
- **Breaking**: `AuditSink` trait — added `verify_entry()`, `query()`, `seal()` methods (default impls provided)
- **Breaking**: `SecretStore` trait — added `rotate()`, `search_by_tag()` methods (default impls provided)
- **Breaking**: `AgentId::from_str` and `UserId::from_str` now return `AgnostikError` instead of `uuid::Error` — consistent with all other `FromStr` impls in the crate
- Error message capitalization standardized: "I/O error" → "i/o error" (lowercase, matching all other variants)

### Fixed
- `AgentId::from_str` and `UserId::from_str` error messages now include expected format and underlying cause (e.g., `"invalid agent id: foo (expected UUID, invalid character)"`)
- `TraceId::from_str` and `SpanId::from_str` error messages now include expected format (e.g., `"expected 32 hex digits"`, `"expected 16 hex digits"`)
- `Version::from_str` parse error improved: `"invalid version part: x"` → `"invalid version component: x (expected unsigned integer)"`

### Testing
- Integration tests expanded from 4 to 26, covering all feature-gated modules
- 249 tests total (223 unit + 26 integration)

### Maintenance
- `deny.toml`: removed 6 unmatched license allowances (GPL-3.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-DFS-2016, Zlib)
- `scripts/bench-history.sh`: fixed broken `--output-format bencher` flag (not valid in Criterion 0.5)
- Dependencies updated: uuid 1.22→1.23, libc, zerocopy, wasm-bindgen

## [2026.3.26] - 2026-03-25

### Added

#### Classification Module (new)
- **ClassificationLevel** — Public/Internal/Confidential/Restricted (ordered by sensitivity)
- **PiiKind** — Email, Phone, SSN, CreditCard, IpAddress, Passport, DriversLicense, DateOfBirth, Custom
- **ClassificationResult** — level, auto_level, rules_triggered, pii_found, keywords_found

#### Validation Module (new)
- **ValidationSeverity** — Low/Medium/High (ordered)
- **ValidationWarning** — code, message, severity, position, pattern
- **ValidationResult** — valid, sanitized, warnings, blocked, block_reason, injection_score

#### Hardware Module (new)
- **DeviceFamily** — Gpu, Tpu, Npu, AiAsic, Cpu
- **DeviceVendor** — Nvidia, Amd, Intel, Apple, Google, Qualcomm, Habana, Aws, Custom
- **AcceleratorFlags** — cuda, rocm, metal, vulkan, oneapi, tpu availability
- **AcceleratorDevice** — full device descriptor with VRAM, utilization, temperature, driver, compute capability
- **AcceleratorSummary** — device list with `by_family()` filter

#### Security RBAC & Sandbox Capabilities
- **Role** — Admin, Operator, Auditor, Viewer, Service
- **ConditionOperator** — Eq/Neq/In/Nin/Gt/Gte/Lt/Lte for permission conditions
- **PermissionCondition**, **RolePermission** — resource-level RBAC with conditions
- **TokenPayload** — JWT claims (sub, role, permissions, iat, exp, jti, email, display_name)
- **AuthContext** — agent_id + role + permissions
- **SeccompMode** — Disabled/Strict/Filter/Unsupported
- **SandboxCapabilities** — seccomp, landlock ABI version, cgroup v2, namespace detection

#### Audit Integrity Chain
- **IntegrityFields** — version, HMAC-SHA256 signature, previous_entry_hash
- **GENESIS_HASH** constant for chain initialization
- **IntegrityFields::genesis()**, **is_genesis()** helpers
- **AuditEntry** restructured with id, correlation_id, user_id, integrity chain
- **AuditSink** trait — append, verify_chain

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
- **Display**, **FromStr** added to UserId (UUID parsing/formatting)
- **From\<Uuid\>** for AgentId and UserId
- **From\<std::io::Error\>** for AgnostikError (new `Io` variant)
- **From\<serde_json::Error\>** for AgnostikError (into `Serialization` variant)
- **Hash** added to AgentType, AgentStatus, LlmProvider, FinishReason, SystemFeature
- **Eq** added to ResourceLimits, ResourceUsage, TokenUsage
- **PartialEq**, **Eq** added to Capabilities
- Crate-root re-exports for all feature-gated modules' key types (consumers can use `agnostik::AuditEntry` instead of `agnostik::audit::AuditEntry`)

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
- **Breaking**: `Version` serde format changed from struct `{"major":1,"minor":0,"patch":0}` to string `"1.0.0"` (matches SemVer convention)
- **Breaking**: `AgentManifest.version` changed from `String` to `Version`
- **Breaking**: `AgentDependency.min_version` changed from `Option<String>` to `Option<Version>`
- **Breaking**: `AgentMessage.timestamp` changed from `Option<DateTime<Utc>>` to `DateTime<Utc>` (now required)
- **Breaking**: `AuditEntry.user_id` changed from `Option<String>` to `Option<UserId>`
- **Breaking**: `IntegrityFields.version` changed from `String` to `Version`
- **Breaking**: `Span.start_ms` renamed to `started_at` and changed from `u64` to `chrono::DateTime<Utc>`
- `agent` feature now implies `security` feature
- `audit` and `secrets` features now depend on `chrono`
- `Secret` Serialize/Deserialize omission documented in doc comments

### Performance
- Serde benchmarks added: AgentId (37/60 ns ser/de), TraceContext (166/320 ns), SandboxConfig (326/336 ns)
- New serde benchmarks: InferenceRequest (700 ns/1.22 µs), AuditEntry (800 ns/1.02 µs), AcceleratorDevice (483/480 ns)
- No regressions in existing benchmarks

### Testing
- 194 tests total (190 unit + 4 integration), up from 49
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
