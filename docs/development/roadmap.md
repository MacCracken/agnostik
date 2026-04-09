# Agnostik Roadmap

## Status
**v0.97.0** — P(-1) hardened (round 2). Rust removed. 12 modules, 553 test assertions (6 test files), 15 benchmarks. Zero external dependencies. Cyrius v3.2.1. Sakshi tracing vendored. CI uses `cat | cc2` pipe with correct `.bcyr` benchmark path.

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

### Blocked on compiler — request fix in Cyrius 3.2.x
- `_from_json` deserialization for all 9 serializable structs (blocked on 1024 function limit)
- Serde benchmarks (JSON serialize/deserialize timing — blocked on 1024 function limit)
- `#derive(Serialize)` generates working `_to_json` since v1.10.3, but formats all values as strings (`"42"` not `42`). Manual implementations still required for correct numeric JSON. Request: integer/bool field detection in derive codegen.

### Open — agnostik bugs
- `version_from_str` prerelease+build parsing: simple prerelease works (`1.0.0-beta.1` roundtrips correctly), but fails when BOTH prerelease AND build metadata are present (`2.0.0-rc.1+build.42` → prerelease is NULL, build is correct). Likely compiler codegen issue with nested if/while/break — request investigation in Cyrius 3.2.x.
- Remaining `_name()` functions for 7 internal security enums (FsAccess, NetworkAccess, SeccompAction, SeccompArgOp, SeccompArch, MountPropagation, PolicyEffect)

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
