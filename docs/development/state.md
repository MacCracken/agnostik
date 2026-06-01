# Agnostik — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures (durable);
> this file is **state** (volatile). Bumped via `scripts/version-bump.sh`.

## Version

**1.3.0** — Toolchain refresh + refactoring/optimization closeout on
top of 1.2.3. Cyrius pin `6.0.14` → `6.0.26`. Four internal
improvements from the review (no public API or wire-format change):
(1) `src/proto.cyr` `_proto_string`/`_proto_bytes`/`_proto_message`
swapped per-byte `str_builder_putc` copy loops for a single
`str_builder_add` (grow+memcpy) — OTLP encode hot path; (2)
`src/audit.cyr` caches the 64-char `GENESIS_HASH` Str once
(`_genesis_hash_cached`) instead of re-wrapping it per `audit_entry_new`
/ `integrity_is_genesis`; (3) byte-identical `_hex_nibble` /
`_json_hex_digit` merged into one `_hex_nibble` with the five
open-coded hex ladders (agent_id/trace_id/span_id/`\uXXXX`) routed
through it; (4) **F-013 (LOW)** buffer-safety fix — `version_from_str`
replaced an unbounded `strchr` separator scan (over-read past `slen` on
non-NUL-terminated `Str` slices) with a bounded forward scan. New
`tests/tcyr/test_v130_slice_safety.tcyr` (+7) exercises the bound where
the always-NUL-terminating fuzz harness could not. Audit in
[`docs/audit/2026-06-01-audit.md`](../audit/2026-06-01-audit.md).
Bench-regression gate clean (25/25, 0 regressions vs the v1.2.0
baseline); 6.x `_from_json` codegen wins held. 858/858 tests pass (was
851); lint/fmt/vet clean; `dist/agnostik.cyr` re-bundled. DCE binary
`313,344 B` → `311,264 B` (−2 KB) from the removed copy loops + ladders.
Public API unchanged at 871 fns. Deferred low-priority cleanups
(dead-code clusters, setter-less `mcap_supports_*` getters,
`secret_metadata_new` over-alloc) logged in the audit + roadmap backlog.

**1.2.3** — Major-toolchain-refresh patch on top of 1.2.2. Cyrius pin
`5.10.44` → `6.0.14` (first 6.x pin). No agnostik-side source
changes. The one project-visible change is the stdlib workflow:
under 6.0.x, `cyrius lib sync` (not `cyrius deps`) copies the
version-pinned stdlib snapshot into `./lib/`; `cyrius deps` now does
git deps only and presence-checks the `[deps] stdlib` array.
`build`/`test`/`bench` resolve stdlib from the snapshot directly and
need no `./lib/`. CI (`ci.yml`/`release.yml`) and the CLAUDE.md Quick
Start gained a `cyrius lib sync` step before `cyrius deps`; the
`[deps] stdlib` list itself is unchanged (consumers still rely on it
per `cyrius distlib`). Bench-regression gate clean (25/25 checked, 0
regressions vs the v1.2.0 history.csv baseline); v1.2.1/1.2.2 hot-path
wins held across the major-version boundary; `accelerator_device_full`
drifted further to 133ns. DCE binary `~305 KB` → `~306 KB` (313,344
bytes; +1 KB nominal drift). 851/851 tests pass; api-surface unchanged
at 871 public fns; lint/fmt/vet clean; `dist/agnostik.cyr` re-bundled
for the version banner only. Cross-consumer build sweep re-pinned
v1.2.3 → v1.2.4; OTLP completion re-pinned v1.2.4 → v1.2.5.

