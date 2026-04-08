# Agnostik Roadmap

## Status
**v0.91.0** — P(-1) hardened. 12 modules, 198 tests, 15 benchmarks. Zero external dependencies. 145 KB test binary. Cyrius v1.11.4.

## Migration

agnostik is the shared types crate for the AGNOS ecosystem, replacing agnos-common. Ported from 7,121 lines of Rust to ~3,000 lines of Cyrius.

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

## v1.0.0 Criteria
- API frozen — no breaking changes ✅
- Consistent error types — all parse functions return Result ✅
- Descriptive error messages with expected format hints ✅
- All public types have JSON serialization (9 struct types) ✅
- All enums have name functions ✅
- Zero panic in library code ✅
- Serde roundtrip tests for serializable types ✅
- W3C traceparent roundtrip (to/from) ✅
- SandboxConfig defaults match Rust (LocalhostOnly) ✅
- AcceleratorDevice full field parity with Rust ✅
- InferenceRequest field parity (service_tier, metadata, reasoning_effort) ✅
- Integration tests covering all Rust integration.rs scenarios ✅
