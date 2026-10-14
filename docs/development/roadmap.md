# Agnostik Roadmap

## Status

**v1.6.5** is the most recent stable release, on Cyrius `6.6.20`. It has 12
modules plus `src/proto.cyr` (OTLP wire helpers), 923 public fns, and 1,536
test assertions across 18 `.tcyr` files, passing on x86_64-linux and on aarch64
under `qemu-aarch64`. There are 25 benchmarks and no external dependencies.
Five repos declare `[deps.agnostik]`: aegis, anuenue, ark, kybernet and mela.

Live numbers are in [`state.md`](state.md). Release history is in
[`../../CHANGELOG.md`](../../CHANGELOG.md). The most recent audit is
[`../audit/2026-10-14-audit.md`](../audit/2026-10-14-audit.md).

Every item below is pinned to a release. Shipped work leaves this file for the
CHANGELOG: if work is worth doing, it has a slot, and once it ships it is no
longer here.

| Slot | Kind | Theme |
|---|---|---|
| [v1.7.0](#v170--consumer-safety-net-minor) | minor | Cross-consumer sweep, private-helper namespacing, `ClassificationResult` setters |
| [v1.7.1](#v171--ci-and-release-hardening-patch) | patch | Raw-syscall gate, aarch64 tests in CI, toolchain checksum, orphan `keccak.cyr` |
| [v1.7.2 – v1.7.3](#documentation-debt-burn-down-v172-onward) | patch | Documentation debt, small modules |
| [v1.8.0](#v180--cyrius-679-pin-and-enum-nameparse-completion-minor) | minor | Cyrius pin `6.6.20` → `6.7.9`; `*_name` / `*_parse` for the 26 enums without them |
| [v1.8.1 – v1.8.2](#documentation-debt-burn-down-v172-onward) | patch | Documentation debt, mid-size modules |
| [v1.9.0](#v190--otlp-wire-format-completion-minor) | minor | OTLP `LogRecord`, `MetricDataPoint`, `Span` attributes / events / links |
| [v1.9.1 – v1.9.3](#documentation-debt-burn-down-v172-onward) | patch | Documentation debt, large modules, ending at 0 |
| [v2.0.0](#v200--breaking-changes-next-major) | major | Accessor rename, dead-helper removal (`_stik_json_int` Result may move earlier) |

---

## v1.7.x — Consumer safety net and hardening

### v1.7.0 — Consumer safety net (minor)

All three items are done under `[Unreleased]` (see the CHANGELOG). The slot
leaves this file at the 1.7.0 cut.

1. ✅ **Cross-consumer build sweep**: `scripts/consumer-sweep.sh`, wired into
   CLAUDE.md's closeout step 8. It discovers every sibling repo that declares
   `[deps.agnostik]` (five today: aegis, anuenue, ark, kybernet, mela). It
   builds and tests each one twice on its own pin, once at the agnostik tag it
   pins and once resolving this working tree through a `path` dep, and it
   verifies that the second run really vendored the in-flight code. A CI job
   can follow if the consumer repos become cloneable from the runner.
2. ✅ **Private-helper namespacing**: all 21 `_`-prefixed helpers in shipped
   modules are now `_stik_*`, and a CI step rejects an unprefixed one. The
   sweep found `_hex_nibble` colliding with sigil in four of five consumers. A
   wider scan found `_json_int` / `_json_str` colliding, at a different arity,
   with ai-hwaccel and itihas, and `_hex_byte` with bote.
3. ✅ **`ClassificationResult` setters**: `cresult_set_level`,
   `cresult_set_auto_level` and `cresult_set_confidence` validate and return
   a `Result`. Confidence is parts per thousand (0..1000), and
   `cresult_has_confidence` tells a set 0 from an unset score.

### v1.7.1 — CI and release hardening (patch)

- **Raw-syscall gate** ([ADR-003](../adr/003-roadmap-tracks-agnostik-work-only.md)).
  `src/error.cyr`'s four stderr writes are the last raw `syscall(1, 2, …)` sites
  in `src/`. They are correct on every target today (1 is `write` on
  x86_64-linux and agnos, and it is routed on aarch64 and Windows), but the
  compiler diagnoses raw numbers for aarch64 and Windows, not agnos. Move them
  to `sys_write(2, …)`,
  then make the CI security scan fail on any `syscall(` in `src/`. F-022 and
  F-023 were both raw numbers that meant something else on another target.
- **Run the aarch64 suite in CI.** CI cross-builds aarch64 but does not run it.
  The full suite passes locally under `qemu-aarch64`. Install `qemu-user` on
  the runner and run `cyrius tests --aarch64`.
- **Verify the toolchain download.** CI and release install Cyrius from the
  release tarball without checking it. Check it against the published
  `.sha256` sidecar before unpacking. cyrius 6.6.20's CYRIUS-2026-0035 hardened
  its own installers against stripped signatures, and this job bypasses them.
- **Remove the orphan `lib/keccak.cyr`.** It is git-tracked only because the
  ecosystem template's `!lib/k*.cyr` ignore exception matched it. No module
  includes it, `[deps] stdlib` does not declare it, and `cyrius deps --verify`
  now has to vouch for it. Delete it and the exception, relock, and update
  CLAUDE.md's `.gitignore` note.

### Documentation-debt burn-down (v1.7.2 onward)

`scripts/doc-debt.sh` has gated CI since v1.6.3. A new undocumented fn fails,
and so does a grandfathered one that was documented without recording it. The
849 fns in `docs/undocumented.baseline` are paid down one or more modules per
patch, smallest first, so each release's review stays small:

| Slot | Modules (undocumented fns) | Running total |
|---|---|---|
| v1.7.2 | `error` (18), `secrets` (21), `config` (22) | 849 → 788 |
| v1.7.3 | `main` (23), `validation` (26), `types` (40) | 788 → 699 |
| v1.8.1 | `audit` (44), `hardware` (55) | 699 → 600 |
| v1.8.2 | `telemetry` (126) | 600 → 474 |
| v1.9.1 | `llm` (148) | 474 → 326 |
| v1.9.2 | `security` (152) | 326 → 174 |
| v1.9.3 | `agent` (174) | 174 → 0 |

At 0, the baseline file goes and the gate becomes "no undocumented fn in
`src/`". `cyrius audit` will still exit non-zero on its docs phase, because
that phase counts the test and bench files too. Whether to document test
helpers or narrow the audit's scope is decided at v1.9.3.

---

## v1.8.0 — Cyrius 6.7.9 pin and enum name/parse completion (minor)

### Cyrius pin 6.6.20 → 6.7.9

🔧 The toolchain refresh rides this minor rather than a patch of its own. It is
the first move onto the 6.7.x line. Same checklist as the 1.6.5 refresh:

- Read the cyrius 6.7.0 – 6.7.9 CHANGELOG entries for anything that names
  agnostik or changes `lib sync`, `deps`, `distlib`, `api-surface`, `fmt`,
  `lint` or `#derive`.
- Re-vendor with `cyrius lib sync` + `cyrius deps`, commit the rewritten
  `cyrius.lock`, and regenerate the api-surface snapshot and the dist bundle
  if the toolchain changed their shape.
- Review the security advisories in range against what agnostik reaches, and
  record the pass in `docs/audit/`.
- Replay every `ci.yml` step from the published 6.7.9 tarball, run the suite
  on aarch64 and build agnos and Windows, and run the v1.7.0 consumer sweep
  against the new bundle.
- Bench gate plus an interleaved 6.6.20 / 6.7.9 A/B on identical source.

### Enum name/parse completion

✨ CLAUDE.md requires every public enum to have `*_name()` and `*_parse(s)`.
v1.4.0 (F-018) covered the 31 enums that had a `_name`, and v1.5.0 added
`LinuxCapability`. **26 enums still have neither**:

- **agent**: `RestartPolicy`, `ProbeType`, `StopReason`, `CircuitBreakerState`,
  `LabelOperator`
- **error**: `AgnostikErrorCode`
- **llm**: `ContentBlockType`, `CacheControl`, `ToolChoice`, `ResponseFormat`,
  `SafetyCategory`, `SafetyProbability`, `StreamEventType`, `BatchStatus`
- **security**: `ConditionOperator`, `SeccompMode`, `SystemFeature`,
  `LandlockFsAccess`, `LandlockNetAccess`, `LandlockScope`, `DeviceType`,
  `RlimitType`
- **telemetry**: `EventType`, `MetricKind`, `AggregationTemporality`,
  `MetricValueType`

Most are ordinal state sets, where a name pair is plainly useful in config and
logs. The three `Landlock*` enums are bit sets, so a name names one bit, not a
value. They get per-bit names, or an ADR that records "no string form" and
amends the CLAUDE.md rule. Same rules as F-018: exact match, no case-folding,
an unknown string is `Err`. `tests/tcyr/test_v140_enum_parse.tcyr`'s
exhaustive `parse(name(v)) == v` check extends to every new member.

## v1.9.0 — OTLP wire-format completion (minor)

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
encoder. If a consumer needs this before v1.9.0, it can move ahead of v1.8.0.
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
  breaking changes batched at majors. Public API additions ride minors. Each
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
