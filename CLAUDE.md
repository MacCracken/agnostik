# Agnostik — Claude Code Instructions

> **Core rule**: this file is **preferences, process, and procedures** — durable rules that change rarely. Volatile state (current version, binary sizes, test counts, in-flight work, consumers, verification hosts, quirks of the current toolchain pin) lives in [`docs/development/state.md`](docs/development/state.md), bumped every release. Do not inline state here — inlined state rots within a minor.

---

## Project Identity

**Agnostik** (agnostic) — Shared types and domain primitives for AGNOS

- **Type**: Shared library (Cyrius)
- **License**: GPL-3.0-only
- **Language**: Cyrius (toolchain pinned in `cyrius.cyml [package].cyrius`)
- **Version**: `VERSION` at the project root is the source of truth — do not inline the number here
- **Genesis repo**: [agnosticos](https://github.com/MacCracken/agnosticos)
- **Standards**: [First-Party Standards](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-standards.md) · [First-Party Documentation](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-documentation.md)
- **Shared crates**: [shared-crates.md](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/shared-crates.md)
- **Recipes**: [zugot](https://github.com/MacCracken/zugot) — takumi build recipes

## Goal

Own the AGNOS type vocabulary. Every component speaks agnostik types: agents, classifications, secrets, telemetry, audit, LLM messaging, hardware descriptors, security envelopes. Zero domain logic — pure type definitions, parsers, validators, and serde primitives. If two AGNOS components need to communicate about a shared concept, the concept lives here.

## Current State

> Volatile state lives in [`docs/development/state.md`](docs/development/state.md) —
> current version, binary sizes, test/assertion counts, in-flight slots, recent
> shipped releases, consumers, verification hosts, toolchain notes. Refreshed
> every release. Historical release narrative lives in [`CHANGELOG.md`](CHANGELOG.md).

This file (`CLAUDE.md`) is durable rules.

## Scaffolding

Project was ported from a Rust crate. **Do not manually create project structure** — use `cyrius` tools. If the tools are missing something, fix the tools.

## Quick Start

```bash
cyrius lib sync                                        # vendor the pinned stdlib's declared [deps].stdlib subset + platform peers into lib/
cyrius deps                                            # add those modules' transitive stdlib includes; resolves [deps.NAME] git deps (agnostik has none)
cyrius deps --verify                                   # check lib/ against the committed cyrius.lock (CI runs it)
cyrius build src/main.cyr build/agnostik               # build the test harness binary
cyrius test tests                                      # run every .tcyr under tests/ (one file: cyrius test tests/tcyr/<name>.tcyr)
cyrius bench tests/bcyr/agnostik.bcyr                  # benchmarks
cyrius lint src/*.cyr                                  # static checks
cyrius fmt --check src/types.cyr                       # format check, one file; drift is reported by exit code only
CYRIUS_DCE=1 cyrius build src/main.cyr build/agnostik  # release build (dead code eliminated)
cyrius distlib                                         # regenerate dist/agnostik.cyr (CI fails when it is stale)
bash scripts/bench-regression.sh                       # benchmark gate vs history.csv — mandatory before every release, on the host that records history.csv
bash scripts/bench-paired.sh                           # CI's bench gate: paired, interleaved A/B against the last release tag, each tree on its own pin
bash scripts/api-surface.sh check                      # public-API snapshot gate (`update` after an intentional API change)
bash scripts/doc-debt.sh check                         # doc gate: every fn in src/ has a doc comment
bash scripts/consumer-sweep.sh                         # build + test every [deps.agnostik] consumer against this working tree, on its own pin
cyrius audit                                           # sweep: fmt / lint / docs / tests / bench — exits 0 since v1.9.0; a non-zero exit is a regression
```

Refresh the vendored stdlib with `lib sync` + `deps`. (`cyrius update` has done the same `lib sync` since cyrius 6.7.12; never pass it or `lib sync` `--full`, which copies the whole snapshot into `lib/`.) `deps` rewrites `cyrius.lock`; commit it with the pin move. **On a pin move, run `cyrius deps` alone**, not `lib sync` first: since cyrius 6.7.13 a scoped `lib sync` refuses ("nothing was copied") while `lib/` holds transitive leaves outside the declared subset that the new snapshot changed, and `deps` re-vendors the declared closure at the new pin and re-locks. Then check that `cyrius deps --verify` passes and that a fresh checkout (`lib sync` + `deps` from an empty `lib/`) reproduces the lock byte for byte. `deps` notes `lib/hashmap_fast.cyr` as outside the declared footprint; `lib sync` vendors it and a fresh checkout locks it, so keep it. `cyrius deps --relock` is only for a snapshot that changed under an unchanged pin ("DISAGREE under an unchanged pin"). Cross targets: `cyrius build --aarch64` / `--agnos` / `--win`; `cyrius test --aarch64 tests` runs the suite under `qemu-aarch64` when it is installed.

## Key Principles

- **Correctness is the optimum sovereignty** — if it's wrong, you don't own it; the bugs own you
- Test after EVERY change, not after the feature is "done"
- ONE change at a time — never bundle unrelated changes
- Research before implementation — check vidya for existing patterns
- `cyrius build`/`test`/`bench` auto-inject the `main()` caller and lazy-init the heap (since 5.10.x); do not write `var r = main(); syscall(SYS_EXIT, r);` or call `alloc_init()` explicitly
- **Build with `cyrius build`, never raw `cat file | cycc`** — the manifest auto-resolves deps and prepends includes
- Library modules in `src/` carry no stdlib includes — the manifest resolves them. The harness entry points (`src/main.cyr`, `tests/tcyr/*.tcyr`, `tests/bcyr/*.bcyr`) list their includes explicitly, and must include every module the included code calls (`telemetry.cyr` calls into `proto.cyr`)
- Every buffer declaration is a contract: `var buf[N]` = N **bytes**, not N entries
- Fuzz every parser path — edge cases get invariants, not assertions
- Benchmark before claiming perf — numbers or it didn't happen
- **Own the stack** — agnostik IS the stack's type vocabulary; consumers should not redefine these
- Every fn gets a `#` doc comment directly above it — `scripts/doc-debt.sh` fails CI on any undocumented fn in `src/` (absolute since v1.9.0, when the grandfathered baseline reached zero). Test and bench fns get one too, which is what keeps `cyrius audit`'s docs phase green
- All public enums must have `*_name()` (string representation) and `*_parse(s)` (roundtrip)
- Every serializable struct must have a `*_to_json(ptr, sb)` function (or `#derive(Serialize)`)
- Zero panic in library code — use `Result` (`Ok` / `Err`) for fallible operations
- Every parse function must have a roundtrip test

## Rules (Hard Constraints)

- **Read the genesis repo's CLAUDE.md first** — [agnosticos/CLAUDE.md](https://github.com/MacCracken/agnosticos/blob/main/CLAUDE.md)
- **Do not commit or push** — the user handles git. Local commits or branches only when the user explicitly asks; never push
- **Do not bump the version speculatively** — in-flight work goes under `## [Unreleased]` in the CHANGELOG. `VERSION` and a `## [X.Y.Z]` header change only when the user cuts a release (`scripts/version-bump.sh X.Y.Z` does both)
- **NEVER use `gh` CLI** — use `curl` to the GitHub API if needed
- Do not add unnecessary dependencies
- Do not skip tests before claiming changes work
- Do not skip fuzz / benchmark verification before claiming a feature works
- **Do not ship any release without the benchmark gate** — `scripts/bench-regression.sh` must pass with 0 regressions (or a `[bench-regression-ack]`) AND the delta table recorded in the CHANGELOG Performance section, on every patch/minor/major. See [Benchmark Gate](#benchmark-gate-before-every-release--patch-minor-major)
- Do not use `sys_system()` with unsanitized input — command injection risk
- Do not trust external data (file content, network input, user args) without validation
- Do not use `break` in while loops with `var` declarations — use flag + `continue` (the ecosystem-wide convention; upstream stdlib code still treats that `break` as unreliable)
- Do not add stdlib includes to `src/` library modules — the manifest resolves them
- Do not hardcode toolchain versions in CI YAML — the `cyrius = "X.Y.Z"` pin in `cyrius.cyml` is the only source of truth
- Do not break public API without a major version bump (consumer count is large — every component depends on agnostik). Public API additions normally ship in a minor, not a patch

## Process

### P(-1): Scaffold / Project Hardening (before any new features)

1. **Cleanliness** — `cyrius build`, `cyrius lint`, `cyrius audit`; all tests pass
2. **Benchmark baseline** — `cyrius bench`, save CSV for comparison
3. **Internal deep review** — gaps, optimizations, correctness, docs
4. **External research** — domain completeness, best practices, existing CVE patterns
5. **Security audit** — input handling, syscall usage, buffer sizes, pointer validation. File findings in `docs/audit/YYYY-MM-DD-audit.md`
6. **Additional tests / benchmarks** from findings
7. **Post-review benchmarks** — prove the wins against step 2
8. **Documentation audit** — ADRs for decisions, source citations, guides for public API
9. **Repeat if heavy** — keep drilling until clean

### Work Loop (continuous)

1. **Work phase** — new features, roadmap items, bug fixes
2. **Build check** — `cyrius build`
3. **Test + benchmark additions** for new code
4. **Internal review** — performance, memory, correctness, edge cases
5. **Security check** — any new syscall usage, user input handling, buffer allocation
6. **Documentation** — update CHANGELOG (under `[Unreleased]`), roadmap, `docs/development/state.md`, `docs/doc-health.md` (rewrite-in-place the rows for any docs touched this loop), any ADR the change earned
7. **Version check** — at a release cut, `VERSION`, `cyrius.cyml`, CHANGELOG header in sync
8. **Return to step 1**

### Security Hardening (before every release)

Every release runs a security audit pass. Minimum:

1. **Input validation** — every function accepting external data validates bounds, types, ranges
2. **Buffer safety** — every `var buf[N]` verified; N is **bytes**, max access < N, no adjacent-variable overflow
3. **Syscall review** — every syscall validated: args checked, returns handled, error paths complete — and correct on **every target the toolchain builds** (x86_64-linux, aarch64, agnos, Windows), not only x86_64-linux
4. **Pointer validation** — no raw pointer dereference of untrusted input without bounds
5. **No command injection** — use `exec_vec()` with explicit argv; never `sys_system()` with unsanitized input
6. **No path traversal** — file paths from external input validated, no `../` escape
7. **Known CVE review** — check dependencies and patterns against current CVE databases
8. **Document findings** — all issues in `docs/audit/YYYY-MM-DD-audit.md`

Severity levels: **CRITICAL** (remote / privilege escalation), **HIGH** (moderate effort), **MEDIUM** (specific conditions), **LOW** (defense-in-depth).

### Benchmark Gate (before EVERY release — patch, minor, major)

Mandatory on every version bump, no exceptions — toolchain-only refresh
patches included. A release does not ship until this passes and its
results are recorded.

1. **Run the gate** — `bash scripts/bench-regression.sh`. It compares
   per-op averages against the most recent committed baseline in
   `docs/benchmarks/history.csv` and **fails on any regression** beyond
   threshold (50% ns-bracket / 80% us-bracket, with absolute floors for
   jitter). The release is blocked while the gate fails. Its baselines
   are absolute numbers from the host that records `history.csv`, so it
   is only meaningful there. On any other machine (a GitHub runner, a
   contributor's laptop) use `scripts/bench-paired.sh`, which CI runs.
2. **Record the deltas** — the full delta table (baseline → current,
   delta%, regressions count) goes in the CHANGELOG **Performance**
   section for the release. "0 regressions" alone is not enough — note
   the notable movers (wins and drifts) with numbers.
3. **Intentional trade-offs** — if a regression is deliberate, ack it
   with `[bench-regression-ack]` in the HEAD commit message AND justify
   it in the CHANGELOG. Never silence the gate without a written reason.
4. **Refresh the baseline** — append the release's run with
   `bash scripts/bench-history.sh` and commit it with the release, so the
   next gate compares against it.
5. **Subtle perf work** — single-run benches bounce ±20–30% on load;
   the gate catches catastrophes, not drift. For claimed micro-wins,
   run benches multiple times and report the median, not one sample.
   To isolate a code change from host drift, interleave runs of the old
   and new tree rather than comparing against an older baseline:
   `BENCH_PAIRED_ROUNDS=15 bash scripts/bench-paired.sh [REF]` does that
   against the last release tag (or REF), each tree on its own pin, and
   prints the per-row paired median and how many rounds were slower.

Numbers or it didn't happen — every performance claim in the CHANGELOG
cites benchmark figures from this gate.

### Closeout Pass (before every minor/major bump)

1. **Full test suite** — all `.tcyr` pass, zero failures
2. **Benchmark gate** — run the mandatory [Benchmark Gate](#benchmark-gate-before-every-release--patch-minor-major): `scripts/bench-regression.sh` passes, deltas recorded in CHANGELOG, baseline appended to `history.csv`
3. **Dead code audit** — remove unused functions; record remaining floor in CHANGELOG
4. **Refactor pass** — consolidate the minor's additions where parallel codepaths accreted
5. **Code review pass** — walk diffs end-to-end for missed guards, ABI leaks, off-by-ones, silently-ignored errors
6. **Cleanup sweep** — stale comments, dead branches, unused includes, orphaned files
7. **Security re-scan** — quick grep for new `sys_system`, raw `syscall(` numbers, unchecked writes, unsanitized input, buffer size mismatches
8. **Downstream check** — `bash scripts/consumer-sweep.sh`: every repo that declares `[deps.agnostik]` builds and passes its tests against the in-flight tree, on its own pin, with no new agnostik-involved duplicate definitions
9. **Doc sync** — CHANGELOG, roadmap, `docs/development/state.md`, `docs/doc-health.md` (re-bucket any rows touched this cycle and refresh the at-a-glance counts), `SECURITY.md` (Supported Versions names the new line; audits table row), CLAUDE.md (if durable content changed)
10. **Version verify** — `VERSION`, `cyrius.cyml`, CHANGELOG header, intended git tag all match (tags are bare: `1.6.2`, not `v1.6.2`)
11. **Full build from clean** — `rm -rf build && cyrius lib sync && cyrius deps && CYRIUS_DCE=1 cyrius build src/main.cyr build/agnostik` passes clean

### Task Sizing

- **Low/Medium effort**: batch freely — multiple items per work loop cycle
- **Large effort**: small bites only — break into sub-tasks, verify each before moving to the next
- **If unsure**: treat it as large

### Refactoring Policy

- Refactor when the code tells you to — duplication, unclear boundaries, measured bottlenecks
- Never refactor speculatively. Wait for the third instance
- Every refactor must pass the same test + fuzz + benchmark gates as new code
- 3 failed attempts = defer and document — don't burn time in a rabbit hole

## Cyrius Conventions

Rules below were checked against the pinned compiler; when an older note elsewhere disagrees, trust these and re-probe if in doubt.

- Struct fields default to 8 bytes (`i64`), accessed via `load64` / `store64` with offset. Sub-byte widths (`i8`/`i16`/`i32`) are allowed when the value range fits — see `InjectionScores` and `AcceleratorFlags` (v1.1.1 — both are 5- and 9-byte structs of i8 fields respectively). When narrowing a struct, every accessor + setter must use the matching `load8`/`store8` (etc.) and direct `store64(s + N, v)` callers must migrate before the alloc shrinks (otherwise OOB-write).
- Allocate with `alloc()` (bump). The freelist (`fl_alloc` / `fl_free`) is not a declared dependency of this repo.
- Heap-allocate large buffers. A `var buf[N]` array is static storage in the binary image: a fn-local one adds N bytes, a global one reserves 8 bytes per unit (8·N).
- Lazy initialization pattern: `_stik_lazy_vec(ptr)` / `_stik_lazy_map(ptr)` (in `src/security.cyr`) for deferred collection creation
- **Private helpers are `_stik_`-prefixed.** Consumers compile agnostik in one unit with other libraries, where a same-named fn silently replaces another or, at a different arity, fails the build (`_hex_nibble` collided with sigil, `_json_int` / `_json_str` with ai-hwaccel). Every `_`-prefixed fn in a shipped module is `_stik_<name>`; CI's "Private helpers are namespaced" step enforces it. Test-harness helpers in `src/main.cyr` and `tests/` are exempt, since they never ship
- **`Result` is a value form (cyrius ≥ 6.6.0).** `Ok(v)` / `Err(e)` return a `(tag, payload)` register pair and allocate nothing. Always bind both halves — `var r_t, r = f();` (a single-variable bind is a compile error). Test the tag with `is_ok(r_t)` / `is_err_result(r_t)`; propagate with `f()?`. A fn returning a Result must return a pair on every path. A one-argument fn cannot receive a Result, because the payload never reaches a parameter — pass the tag too, as `result_print_agnostik_err(res_t, res)` does.
- **Tagged boxes**: `tagged_new(tag, value)` builds a 16-byte heap box for enums with data. Read a box only with `boxed_tag(b)` / `boxed_payload(b)`. The Result predicates (`is_ok` etc.) compile against a box and silently answer wrongly; `tag()` / `payload()` no longer exist.
- Trait objects via vtable dispatch: `trait_obj_new(vtable, data)`
- Function pointers via `fncall0` / `fncall1` / `fncall2` (inline asm)
- `#derive(Serialize)` generates correct `_to_json` — integers as bare numbers, `: Str` fields as quoted strings
- **Syscalls go through the stdlib `sys_*` wrappers** (`sys_getrandom`, `sys_write`, `sys_exit`, …), not raw `syscall(N, …)`. A raw number is x86_64-linux numbering: aarch64 and Windows translate only some numbers, and agnos numbers its own (audit F-022 / F-023). `src/` has no raw `syscall(` at all since 1.7.1, and the CI security scan fails on one; add a stdlib wrapper use instead.
- Enum values for constants — don't consume `gvar_toks` slots (4,096 initialized globals limit)
- **`var` is block-scoped.** A `var` declared inside `{ … }` ends at the `}` (using it afterwards is a compile error), and re-declaring an outer name inside a block creates a new variable that shadows it: `var t = 1; if (c) { var t = 5; } return t;` returns 1. To carry a value out of a block, declare it above the block and assign inside. Older notes calling `var` "function-scoped" are wrong.
- `break` in a while loop with `var` declarations: don't — flag + `continue` (see Rules)
- `match` and the compiler's other builtin names are reserved and cannot be identifiers; the compiler names the clash
- Always return a value (`return 0;`). A bare `return;` is a compile error in a fn that returns a struct or a vector
- Negative literals (`-1`) and mixed `&&` / `||` both compile correctly — C precedence for `&&` / `||` since cyrius 6.3.36. House style still writes `0 - N` and nests or parenthesizes mixed conditions; match the surrounding code
- The per-compilation-unit limit that matters is 4,096 globals with non-literal initializers (functions and variable-table entries are capped far higher). Counting rule: only a top-level `var NAME = <non-literal>;` (call / identifier / expression initializer) consumes an initialized-globals slot; a bare integer-literal init (`var x = 42;`) takes the static-init fast path and enum members are const-folded, so neither counts. See the cyrius guide's **Global Initializers** section (`docs/guides/cyrius-guide.md` in the cyrius repo)

## CI / Release

- **Toolchain pin**: `cyrius = "X.Y.Z"` field in `cyrius.cyml [package]`. CI and release both read this; no hardcoded version strings in YAML.
- **Verified toolchain install**: CI and release install cyrius through `scripts/ci-install-cyrius.sh`, which checks `SHA256SUMS.sig` against the cyrius release key pinned in that script (stock OpenSSL Ed25519), then the tarball against `SHA256SUMS`, before unpacking. It fails closed. If cyrius rotates its key, confirm the new one out of band (the cyrius repo's `keys/` and `install.sh`) before updating the pin.
- **Toolchain binary names**: the compiler is `cycc` and the aarch64 cross-compiler `cycc_aarch64` (renamed from `cc5` / `cc5_aarch64` at cyrius 6.0.0; no 6.x release tarball ships the old names). A workflow step that probes for a binary must use these names — a probe for a missing name makes its step skip silently.
- **Dead code elimination**: every `cyrius build` in CI and release runs with `CYRIUS_DCE=1`. Binary size is a release metric — track it.
- **Tag filter**: release workflow triggers on semver tags only (`v1.2.3` or `1.2.3`; this repo's tags are bare `1.2.3`). Non-numeric tags do not ship a release.
- **Version-verify gate**: release asserts `VERSION == cyrius.cyml version == git tag` before building. Mismatch fails the run.
- **Lint step**: CI runs `cyrius lint` per source file. Warnings fail.
- **Workflow layout**:
  - `.github/workflows/ci.yml` — stdlib sync, fmt, lint, vet, DCE build, type-check, API-surface snapshot, documentation debt, dist-bundle sync, aarch64 cross-build, tests, the suite again on aarch64 under qemu-user, bench + the paired bench gate against the last release tag (`scripts/bench-paired.sh`; the checkout fetches full history for the tags), security scan (including no raw `syscall(` and `_stik_`-prefixed private helpers), required-docs and version checks; reusable via `workflow_call`
  - `.github/workflows/release.yml` — version gate → CI gate → DCE builds (x86_64, aarch64) → artifacts (source tarball, bundled `.cyr`, DCE binaries, `cyrius.lock` when one exists, SHA256SUMS)
- **Concurrency**: CI uses `cancel-in-progress: true` keyed on workflow + ref — only the latest push is tested.

## Docs

- [`docs/adr/`](docs/adr/) — architecture decision records, written from the template in `docs/adr/README.md`. *Why did we choose X over Y?*
- [`docs/architecture/`](docs/architecture/) — non-obvious constraints and quirks. *What can't I derive from the code alone?*
- [`docs/audit/`](docs/audit/) — security audit reports (`YYYY-MM-DD-audit.md`); frozen once superseded.
- [`docs/benchmarks/history.csv`](docs/benchmarks/history.csv) — benchmark-gate baseline history.
- [`docs/api-surface.snapshot`](docs/api-surface.snapshot) — the public-fn surface `scripts/api-surface.sh check` diffs against.
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — pinned future slots and the unpinned backlog; shipped work leaves it for the CHANGELOG.
- [`docs/development/state.md`](docs/development/state.md) — **live state snapshot, refreshed every release**.
- [`docs/development/issues/`](docs/development/issues/) — records of upstream (cyrius) issues; move to `archive/` once fixed upstream.
- [`docs/doc-health.md`](docs/doc-health.md) — **doc-currency ledger** (fresh / stale / archived per file). Refreshed in place whenever docs are touched.
- [`CHANGELOG.md`](CHANGELOG.md) — source of truth for all changes.

New quirks land in `docs/architecture/` as numbered items (`NNN-kebab-case.md`). New decisions land in `docs/adr/` using the template. **Never renumber either series.**

**Required root files** (CI checks them): README.md, CHANGELOG.md, CLAUDE.md, CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md, LICENSE, VERSION, cyrius.cyml — plus `docs/development/state.md` and `docs/development/roadmap.md`.

**Not yet present**: the first-party documentation standard also expects `docs/guides/` (task-oriented how-tos) and `docs/examples/` (runnable examples); neither exists here yet. When earned: `docs/sources.md` (source citations), `docs/proposals/` (pre-ADR drafts), `docs/api/` (curated public-surface reference), `standards/`, `compliance/`, `faq.md`.

**`.gitignore`**: the repo's own file is authoritative. `dist/` is **tracked** — `dist/agnostik.cyr` is the bundle consumers vendor, and CI fails when it drifts from `src/` — so the ecosystem template's `/dist/` line does not apply here. `lib/**/*.cyr` is ignored, all of it, subdirectories included (`bayan` has pulled in `lib/unicode/` since cyrius 6.7.13): `lib/` is vendored by `cyrius lib sync` + `cyrius deps`, and the committed `cyrius.lock` pins its hashes. (The template's `!lib/k*.cyr` exception kept an orphan `lib/keccak.cyr` tracked until 1.7.1.)

## CHANGELOG Format

Follow [Keep a Changelog](https://keepachangelog.com/). In-flight work goes under `## [Unreleased]` until the user cuts a release. Performance claims **must** include benchmark numbers, and every release entry carries the benchmark gate's delta table. Breaking changes get a **Breaking** section with migration guide. Security fixes get a **Security** section with CVE references where applicable.
