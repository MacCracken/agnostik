# Agnostik Roadmap

## Status

**v1.6.2** — most recent stable. 12 modules + `src/proto.cyr` (OTLP wire
helpers), **1,412 test assertions across 18 `.tcyr` files** (passing on
x86_64-linux and on aarch64 under `qemu-aarch64`), 25 benchmarks, zero
external dependencies, Cyrius `6.6.6`.

v1.6.2 is a toolchain refresh (`6.6.2` → `6.6.6`, `lib/` re-vendored) plus
**F-022**: `_fill_random` called `getrandom` by its raw x86_64 number, which
is `-ENOSYS` on aarch64 and not a syscall on agnos. It now calls the stdlib's
portable `sys_getrandom`. It also re-enables the aarch64 CI cross-build, which
had probed a compiler name (`cc5_aarch64`) that no 6.x toolchain ships.
v1.6.1 renamed `health_check_new` → `agnostik_health_check_new` (the argonaut
collision in kybernet). v1.6.0 migrated to the cyrius 6.6.x value form (pin
`6.5.35` → `6.6.2`; `result_print_agnostik_err` takes the tag); v1.5.1 added
the three missing `cglim_set_*` setters. See
[`../audit/2026-09-23-audit.md`](../audit/2026-09-23-audit.md) for the most
recent audit.

v1.5.0 was a security minor: `enum LinuxCapability`'s values are Linux
capability numbers again. They are fed to `capset(2)` via `1 << cap`, and
the enum omitted `CAP_MAC_OVERRIDE`/`CAP_MAC_ADMIN` (shifting its tail by
two) and transposed `CAP_AUDIT_READ`/`CAP_AUDIT_CONTROL`. Adds
`capability_name`/`capability_parse`, completing F-018 for that enum.
Surfaced from kybernet 1.5.2; argonaut 1.11.0 corrects its same-named enum
in lockstep.

v1.4.0 closed the two contract gaps the 2026-08-24 P(-1) audit found:
**F-018** (31 enums had `*_name()` and **zero** `*_parse()`, so the
roundtrip contract held for no enum — now 31 `*_parse` covering 204
members, with an exhaustive 481-assertion `parse(name(v)) == v` test) and
**F-021** (9 fields had getters but no setters and read `0` forever —
`SecretMetadata.expires_at`/`.owner` plus the seven `mcap_supports_*`
flags). Additive only: surface 871 → **911** fns, zero removals, no wire
or layout change.

v1.3.7 was the P(-1) hardening sweep that found them — **F-014** (a
signed-comparison sentinel in `_fill_random` that retried at `buf - 1`
with `n + 1` bytes, or hung), **F-015/F-016/F-017** (W3C traceparent
validation: all-zero ids, unvalidated version, uppercase hex),
**F-019/F-020**. Prior: v1.3.6 (Cyrius `6.5.27` → `6.5.35` + the CI
format-gate fix), v1.3.5 (`6.4.62` → `6.5.27`), v1.3.4 (`6.3.15` →
`6.4.62`), v1.3.3 (error-family namespacing `ERR_* → STIK_ERR_*`,
symbol-level breaking). See [`state.md`](state.md) for the live snapshot,
[`../audit/2026-08-24-audit.md`](../audit/2026-08-24-audit.md) for the
last full-source audit, and [`../../CHANGELOG.md`](../../CHANGELOG.md) for
full release history.

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
- ~~**Setter-less `mcap_supports_*` getters** (`llm.cyr`) — seven flag
  getters with no matching setter (can only read 0).~~ **RESOLVED in
  v1.4.0** (F-021): setters added, alongside `SecretMetadata`'s, so the
  library gives one consistent answer on setter-less getters rather than
  two.
- ~~**`secret_metadata_new` over-alloc** (`secrets.cyr`) — 72 B / 9 slots,
  3 unreachable (offsets 24/56/64).~~ **RESOLVED in v1.3.7** (F-020).
  The gating question — whether an external consumer writes offsets
  56/64 by raw pointer — was answered instead of waived: all eleven
  consumer repos grep to **zero** references to `smeta_` /
  `secret_metadata`, so the layout is private to the file. Shrunk to
  56 B. See [`../audit/2026-08-24-audit.md`](../audit/2026-08-24-audit.md)
  §F-020.

These are not security exposures — F-013 (the one real finding) shipped
in v1.3.0. Trigger for action: the v1.2.4 cross-consumer sweep landing
(gives the ABI-safety signal removal needs), or a consumer surfacing a
concrete need.

---

## Backlog — v1.3.1 toolchain review (unpinned, revisit later)

Two items surfaced by the 6.2.11 pin that were **accepted as-is** at the
v1.3.1 cut (toolchain trade-offs, no source-side fix) but warrant a later
look. Full numbers in the CHANGELOG `[1.3.1]` Performance section.