**1.2.2** — Toolchain-refresh patch on top of 1.2.1. Cyrius pin
`5.10.34` → `5.10.44` (10 upstream slots). No agnostik-side source
changes; codegen-only patch. Bench-regression gate clean (25/25
checked, 0 regressions vs the v1.2.0 history.csv baseline); v1.2.1
hot-path wins held through the new pin. One additional notable
improvement: `accelerator_device_full` 177ns → 155ns (−12.4%) —
the bench that needed a `[bench-regression-ack]` at the v1.2.1 cut
due to CI jitter is now cleanly clear of the noise band. DCE binary
`~304 KB` → `~305 KB` (+1 KB nominal codegen drift). 851/851 tests
pass; api-surface unchanged at 871 public fns; `dist/agnostik.cyr`
re-bundled for the version banner only. Cross-consumer build sweep
re-pinned v1.2.2 → v1.2.3; OTLP completion re-pinned v1.2.3 → v1.2.4.

**1.2.1** — Toolchain-refresh patch on top of 1.2.0. Cyrius pin
`5.10.20` → `5.10.34` (14 upstream slots). No agnostik-side source
changes; codegen wins came in via the new pin. Bench-regression gate
clean (25/25 checked, 0 regressions); top hot-path improvements:
`resource_limits_from_json` 3000ns → 476ns (−84.1%),
`token_usage_from_json` 2000ns → 422ns (−78.9%),
`version_to_str` 191ns → 166ns (−13.1%). DCE binary `~311 KB` →
`~304 KB` (−7 KB nominal codegen shrink). 851/851 tests pass; api-
surface unchanged at 871 public fns; `dist/agnostik.cyr` re-bundled
for the version banner only. Cross-consumer build sweep re-pinned
v1.2.1 → v1.2.2; OTLP completion re-pinned v1.2.2 → v1.2.3.

**1.2.0** — First v1.2.x minor: OTLP wire-format primitives. New
`src/proto.cyr` ships protobuf wire helpers (varint, tag,
length-delimited string/bytes, fixed64, nested message); new
`Span_to_otlp_proto(ptr, sb)` in `src/telemetry.cyr` encodes
agnostik's `Span` to `opentelemetry.proto.trace.v1.Span` on the
wire across all scalar fields (trace_id, span_id, parent_span_id,
name, kind, start/end times, status, dropped-counts). 66 byte-exact
test assertions in `tests/tcyr/test_v120_otlp.tcyr`. Repeated nested-
message fields (attributes/events/links) + LogRecord/MetricDataPoint
encoders deferred to v1.2.2 when a consumer surfaces the pin.
Cross-consumer build sweep re-pinned v1.2.0 → v1.2.1.

**1.1.2** — Fuzz harness on top of 1.1.1 (per the v1.1.2 roadmap
pin). 8 parser entry points exercised with 200 deterministic
xorshift64-driven inputs each plus all F-002..F-010 audit-finding
regression seeds. Survival contract: parsers must accept any byte
sequence without crashing or OOB access. ~1680 calls per run;
milliseconds wall-clock. File: `tests/tcyr/test_v112_fuzz.tcyr`
(~290 LoC). 785/785 tests pass (+8 survival counters); no public
API surface change.

**1.1.1** — Sub-byte field widths on top of 1.1.0 (per the v1.1.1
roadmap pin). `InjectionScores` (5 fields i64 → i8: 40 B → 5 B) and
`AcceleratorFlags` (9 fields i64 → i8: 72 B → 9 B) shrunk 87.5%
per-instance with no wire-format change (cyrius derive emits the
same `{"k":N,"k":N}` shape regardless of width). 5 new `iscore_set_*`
setters added since `InjectionScores` lacked them pre-1.1.1.
**Breaking:** direct `store64(is + N, v)` writes to `InjectionScores`
no longer safe — alloc shrank from 40 B to 5 B; callers must use
`iscore_set_*`. Filed cyrius bug at
[`docs/development/issues/cyrius-derive-comments-in-struct-body-2026-05-10.md`](../development/issues/cyrius-derive-comments-in-struct-body-2026-05-10.md)
— `#`-comments inside derive struct bodies corrupt cyrius 5.10.14's
codegen; workaround applied (comments above the directive). 777/777
tests pass (was 735; +42 from `test_v111_subbyte_widths.tcyr`).

