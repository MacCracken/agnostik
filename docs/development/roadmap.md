# Agnostik Roadmap

## Status
**v0.96.0** — P(-1) hardened. Rust removed. 12 modules, 226 tests, 15 benchmarks. Zero external dependencies. Cyrius v2.6.4. Sakshi tracing vendored. CI uses `cyrius` CLI tooling.

## Migration

Complete. 7,121 lines of Rust → ~3,200 lines of Cyrius. rust-old/ deleted. Benchmark comparison preserved in `benchmark-rustvcyrius.md`.

Consumers use `include "src/lib.cyr"` or include individual modules.

## Completed (0.91.0–0.96.0)
- Rust→Cyrius port (all 12 modules, all public items verified)
- P(-1) scaffold hardening (critical bugs, serialize hardening, elif chains)
- 14 `_name()` functions for consumer-facing enums
- 12 missing accessors/setters/factories
- Stdlib synced to vidya 2.0 (9 files + sakshi)
- Compiler bug #30 found and fixed (str_data buffer overflow)
- CI/release workflows rewritten for `cyrius` CLI
- `cyrfmt` formatting pass
- Rust benchmark reference captured, rust-old/ removed

## Backlog

### Blocked on compiler (function limit increase)
- Add `_from_json` deserialization for all 9 serializable structs
- Add serde benchmarks (JSON serialize/deserialize timing)
- Split integration tests into per-module functions

### Open
- `version_from_str` prerelease field not populated on parse (set/get works — needs compiler investigation)
- `#derive(Serialize)` generates no-op stubs — manual `_to_json` required (compiler fix or document as intentional)
- Remaining `_name()` functions for internal security enums (FsAccess, NetworkAccess, SeccompAction, SeccompArgOp, SeccompArch, MountPropagation, PolicyEffect)

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
