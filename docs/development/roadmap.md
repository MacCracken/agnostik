# Agnostik Roadmap

## Status
**v0.97.0** — P(-1) hardened (round 2). Rust removed. 12 modules, 613 test assertions (7 test files), 15 benchmarks. Zero external dependencies. Cyrius v3.2.4. Sakshi tracing vendored. CI uses `cat | cc2` pipe with correct `.bcyr` benchmark path.

## Migration

Complete. 7,121 lines of Rust → ~3,200 lines of Cyrius. rust-old/ deleted. Benchmark comparison preserved in `benchmark-rustvcyrius.md`.

Consumers use `include "src/lib.cyr"` or include individual modules.

## Completed (0.91.0–0.97.0)
- Rust→Cyrius port (all 12 modules, all public items verified)
- P(-1) scaffold hardening rounds 1 & 2
- 14 `_name()` functions for consumer-facing enums
- 12 missing accessors/setters/factories
- Stdlib synced to vidya 2.0 (9 files + sakshi)
- Compiler bug #30 found and fixed (str_data buffer overflow)
- CI/release workflows rewritten for `cyrius` CLI
- CI benchmark path fixed (`bench.cyr` → `bench.bcyr`)
- `cyrfmt` formatting pass
- Rust benchmark reference captured, rust-old/ removed
- Extended test coverage: agent lifecycle/capabilities/scheduling, security contexts/policies/capabilities, LLM tools/sampling/streaming/content blocks, telemetry spans/metrics/logs/exemplars, audit entries/integrity, hardware device accessors/flags/summary
- CONTRIBUTING.md and SECURITY.md rewritten for Cyrius
- Integration tests split into per-module files (test_coverage_1–4)

## Backlog

### Completed in v0.97.0 (from_json)
- `_from_json` deserialization for 8 of 9 serializable structs (ResourceLimits, ResourceUsage, AgentStats, TokenUsage, AcceleratorFlags, EdgeResourceOverrides, TelemetryConfig, InjectionScores) + AgentInfo (with UUID parsing)
- Break-free JSON field extractors (`_json_int`, `_json_str`, `_jfind`, `_str_find`) — workaround for compiler nested break bug
- 36 roundtrip test assertions (serialize → deserialize → verify all fields)

### Completed in v0.97.0 (serde benchmarks)
- 10 serde benchmarks (5 to_json + 5 from_json): AgentStats, InjectionScores, TokenUsage, ResourceLimits, AcceleratorFlags
- Serialization: ~1us (3-field) to ~2us (9-field)
- Deserialization: ~2us (3-field) to ~25us (9-field) — linear scan per field

### Resolved by Cyrius 3.2.3
- `#derive(Serialize)` now generates correct JSON for mixed structs: integers as bare numbers, `: Str` annotated fields as quoted strings. Manual `_to_json` implementations can be replaced with `#derive(Serialize)` using field type annotations.
- `version_from_str` prerelease+build parsing fixed — `2.0.0-rc.1+build.42` now correctly parses prerelease="rc.1", build="build.42".

### Completed in v0.97.0 (late)
- 7 `_name()` functions for internal security enums: `fs_access_name`, `net_access_name`, `seccomp_action_name`, `seccomp_arg_op_name`, `seccomp_arch_name`, `mount_propagation_name`, `policy_effect_name`

## v1.0.0 Criteria
- API frozen — no breaking changes ✅
- Consistent error types — all parse functions return Result ✅
- Descriptive error messages with expected format hints ✅
- All public types have JSON serialization (9 struct types) ✅
- All consumer-facing enums have name functions ✅
- Zero panic in library code ✅
- Serde roundtrip tests for serializable types ✅
- W3C traceparent roundtrip (to/from) ✅
- SandboxConfig defaults match Rust (LocalhostOnly) ✅
- AcceleratorDevice full field parity with Rust ✅
- InferenceRequest field parity (service_tier, metadata, reasoning_effort) ✅
- Integration tests covering all Rust integration.rs scenarios ✅
- Kernel boots with agnostik types — pending (v1.0.0 gate)
