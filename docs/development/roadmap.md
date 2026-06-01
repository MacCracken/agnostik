# Agnostik Roadmap

## Status

**v1.3.0** — most recent stable. 12 modules + `src/proto.cyr` (OTLP wire
helpers), 858 test assertions across 15 `.tcyr` files (incl. byte-exact
serde golden + 8-parser fuzz harness + OTLP coverage + slice-safety
regression), 25 benchmarks, zero external dependencies, Cyrius `6.0.26`.
v1.3.0 was a toolchain refresh + refactoring/optimization closeout
(proto OTLP-encode memcpy, audit genesis-hash caching, hex-decode
consolidation, F-013 buffer-safety fix) with no public API or wire
change. See [`state.md`](state.md) for the live snapshot,
[`../audit/2026-06-01-audit.md`](../audit/2026-06-01-audit.md) for the
most recent audit, and [`../../CHANGELOG.md`](../../CHANGELOG.md) for full
release history.

Every item below is pinned to a specific release. Shipped work is recorded
in `CHANGELOG.md` and not duplicated here — the principle: if work is worth
doing, it has a slot; if it has shipped, it isn't on the roadmap any more.

---

## v1.2.x — Ecosystem expansion

### v1.2.4 — Cross-consumer build sweep automation

✨ **Feature** — A CI workflow (or downstream-triggered job) that, for each
of the 11 consumers in `state.md`, clones the consumer repo at its main
HEAD, swaps `cyrius.cyml`'s agnostik dep to the in-flight commit, and runs
the consumer's `cyrius build` + `cyrius test`. Reports per-consumer
green/red. Catches accessor-ABI breaks, struct-layout drift, and serde-
shape changes before they propagate.

Originally bundled with v1.2.0 OTLP work; pushed because OTLP took the
slot. Re-pinned across v1.2.1 (toolchain refresh `5.10.20 → 5.10.34`),
v1.2.2 (toolchain refresh `5.10.34 → 5.10.44`), and v1.2.3 (major
toolchain refresh `5.10.44 → 6.0.14`). Infrastructure cost
(orchestrating 11 repos, caching toolchain, surfacing per-consumer
output) is high enough to be its own slot anyway.

### v1.2.5 — OTLP wire-format completion

✨ **Feature** — extends the v1.2.0 `Span_to_otlp_proto` foundation to the
remaining OpenTelemetry data-plane shapes:

- **`LogRecord_to_otlp_proto`** + **`MetricDataPoint_to_otlp_proto`** — same
  shape as the v1.2.0 Span encoder; uses the existing `src/proto.cyr`
  primitives.
- **Span repeated nested-message fields** (skipped in v1.2.0):
  - field 9: `attributes` (repeated `KeyValue`)
  - field 11: `events` (repeated `Event`)
  - field 13: `links` (repeated `Link`)

  Requires nested encoders for `KeyValue` (string-typed key + `AnyValue`
  union), `Event` (timestamp + name + attributes), and `Link` (trace_id +
  span_id + attributes). The `KeyValue` / `AnyValue` cluster is the
  largest sub-suite — about half the slot's effort.

  Trigger: a consumer (likely `stiva`) surfaces the need, OR v1.2.4's
  cross-consumer sweep flags consumers that already work around the gap.

---

## Backlog — v1.3.0 review deferrals (unpinned)

The v1.3.0 refactoring/optimization review surfaced cleanups that were
**not** applied because they touch public API surface (removal/rename is
breaking → needs a major) or are low-value layout changes. Recorded so
they aren't re-discovered each cycle. Full context in
[`../audit/2026-06-01-audit.md`](../audit/2026-06-01-audit.md) §Deferred.

- **Dead/vestigial public helpers** — `seccomp_errno`/`seccomp_trace`,
  the `SeccompArg` cluster, `id_mapping_*`, `network_policy_*`
  (`security.cyr`); `stream_usage` (`llm.cyr`); `AgentInfo_from_json`
  (`agent.cyr`). The last one also **cannot round-trip its own
  `_to_json`** (emits `agent_type`/`status` name strings, reads
  `agent_type_id`/`status_id` ints) and has no test — fold its fix or
  removal into the v2.0.0 break, or fix-and-test it sooner if a consumer
  needs it. All are in `docs/api-surface.snapshot`; gate any
  removal/rename on the v1.2.4 cross-consumer sweep confirming no
  external dependency.
- **Setter-less `mcap_supports_*` getters** (`llm.cyr`) — seven flag
  getters with no matching setter (can only read 0). Decide: add setters
  (complete the API) or drop (decorative). Additive (setters) is
  non-breaking; pin to a minor when a consumer needs to *set* them.
- **`secret_metadata_new` over-alloc** (`secrets.cyr`) — 72 B / 9 slots,
  3 unreachable (offsets 24/56/64). Shrink to 56 B is a layout change;
  no in-repo raw writers, but external consumers may. Confirm via the
  cross-consumer sweep before trimming.

These are not security exposures — F-013 (the one real finding) shipped
in v1.3.0. Trigger for action: the v1.2.4 cross-consumer sweep landing
(gives the ABI-safety signal removal needs), or a consumer surfacing a
concrete need.

---

## v2.0.0 — Breaking changes (next major)

The two items here are the only breaking changes on the horizon. Batching
them at a major release lets every consumer absorb migration cost in one
cycle rather than chasing point-version churn.

### `_json_int` Result return signature

🔧 **Optimization (breaking)** — `_json_int(src: Str, key: Str)` currently
returns `i64` and conflates "missing key" with "literal 0". F-003 hardened
the overflow path but left the missing-key ambiguity. Switch the return to
`Result<i64, Err>` so consumers can distinguish missing from zero. Every
caller updates from `var n = _json_int(s, k);` to `var n = _json_int(s, k)?;`
or pattern-match.

### `#derive(accessors)` migration with prefix rename

🔧 **Optimization (breaking)** — Cyrius's `#derive(accessors)` generates
`<Struct>_<field>(s)` getters/setters; agnostik's convention is
`<prefix>_<field>(s)` (e.g. `amsg_*`, `aentry_*`, `secctx_*`). Today's
~470 hand-written single-line accessors collapse to derive markers if we
either (a) rename to match derive's default shape (consumer-visible break)
or (b) wait for upstream to ship derive-with-prefix support. v2.0.0
absorbs the rename cost.

---

## Working agreement

- **Default shape**: small fast-follows in patch slots; bundled minors;
  breaking changes batched at majors. Each minor cut runs the security
  audit pass per CLAUDE.md (cadence established at v1.0.8 and re-verified
  at every minor since).
- **Adding new items**: draft a proposal under `docs/proposals/`, cite the
  trigger and the slot pin. New items without a slot don't go on this
  roadmap — they go in proposals until they earn a slot.
- **Removing items**: when a slot's work ships, the roadmap entry is
  deleted (CHANGELOG owns shipped-work history). When an item is
  abandoned, the rationale is recorded in an ADR before deletion.
