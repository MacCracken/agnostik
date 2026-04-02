# Agnostik Roadmap

## Status
**v0.90.0** — 12 modules, 216 tests. API frozen. All quality gates passing. Alpha/beta for OS integration testing before v1.0.0.

## Migration

agnostik is the shared types crate for the AGNOS ecosystem, replacing agnos-common. All consumers import directly from `agnostik::*`.

## Backlog

_(No outstanding backlog items — all planned features implemented)_

## v1.0.0 Criteria
- API frozen — no breaking changes ✅
- Consistent error types — all FromStr impls return AgnostikError ✅
- Descriptive error messages with expected format hints ✅
- All types Serialize + Deserialize with roundtrip tests ✅
- All enums #[non_exhaustive] ✅
- Zero unwrap/panic ✅
- 90%+ coverage ✅ (99.53%)