**1.1.0** — Modernization minor: `#derive(Serialize)` revived for
7 of 9 trivial all-int structs (per ADR-002 superseding ADR-001 —
cyrius 5.10.14's derive can't replicate AgentInfo/TelemetryConfig's
custom shapes); 14 hand-written serde fns deleted (~280 LoC). The 2
custom-shape structs retain hand-written impls but adopt derive's
compact byte format for library uniformity. Public API change:
`<Struct>_from_json(s: Str)` removed for the 7 derive structs;
consumers use `<Struct>_from_json_str(str_data(j))` (cyrius-emitted
shape). Wire format change: JSON output is compact across all 9
structs (RFC-permissive; consumer parsers handle either form).
5 new `LlmProvider` variants (Together/Fireworks/Bedrock/Vertex/
Cohere) + 3 new `ModelCapabilities` flags (video_input/caching/
parallel_tool_calls). 735/735 tests pass (was 701; +34 from new
golden corpus).

**1.0.8** — Security audit pass + 1 INFO finding fixed. First audit
since 2026-04-26 (cumulative 1.0.1..1.0.7 diff scope). Findings in
[`docs/audit/2026-05-10-audit.md`](../audit/2026-05-10-audit.md):
**F-012 (INFO)** — `_fill_random` fatal-message stderr write
off-by-one (passed 67 bytes for a 68-byte literal); cosmetic, fixed
via `strlen()`-based length computation. F-001..F-011 re-verified
closed. v1.0.7's `\uXXXX` decoder verified clean across input
validation, buffer safety, syscall review, and pointer validation.
Established cadence: audit at every minor cut. No public API
changes; 701/701 tests pass.

**1.0.7** — Two additive features on top of 1.0.6. JSON `\uXXXX`
Unicode escape decoder lands in `_json_str`, closing the
F-002/F-004 follow-up note: BMP single + surrogate-pair paths +
U+FFFD fallback for malformed escapes; 3 new private helpers
(`_json_hex_digit`, `_json_parse_u4`, `_utf8_encode`). Three new
`PiiKind` variants — `PII_GENETIC`, `PII_BIOMETRIC_TEMPLATE`,
`PII_PRECISE_GEOLOCATION` — appended after `PII_CUSTOM` to
preserve wire-format tag values. Both additions purely additive;
no public API removals. Test count grew 653 → 701 (+48 from
`test_v107_unicode_pii.tcyr`).

**1.0.6** — Performance observability on top of 1.0.5. Adds a
**bench-regression CI gate** (`scripts/bench-regression.sh`) that
compares per-op averages against the most recent committed baseline
in `docs/benchmarks/history.csv` and fails on slowdown beyond the
threshold (50% ns-bracket, 80% us-bracket — tuned for cyrius's
whole-µs rounding + CI jitter). Intentional perf trade-offs ack'd
via `[bench-regression-ack]` in the HEAD commit message. The
**compile-time profile pass** ran (CYRIUS_PROF=1) and recorded
findings: lex dominates at 68%, all top phases upstream-bound; no
agnostik-side action — slot closes with `docs/development/compile-profile-2026-05-09.md`.
Three stabilization tail-fixes from the 1.0.5 line folded in: the
1.0.5 api-surface snapshot wasn't portable (stdlib platform peers +
locale-sensitive sort) and `bench-history.sh` was dropping
us-bracket rows via a too-narrow regex. No public API changes;
653/653 tests pass.

**1.0.5** — Test + API hygiene on top of 1.0.4. Adopts
`lib/test.cyr`'s `test_each` helper for the F-005 + F-010
audit-regression clusters (homogeneous accept/reject + whitespace
shapes); heterogeneous clusters stay as direct test fns. Adds an
**API surface snapshot gate** in CI — `cyrius_api_surface` diffs
the live public-fn surface (1317 fns at the 1.0.5 baseline) against
`docs/api-surface.snapshot` and fails on unexplained drift.
Intentional API bumps regenerate via `cyrius_api_surface --update`
and commit alongside. No public API changes; 653/653 tests pass.

