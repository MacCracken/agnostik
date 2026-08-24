# Agnostik — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures (durable);
> this file is **state** (volatile). Bumped via `scripts/version-bump.sh`.

## Version

> **In flight (unreleased, staged for `1.3.7`)** — P(-1) hardening sweep
> per CLAUDE.md §P(-1). Full report:
> [`../audit/2026-08-24-audit.md`](../audit/2026-08-24-audit.md). Six new
> findings; four repaired here (**F-014** `_fill_random`'s error sentinel
> neither exited its loop nor stayed in bounds — signed compare made
> `off = 0 - 1` retry at `buf - 1` with `n + 1` bytes, or hang;
> **F-015/F-016/F-017** W3C traceparent validation — all-zero
> trace-id/parent-id, unvalidated version field, uppercase hex;
> **F-019** missing `user_id_from_str` roundtrip test; **F-020**
> `secret_metadata_new` 72 B → 56 B, cross-consumer-confirmed safe). Two
> contract gaps quantified and pinned to **v1.4.0** because they need
> additive public API: **F-018** (58 enums, 54 `*_name()`, **zero** enum
> `*_parse()`) and **F-021** (`SecretMetadata.expires_at`/`.owner` are
> unsettable, so `smeta_expires_at()` returns 0 for every secret ever
> released). 886/886 tests; lint/fmt/vet clean; api-surface unchanged at
> 871 fns; bench gate 25 checked / 0 regressions. `VERSION` still reads
> `1.3.6` — the cut is the maintainer's call.

**1.3.6** — Toolchain-refresh patch on top of 1.3.5, plus the gate and
doc reconciliation the 1.3.5 cut skipped. Cyrius pin `6.5.27` → `6.5.35`.
No agnostik-side logic changes: public API (**871 fns**, matches
`docs/api-surface.snapshot`) and every wire format byte-for-byte unchanged,
so this is drop-in for all 11 consumers. `lib/` re-synced from the 6.5.35
snapshot: `cyrius lib sync` resolves the **15** declared `[deps].stdlib`
modules to **25 files** (15 + 10 platform peers); `lib/` holds **27**
because `atomic.cyr` (written by `cyrius deps`) and the vendored,
git-tracked `keccak.cyr` sit outside `lib sync`. All 27 byte-match the
snapshot. Stdlib **module set unchanged at 101** — no reorg this cycle,
`[deps].stdlib` untouched. Three vendored modules changed content:
**`bayan`** `215,481` → `641,083` B (431 → 799 fns) as 6.5.3x folds a full
PDF parse/encode subsystem into the bundle — **361 PDF fns**: 209 private
`_pdf*` internals across eleven families plus 152 public `bayan_pdf_*`
entry points, none of either present at 6.5.27; **`fmt`**
`11,428` → `12,844` B (upstream `fmt_float_buf` carry fix — not
user-visible here, agnostik formats no floats); **`syscalls_windows`**
`16,361` → `18,078` B. Build warnings **0** — the three `lib/bayan.cyr`
`_toml_parse_*` non-pointer-to-typed-pointer warnings present at the 1.3.5
pin are fixed upstream.

Two fixes landed, but **neither is attributable to 6.5.35** — both came
from running gates that the 1.3.5 cut skipped, and an earlier draft of this
entry credited them to the toolchain in error.

