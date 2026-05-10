# Agnostik Roadmap

## Status

**v1.0.3** — most recent stable. 12 modules, 653 test assertions across 9 test files,
25 benchmarks, zero external dependencies, Cyrius 5.10.14. See [`state.md`](state.md)
for the live snapshot, [`../audit/2026-04-26-audit.md`](../audit/2026-04-26-audit.md)
for the 1.0.0 audit report, and [`../../CHANGELOG.md`](../../CHANGELOG.md) for full
release notes.

Every item in this roadmap is pinned to a specific release — no "Future
Considerations" bucket, no held-without-trigger items. The principle: if work is worth
doing, it has a slot; if it isn't, it isn't here.

## Shipped (terse)

- ✅ **v1.0.0** — first stable; toolchain refresh 3.2.5 → 5.7.12; manifest format
  migration; layout aligned with vidya/yukti; F-001..F-011 audit closed.
- ✅ **v1.0.1** — toolchain refresh 5.7.12 → 5.10.3; doc cleanup (CONTRIBUTING,
  SECURITY, roadmap status); benchmark legacy file moved.
- ✅ **v1.0.2** — Cyrius 5.10 modernization: test boilerplate drop, `chrono` adoption
  (`clock_now_ns`), `?` operator in parse fns, `: Str` annotation pass across 12
  source files, single-char `str_builder_putc`.
- ✅ **v1.0.3** — toolchain refresh 5.10.3 → 5.10.14; `CYRIUS_TYPE_CHECK=1` CI gate;
  `'}'` putc workaround reverted; `docs/development/issues/archive/` convention; CI
  install layout fix for the version-pinned lib path.

---

## v1.0.x — Fast-follow chain (small, additive, non-breaking)

Each slot is a small patch release: hygiene, hardening, or additive feature work
that doesn't earn a minor bump. Order is roughly priority + dependency.

### v1.0.4 — Doc + ergonomic small wins

🧹 **Hygiene** — Add `docs/adr/` directory + ADR-001 (the v1.1.0 derive-revival
decision: trigger conditions, golden-corpus plan, F-002/F-003/F-008 byte-equivalence
requirement). CLAUDE.md and the Documentation Structure section both reference
`docs/adr/` but the directory doesn't exist yet.

🔧 **Optimization** — Pointer-to-struct dot syntax adoption (`s.data` / `s.len`
in place of `str_data(s)` / `str_len(s)` where readability wins). Selective
refactor — not wholesale; touch only call-sites already being edited for other
reasons. v5.8.17 syntax + v5.10.4 inference make this clean.

### v1.0.5 — Test + API hygiene

🧹 **Hygiene** — Adopt `lib/test.cyr`'s `test_each` table-driven helper in the
F-008/F-009/F-010 regression files. The current shape is hand-rolled `assert_eq`
chains; collapse to one case-table per finding-cluster. Smaller files, easier to add
new regression rows.

🛡️ **Hardening** — API surface snapshot in CI. Cyrius ships `cyrius_api_surface`;
generate the snapshot at release time, commit it as `docs/api/surface.snapshot`,
add a CI gate that diffs against committed and fails on unexplained drift. Catches
unintended public-API removals/renames between releases.

### v1.0.6 — Performance observability

🛡️ **Hardening** — Bench-regression CI gate. We already track
`docs/benchmarks/history.csv`; add a small script that compares the current
benchmark CSV row against the previous tagged release's row and fails CI on a
regression beyond a per-bench threshold (e.g. >15% per-op slowdown). No
auto-revert; requires human ack via a `[bench-regression-ack]` commit-message tag.

