---
name: Agnostik Documentation Health
description: Living state of doc currency in the agnostik repo — fresh / stale / archived / open-question, refreshed as docs are touched
type: state
---

# Documentation Health — agnostik

> **Last refresh**: 2026-05-28 (rows refreshed for the v1.2.3 major-toolchain-refresh cut — Cyrius `5.10.44` → `6.0.14`, stdlib workflow moved to `cyrius lib sync`) | **Refresh cadence**: when docs are touched, update the affected row.
> **Scope**: This repo only (`agnostik`) — root-level files (README, CHANGELOG, CLAUDE.md, etc.) plus the entire `docs/` tree. Cross-repo Cyrius pin/version drift lives in [`development/state.md`](development/state.md), not here.

This is a **ledger**, not a one-time audit. Rewrite-in-place as docs change. The doc surface is small (~22 files) but every file is load-bearing — agnostik is the type vocabulary every AGNOS component depends on, and stale type docs propagate downstream.

Pattern lifted from the genesis-repo ledger ([`agnosticos/docs/doc-health.md`](https://github.com/MacCracken/agnosticos/blob/main/docs/doc-health.md)) — same buckets, smaller scale.

---

## At a glance — 2026-05-09 inventory

**~22 markdown files** total. Bucket counts after the v1.2.0 doc audit (commit `2c51e07`):

| Bucket | Count | What it means |
|---|---|---|
| ✅ **Fresh / refreshed in this audit** | ~14 | Touched 2026-05-09 in the v1.2.0 sweep — README, CHANGELOG, CLAUDE.md, CONTRIBUTING, SECURITY, architecture/overview, roadmap, state, ADR-001, ADR-002, ADR README, both audit reports, derive-comments issue. |
| 🟡 **Stale — refresh in place** | 0 | None known. Audit pass closed the surface. |
| 🟠 **Read-through outstanding** | ~3 | `cyrius-audit-missing-check-script-2026-04-26.md` (5.7.x-era, may be fixed upstream); the two issue-archive files (date-stamped — verify still classified correctly). |
| 🔵 **Probably evergreen** | ~2 | `CODE_OF_CONDUCT.md`, archive `README.md`. Re-read pass annually. |
| 📦 **Archive / frozen by design** | ~3 | `docs/benchmarks/benchmark-rust-v-cyrius-legacy.md` (self-marks legacy), `docs/development/compile-profile-2026-05-09.md` (one-shot v1.0.6 artifact), the issues `archive/` set. |
| ❓ **Open strategic question** | 0 | None outstanding — see [Open questions](#open-strategic-questions) for the empty list and the criteria that would re-open it. |

**Doc audit completed 2026-05-09** as part of the v1.2.0 cut:
- ✅ README.md substantially rewritten (Status block, modules table including `proto.cyr`, OTLP example, gate scripts listed, Decisions section).
- ✅ CONTRIBUTING.md — `cyrius lint` → `cyrlint` fix, ADR-002 derive guidance, sub-byte field rule, type-check + api-surface + bench-regression workflow added.
- ✅ SECURITY.md — supported versions, scope rewritten by attack surface, audits table.
- ✅ CLAUDE.md — durable "all fields i64" rule softened to acknowledge sub-byte widths.
- ✅ docs/architecture/overview.md — `proto.cyr` added; `LlmProvider 13→18`, `ModelCapabilities 12→15`, `PiiKind 16→19`; serialization rewritten as 9-row table per ADR-002.
- ✅ docs/development/roadmap.md — all shipped sections removed; v1.2.1 / v1.2.2 / v2.0.0 pins kept.

---

## Tier 1 — Root files

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | 2026-05-28 | ✅ Fresh | v1.2.0 audit rewrite (Status block + module table + OTLP example + gate-script list + Decisions section). Status block bumped at v1.2.3 (toolchain `6.0.14`); Quick Start gained `cyrius lib sync` step. |
| `CHANGELOG.md` | 2026-05-28 | ✅ Fresh | Source of truth for shipped work. Updated through 1.2.3 (major toolchain refresh `5.10.44 → 6.0.14`; stdlib workflow `deps` → `lib sync`). |
| `CLAUDE.md` | 2026-05-28 | ✅ Fresh | Durable rules. Quick Start gained `cyrius lib sync` (6.0.x stdlib resolution). Sub-byte-field rule intact; state pointer intact. |
| `CONTRIBUTING.md` | 2026-05-28 | ✅ Fresh | `cyrlint` fix + derive guidance + sub-byte rule + new gate-script steps. Setup snippet gained `cyrius lib sync` (6.0.x). |
| `SECURITY.md` | 2026-05-09 | ✅ Fresh | Supported lines (1.0/1.1/1.2), scope by attack surface, audits table. |
| `CODE_OF_CONDUCT.md` | 2026-05-03 | 🔵 Evergreen | Standard. |
| `VERSION` | 2026-05-28 | ✅ Fresh | `1.2.3` — source of truth, matches `cyrius.cyml`. |

---

## Tier 2 — Project state (`docs/development/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `state.md` | 2026-05-28 | ✅ Fresh | Live volatile state (version, sizes, test count, consumers, verification hosts). Refreshed at v1.2.3 (toolchain `6.0.14`, DCE ~306 KB; 6.0.x stdlib-resolution note added). |
| `roadmap.md` | 2026-05-28 | ✅ Fresh | Status block bumped to v1.2.3 (Cyrius `6.0.14`); cross-consumer sweep re-pinned v1.2.3→v1.2.4; OTLP completion re-pinned v1.2.4→v1.2.5. |

---

## Tier 3 — Architecture (`docs/architecture/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `overview.md` | 2026-05-09 | ✅ Fresh | Module map, library deps, traits, serialization split. Rewritten in v1.2.0 audit; counts re-verified against `src/`. |

---

## Tier 4 — ADRs (`docs/adr/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | 2026-05-09 | ✅ Fresh | ADR index. Both ADRs cross-linked. |
| `001-revive-derive-serialize.md` | 2026-05-09 | ✅ Fresh | Marked **superseded by ADR-002**. |
| `002-derive-serialize-7-of-9.md` | 2026-05-09 | ✅ Fresh | Current decision. 7 trivial structs use `#derive(Serialize)`; `AgentInfo` + `TelemetryConfig` retain hand-written impls. |

**ADR posture**: small surface, low decision-velocity. Only architecturally significant calls earn an ADR — minor decisions ride CHANGELOG + design comments. Re-evaluate at v2.0.0 cut.

---

## Tier 5 — Audit reports (`docs/audit/`)

Date-stamped, frozen by design. Each minor cut runs an audit pass per CLAUDE.md cadence and lands a new report — old reports stay verbatim as the historical record.

| File | Date | Status | Notes |
|---|---|---|---|
| `2026-04-26-audit.md` | 2026-04-26 | 📦 Frozen | Pre-1.0.0 hardening — F-001..F-011 closed. |
| `2026-05-10-audit.md` | 2026-05-09 (filed under 05-10) | ✅ Fresh | Post-1.0 cumulative diff (1.0.1..1.0.7) — F-012 INFO closed, prior findings re-verified. |

Next audit slot: at v1.3.0 cut (or sooner if a CVE pattern surfaces in agnostik's input-handling paths or the cyrius toolchain's parser/serde dependencies).

---

## Tier 6 — Engineering issues (`docs/development/issues/`)

Filed-upstream issue records. Open issues sit at the top level; resolved issues move to `archive/` when the upstream fix lands and a workaround is no longer needed.

| File | Last touched | Status | Notes |
|---|---|---|---|
| `cyrius-derive-comments-in-struct-body-2026-05-10.md` | 2026-05-09 | ✅ Fresh — open | Filed during v1.2.0. Workaround in CONTRIBUTING.md. |
| `cyrius-audit-missing-check-script-2026-04-26.md` | 2026-05-28 | 🟠 Read-through — still open | Filed against 5.7.x; re-confirmed **still broken on 6.0.14** at the v1.2.3 cut (`cyrius audit` → `script not found: .../bin/check.sh`). Workaround (run `self`/`test`/`fmt`/`lint` individually) stays. Re-check at next cut. |
| `archive/README.md` | 2026-05-08 | 🔵 Evergreen | Index for resolved-issue archive. |
| `archive/cyrius-lint-ufcs-pascal-prefix-snake-case-2026-04-26.md` | 2026-05-08 | 📦 Frozen | Closed — fixed upstream. |
| `archive/cyrlint-char-literal-brace-bug-2026-05-09.md` | 2026-05-08 | 📦 Frozen | Closed — fixed upstream (5.10.6 / 5.10.10). |

---

## Tier 7 — Dated artifacts (one-shot, intentionally frozen)

These are timestamped engineering artifacts — captured at a point in time, referenced by CHANGELOG/audit, never refreshed in place.

| File | Last touched | Status | Notes |
|---|---|---|---|
| `docs/benchmarks/benchmark-rust-v-cyrius-legacy.md` | 2026-05-03 | 📦 Frozen | Self-marks legacy at the top of the file. Live benchmark data is `docs/benchmarks/history.csv`. |
| `docs/development/compile-profile-2026-05-09.md` | 2026-05-09 | 📦 Frozen | One-shot v1.0.6 compile-profile pass. Conclusion already folded into roadmap (no action triggered). |

`docs/benchmarks/history.csv` is the live perf surface — not a doc, not bucketed here.

---

## Open strategic questions

None outstanding for the v1.2.0 cut. This section will repopulate when:

- A new doc category appears that doesn't fit an existing tier (e.g. a `docs/guides/` if/when consumer onboarding docs become a thing).
- The lib starts accumulating per-module deep-dives (today the architecture overview carries the whole module map; that pattern won't scale forever).
- An ADR needs to be retired without a successor — would force a posture call (close the series vs. write a closure ADR).

---

## In-flight (blocked, not stale)

- `cyrius-audit-missing-check-script-2026-04-26.md` — re-confirmed still broken on 6.0.14 at the v1.2.3 cut (`check.sh` still absent). Owner: agnostik audit pass; trigger: re-check next time the pin bumps.

---

## Forward doc-policy commitments

| # | Commitment | Trigger | Source | Notes |
|---|---|---|---|---|
| 1 | **Audit report retention** — keep all `docs/audit/YYYY-MM-DD-audit.md` reports verbatim through at least v2.0.0; re-evaluate at the major cut whether pre-1.0 reports get folded into a single historical summary. | v2.0.0 cut | This file | Today's surface is 2 reports — purge pressure is zero. |
| 2 | **Issue archive purge** — the `docs/development/issues/archive/` set is a record of upstream cyrius bugs that landed during agnostik development. Keep through v2.0.0; at major cut, decide whether to roll them up into a single CHANGELOG-of-cyrius-quirks file. | v2.0.0 cut | This file | Same low-pressure shape. |

---

## Refresh procedure

When docs are touched:

1. Find the affected row in the relevant tier table.
2. Update **Last touched** column to the new date.
3. Update **Status** column if the bucket changed.
4. Update **Notes** column if the next step changed.
5. If a doc moved or was archived, update its row to reflect the new home.
6. Re-anchor "Last refresh" date in the header.

When the bucket counts at the top drift by more than ~3 in any cell, refresh the at-a-glance table.

This file's refresh cadence is **opportunistic** (touched when other docs are touched), not periodic. The v1.2.0 audit established the baseline; each minor cut's doc-sync step (CLAUDE.md Closeout Pass §9) updates this file alongside CHANGELOG + roadmap + state.md.

---

## What this file is NOT

- Not a substitute for [`development/state.md`](development/state.md) (which holds live version/size/test/consumer state).
- Not a CHANGELOG (which records what shipped, not what's stale).
- Not a roadmap (forward work lives in [`development/roadmap.md`](development/roadmap.md)).
- Not a per-doc review log (we record the result of an audit pass, not the per-doc reasoning).

---

*Last refresh: 2026-05-28 (rows refreshed for the v1.2.3 major-toolchain-refresh cut — Cyrius `5.10.44` → `6.0.14`). Refresh in place when docs are touched.*