(1) **`cyrius audit` is usable, and already was on the previous pin.**
Bisecting the installed toolchains against this exact tree: the original
missing-`check.sh` breakage ended at **6.2.24** (6.2.23 still errors
`script not found: ~/.cyrius/bin/check.sh`; 6.2.24 runs its phases inline —
the 2026-07-13 note recording this as "resolved at 6.4.62" simply reflects
the next version agnostik re-tested), and the follow-on `tests`/`bench`
stdlib-preamble defect ended at **6.4.73** (6.4.72 → `10 passed, 5 failed`;
6.4.73 → `15 passed, 0 failed`). It was therefore already fixed in
**6.5.27**, the pin this release bumps *from* — running
`~/.cyrius/versions/6.5.27/bin/cyrius audit` here produces the same clean
sweep as 6.5.35. The
`cyrius-audit-missing-check-script-2026-04-26` issue nevertheless stays
**open**, narrowed to `cyrius self`: standalone it still false-fails with
the same two undefined symbols and `FAIL: cycc!=cycc` (verified on 6.5.27,
6.5.30 and 6.5.35, so not a 6.5.35 regression). `audit` does not cover it
and has not since **6.2.24**, when the self-host phase left its phase list
— `cyrius --help` read "full check: self-host, test, fmt, lint" from 6.0.1
through **6.2.10**, "local item suite" from 6.2.11, and "project sweep:
fmt/lint/docs/tests/bench" since 6.2.24. `cyrius audit` also exits 1 on its
`docs` phase — **853 undocumented public fns**, unchanged by this bump
(6.5.27 reports the identical count) and agnostik's own gap, now a roadmap
backlog item. Tooling note: `cyrius doc --check` takes a single file and
exits with *that file's* count (`src/main.cyr` → 23, `src/agent.cyr` → 177,
`src/lib.cyr` → 0); 853 is the sum across the 15 `src/*.cyr`. Read `audit`'s
per-phase verdicts, not its exit code.

(2) Running audit's bench phase surfaced **`tests/bcyr/agnostik.bcyr`
missing `include "src/proto.cyr"`** — the same defect v1.3.4 fixed in
`tests/tcyr/agnostik.tcyr` and missed in the bench harness. It includes
`src/telemetry.cyr` (whose `Span_to_otlp_proto` calls the `_proto_*`
helpers) but not `src/proto.cyr`, leaving 5 undefined-function references.
Benign — no bench calls `Span_to_otlp_proto`, so they were unreachable and
the harness linked and ran correctly — and **not** hidden by tooling: a
plain `cyrius bench` prints all five warnings on 6.4.62 and 6.5.27 as well
as 6.5.35. Latent since v1.2.0 and simply overlooked in bench output for
four releases. Include added mirroring `src/lib.cyr` order; bench-harness
warnings back to 0.

Bench gate on the fixed harness: **25 checked, 0 regressions**, **23 of 25
ops faster** (medians of 3: `inference_request_full` 684→378 −44.7%,
`sandbox_config_default` 51→30 −41.2%, `message_build_3turn` 592→360
−39.2%, `version_to_str` 164→111 −32.3%, `token_usage_update` 41→27
−34.1%, `accelerator_device_full` 154→110 −28.6%, `version_roundtrip`
360→267 −25.8%). The two above baseline — `audit_entry_full` +8.4%,
`trace_context_new` +8.0% — sit far inside the 80% us-bracket threshold and
compare against whole-µs-**truncated** 2026-06-20 baselines (pre-6.2.15
`_fmt_time` rendered µs as `major = ns / 1000`, integer division with no
fraction), so a `2,000` baseline encodes 2,000–2,999 ns and both figures
sit inside noise. Because 1.3.5 appended no baseline the comparison is
split: 20 of 25 benches baseline to the 2026-07-13 (v1.3.4-line) run,
spanning `6.4.62 → 6.5.27 → 6.5.35`, while the five that run dropped
(`accel_flags_to_json`, `audit_entry_full`, `resource_limits_to_json`,
`trace_context_new`, `traceparent_format`) fall back to the 2026-06-20 run
captured under **6.2.11** and span `6.2.11 → 6.5.35`. No row is a
single-pin-step delta. The 1.3.6 run is appended to
`history.csv` (25 rows — the first full append since the parser fix, vs 20
before).

858/858 tests pass; lint clean (0 warnings, 15 src files); fmt clean across
31 files; `cyrius vet` 24 deps / 0 untrusted / 0 missing; api-surface gate
matches at 871 fns; `dist/agnostik.cyr` re-bundled (banner-only diff,
121,132 B). Binary `413,512 B` → `629,032 B` (**+215,520 B, +52.1%**) —
entirely the NOPed `bayan` PDF subsystem; nothing new is reachable
(unreachable-fn accounting 1,503 fns / 291,041 B → 1,871 fns / 445,949 B).
**Note** `CYRIUS_DCE=1` and a plain build now emit the *identical* byte
count at both pins: DCE NOP-fills in place rather than
removing, so it no longer shrinks the artifact — it only guarantees dead
code cannot execute. Also reconciled from the skipped 1.3.5 gates:
`cyrius fmt --check` debt in `src/main.cyr` + `tests/tcyr/agnostik.tcyr` +
`tests/tcyr/test_v110_serde_golden.tcyr` canonicalized (whitespace-only, 16
lines; the 6.5.27 and 6.5.30 formatters reject the same three files, so the
debt predates this bump), and README / state.md / roadmap / doc-health
Status blocks moved off **1.3.4 / 6.4.62**.