🔧 **Optimization** — Compile-time profile pass. Run `CYRIUS_PROF=1 cyrius build`
on agnostik and snapshot the per-phase distribution. Action only if a phase
exceeds 30% of total compile time *and* the cause is something agnostik can fix
(heavy include count, large gvar init block). Otherwise close the slot with a
no-action note. (Was the v5.10.0 framing's "Future Considerations" item.)

### v1.0.7 — Small additive features

✨ **Feature** — JSON `\uXXXX` Unicode escape decoder (`src/types.cyr:_json_str`).
Currently passes 6-byte literal through; consumers carrying non-ASCII text
through serde get garbage. Additive: existing ASCII paths unchanged. Pinned here
not because a consumer surfaced — but because if we're going to do it eventually,
1.0.7 is the cheap slot and it removes a documented limitation that's tracked in
the audit report's F-004 follow-up note.

✨ **Feature** — New `PiiKind` variants for emerging regulatory-attention
categories (proposal: `PII_GENETIC`, `PII_BIOMETRIC_TEMPLATE`,
`PII_PRECISE_GEOLOCATION`). Additive enum extensions; existing consumers ignore
unknown variants by falling through to `PII_CUSTOM`. Backed by GDPR Article 9 +
CCPA "sensitive personal information" + state-level expansions (CO/CT/VA).
Tracks ecosystem reality without requiring a consumer pin since the change is
purely additive.

### v1.0.8 — Post-1.0 security audit pass

🛡️ **Hardening** — Run a fresh security audit per CLAUDE.md's "Security
Hardening (before every release)" procedure. The 2026-04-26 audit was pre-1.0
and closed F-001..F-011; the 1.0.x line has added `: Str` annotations and
swapped `now_ns` to `clock_now_ns` (CLOCK_MONOTONIC) — neither is plausibly
security-relevant, but the audit cadence shouldn't lapse for an entire minor
line. New findings file at `docs/audit/<date>-audit.md`. **Established cadence:
audit at every minor cut and on demand if a CVE/0-day pattern surfaces; not
literally every patch.**

---

## v1.1.0 — Modernization + features bundle (the next minor)

Larger scope than 1.0.x: bundles ~3 weeks of mixed work — optimizations that
benefit from cyrius 5.10.x type-system maturity, new LLM domain types tracking
ecosystem reality, and the fuzz hardening that should have predated 1.0 but
didn't. Order inside this section reflects implementation dependency.

### Revive `#derive(Serialize)` — eliminates 18 hand-written serde fns

🔧 **Optimization (centerpiece)**

- **Surface today**: 9 structs across `agent.cyr` (ResourceLimits, ResourceUsage,
  AgentInfo, AgentStats), `config.cyr` (EdgeResourceOverrides), `validation.cyr`
  (InjectionScores), `telemetry.cyr` (TelemetryConfig), `llm.cyr` (TokenUsage),
  `hardware.cyr` (AcceleratorFlags) carry hand-written `<Struct>_to_json` +
  `<Struct>_from_json` pairs — ~18 functions of rote field-by-field JSON
  wiring.
- **Why removed pre-1.0**: F-011 (audit 2026-04-26) — old compiler emitted
  dead-code derive stubs that shadowed the hand-written impls. Cyrius v5.9.30/.31
  /.36 fixed typed-i64, narrow-int, and API-rename bugs; v5.9.39 closed the
  Mach-O ARM64 fn-pointer ASLR cascade; v5.10.7 closed Str-field positional-init;
  v5.10.8 fixed JSON escape (quote / backslash / control chars); v5.10.14 added
  multi-stack `#derive(...)` directive support.
- **Plan**: capture a golden-corpus snapshot of every `<Struct>_to_json` output
  against the current hand-written impls before swapping. Keep current tests;
  add a roundtrip diff against the golden corpus. Hand-written impls retain
  F-002 / F-003 / F-008 audit fixes; verify derive emits equivalent bytes for
  the security-relevant cases (escaped quotes, negative ints, max-int boundaries,
  truncated null).
- **Risk**: if derive output drifts from hand-written form on any boundary case,
  every consumer's parser breaks silently. The golden-corpus diff is the
  must-have safety net.

### Sub-byte field widths — shrinks several hot structs (deferred)

🔧 **Optimization** — *deferred from v1.1.0 to v1.1.1.* The v1.1.0
derive-revival landed on a smaller scope than ADR-001 envisioned (7 of
9 structs, per ADR-002 — AgentInfo + TelemetryConfig retained
hand-written). Sub-byte widths apply cleanly to derive-driven structs
only; tackling them as a separate slot keeps v1.1.0 focused on the
derive transition. Targets unchanged: `InjectionScores` (5×i64 → i8:
40 B → 5 B); `AcceleratorFlags` (9×i64 → i8: 72 B → 9 B). ABI
verification still required.

### New LLM capabilities tracking ecosystem reality

✨ **Feature**

- **Surface today**: `src/llm.cyr` ships `LlmProvider` (13 entries) and
  `ModelCapabilities` (12 boolean flags). Provider list and capability flags
  haven't tracked the 2026-Q1/Q2 wave (newer Anthropic / OpenAI / Google
  releases with extended thinking, larger context windows, audio I/O variations,
  output-token caps near 128K).
