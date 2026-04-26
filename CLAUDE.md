# Agnostik — Claude Code Instructions

> **Core rule**: this file is **preferences, process, and procedures** — durable rules that change rarely. Volatile state (current version, binary sizes, test counts, in-flight work, consumers, verification hosts) lives in [`docs/development/state.md`](docs/development/state.md), bumped every release. Do not inline state here — inlined state rots within a minor.

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
> shipped releases, consumers, verification hosts. Refreshed every release.
> Historical release narrative lives in [`CHANGELOG.md`](CHANGELOG.md).

This file (`CLAUDE.md`) is durable rules.

## Scaffolding

Project was ported from a Rust crate. **Do not manually create project structure** — use `cyrius` tools. If the tools are missing something, fix the tools.

## Quick Start

```bash
cyrius deps                                     # resolve stdlib + git deps into lib/
cyrius build src/main.cyr build/agnostik        # build the test harness binary
cyrius test tests/tcyr/agnostik.tcyr            # run the unit-test harness
cyrius bench tests/bcyr/agnostik.bcyr           # benchmarks
cyrius lint src/*.cyr                           # static checks
cyrius audit                                    # full check: self-host, test, fmt, lint
CYRIUS_DCE=1 cyrius build ...                   # dead-code-eliminated release build
```

## Key Principles

- **Correctness is the optimum sovereignty** — if it's wrong, you don't own it; the bugs own you
- Test after EVERY change, not after the feature is "done"
- ONE change at a time — never bundle unrelated changes
- Research before implementation — check vidya for existing patterns
- Programs call `main()` at top level: `var exit_code = main(); syscall(60, exit_code);`
- **Build with `cyrius build`, never raw `cat file | cc5`** — the manifest auto-resolves deps and prepends includes
- Source files only need project includes — stdlib / external deps auto-resolve from `cyrius.cyml`
- Every buffer declaration is a contract: `var buf[N]` = N **bytes**, not N entries
- Fuzz every parser path — edge cases get invariants, not assertions
- Benchmark before claiming perf — numbers or it didn't happen
- **Own the stack** — agnostik IS the stack's type vocabulary; consumers should not redefine these
- All public enums must have `*_name()` (string representation) and `*_parse(s)` (roundtrip)
- Every serializable struct must have a `*_to_json(ptr, sb)` function (or `#derive(Serialize)`)
- Zero panic in library code — use `Result` (`Ok` / `Err`) for fallible operations
- Every parse function must have a roundtrip test

## Rules (Hard Constraints)

- **Read the genesis repo's CLAUDE.md first** — [agnosticos/CLAUDE.md](https://github.com/MacCracken/agnosticos/blob/main/CLAUDE.md)
- **Do not commit or push** — the user handles all git operations
- **NEVER use `gh` CLI** — use `curl` to the GitHub API if needed
- Do not add unnecessary dependencies
- Do not skip tests before claiming changes work
- Do not skip fuzz / benchmark verification before claiming a feature works
- Do not use `sys_system()` with unsanitized input — command injection risk
- Do not trust external data (file content, network input, user args) without validation
- Do not use `break` in while loops with `var` declarations — use flag + `continue`
- Do not add Cyrius stdlib includes in individual src files — the manifest resolves them
- Do not hardcode toolchain versions in CI YAML — the `cyrius = "X.Y.Z"` pin in `cyrius.cyml` is the only source of truth
- Do not break public API without a major version bump (consumer count is large — every component depends on agnostik)

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
6. **Documentation** — update CHANGELOG, roadmap, `docs/development/state.md`, any ADR the change earned
7. **Version check** — `VERSION`, `cyrius.cyml`, CHANGELOG header in sync
8. **Return to step 1**

### Security Hardening (before every release)

Every release runs a security audit pass. Minimum:

1. **Input validation** — every function accepting external data validates bounds, types, ranges
2. **Buffer safety** — every `var buf[N]` verified; N is **bytes**, max access < N, no adjacent-variable overflow
3. **Syscall review** — every syscall validated: args checked, returns handled, error paths complete
4. **Pointer validation** — no raw pointer dereference of untrusted input without bounds
5. **No command injection** — use `exec_vec()` with explicit argv; never `sys_system()` with unsanitized input
6. **No path traversal** — file paths from external input validated, no `../` escape
7. **Known CVE review** — check dependencies and patterns against current CVE databases
8. **Document findings** — all issues in `docs/audit/YYYY-MM-DD-audit.md`

Severity levels: **CRITICAL** (remote / privilege escalation), **HIGH** (moderate effort), **MEDIUM** (specific conditions), **LOW** (defense-in-depth).

### Closeout Pass (before every minor/major bump)

1. **Full test suite** — all `.tcyr` pass, zero failures
2. **Benchmark baseline** — `cyrius bench`, save CSV; compare against prior closeout
3. **Dead code audit** — remove unused functions; record remaining floor in CHANGELOG
4. **Refactor pass** — consolidate the minor's additions where parallel codepaths accreted
5. **Code review pass** — walk diffs end-to-end for missed guards, ABI leaks, off-by-ones, silently-ignored errors
6. **Cleanup sweep** — stale comments, dead branches, unused includes, orphaned files
7. **Security re-scan** — quick grep for new `sys_system`, unchecked writes, unsanitized input, buffer size mismatches
8. **Downstream check** — every consumer in `state.md` still builds and passes tests against the new version
9. **Doc sync** — CHANGELOG, roadmap, `docs/development/state.md`, CLAUDE.md (if durable content changed)
10. **Version verify** — `VERSION`, `cyrius.cyml`, CHANGELOG header, intended git tag all match
11. **Full build from clean** — `rm -rf build && cyrius deps && CYRIUS_DCE=1 cyrius build` passes clean

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