**1.3.5** — Toolchain-refresh patch on top of 1.3.4 (backfilled here at
the 1.3.6 cut; the 1.3.5 release did not run a doc-sync pass). Cyrius pin
`6.4.62` → `6.5.27`, aligning agnostik with the rest of the AGNOS desktop
stack. No source changes; 15/15 test files green as shipped. **The release
skipped two mandatory gates** — the benchmark gate (required on *every*
version bump per CLAUDE.md, toolchain-only patches included) and the
closeout doc-sync. Consequences carried into 1.3.6: no 1.3.5 row in
`docs/benchmarks/history.csv` (so the 1.3.6 gate's baseline is the
2026-07-13 / 1.3.4-line run and its deltas span `6.4.62 → 6.5.27 →
6.5.35`, not one pin step), `cyrius.cyml` left pinned at `6.5.27` against
a `6.5.35` wrapper (toolchain-drift warning on every build), fmt debt left
unflagged in three files, and every Status block still reading **1.3.4 /
6.4.62**. All reconciled in 1.3.6; 1.3.5 itself stands as shipped. Its
CHANGELOG line quotes `Build 354,112 -> 417,608 bytes`, which does not
reproduce under either a DCE or a plain build at that pin (both measure
`413,512 B` on the local x86_64-linux host) — superseded by the 1.3.6
figures.

