# Agnostik — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures (durable);
> this file is **state** (volatile). Bumped via `scripts/version-bump.sh`.

## Version

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

- **Cyrius**: `5.10.14` (pinned in `cyrius.cyml [package].cyrius`)
- **Compiler**: `cc5` — invoked via `cyrius {build,test,bench}`; raw
  `cat | cc5` is forbidden (manifest auto-resolves deps and prepends includes)
- **Locally installed vs released**: `cyrius --version` may report
  a newer dev build; the manifest always pins to the latest
  **released** version so CI and external contributors get a
  reproducible toolchain. Bump the pin only when a new release ships.
- **`cyrius audit`** is still broken on 5.10.x (missing `check.sh`
  — same upstream issue as 5.7.12, filed in
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
| Source LOC (src/)     | ~3,200    | down from 7,121 LOC Rust; derive markers removed in F-011 |
| Module count          | 12        |                                    |
| Test files            | 9         | tests/tcyr/                        |
| Test assertions       | 653       | 0 failed; +40 audit regressions vs pre-1.0 |
| Benchmarks            | 25        | tests/bcyr/                        |
| Test binary (DCE)     | 274 KB    | `build/agnostik` after `CYRIUS_DCE=1 cyrius build` (261→273 KB at 1.0.2; 274 KB at 1.0.3+; 1.0.4 nominal +48 B from dot-syntax codegen) |
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

See [`CHANGELOG.md`](../../CHANGELOG.md). Most recent stable: `1.0.5` (`lib/test.cyr` table-driven adoption for F-005/F-010 + API surface snapshot CI gate at the 1317-fn baseline).

## Verification hosts

- Local: x86_64-linux (LTS kernel 6.18)
- CI: `ubuntu-latest` (GitHub Actions)
- Cross: aarch64 best-effort via `cc5_aarch64` when shipped in toolchain