- **Plan**: additive — new `mcap_supports_*` flags for any capability now in
  production at ≥2 of the existing providers. Document each in
  `docs/architecture/overview.md`. No removals, no renames; existing consumers
  see unchanged behavior on the existing flags.
- **Tracks ecosystem reality** — additive enum/flag extensions don't require
  a consumer pin since the change is purely additive (existing consumers ignore
  unknown flags via the `_json_int` fall-through-zero path).

### Fuzz harness for parsers (deferred)

🛡️ **Hardening** — *deferred from v1.1.0 to v1.1.2.* Same scoping
rationale as sub-byte widths: keep v1.1.0 focused on the derive
transition. Surface (8 parsers) and plan unchanged from the
original v1.1.0 framing. Audit-finding seeds (F-002..F-005,
F-008..F-010) become regression rows in the harness corpus.

---

## v1.2.0 — Ecosystem expansion

### OTLP wire primitives — **v1.2.0 partial; rest pinned to v1.2.2**

✨ **Feature** — Spans, metrics, and log records model the OpenTelemetry data
plane shapes and now ship wire-format encoders. v1.2.0 delivered:

- `src/proto.cyr` — minimal protobuf wire helpers (varint, tag,
  length-delimited string/bytes, fixed64, nested message). Not a
  general-purpose proto library; just what OTLP types need.
- `Span_to_otlp_proto(ptr, sb)` covering scalar fields (trace_id,
  span_id, parent_span_id, name, kind, start/end times, status,
  dropped-counts).

**Deferred to v1.2.2:**
- `LogRecord_to_otlp_proto` + `MetricDataPoint_to_otlp_proto`.
- Repeated nested-message fields on Span (attributes, events, links)
  — require nested KeyValue/Event/Link encoders. Pin when a consumer
  surfaces the need.

### Cross-consumer build sweep automation — **re-pinned v1.2.0 → v1.2.1**

✨ **Feature** — A CI workflow (or downstream-triggered job) that, for each of
the 11 consumers in `state.md`, clones the consumer repo at its main HEAD,
swaps `cyrius.cyml`'s agnostik dep to the in-flight commit, and runs the
consumer's `cyrius build` + `cyrius test`. Reports per-consumer green/red.
Catches accessor-ABI breaks, struct-layout drift, and serde-shape changes
before they propagate. Pinned to v1.2.1 (originally v1.2.0; pushed when OTLP
took the v1.2.0 slot per user direction). Pairs naturally with the v1.1.0
sub-byte-widths work but the infrastructure cost is high enough to be its
own slot.

---

## v2.0.0 — Breaking changes (next major)

The two items here are the only breaking changes on the horizon. Pinning to a
single major release lets every consumer absorb the migration cost in one cycle
rather than chasing point-version churn.

### `_json_int` Result return signature

🔧 **Optimization (breaking)** — `_json_int(src: Str, key: Str)` currently
returns `i64` and conflates "missing key" with "literal 0". F-003 hardened the
overflow path but left the missing-key ambiguity. Switch the return to
`Result<i64, Err>` so consumers can distinguish missing from zero. Every
caller updates from `var n = _json_int(s, k);` to `var n = _json_int(s, k)?;`
or pattern-match. Audit uses ride along.

### `#derive(accessors)` migration with prefix rename

🔧 **Optimization (breaking)** — Cyrius's `#derive(accessors)` generates
`<Struct>_<field>(s)` getters/setters; agnostik's convention is
`<prefix>_<field>(s)` (e.g. `amsg_*`, `aentry_*`, `secctx_*`). Today's ~470
hand-written single-line accessors collapse to derive markers if we either
(a) rename to match derive's default shape (consumer-visible break) or
(b) wait for upstream to ship derive-with-prefix support. v2.0.0 absorbs the
rename cost; it's the only consumer-visible API churn in the slot.

---

## Working agreement

- **Default shape**: small fast-follows in v1.0.x; bundled minors at v1.1.x;
  breaking changes batched at majors. Each minor cut runs the security
  audit pass per CLAUDE.md.
- **Adding new items**: open an ADR draft in `docs/proposals/` (to be created
  alongside `docs/adr/` in v1.0.4); cite the trigger and the slot pin. New
  items without a slot don't go on this roadmap — they go in proposals.
- **Removing items**: when a slot's work ships, move the entry to
  `## Shipped` (terse) and link to the CHANGELOG entry. When an item is
  abandoned, delete it from the roadmap and record the rationale in an ADR.