**1.0.4** — Doc + ergonomic small wins on top of 1.0.3. Introduces
the `docs/adr/` convention (ADR-001 captures the v1.1.0 derive
revival decision: trigger conditions, golden-corpus verification
plan, F-002/F-003/F-008 byte-equivalence requirement); adopts
pointer-to-struct dot syntax (`s.data` / `s.len`) in 8 parsers
across `types.cyr` and `telemetry.cyr` — selective, not wholesale.
Also folds in two post-1.0.3 fixes that surfaced via the new CI
type-check gate: dropped over-aggressive `: Str` annotations on the
4 baggage/textmap pass-through helpers (these forward to opaque
hashmap slots), and corrected the CI filter pattern that was
missing stdlib self-flags. No public API changes; 653/653 tests
pass; DCE binary `274 KB` → `274 KB` (+48 B nominal codegen drift).

**1.0.3** — Toolchain refresh + CI hygiene on top of 1.0.2. Manifest
pin `5.10.3` → `5.10.14` (picks up the rest of the v5.10.x
type-system arc plus the cyrfmt + cyrlint char-literal brace fixes
that closed the 1.0.2 putc workaround); CI gains a
`CYRIUS_TYPE_CHECK=1` step that fails on agnostik-side annotation
drift; CI install steps rewired to the version-pinned lib layout
that 5.10.9+ requires. The 1.0.2 `'}'` → `125` putc workaround at 8
sites reverted to the readable char-literal form. Two upstream-
resolved issue files moved to a new `docs/development/issues/archive/`
subdirectory. No public API changes; all 653 assertions pass;
DCE binary `273 KB` → `274 KB`.

**1.0.2** — Cyrius 5.10.3 modernization on top of 1.0.1. Test
boilerplate dropped (cyrius auto-injects the `main()` caller and
lazy-inits the heap); `result` + `chrono` added to `[deps] stdlib`
and inlined `now_ns()` replaced with `chrono::clock_now_ns()`
(CLOCK_MONOTONIC) at 10 call sites; `?` operator adopted in
`tctx_from_traceparent`; `: Str` annotation pass across 12 source
files (~120 annotations) verified clean under `CYRIUS_TYPE_CHECK=1`;
single-char `str_builder_add_cstr` → `str_builder_putc` at 15 call
sites. No public API changes; all 653 assertions pass; DCE binary
grew `261 KB` → `273 KB` from chrono dependency surface DCE didn't
fully eliminate.

**1.0.1** — documentation cleanup + toolchain refresh on top of
1.0.0. Manifest pin moved from Cyrius `5.7.12` to `5.10.3`; stdlib
deps re-resolved via `cyrius deps`; bench banner stripped of its
hardcoded toolchain literal. Public API unchanged, all 653 assertions
across 9 test files pass; DCE binary grew `214 KB` → `261 KB` purely
from codegen differences.

**1.0.0** — first stable release. Toolchain refresh to Cyrius
5.7.12, manifest migration `cyrius.toml` → `cyrius.cyml`, P(-1)
scaffold hardening, security audit pass (11 findings closed,
F-006 resolved upstream in 5.7.7, 1 new upstream issue filed),
and layout aligned with vidya/yukti conventions. See
[`docs/audit/2026-04-26-audit.md`](../audit/2026-04-26-audit.md)
for security findings and [`CHANGELOG.md`](../../CHANGELOG.md)
for full release notes.

## Toolchain

- **Cyrius**: `6.0.26` (pinned in `cyrius.cyml [package].cyrius`)
- **Stdlib resolution (6.0.x)**: `cyrius lib sync` copies the
  version-pinned snapshot into `./lib/`; `cyrius deps` resolves git
  deps only and presence-checks the `[deps] stdlib` array. Run
  `lib sync` before `deps` on a fresh checkout. `build`/`test`/`bench`
  resolve stdlib from the snapshot directly (no `./lib/` required).
- **Compiler**: `cc5` — invoked via `cyrius {build,test,bench}`; raw
  `cat | cc5` is forbidden (manifest auto-resolves deps and prepends includes)
- **Locally installed vs released**: `cyrius --version` may report
  a newer dev build; the manifest always pins to the latest
  **released** version so CI and external contributors get a
  reproducible toolchain. Bump the pin only when a new release ships.