- **Ack'd bench regressions** — 3 consistent on an unloaded runner
  (`version_roundtrip` 371→~668ns, `accelerator_device_full` 177→~284ns,
  `version_to_str` 191→~296ns), rising to 6 under CI load as sub-µs ops
  (`traceparent_format`, `sandbox_config_default`, `token_usage_update`)
  inflate past threshold — jitter, not drift. Pure 6.2.11 codegen / runner
  contention; source unchanged, nothing to optimize agnostik-side, and the
  net is strongly positive (JSON-decode hot paths −67…87%). Revisit if a
  later cyrius pin recovers the three real small-op paths, or if a
  consumer's profile shows these constructor/format ops on a hot path.
  Ack'd via `[bench-regression-ack]` in the release commit (whole-run skip).
- ~~**DCE binary +81 KB** (`311,264 B` → `392,840 B`). Two causes: 6.2.11 DCE
  *NOPs* unreachable fns in place instead of stripping, and the `bayan`
  bundle (base64+json+csv+toml) adds ~119 KB of now-NOPed dead code the
  former standalone `json.cyr` did not.~~ **RESOLVED upstream at cyrius
  6.5.72**, which made `CYRIUS_DCE=1` strip rather than NOP-fill on x86_64.
  agnostik first pinned past it at v1.6.0, and it was recorded at v1.6.2: the
  DCE binary is **127,128 B** on 6.6.6. `bayan` stays declared for the
  `bayan_json_get` preamble reference, but its dead code no longer reaches
  the artifact.

---

## Backlog — v1.3.6 toolchain review (unpinned, revisit later)

Three items surfaced by the 6.5.35 pin, all **accepted as-is** at the
v1.3.6 cut. Full numbers in the CHANGELOG `[1.3.6]` sections.

- **853 undocumented public fns** — the *sole* reason `cyrius audit` exits
  non-zero (`cyrius doc --check` takes one file and exits with that file's
  count: `src/main.cyr` → 23, `src/agent.cyr` → 177; 853 is the sum over
  the 15 `src/*.cyr`) (its fmt / lint /
  tests / bench phases all pass). Pre-existing and unchanged by the bump —
  6.5.27 reports the identical count — but it is agnostik's own gap, not a
  toolchain bug, and it is what stops `audit` from being usable as a
  single-command CI gate. Sizing it honestly: 853 fns is a large sustained
  effort, so the realistic shape is incremental (document a module per
  cycle, gate new fns at the api-surface check) rather than one sweep.
  Until then, read `audit`'s per-phase verdicts, not its exit code.
  **At v1.6.2 / 6.6.6:** the `src/*.cyr` per-file sum is **860** (plus 187
  over the test and bench files). `audit`'s docs phase reports **1,421** over
  its `src tests` scope, which is not the sum of the per-file counts (1,047),
  so track the per-file `src` figure release to release.

- ~~**Binary +215,520 B (+52.1%)** — `413,512` → `629,032` B, entirely the
  361-fn PDF parse/encode subsystem (209 private `_pdf*` + 152 public
  `bayan_pdf_*`) that 6.5.3x folds into the bundled `bayan`
  module (`215,481` → `641,083` B of source). agnostik reaches none of it;
  it lands NOPed.~~ **RESOLVED upstream at cyrius 6.5.72** (strip-not-NOP
  DCE on x86_64). Crossed at v1.6.0 and recorded at v1.6.2: the DCE
  artifact went 629,032 → 126,960 B (6.6.2) → **127,128 B** (6.6.6), with
  553,555 B of unreachable code eliminated. On x86_64, binary size tracks
  reachable surface again. aarch64 DCE still NOP-fills (1,024,048 B).

- **`cyrius self` still false-fails** — same `clock_now_ns` /
  `bayan_json_get` preamble-resolution defect, verified on 6.5.27, 6.5.30,
  6.5.35 and **6.6.6** (v1.6.2), so not a regression at any of them. `cyrius audit` does not cover it
  and has not since 6.2.24, when the self-host phase left `audit`'s phase
  list; the `audit`-side preamble fix landed at **6.4.73**, already present
  in the previous 6.5.27 pin — the bug was routed around, not repaired.
  Tracked in
  [`issues/cyrius-audit-missing-check-script-2026-04-26.md`](issues/cyrius-audit-missing-check-script-2026-04-26.md);
  archive that file when `self` resolves its preamble.

---

## Backlog — v1.6.2 toolchain review (unpinned, revisit later)

Surfaced by the 6.6.6 pin and the cross-target syscall review. Full
write-ups are in [`../audit/2026-09-23-audit.md`](../audit/2026-09-23-audit.md)
and the CHANGELOG `[1.6.2]` sections.

