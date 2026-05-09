# ADR-001: Revive `#derive(Serialize)` for the 9 hand-written serde structs

**Status:** Accepted
**Date:** 2026-05-09
**Slot:** v1.1.0

## Context

Pre-1.0, agnostik used `#derive(Serialize)` markers on 9 struct types to
auto-generate `<Struct>_to_json` and `<Struct>_from_json` functions. Cyrius
v5.7.9 introduced a `duplicate fn` warning that surfaced a long-latent bug:
the compiler was emitting *empty* derive-generated bodies that were silently
shadowed at link time by hand-written adapters living alongside them. The
hand-written impls were what consumers actually called; the dead derive bodies
were just bloating the binary.

The 1.0.0 audit closed this as F-011: dropped the 9 derive markers, kept the
hand-written adapters as canonical (they incorporated the F-002 / F-003 / F-008
audit fixes that the early derive codegen lacked anyway). 18 hand-written
functions remain in the source today — rote field-by-field JSON wiring
across `agent.cyr`, `config.cyr`, `validation.cyr`, `telemetry.cyr`,
`llm.cyr`, `hardware.cyr`.

Between 1.0.0 and 1.0.3 the cyrius derive codegen matured substantially:

| Cyrius slot | Closed |
|---|---|
| v5.9.30 | typed-i64 emit |
| v5.9.31 | API rename (`<Struct>_to_json` shape stabilized) |
| v5.9.36 | narrow-int (i8/i16/i32) |
| v5.9.39 | Mach-O ARM64 fn-pointer ASLR — Linux + Mach-O cross-host parity |
| v5.10.7 | Str-typed struct field positional init |
| v5.10.8 | JSON escape (quote / backslash / control chars) |
| v5.10.14 | Multi-stack `#derive(...)` directives |

Every consumer-pain shape from the original F-011 incident is now closed.

## Options considered

### Option A — Keep hand-written adapters as the canonical form (status quo)

- **Pros**: zero risk of audit regressions; every byte that goes out the wire
  is one we wrote and reviewed; F-002 / F-003 / F-008 fixes are right where
  the reader expects them.
- **Cons**: 18 functions of mechanical, easily-out-of-sync wiring; new
  serializable structs cost one new pair each; field additions cost one new
  store64+_json_int line in two functions; readers have to verify by eye that
  every field is mirrored to/from JSON.

### Option B — Replace with `#derive(Serialize)` outright

- **Pros**: 18 functions disappear; field additions become a single struct
  declaration edit; cyrius emits identical JSON shape to what the hand-written
  impls produce when the struct shape matches.
- **Cons**: derive output bytes must round-trip *identically* to the
  hand-written form, including field order and edge-case handling for
  escaped quotes, negative ints, and i64-MAX-boundary values. Any silent
  drift breaks every consumer's parser. The audit fixes are spread across
  `_to_json` / `_from_json` and not every fix is reproducible in derive
  emit.

### Option C — Hybrid: derive only for new structs, keep hand-written for the existing 9

- **Pros**: avoids the round-trip-equivalence risk for the existing surface.
- **Cons**: two parallel conventions in the same codebase; the 9 hand-written
  impls remain forever; the migration cost is paid lazily but never paid.

## Decision

**Option B**, gated by a golden-corpus byte-equivalence test that must pass
before deletion.

## Consequences

After v1.1.0 lands:

- 18 functions deleted across 6 source files (~360 LoC removed).
- 9 struct types carry `#derive(Serialize)` markers.
- New serializable structs cost zero serde-wiring effort — just declare the
  struct with typed fields and add the marker.
- F-002 / F-003 / F-008 audit fixes are no longer in agnostik source — they
  live in cyrius's derive codegen (5.10.7 / 5.10.8 closed the boundaries).
  Audit lineage shifts: any future regression to those boundary cases is a
  cyrius bug to file upstream, not an agnostik patch.
- Field additions: one struct declaration line; serde wires through.

This decision **forecloses**:

- Custom JSON shapes per struct (e.g. one struct emitting snake_case, another
  emitting camelCase). Derive emits a single shape; if a consumer wants a
  variant we'd have to either fork the struct or add custom adapter code that
  partially re-introduces the hand-written form.
- Per-field control over null / 0 / empty-string defaults. Today's
  hand-written code makes case-by-case calls (e.g. `AgentInfo_to_json` emits
  `null` for missing names; `TelemetryConfig_to_json` emits `null` for
  missing endpoint). Derive uses one consistent strategy — verify it matches
  every existing case before swapping.

This decision **does not foreclose**:

- Future ADR superseding this one if a consumer needs a non-derive shape and
  the hybrid (Option C) becomes the lesser evil.

## Verification

Before swapping any struct from hand-written to derive:

1. **Golden corpus**: write `tests/corpus/serde_golden.cyr` that exercises
   every `<Struct>_to_json` with representative inputs (zero values, max
   values, escaped strings, empty optional fields, populated optional fields,
   negative ints where allowed) and asserts on the exact byte output.
2. **Boundary cases**: for each of F-002 (escaped quotes), F-003 (negatives),
   F-008 (i64-MAX-boundary 19-digit overflow), F-009 (truncated null literal),
   add explicit corpus rows.
3. **Roundtrip**: every emit→parse→re-emit cycle must produce identical bytes
   to the original emit.
4. **Consumer sweep**: rebuild the 11 consumers in `state.md` against the
   in-flight branch; all must pass their own test suites unchanged.

The work merges only when all four pass. If any fails, debug the derive output
or file the cyrius bug upstream and either revert this ADR (move to Option A)
or wait for the upstream fix (Option C as transitional).

## References

- [`docs/audit/2026-04-26-audit.md` §F-011](../audit/2026-04-26-audit.md) —
  original derive-collision dead-code finding.
- [Cyrius CHANGELOG.md `#derive(Serialize)` arc](https://github.com/MacCracken/cyrius/blob/main/CHANGELOG.md)
  — slots v5.9.30, v5.9.31, v5.9.36, v5.9.39, v5.10.7, v5.10.8, v5.10.14.
- [`docs/development/roadmap.md` §v1.1.0](../development/roadmap.md) — slot
  pin and surrounding scope.