- All struct fields are 8 bytes (`i64`), accessed via `load64` / `store64` with offset
- Heap allocation via `fl_alloc()` / `fl_free()` (freelist) for data with individual lifetimes
- Bump allocation via `alloc()` for long-lived data (vec, str internals)
- Lazy initialization pattern: `_lazy_vec(ptr)` / `_lazy_map(ptr)` for deferred collection creation
- Tagged unions via `tagged_new(tag, value)` for enums with data
- Trait objects via vtable dispatch: `trait_obj_new(vtable, data)`
- Function pointers via `fncall0` / `fncall1` / `fncall2` (inline asm)
- `#derive(Serialize)` generates correct `_to_json` — integers as bare numbers, `: Str` fields as quoted strings
- Enum values for constants — don't consume `gvar_toks` slots (256 initialized globals limit)
- Heap-allocate large buffers — `var buf[256000]` bloats the binary by 256 KB
- `break` in while loops with `var` declarations is unreliable — use flag + `continue`
- No negative literals — write `(0 - N)` not `-N`
- No mixed `&&` / `||` in one expression — nest `if` blocks instead
- `match` is reserved — don't use as a variable name
- `return;` without value is invalid — always `return 0;`
- All `var` declarations are function-scoped — no block scoping
- Max limits per compilation unit: 4,096 variables, 1,024 functions, 256 initialized globals

## CI / Release

- **Toolchain pin**: `cyrius = "X.Y.Z"` field in `cyrius.cyml [package]`. CI and release both read this; no hardcoded version strings in YAML.
- **Dead code elimination**: every `cyrius build` in CI and release runs with `CYRIUS_DCE=1`. Binary size is a release metric — track it.
- **Tag filter**: release workflow triggers on semver tags only (`v1.2.3` or `1.2.3`). Non-numeric tags do not ship a release.
- **Version-verify gate**: release asserts `VERSION == cyrius.cyml version == git tag` before building. Mismatch fails the run.
- **Lint step**: CI runs `cyrius lint` per source file. Warnings fail.
- **Workflow layout**:
  - `.github/workflows/ci.yml` — build, lint, fmt, vet, test, bench; reusable via `workflow_call`
  - `.github/workflows/release.yml` — version gate → CI gate → DCE build → artifacts (source tarball, bundled `.cyr`, DCE binary, `cyrius.lock`, SHA256SUMS)
- **Concurrency**: CI uses `cancel-in-progress: true` keyed on workflow + ref — only the latest push is tested.

## Docs

- [`docs/adr/`](docs/adr/) — architecture decision records. *Why did we choose X over Y?*
- [`docs/architecture/`](docs/architecture/) — non-obvious constraints and quirks. *What can't I derive from the code alone?*
- [`docs/guides/`](docs/guides/) — task-oriented how-tos.
- [`docs/examples/`](docs/examples/) — runnable examples.
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — completed, backlog, future, v1.0 criteria.
- [`docs/development/state.md`](docs/development/state.md) — **live state snapshot, refreshed every release**.
- [`docs/audit/`](docs/audit/) — security audit reports (`YYYY-MM-DD-audit.md`).
- [`CHANGELOG.md`](CHANGELOG.md) — source of truth for all changes.

New quirks land in `docs/architecture/` as numbered items (`NNN-kebab-case.md`). New decisions land in `docs/adr/` using the template. **Never renumber either series.**

## Documentation Structure

```
Root files (required):
  README.md, CHANGELOG.md, CLAUDE.md, CONTRIBUTING.md,
  SECURITY.md, CODE_OF_CONDUCT.md, LICENSE,
  VERSION, cyrius.cyml

docs/ (minimum):
  adr/ — architectural decision records
  architecture/ — non-obvious invariants
  guides/ — task-oriented how-tos
  examples/ — runnable examples
  development/
    roadmap.md — completed, backlog, future
    state.md — live state snapshot (volatile; release-bumped)

docs/ (when earned):
  audit/ — security audit reports (YYYY-MM-DD-audit.md)
  sources.md — source citations for algorithms/formulas
  proposals/ — pre-ADR design drafts
  api/ — curated public-surface reference
  standards/, compliance/, faq.md — as applicable
```

## .gitignore (Required)

```gitignore
# Build
/build/
/dist/

# Resolved deps (auto-generated by cyrius deps)
lib/*.cyr
!lib/k*.cyr

# Release / toolchain artifacts
cyrius-*.tar.gz
*.tar.gz
SHA256SUMS

# IDE
.idea/
.vscode/
*.swp
*~

# OS
.DS_Store
Thumbs.db
```

## CHANGELOG Format

Follow [Keep a Changelog](https://keepachangelog.com/). Performance claims **must** include benchmark numbers. Breaking changes get a **Breaking** section with migration guide. Security fixes get a **Security** section with CVE references where applicable.