**1.3.4** — Toolchain-refresh patch on top of 1.3.3. Cyrius pin
`6.3.15` → `6.4.62`. No agnostik-side source changes; the sole code
change is a test-harness fix: `tests/tcyr/agnostik.tcyr` was missing
`include "src/proto.cyr"` (it includes `telemetry.cyr`, whose
`Span_to_otlp_proto` calls the `_proto_*` helpers) — benign since v1.2.0
as those fns were unreachable in that unit, but 6.4.62's link
diagnostics surfaced it as 5 warnings; include added mirroring
`main.cyr`. Public API (871 fns) and all wire formats byte-for-byte
unchanged. 6.4.x `lib sync` now copies only the declared
`[deps].stdlib` subset (25 `.cyr`) by default (`--full` for the whole
snapshot); `bayan` still ships (no `json`→`bayan`-style reorg this
cycle). Bench-regression gate: 20 checked, **0 regressions** — 6.4.62
codegen is uniformly faster, **every** op improved (medians of 3 runs:
`token_usage_update` 88→~42ns −52%, `sandbox_config_default` 99→~50ns
−49%, `version_roundtrip` 666→~350ns −47%, `accelerator_device_full`
296→~159ns −46%, `version_to_str` 292→~170ns −42%, `message_build_3turn`
1000→~618ns −38%). 858/858 tests pass; lint/fmt clean; api-surface
unchanged at 871 fns; `dist/agnostik.cyr` re-bundled. DCE binary
`392,840 B` → `350,016 B` (−43 KB — 6.4.62's DCE NOP-fill is leaner).
`cyrius audit` now runs on 6.4.62 (the long-standing missing-`check.sh`
breakage is **resolved**) but its test/bench sub-phases false-fail on
unresolved manifest stdlib (`clock_now_ns`/`bayan_json_get`); workaround
(run gates individually) unchanged — see the audit issue file.

**1.3.3** — Error-family namespacing on top of 1.3.2. To end symbol
collisions when the bundle is co-included with sibling libs that define
their own `ERR_*` (notably agnodrm): `ERR_*` → `STIK_ERR_*`, `err_*` →
`stik_err_*`, `syserr_*` → `stik_syserr_*`. `dist/agnostik.cyr`
regenerated. **Breaking (symbol-level):** downstream consumers must
migrate their references (known consumer: aegis). No wire-format or
type-shape change.

**1.3.2** — Toolchain refresh on top of 1.3.1 — leaf step of the
coordinated base-security-stack migration to cyrius `6.3.15` (agnostik
is the shared-types leaf under aegis). Cyrius pin `6.2.11` → `6.3.15`.
Public API surface and all wire formats byte-for-byte unchanged; all
858 assertions across the 15 test files pass on the new stack;
bench-regression gate green (0 regressions). One perf improvement:
`sandbox_config_new` unrolled (`src/security.cyr`) — the 11-iteration
`while` zero-loop (zeroed all 11 u64 slots, then overwrote two)
replaced with 11 direct `store64`s (two live fields with their values,
nine zeroed inline), dropping the per-slot loop counter/compare/branch
from the hot constructor: **~101ns → ~90ns** (−10%) on
`sandbox_config_default`. No behavior change. (The shipped 1.3.2
CHANGELOG entry says "810 assertions" — an undercount that omits
`test_v107`'s 48; the real total is 858.)

**1.3.1** — Toolchain-refresh patch on top of 1.3.0. Cyrius pin
`6.0.26` → `6.2.11`. No agnostik-side source changes; public API
(871 fns) and all wire formats byte-for-byte unchanged. Project-visible
change: 6.2.x folds standalone stdlib `json` (with `base64`/`csv`/`toml`)
into the bundled `bayan` module, so `[deps] stdlib` swaps `json` →
`bayan`. agnostik never called stdlib json (its parsing is the
hand-rolled `_json_*` family in `src/types.cyr`), but the 6.2.11 build
references `bayan_json_get` from the auto-resolved stdlib preamble, so
`bayan` must be declared for a warning-free build. `lib/` wiped and
re-synced from the 6.2.11 snapshot (`cyrius lib sync`, 97 `.cyr`).
Bench-regression gate: 25 checked, **3 regressions** vs the v1.2.0
baseline, ack'd with `[bench-regression-ack]` — net strongly positive
6.2.11 codegen: JSON-decode hot paths collapsed 67–87%
(`accel_flags_from_json` 6000→~810, `resource_limits_from_json`
3000→~500, `injection_scores_from_json` 2000→~360,
`token_usage_from_json` 2000→~485, `agent_stats_from_json` 1000→~330),
while three sub-300ns constructor/format ops gained ~100ns each
(`version_roundtrip` 371→~668, `accelerator_device_full` 177→~284,
`version_to_str` 191→~296). Source unchanged → nothing to optimize.
858/858 tests pass; lint/fmt clean; api-surface unchanged at 871 fns;
`dist/agnostik.cyr` re-bundled. DCE binary `311,264 B` → `392,840 B`
(+81 KB): 6.2.11 DCE **NOPs** unreachable fns in place rather than
stripping (257,630 B NOPed), and the `bayan` bundle adds ~119 KB of
now-NOPed dead code the leaner `json.cyr` did not. None reachable.

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

- **Cyrius**: `6.5.35` (pinned in `cyrius.cyml [package].cyrius`) —
  shipped in `1.3.6` (bumped from `6.5.27`). Every functional gate green:
  858/858 tests, lint clean (0 warnings), fmt clean (31 files), `vet` 24
  deps / 0 untrusted / 0 missing, api-surface locked at 871 fns, bench gate
  25 checked / 0 regressions with 23 of 25 ops faster.
- **Stdlib resolution (6.4.x+)**: `cyrius lib sync` copies the
  version-pinned snapshot into `./lib/`; `cyrius deps` resolves git
  deps only and presence-checks the `[deps] stdlib` array. Run
  `lib sync` before `deps` on a fresh checkout. `build`/`test`/`bench`
  resolve stdlib **from the pinned snapshot** (`~/.cyrius/versions/<pin>/lib`),
  not from `./lib/` — verified at the 1.3.6 cut by swapping `./lib/`
  contents and observing zero effect on the emitted binary. `./lib/` is
  vendored for CI, offline builds, and consumers; the manifest pin is what
  actually selects the stdlib. **6.4.x+ behavior**: `lib sync` copies only
  the declared `[deps].stdlib` subset (25 `.cyr`, 27 files with platform
  peers) by default, not the whole snapshot; pass `--full` for the
  complete set (101 modules).
- **Stdlib layout**: standalone `json.cyr` (with `base64`/`csv`/`toml`)
  was folded into the bundled `bayan.cyr` distribution module back at
  6.2.x, and `bayan` still ships in 6.5.35 (no reorg this cycle — module
  set unchanged at 101). `[deps] stdlib` lists `bayan` (not `json`).
  agnostik uses none of stdlib json directly, but the build references
  `bayan_json_get` from the auto-resolved preamble, so `bayan` must be
  declared for a 0-warning build. **6.5.3x grew `bayan` from 215,481 to
  641,083 B** (431 → 799 fns) by folding in a full PDF parse/encode
  subsystem (361 `_pdf*` fns). agnostik reaches none of it; it lands in
  the binary NOPed, and is the sole cause of the +215,520 B size jump at
  1.3.6.
- **DCE**: `CYRIUS_DCE=1` **NOP-fills** unreachable code in place rather
  than removing it — a DCE build and a plain build emit byte-identical
  artifacts (measured at both the 6.5.27 and 6.5.35 pins). This is
  long-standing, not a 6.x-era change: a probe with 3,000 unreferenced fns
  measures identical under both modes on **6.0.1**, the oldest 6.x on this
  host, and every version since.
  DCE is a guarantee that dead code cannot execute, **not** a size
  reduction. Binary size is still tracked as a release metric, but it
  tracks total stdlib surface, not reachable surface.
- **Compiler**: `cc5` — invoked via `cyrius {build,test,bench}`; raw
  `cat | cc5` is forbidden (manifest auto-resolves deps and prepends includes)
- **Locally installed vs released**: `cyrius --version` may report
  a newer dev build; the manifest always pins to the latest
  **released** version so CI and external contributors get a
  reproducible toolchain. Bump the pin only when a new release ships.
  `cyrius --version` prints the manifest pin alongside the wrapper version
  and flags drift explicitly (`manifest-pin: X (drift — wrapper is Y)`) —
  the 1.3.5 tree sat in that drifted state until this release.
- **`cyrius audit`** — usable, and **already was before this bump**. Phase
  list is `fmt / lint / docs / tests / bench`. Both historical failure modes
  closed earlier than agnostik's notes previously recorded, bisected against
  this tree:
  - missing-`check.sh` — closed at **6.2.24** (6.2.23 errors `script not
    found: ~/.cyrius/bin/check.sh`; 6.2.24 runs the phases inline). The
    2026-07-13 note saying "resolved at 6.4.62" reflects the next version
    agnostik happened to re-test, not the fix point.
  - `tests`/`bench` stdlib-preamble resolution — closed at **6.4.73**
    (6.4.72 → `10 passed, 5 failed`; 6.4.73 → `15 passed, 0 failed`), so it
    was already working on **6.5.27**, the pin 1.3.6 bumps from.

  [`docs/development/issues/cyrius-audit-missing-check-script-2026-04-26.md`](issues/cyrius-audit-missing-check-script-2026-04-26.md)
  stays **open** on two counts:
  1. **`cyrius self` still false-fails** — same two undefined symbols,
     `FAIL: cycc!=cycc`, rc 1 (verified 6.5.27 / 6.5.30 / 6.5.35, so not a
     6.5.35 regression). `audit` has not covered it since **6.2.24**, when
     the self-host phase left its phase list: `cyrius --help` read "full
     check: self-host, test, fmt, lint" from 6.0.1 through **6.2.10**,
     "local item suite (check.sh: fmt/lint/format/tests)" from 6.2.11, and
     "project sweep: fmt/lint/docs/tests/bench" since 6.2.24. The preamble
     defect was routed around, not repaired, and the one gate that still
     trips it sits outside `audit`.
  2. **`audit` exits 1** on its `docs` phase — **853 undocumented public
     fns**, agnostik's own gap (6.5.27 reports the identical count), a
     roadmap backlog item. Tooling note: `cyrius doc --check` takes a
     **single file** and exits with *that file's* undocumented count
     (`src/main.cyr` → 23, `src/agent.cyr` → 177, `src/lib.cyr` → 0); 853
     is the sum across the 15 `src/*.cyr`, not any one command's rc.

  Practical guidance: run `cyrius audit` for `fmt`/`lint`/`tests`/`bench`
  and read its **per-phase verdicts, not its exit code**; run `cyrius self`
  separately, expecting a known false failure.

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
| Test files            | 16        | tests/tcyr/ (+test_v137_hardening at the 2026-08-24 P(-1) pass) |
| Test assertions       | 886       | 0 failed; +28 F-014..F-019 hardening regressions at the 2026-08-24 P(-1) pass (858 through v1.3.6; 851 through v1.2.3) |
| Benchmarks            | 25        | `tests/bcyr/agnostik.bcyr` — gained the missing `src/proto.cyr` include at 1.3.6 |
| Test binary           | 629,032 B | `build/agnostik`. DCE and plain builds are now byte-identical (see Toolchain — DCE NOP-fills in place, it does not shrink). History: 261→273 KB at 1.0.2; 274 KB at 1.0.3+; ~311 KB at 1.2.0 from chrono+proto surface; ~304 KB at 1.2.1; ~306 KB / 313,344 B at 1.2.3 across the 6.0.x boundary; 311,264 B at 1.3.0; 392,840 B at 1.3.1 (6.2.11 NOPs in place + ~119 KB `bayan`); 350,016 B at 1.3.4 (6.4.62, −43 KB, leaner NOP-fill); 413,512 B at 1.3.5 (6.5.27); **629,032 B at 1.3.6** on the 6.5.35 pin, **+215,520 B** — 6.5.3x folds a 361-fn PDF subsystem into `bayan`, all NOPed, none reachable |
| Build warnings        | 0         | 3 vendored-`bayan` TOML warnings at the 1.3.5 pin fixed upstream in 6.5.35; 5 bench-harness `_proto_*` warnings fixed at 1.3.6 |
| Lint warnings         | 0         | (28 UFCS false positives resolved upstream in cyrius 5.7.7) |
| Lib bundle (dist/)    | 121,132 B | regenerated by `cyrius distlib`; tracked in CI sync check. 1.3.6 diff is the version banner only |
| Undocumented pub fns  | 853       | `cyrius doc --check` rc 23 — the sole reason `cyrius audit` exits non-zero on 6.5.35; pre-existing, backlog item |

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

See [`CHANGELOG.md`](../../CHANGELOG.md). Most recent stable: `1.3.6`
(toolchain-refresh patch Cyrius `6.5.27` → `6.5.35`; no source-logic
changes; 858/858 tests, api-surface unchanged at 871 fns, bench gate 25
checked / 0 regressions with 23 of 25 ops faster; binary +215,520 B to
629,032 B, entirely the NOPed `bayan` PDF subsystem; two fixes found by
running the gates 1.3.5 skipped, neither attributable to 6.5.35 — the
`agnostik.bcyr` missing-`proto.cyr` include, and the narrowing of the
2026-04-26 issue to `cyrius self`, which stays **open**; plus the fmt,
`history.csv` baseline, and doc-Status reconciliation the 1.3.5 cut
skipped. No public API/wire change). Prior: `1.3.5` (Cyrius `6.4.62` →
`6.5.27`, aligning with the AGNOS desktop stack; shipped without its
benchmark gate or doc-sync — reconciled at 1.3.6); `1.3.4`
(Cyrius `6.3.15` → `6.4.62` + `agnostik.tcyr` proto-include fix);
`1.3.3` (error-family namespacing `ERR_* → STIK_ERR_*` — symbol-level
breaking, consumers migrate; known consumer aegis); `1.3.2` (Cyrius
`6.2.11` → `6.3.15`, base-security-stack leaf migration +
`sandbox_config_new` unroll −10%).

## Verification hosts

- Local: x86_64-linux (LTS kernel 6.18)
- CI: `ubuntu-latest` (GitHub Actions)
- Cross: aarch64 best-effort via `cc5_aarch64` when shipped in toolchain
