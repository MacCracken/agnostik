# Agnostik Roadmap

## Status
**v0.1.0** — Initial extraction from agnos-common. 9 modules, feature-gated.

## Migration Plan

1. agnos-common currently owns these types in the monolith
2. agnostik extracts them as standalone crate
3. agnos-common becomes thin re-export over agnostik (same pattern as agnos-sys → agnosys)
4. Consumers gradually migrate `agnos_common::*` → `agnostik::*`
5. agnos-common deprecated once all consumers migrated

## Future Features (demand-gated)

### Environment Profiles
- Edge-specific resource limits
- Fleet-wide configuration distribution
- Profile inheritance (edge inherits from production, overrides specific fields)

### Agent Manifest v2
- Capability-based permissions (replaces enum list)
- Resource negotiation (agent requests, runtime approves/modifies)
- Dependency declaration (agent A requires agent B)

### Telemetry v2
- SpanCollector trait for pluggable backends
- Metric types (counter, gauge, histogram) alongside traces
- Budget-aware sampling

## v1.0.0 Criteria
- API frozen — no breaking changes
- All types Serialize + Deserialize with roundtrip tests
- All enums #[non_exhaustive]
- Zero unwrap/panic
- 90%+ coverage
- 3+ downstream consumers in production