- **F-023 (LOW) — `_fill_random`'s fallback still uses raw x86_64-linux
  syscall numbers.** Its `open` / `read` / `close` / `write` / `exit` are 2 /
  0 / 3 / 1 / 60. aarch64 and Windows route them; on agnos they are getpid /
  exit / spawn / write / winsize. After F-022 this path runs only if
  `sys_getrandom` fails. There are two fixes, and choosing between them is the
  work:
  1. Route through `sys_read` / `sys_close` / `sys_write` / `sys_exit`, which
     are uniform on every peer, plus an agnos arm for `sys_open`, whose
     signature there is `(name, namelen, flags)`.
  2. Drop the `/dev/urandom` fallback now that `sys_getrandom` covers every
     target, keeping only the fail-loud path.
  Either is a behaviour change for kernels without `getrandom(2)` (Linux
  < 3.17), so it wants a minor, not a toolchain patch.
- **Bench windows vs the 6.6.5 resolution bar.** `message_build_3turn` and
  `resource_limits_from_json` run 500-iteration windows that sit near the
  ~222 µs bar (100 × (clock floor + tick) on this host). Only their slower
  windows resolve, so `min` can exceed `avg`. The gate reads `avg` and is
  unaffected. Raising those batch sizes would make every window resolve, but
  it changes what those rows measure, so land it with a fresh baseline and
  a note.
- **Upstream (cyrius) papercuts to file.**
  - `cyrius update` on a project with no `[deps.NAME]` entries copies the
    whole stdlib snapshot into `./lib/`: 111 files, including an untracked
    `lib/unicode/` that the `lib/*.cyr` ignore rule misses.
  - No raw-syscall diagnostic fires for `--agnos` builds, although agnos
    numbering diverges from x86_64-linux further than aarch64's does. The
    aarch64 diagnostic is what caught F-022; nothing would have caught F-023.
- **aarch64 is build-checked in CI, not run.** CI now cross-builds aarch64
  again (the probe was `cc5_aarch64` through 1.6.1), and the full suite passes
  locally under `qemu-aarch64`. Running `cyrius test --aarch64` in CI would
  need `qemu-user` on the runner; it is a small step if aarch64 consumers
  appear.

---

## v1.4.0 — Contract completeness ✅ SHIPPED

Both findings from the 2026-08-24 P(-1) audit, held out of the v1.3.7
patch because they are additive public API. Full write-ups in
[`../audit/2026-08-24-audit.md`](../audit/2026-08-24-audit.md).

### F-018 — `*_parse()` for every enum with a `*_name()` ✅

Shipped: **31 `*_parse` functions covering 204 enum members.** Before
this, the library had 31 enum `_name` functions and **zero** enum
`_parse`, so the CLAUDE.md roundtrip contract held for no enum and all
eleven consumers hand-rolled their own string→enum mappings.

What the implementation settled:
- **Exact match, no coercion** — no case-folding, trimming or aliases.
  An unrecognised string is `Err`, not a sentinel variant, so a config
  typo surfaces instead of silently becoming `Unknown`.
- **Seven members are named by their `_name` fallback**, not an explicit
  arm (`STIK_ERR_UNKNOWN`, `HEALTH_UNKNOWN`, `VENDOR_CUSTOM`,
  `LLM_CUSTOM`, `MEM_UNKNOWN`, `PII_CUSTOM`, `STATUS_UNKNOWN`). Their
  `_parse` maps the fallback string back so the roundtrip is total; the
  other 24 enums reject theirs, since no member owns it.
- **The test is the deliverable as much as the functions.**
  `tests/tcyr/test_v140_enum_parse.tcyr` asserts `parse(name(v)) == v`
  for all 204 members plus per-enum rejection cases — 481 assertions,
  mechanically derived. A 31-enum gap survived eight releases because
  nothing checked it; now something does.

Correction recorded during implementation: the audit's first draft said
"54 `*_name()` functions". That grep conflated enum name functions with
20 struct-field accessors also called `*_name` (`span_name`,
`smeta_name`, …). The real figure is 31 enum name functions / 204
members.

**Still open (deliberately out of scope):** 26 declared enums have no
`*_name()` *or* `*_parse()` — mostly bitflag/constant sets where a string
name may not be meaningful. Whether they need one is a separate decision;
pick it up when a consumer asks.

`LinuxCapability` (41 values) came off that list at **v1.5.0**: a consumer
did ask. kybernet needed per-service capability policy expressed as config
data, which means parsing capability names, so it got `capability_name` /
`capability_parse` — the lowercase `cap_` spelling `capsh(1)` and
`capability(7)` use. That release also corrected the enum's values, which
were not kernel capability numbers.

### F-021 — setters for previously unsettable fields ✅

Shipped: **9 setters.** `smeta_set_expires_at` / `smeta_set_owner` on
`SecretMetadata`, and the seven original `mcap_set_supports_*` flags on
`ModelCapabilities` — closing the v1.3.0 backlog item in the same change
so the library has one consistent answer on setter-less getters.

Chose setters (additive) over a wider constructor signature (breaking,
would have needed v2.0.0). Consumers should note that any expiry handling
built against `smeta_expires_at` before 1.4.0 was reading a permanent `0`
and was a no-op.

Surface 871 → **911** fns, zero removals; snapshot regenerated.

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
