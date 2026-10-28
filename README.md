# Agnostik

**Agnostik** (agnostic) — shared types, error handling, domain primitives, and
wire-format encoders for the [AGNOS](https://github.com/MacCracken/agnosticos)
ecosystem.

Written in [Cyrius](https://github.com/MacCracken/cyrius). Zero external
dependencies (stdlib only). Originally ported from Rust (~7,121 lines) to
Cyrius (~3,500 lines); v1.x has progressively modernized the implementation
against successive Cyrius type-system slots.

## Status

- **Current**: 2.0.0, which renamed every accessor to `<Struct>_<field>`. Upgrading from 1.x:
  [`docs/migration/README.md`](docs/migration/README.md) and `scripts/migrate-2.0.0.sh`.
- **Toolchain**: Cyrius `6.7.18` (pinned in `cyrius.cyml`)
- **Tests**: 2,582 assertions across 27 `.tcyr` files, passing on x86_64-linux
  and on aarch64 (under `qemu-aarch64`); `CYRIUS_TYPE_CHECK=1` clean; OTLP
  encoders byte-identical to protoc's encoding of the official schema;
  api-surface gate locked at 1,295 public fns, every one documented;
  `cyrius audit` clean; allocation-failure probes under `prlimit`; all five
  consumers swept green
- **Audits**: 2026-04-26 (pre-1.0, 11 findings closed), 2026-05-10
  (1.0.x, 1 INFO fixed), 2026-06-01 (v1.3.0, F-013 buffer safety),
  2026-08-24 (P(-1) sweep, F-014..F-021 — 6 repaired in v1.3.7, the two
  contract gaps shipped in v1.4.0), 2026-09-23 (v1.6.2 cross-target syscall
  review — F-022 fixed in v1.6.2; F-023 and F-024 fixed in v1.6.3),
  2026-10-14 (v1.6.5 toolchain refresh — no findings; v1.7.0 minor-cut
  pass — F-025 helper namespacing, fixed in v1.7.0; v1.7.1 release pass —
  no findings; v1.8.0 minor-cut pass — no findings; v1.9.0 minor-cut pass —
  F-026 JSON string escaping, fixed in v1.9.0; v1.10.0 minor-cut pass —
  F-027 OTLP encoder bounds on malformed values, fixed in v1.10.0; v1.11.0
  and v1.12.0 minor-cut passes — no findings), 2026-10-28 (v1.12.1 toolchain
  refresh, cyrius 6.7.9 → 6.7.18, and v1.12.2 CI pass — no findings; v1.12.3 P(-1)
  pass — F-028..F-047, three MEDIUM, all repaired; v2.0.0 release pass — no new
  finding).
  Cadence: every minor cut and toolchain refresh, plus each P(-1) pass.
- **Per-parser fuzz**: 9 parser entry points × 200 deterministic
  iterations + audit-finding regression seeds — runs every CI build.

See [`docs/development/state.md`](docs/development/state.md) for the live
snapshot, [`CHANGELOG.md`](CHANGELOG.md) for release notes,
[`docs/development/roadmap.md`](docs/development/roadmap.md) for what's next.

## Modules

| Module | Surface |
|---|---|
| `error.cyr` | `AgnostikError` (11 kinds) + numeric error codes (1001-1010) + `Result` helpers |
| `types.cyr` | `AgentId` / `UserId` (UUID v4), `Version` (SemVer), `Capabilities`, `MessageType`, `SystemStatus`, `ComponentConfig`. Hosts the JSON parser primitives (`_stik_json_int`/`_stik_json_str`/`_stik_json_find_value`) with `\uXXXX` Unicode + UTF-8 surrogate-pair decoding (v1.0.7). |
| `proto.cyr` | Protobuf wire-format writer — varint, tag, length-delimited, fixed64, and nested messages written in place behind a patched length (v1.10.0). Foundation for the OTLP encoders. |
| `agent.cyr` | `AgentConfig`, `AgentManifest`, `AgentStatus`, `ResourceLimits`, `HealthCheck`, `LifecycleHooks`, `AgentPool`, `Topic`, `Subscription`, `AgentMessage`. |
| `security.cyr` | `SandboxConfig`, RBAC (`Role`, `TokenPayload`, `AuthContext`), `CgroupLimits`, `NamespaceConfig`, `LandlockRuleset`, `LinuxCapability` (41), `CapabilitySet`, `SeccompProfile`. |
| `telemetry.cyr` | `TraceContext` (W3C), `Span`, `MetricDataPoint`, `LogRecord`, `Resource`, `InstrumentationScope`, `Baggage`, `Exemplar`. Traits: `SpanCollector`, `MetricSink`, `TextMapPropagator`, `TextMapCarrier`. Typed `AnyValue` attribute values. **OTLP wire encoders** for `Span` (with attributes, events, links and status message), `LogRecord`, `NumberDataPoint` (with exemplars) and `Metric` (gauge, sum, histogram), byte-identical to protoc (v1.10.0); `Resource`, `InstrumentationScope` and the traces / logs / metrics export requests, with valid UTF-8 in every string field (v1.12.0). |
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
var parsed_t, parsed = agent_id_from_str(str_from("550e8400-e29b-41d4-a716-446655440000"));
assert_eq(is_ok(parsed_t), 1, "valid UUID parses");      # a Result binds tag + value

# Tracing (W3C TraceContext)
var ctx = trace_context_new();
var child = tctx_child(ctx);                          # inherits trace_id, flags, state
assert_eq(tctx_is_sampled(ctx), 1, "sampled");

# OTLP wire-format encode (v1.10.0: attributes, events, links, logs, metrics)
var span = span_new(str_from("op"), trace_id_new(), span_id_new());
map_set(Span_attributes(span), "http.status_code", any_int(200));   # values are AnyValue boxes
vec_push(Span_events(span), span_event_new(str_from("retry")));
var sb = str_builder_new();
Span_to_otlp_proto(span, sb);                         # opentelemetry.proto.trace.v1.Span bytes

# JSON serde: hand-written, bounded decoders (ADR-008)
var rl = resource_limits_new();
var j = str_builder_new();
ResourceLimits_to_json(rl, j);                        # `{"max_memory":268435456,...}`
```

## Build / Test / Bench

```bash
cyrius lib sync                                       # vendor the pinned stdlib's declared subset into lib/
cyrius deps                                           # add its transitive includes (agnostik has no git deps)
cyrius build src/main.cyr build/agnostik              # compile the test harness
cyrius test tests                                     # every .tcyr under tests/ (counts in state.md)
cyrius bench tests/bcyr/agnostik.bcyr                 # 31 benchmarks
scripts/bench-regression.sh                           # vs baseline in history.csv (release host only)
scripts/bench-paired.sh                               # paired A/B vs the last release tag, any host (CI's gate)
scripts/api-surface.sh check                          # diff vs committed snapshot
scripts/doc-debt.sh check                             # every fn in src/ documented
scripts/consumer-sweep.sh                             # build + test every consumer against this tree
```

## Consumers

agnostik is the AGNOS type vocabulary. Five repos declare `[deps.agnostik]`
today: **aegis** (security daemon), **anuenue**, **ark** (package manager),
**kybernet** (PID 1 init) and **mela** (marketplace). Tags, pins and vendored
modules are in [`docs/development/state.md`](docs/development/state.md).
`scripts/consumer-sweep.sh` builds and tests each of them against the working
tree, on its own cyrius pin, before a release.

## Decisions

- [`docs/adr/001-revive-derive-serialize.md`](docs/adr/001-revive-derive-serialize.md)
  — derive plan (superseded).
- [`docs/adr/002-derive-serialize-7-of-9.md`](docs/adr/002-derive-serialize-7-of-9.md)
  — `#derive(Serialize)` for 7 of 9 trivial structs (superseded by ADR-008).
- [`docs/adr/007-allocation-failure-fails-by-name.md`](docs/adr/007-allocation-failure-fails-by-name.md)
  — every allocation goes through `_stik_alloc`, which names a failure and exits
  71; `agnostik_err_new` returns a static out-of-memory error instead.
- [`docs/adr/008-value-struct-serde-is-hand-written.md`](docs/adr/008-value-struct-serde-is-hand-written.md)
  — the seven value structs' serde is hand-written, bounded and range-checked,
  with the same names, arity and output bytes.

## License

GPL-3.0-only.
