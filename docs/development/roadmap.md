# Agnostik Roadmap

## Status
**v0.95.0** — P(-1) hardened. 12 modules, 226 tests, 15 benchmarks. Zero external dependencies. Cyrius v2.6.4. Sakshi tracing vendored. All consumer-facing enums have `_name()` functions.

## Migration

agnostik is the shared types crate for the AGNOS ecosystem, replacing agnos-common. Ported from 7,121 lines of Rust to ~3,200 lines of Cyrius.

Consumers use `include "src/lib.cyr"` or include individual modules.

## Backlog

### Blocked on compiler (function limit increase)
- Add `_from_json` deserialization for all 9 serializable structs
- Add serde benchmarks (JSON serialize/deserialize timing)
- Restore classification/validation/config modules in bench includes
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