- **`cyrius audit`** is still broken on 6.0.x (missing `check.sh`
  — same upstream issue as 5.7.12 / 5.10.x, filed in
  [`docs/development/issues/cyrius-audit-missing-check-script-2026-04-26.md`](issues/cyrius-audit-missing-check-script-2026-04-26.md)).
  Workaround: run `cyrius self / test / fmt --check / lint` individually.

## Source layout

```
src/
  lib.cyr            — include orchestrator (consumed by main.cyr)
  main.cyr           — test harness entry
  error.cyr          — Result / Err / error kinds
  types.cyr          — version, UUID, timestamp, identifiers
  agent.cyr          — agent ID, capabilities, scheduling, rate limits
  security.cyr       — sandbox, capabilities, auth, policies
  telemetry.cyr      — spans, metrics, logs, exemplars, baggage
  audit.cyr          — entries, integrity, retention
  llm.cyr            — tools, sampling, streaming, content blocks
  secrets.cyr        — metadata, zeroize
  config.cyr         — profiles, fleet
  classification.cyr — classification results
  validation.cyr     — warnings, injection scores
  hardware.cyr       — devices, flags, summary
```

Tests at `tests/tcyr/agnostik.tcyr` + 4 coverage modules + serde
roundtrip + 2 audit regression files (`test_audit_2026_04_26` for
F-001..F-005, `test_audit_5712` for F-008..F-010). Benches at
`tests/bcyr/agnostik.bcyr`.

## Stats

> Updated by the closeout pass. Never inline these in CLAUDE.md.

| Metric                | Value     | Notes                              |
|-----------------------|-----------|------------------------------------|
| Source LOC (src/)     | ~3,180    | down from 7,121 LOC Rust; −2 KB binary at 1.3.0 from copy-loop/ladder removal |
| Module count          | 12        |                                    |
| Test files            | 15        | tests/tcyr/ (+test_v130_slice_safety) |
| Test assertions       | 858       | 0 failed; +7 F-013 slice-safety regression at v1.3.0 (was 851 through v1.2.3) |
| Benchmarks            | 25        | tests/bcyr/                        |
| Test binary (DCE)     | ~304 KB   | `build/agnostik` after `CYRIUS_DCE=1 cyrius build` (261→273 KB at 1.0.2; 274 KB at 1.0.3+; ~311 KB at 1.2.0 from chrono+proto surface; ~304 KB at 1.2.1; ~306 KB / 313,344 B at 1.2.3 across the 6.0.x boundary; 311,264 B at 1.3.0, −2 KB from proto memcpy + hex-ladder collapse) |
| Build warnings        | 0         |                                    |
| Lint warnings         | 0         | (28 UFCS false positives resolved upstream in cyrius 5.7.7) |
| Lib bundle (dist/)    | regenerated by `cyrius distlib` | tracked in CI sync check |

## Consumers

Every AGNOS component depends on agnostik for shared types:

- **daimon** — agent runtime
- **hoosh** — LLM grounding service
- **agnoshi** — shell
- **aegis** — security policy engine
- **argonaut** — agent orchestrator
- **sigil** — capability/auth issuer
- **ark** — packaging / distributable
- **kavach** — sandbox enforcement
- **stiva** — telemetry pipeline
- **nein** — refusal / safety layer
- **yukti** — device abstraction (telemetry types)

## Recent releases

See [`CHANGELOG.md`](../../CHANGELOG.md). Most recent stable: `1.3.0` (toolchain refresh Cyrius `6.0.14` → `6.0.26` + refactoring/optimization closeout — proto OTLP-encode memcpy, audit genesis-hash caching, hex-decode consolidation, F-013 buffer-safety fix; 858/858 tests, DCE binary −2 KB, no public API/wire change).

## Verification hosts

- Local: x86_64-linux (LTS kernel 6.18)
- CI: `ubuntu-latest` (GitHub Actions)
- Cross: aarch64 best-effort via `cc5_aarch64` when shipped in toolchain
