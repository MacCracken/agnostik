# Agnostik Roadmap

## Status

**v1.8.0** is the most recent stable release, on Cyrius `6.7.9`. It has 12
modules plus `src/proto.cyr` (OTLP wire helpers), 977 public fns, and 2,001
test assertions across 20 `.tcyr` files, passing on x86_64-linux and on aarch64
under `qemu-aarch64`. There are 25 benchmarks and no external dependencies.
Five repos declare `[deps.agnostik]` (aegis, anuenue, ark, kybernet, mela), and
`scripts/consumer-sweep.sh` builds and tests all of them against the working
tree before each release. CI runs the suite on x86_64 and on aarch64 (under
`qemu-user`), on a signature-verified toolchain.

Live numbers are in [`state.md`](state.md). Release history is in
[`../../CHANGELOG.md`](../../CHANGELOG.md). The most recent audit is
[`../audit/2026-10-14-audit.md`](../audit/2026-10-14-audit.md) (the 1.6.5, v1.7.0, v1.7.1 and v1.8.0 passes).

Every item below is pinned to a release. Shipped work leaves this file for the
CHANGELOG: if work is worth doing, it has a slot, and once it ships it is no
longer here.

| Slot | Kind | Theme |
|---|---|---|
| [v1.9.0](#v190--restartpolicy-nameparse-and-documentation-debt-minor) | minor | `RestartPolicy` name/parse (after kybernet takes argonaut 1.15.4); documentation debt 849 → 0 |
| [v1.10.0](#v1100--otlp-wire-format-completion-minor) | minor | OTLP `LogRecord`, `MetricDataPoint`, `Span` attributes / events / links |
| [v2.0.0](#v200--breaking-changes-next-major) | major | Accessor rename, dead-helper removal (`_stik_json_int` Result may move earlier) |

---

## v1.9.0 — RestartPolicy name/parse and documentation debt (minor)

### `RestartPolicy` name/parse

v1.8.0 added `*_name` / `*_parse` to 25 of the last 26 enums. `RestartPolicy` was held back:
argonaut ≤ 1.15.3 has its own `enum RestartPolicy`, with the same constant names but other values,
and its own one-argument `restart_policy_parse`. kybernet compiles both libraries, so agnostik's
pair would have been silently replaced there (F-025's class), and the consumer sweep flagged it.
The order agreed on 2026-10-14:
1. ✅ **argonaut 1.15.4** (cut 2026-10-14) makes `RestartPolicy` and `SeccompAction`
   value-identical to agnostik's, and renames its parser `argonaut_restart_policy_parse`.
2. ⏳ **kybernet** bumps `[deps.argonaut]` to 1.15.4 and renames its one call
   (`src/lib/svc_config.cyr:530`) in the same change. Left unrenamed, the call fails the build
   against agnostik ≤ 1.8.x, but once this pair lands it compiles and binds to agnostik's
   PascalCase parser, so kybernet's config tests fail.
3. **agnostik** adds `restart_policy_name` / `_parse` (`"Never"`, `"Always"`, `"OnFailure"`,
   the Kubernetes spelling), and the sweep must show kybernet green with no duplicate.

An end-to-end scratch build of kybernet against argonaut 1.15.4 and this pair passed 876 of 876
with no duplicate warning.

### Documentation debt: 849 → 0

Documentation does not get a release of its own; it rides this minor (it moved here from
v1.8.0, which shipped the pin move and the enum work).
`scripts/doc-debt.sh` has gated CI since v1.6.3. A new undocumented fn fails,
and so does a grandfathered one that was documented without recording it. The
849 fns in `docs/undocumented.baseline` are documented module by module,
smallest first, recording each module with `scripts/doc-debt.sh update`:

| Order | Modules (undocumented fns) | Running total |
|---|---|---|
| 1 | `error` (18), `secrets` (21), `config` (22) | 849 → 788 |
| 2 | `main` (23), `validation` (26), `types` (40) | 788 → 699 |
| 3 | `audit` (44), `hardware` (55) | 699 → 600 |
| 4 | `telemetry` (126) | 600 → 474 |
| 5 | `llm` (148) | 474 → 326 |
| 6 | `security` (152) | 326 → 174 |
| 7 | `agent` (174) | 174 → 0 |

At 0, the baseline file goes and the gate becomes "no undocumented fn in
`src/`". `cyrius audit` will still exit non-zero on its docs phase, because
that phase counts the test and bench files too. Whether to document test
helpers or narrow the audit's scope is decided at the end of this slot.

## v1.10.0 — OTLP wire-format completion (minor)

✨ This extends the v1.2.0 `Span_to_otlp_proto` foundation to the rest of the
OpenTelemetry data plane, using the existing `src/proto.cyr` primitives:

- **`LogRecord_to_otlp_proto`** and **`MetricDataPoint_to_otlp_proto`**, the
  same shape as the Span encoder.
- **Span's repeated nested fields**, skipped in v1.2.0: field 9 `attributes`
  (repeated `KeyValue`), field 11 `events` (repeated `Event`) and field 13
  `links` (repeated `Link`). These need nested encoders for `KeyValue` (a string
  key plus the `AnyValue` union), `Event` and `Link`. The `KeyValue` /
  `AnyValue` cluster is about half the slot.

Golden-byte tests against reference protobuf encodings, plus a fuzz target per
encoder. Map enum values at the encoder, as `_stik_span_kind_to_otlp` does for
`SpanKind`: `AggregationTemporality`'s values are agnostik ordinals
(CUMULATIVE = 0, DELTA = 1), while OTLP has DELTA = 1 and CUMULATIVE = 2. If a consumer needs this sooner, it can move ahead of v1.9.0.
No consumer has asked yet.

---

## v2.0.0 — Breaking changes (next major)

Breaking changes are batched here so every consumer absorbs migration once.

### `_stik_json_int` Result return

🔧 `_stik_json_int(src: Str, key: Str)` (`_json_int` before v1.7.0) returns
`i64` and conflates "missing key" with a literal `0`. F-003 hardened the
overflow path but left that ambiguity. Return a value-form `Result` instead, so
callers can tell missing from zero. Every caller moves to
`var n_t, n = _stik_json_int(s, k);` or `_stik_json_int(s, k)?`.

Note from v1.7.0: the helper is private. It is not in
`docs/api-surface.snapshot`, and no consumer calls it (checked across all five
`[deps.agnostik]` repos for the v1.7.0 rename). Its callers are all inside
agnostik, so this change no longer needs a major and could move to any minor.

### `#derive(accessors)` migration with prefix rename

🔧 Cyrius's `#derive(accessors)` generates `<Struct>_<field>` getters and
setters. agnostik's convention is `<prefix>_<field>` (`amsg_*`, `aentry_*`,
`secctx_*`). About 470 hand-written one-line accessors could become derive
markers, either by renaming to the derive shape (a consumer-visible break) or
by upstream derive-with-prefix support. v2.0.0 absorbs the rename.

### Remove dead and vestigial public helpers

🔧 The v1.3.0 review found public helpers with no consumer. Removing them is
breaking, and was gated on the cross-consumer sweep proving that no consumer
uses them. v1.7.0 delivers the sweep; this slot acts on its census:

- `security.cyr`: `seccomp_errno` / `seccomp_trace`, the `SeccompArg`
  cluster, `id_mapping_*`, `network_policy_*`
- `llm.cyr`: `stream_usage`
- `agent.cyr`: `AgentInfo_from_json`. Its round-trip was fixed in v1.6.3 and
  it now has a test, so it is a keep-or-remove call rather than dead weight.

Context: [`../audit/2026-06-01-audit.md`](../audit/2026-06-01-audit.md)
§Deferred.

---

## Working agreement

- **Default shape**: small fast-follows in patch slots, bundled minors, and
  breaking changes batched at majors. Documentation-only work never gets a
  release of its own; it rides the next minor. Public API additions ride minors. Each
  minor cut runs the security audit pass per CLAUDE.md.
- **Adding items**: draft a proposal under `docs/proposals/`, citing the
  trigger and the slot. An item without a slot stays a proposal until it earns
  one.
- **Removing items**: when a slot's work ships, its entry is deleted, because
  the CHANGELOG owns shipped history. When an item is abandoned, an ADR records
  why before the entry is deleted.
- **Scope**: the roadmap carries work that changes this repository. Toolchain
  defects get an agnostik-side mitigation here, and a record under
  `docs/development/issues/` when one is needed, but not "to file upstream"
  items ([ADR-003](../adr/003-roadmap-tracks-agnostik-work-only.md)).
