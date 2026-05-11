# Agnostik

**Agnostik** (agnostic) — shared types, error handling, domain primitives, and
wire-format encoders for the [AGNOS](https://github.com/MacCracken/agnosticos)
ecosystem.

Written in [Cyrius](https://github.com/MacCracken/cyrius). Zero external
dependencies (stdlib only). Originally ported from Rust (~7,121 lines) to
Cyrius (~3,500 lines); v1.x has progressively modernized the implementation
against successive Cyrius type-system slots.

## Status

- **Current**: 1.2.2
- **Toolchain**: Cyrius `5.10.44` (pinned in `cyrius.cyml`)
- **Tests**: 851 assertions across 14 `.tcyr` files; `CYRIUS_TYPE_CHECK=1`
  clean; api-surface gate locked at 871 public fns
- **Audits**: 2026-04-26 (pre-1.0, 11 findings closed) +
  2026-05-10 (1.0.x line, 1 INFO finding fixed). Cadence: every minor cut.
- **Per-parser fuzz**: 8 parser entry points × 200 deterministic
  iterations + audit-finding regression seeds — runs every CI build.

See [`docs/development/state.md`](docs/development/state.md) for the live
snapshot, [`CHANGELOG.md`](CHANGELOG.md) for release notes,
[`docs/development/roadmap.md`](docs/development/roadmap.md) for what's next.

## Modules

| Module | Surface |
|---|---|
| `error.cyr` | `AgnostikError` (11 kinds) + numeric error codes (1001-1010) + `Result` helpers |
| `types.cyr` | `AgentId` / `UserId` (UUID v4), `Version` (SemVer), `Capabilities`, `MessageType`, `SystemStatus`, `ComponentConfig`. Hosts the JSON parser primitives (`_json_int`/`_json_str`/`_json_find_value`) with `\uXXXX` Unicode + UTF-8 surrogate-pair decoding (v1.0.7). |
| `proto.cyr` | Protobuf wire-format helpers — varint, tag, length-delimited, fixed64, nested-message. Foundation for OTLP encoders (v1.2.0). |
| `agent.cyr` | `AgentConfig`, `AgentManifest`, `AgentStatus`, `ResourceLimits`, `HealthCheck`, `LifecycleHooks`, `AgentPool`, `Topic`, `Subscription`, `AgentMessage`. |
| `security.cyr` | `SandboxConfig`, RBAC (`Role`, `TokenPayload`, `AuthContext`), `CgroupLimits`, `NamespaceConfig`, `LandlockRuleset`, `LinuxCapability` (39), `CapabilitySet`, `SeccompProfile`. |
| `telemetry.cyr` | `TraceContext` (W3C), `Span`, `MetricDataPoint`, `LogRecord`, `Resource`, `InstrumentationScope`, `Baggage`, `Exemplar`. Traits: `SpanCollector`, `MetricSink`, `TextMapPropagator`, `TextMapCarrier`. **OTLP wire encoder** for `Span` (v1.2.0). |
| `audit.cyr` | `AuditEntry` with HMAC-SHA256 integrity-chain shape, `AuditSeverity`, `AuditSink` trait. |
| `llm.cyr` | `LlmProvider` (18 variants — major hosted providers + 5 added in v1.1.0), `Message`, `ContentBlock` (8 kinds), `ToolDefinition`/`ToolCall`/`ToolResult`, `SamplingParams`, `InferenceRequest`/`InferenceResponse`, `StreamEvent`, `ModelCapabilities` (15 flags incl. video / caching / parallel-tool-calls). |
| `secrets.cyr` | `Secret` (zeroize-on-destroy), `SecretMetadata`, `SecretKind`, `SecretStore` trait. |
| `config.cyr` | `EnvironmentProfile`, `AgnosConfig`, `EdgeResourceOverrides`, `ProfileDefinition`, `FleetConfig`. |
| `classification.cyr` | `ClassificationLevel`, `PiiKind` (19 variants — incl. `PII_GENETIC`, `PII_BIOMETRIC_TEMPLATE`, `PII_PRECISE_GEOLOCATION` for emerging regulatory categories), `ClassificationResult`. |
| `validation.cyr` | `ValidationResult`, `ValidationWarning`, `InjectionScores` (5 i8 fields: SQL / XSS / command / path / prompt). |
| `hardware.cyr` | `AcceleratorDevice`, `DeviceFamily` (5), `DeviceVendor` (9), `MemoryType` (12), `AcceleratorFlags` (9 i8 fields), `AcceleratorSummary` (by-family / by-vendor lookups). |

## Quick Start

```cyrius
include "src/lib.cyr"

# Identifiers
var id = agent_id_new();                              # CSPRNG-backed UUID v4
var parsed = agent_id_from_str(str_from("550e8400-e29b-41d4-a716-446655440000"));
assert_eq(is_ok(parsed), 1, "valid UUID parses");

# Tracing (W3C TraceContext)
var ctx = trace_context_new();
var child = tctx_child(ctx);                          # inherits trace_id, flags, state
assert_eq(tctx_is_sampled(ctx), 1, "sampled");

# OTLP wire-format encode (v1.2.0)
var span = span_new(str_from("op"), trace_id_new(), span_id_new());
var sb = str_builder_new();
Span_to_otlp_proto(span, sb);                         # bytes ready for export

# JSON serde — derive-driven for the trivial structs (v1.1.0)
var rl = resource_limits_new();
var j = str_builder_new();
ResourceLimits_to_json(rl, j);                        # `{"max_memory":268435456,...}`
```

## Build / Test / Bench

```bash
cyrius deps                                           # resolve stdlib + git deps into lib/
cyrius build src/main.cyr build/agnostik              # compile the test harness
for t in tests/tcyr/*.tcyr; do cyrius test "$t"; done # 851/851
cyrius bench tests/bcyr/agnostik.bcyr                 # 25 benchmarks
scripts/bench-regression.sh                           # vs baseline in history.csv
scripts/api-surface.sh check                          # diff vs committed snapshot
```

## Consumers

Every AGNOS component depends on agnostik for shared types: **daimon** (agent
runtime), **hoosh** (LLM grounding), **agnoshi** (shell), **aegis** (security
policy), **argonaut** (orchestrator), **sigil** (auth issuer), **ark**
(packaging), **kavach** (sandbox), **stiva** (telemetry pipeline), **nein**
(refusal/safety), **yukti** (device abstraction). Canonical list in
[`docs/development/state.md`](docs/development/state.md).

## Decisions

- [`docs/adr/001-revive-derive-serialize.md`](docs/adr/001-revive-derive-serialize.md)
  — derive plan (superseded).
- [`docs/adr/002-derive-serialize-7-of-9.md`](docs/adr/002-derive-serialize-7-of-9.md)
  — `#derive(Serialize)` for 7 of 9 trivial structs; `AgentInfo` and
  `TelemetryConfig` retain hand-written impls (custom UUID stringification +
  enum-name lookup + null-Str fallback).

## License

GPL-3.0-only.
