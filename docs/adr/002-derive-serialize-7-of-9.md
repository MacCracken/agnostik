# ADR-002: Apply `#derive(Serialize)` to 7 of 9 structs; AgentInfo + TelemetryConfig retain hand-written impls

**Status:** Accepted
**Date:** 2026-05-10
**Slot:** v1.1.0
**Supersedes:** [ADR-001](001-revive-derive-serialize.md)

## Context

ADR-001 chose Option B (replace all 9 hand-written serde fns with
`#derive(Serialize)`) gated by golden-corpus byte equivalence. While
executing in v1.1.0, cyrius 5.10.14's derive codegen was probed against
the actual byte output of each hand-written impl. **Three gaps emerged
that ADR-001's authoring under-anticipated:**

1. **Compact byte format** — cyrius derive emits `{"k":v,"k":v}` (no
   spaces); agnostik hand-written emitted `{"k": v, "k": v}` (spaced).
   This is a byte-level mismatch but a SEMANTIC equivalent (RFC 7159
   makes inter-token whitespace optional). Acceptable to absorb as a
   one-time wire-format change; agnostik's own `_from_json` parser
   handles both forms (verified — `_json_find_value` doesn't require
   the space).

2. **Custom-shape fields don't survive derive.** AgentInfo's
   hand-written `_to_json` emits the `id` field as a 36-char UUID
   string via `agent_id_to_str(load64(ptr))`, and the `agent_type` /
   `status` fields as enum-name strings via `agent_type_name(...)` /
   `agent_status_name(...)`. Derive emits the raw int values
   instead — a semantic change consumers would notice (`{"id": "uuid",
   "agent_type": "User"}` vs `{"id":12345,"agent_type":1}`).

3. **`: Str` field with 0 value crashes derive.** TelemetryConfig's
   `export_endpoint: Str` is `0` when no endpoint is configured; the
   hand-written impl emits `"export_endpoint": null`. Cyrius 5.10.14
   derive on a `: Str`-annotated field calls `str_data(0)` /
   `str_len(0)` unconditionally — SIGSEGV. Verified against a probe
   under cyrius 5.10.14.

ADR-001 considered Option C ("hybrid: derive new structs only; keep
hand-written for the existing 9") and rejected it on the grounds of
"two parallel conventions." But the parallel-convention concern was
framed against the hypothetical of adding NEW structs; the
discovered situation is different — it's about **structs whose
hand-written shape isn't replicable by derive at all**. That's a
firmer dividing line.

## Options reconsidered

### Option A — Don't ship derive in v1.1.0 (purist re-read of ADR-001)

- **Pros:** zero risk; ADR-001 strict-read says any byte mismatch is
  abort.
- **Cons:** 14 hand-written fns (the 7 trivial structs) stay as
  rote `load64`/`store64` wiring forever; the maturity gain from
  cyrius 5.9.30..5.10.14's derive arc isn't captured.

### Option B (adjusted) — derive 7 of 9, retain 2 hand-written

- **Pros:** 14 fns deleted (real surface reduction). Clear dividing
  line: "all-int structs → derive; custom-shape structs → hand
  written."
- **Cons:** two parallel conventions (the concern ADR-001 named).
  Mitigated by adopting derive's **compact byte format** in the
  hand-written impls too — library output is uniformly compact, so
  the only difference between the two camps is field-extraction
  logic, not output format.

### Option C — refactor AgentInfo + TelemetryConfig to be derive-compatible

- Reshape AgentInfo so `id` is a bare int (the 16-byte pointer
  value), expose UUID stringification through a separate fn.
  Same for `agent_type`/`status` via `*_name()` calls outside
  serde.
- **Cons:** consumer-visible API change. Every consumer reading
  `{"id": "uuid"}` from agnostik output now sees `{"id":12345}` and
  has to call `agent_id_from_str` separately. Breaking.

## Decision

**Option B (adjusted).**

- 7 structs adopt `#derive(Serialize)`:
  ResourceLimits, ResourceUsage, AgentStats, EdgeResourceOverrides,
  InjectionScores, TokenUsage, AcceleratorFlags. All have only
  i64 fields with no custom serialization shape.
- 2 structs retain hand-written impls: AgentInfo, TelemetryConfig.
  Reason: they have custom-shape fields (UUID stringification,
  enum-name export, null-Str handling) that current cyrius derive
  cannot replicate.
- **Both camps emit compact byte format** (`{"k":v,"k":v}` no
  spaces) — uniform across the library. The hand-written
  AgentInfo + TelemetryConfig impls get reworked to this format.
- The 14 deleted hand-written fns reduce by ~280 LoC.
- Public `<Struct>_to_json` and `<Struct>_from_json` symbols stay
  for the 7 derive-driven structs (cyrius emits these names by
  default).

## Consequences

- **One-time wire format change**: agnostik JSON output is now
  compact across all 9 structs. Consumers parsing this output are
  unaffected (RFC 7159 whitespace-permissive). Consumers that
  byte-compare against agnostik output (none observed; we don't
  expect any) would need to update fixtures.
- **Two-camp clarity**: trivial all-int structs use derive; custom-
  shape structs are hand-written. The boundary is mechanical and
  documented — no judgment call per struct.
- **Future structs** with all-int fields: add `#derive(Serialize)`,
  done. Future structs with custom-shape fields: hand write,
  document why in this ADR or a successor.
- **Forecloses** ADR-001's "all 9 derive" goal until cyrius lands:
  - null-`Str` graceful handling in derive codegen (filed upstream
    as needed)
  - custom-format hook (e.g. `#derive(Serialize, format="uuid")`
    for `id` fields)
  - enum-name export (e.g. `#derive(Serialize, repr="name")` for
    enum-typed fields)
  Each gap has been verified live against cyrius 5.10.14; upstream
  issues filed if/when agnostik's needs surface them.
- **Sub-byte field widths** (paired roadmap item) defer to a future
  slot if applied to AgentInfo or TelemetryConfig (they retain the
  i64 layout); applied freely to the 7 derive structs since cyrius
  generates width-correct emit for sub-byte fields per v5.9.36.

## Verification

The same four-step plan from ADR-001, narrowed to the 7 derive
structs:

1. **Golden corpus**: `tests/tcyr/test_serde_golden.tcyr` (new at
   v1.1.0) asserts the exact byte output of each `<Struct>_to_json`
   on representative inputs (zero, max, varied).
2. **Boundary cases**: F-002 escaped quotes (none of the 7 structs
   have Str fields, so trivially passes), F-003 negatives (asserted
   for each int-field struct), F-008 i64-MAX boundaries (asserted
   for ResourceLimits / TokenUsage / AcceleratorFlags / etc.),
   F-009 truncated null (TelemetryConfig hand-written path
   exercises this; covered).
3. **Roundtrip**: existing `tests/tcyr/test_serde_roundtrip.tcyr`
   confirms `_to_json` → `_from_json` → field-extract is identity
   across all 9 structs.
4. **Consumer sweep**: deferred to v1.2.0's cross-consumer
   automation slot. Manual verification at v1.1.0 cut: the
   compact-format change is the only consumer-observable diff;
   parsers handle both forms.

## References

- [ADR-001](001-revive-derive-serialize.md) — superseded; original
  "replace all 9" decision and the in-flight discoveries that
  invalidated it for 2 structs.
- [`docs/audit/2026-04-26-audit.md` §F-011](../audit/2026-04-26-audit.md)
  — original derive-collision finding that drove the pre-1.0 removal.
- Cyrius 5.10.14 derive arc references (v5.9.30..v5.10.14) — see
  ADR-001 for the full chronology.
