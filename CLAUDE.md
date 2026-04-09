# Agnostik — Claude Code Instructions

## Project Identity

**Agnostik** (agnostic) — Shared types and domain primitives for AGNOS

- **Type**: Flat library (Cyrius)
- **License**: GPL-3.0-only
- **Version**: `0.97.0` (pre-release, targeting v1.0.0)
- **Genesis repo**: [agnosticos](https://github.com/MacCracken/agnosticos)
- **Philosophy**: [AGNOS Philosophy & Intention](https://github.com/MacCracken/agnosticos/blob/main/docs/philosophy.md)
- **Standards**: [First-Party Standards](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-standards.md)
- **Recipes**: [zugot](https://github.com/MacCracken/zugot) — takumi build recipes
- **Compiler**: Cyrius cc2 (`$HOME/.cyrius/bin/cc2`)

## Consumers

Every AGNOS component: daimon, hoosh, agnoshi, aegis, argonaut, sigil, ark, kavach, stiva, nein, and all consumer apps.

## Development Process

### P(-1): Scaffold Hardening (before any new features)

0. Read roadmap, CHANGELOG, and open issues — know what was intended before auditing what was built
1. Test + benchmark sweep of existing code
2. Cleanliness check: compile clean (`cat src/main.cyr | cc2 > /dev/null`), security scan, docs check, version consistency
3. Get baseline benchmarks (`./scripts/bench.sh`)
4. Internal deep review — gaps, optimizations, security, logging/errors, docs
5. External research — domain completeness, missing capabilities, best practices, world-class accuracy
6. Cleanliness check — must be clean after review
7. Additional tests/benchmarks from findings
8. Post-review benchmarks — prove the wins
9. Documentation audit — ADRs, source citations, guides, examples (see Documentation Standards in first-party-standards.md)
10. Repeat if heavy

### Work Loop / Working Loop (continuous)

1. Work phase — new features, roadmap items, bug fixes
2. Cleanliness check: compile clean, security scan, docs present, versions in sync
3. Test + benchmark additions for new code
4. Run benchmarks (`./scripts/bench.sh`)
5. Internal review — performance, memory, security, throughput, correctness
6. Cleanliness check — must be clean after review
7. Deeper tests/benchmarks from review observations
8. Run benchmarks again — prove the wins
9. If review heavy → return to step 5
10. Documentation — update CHANGELOG, roadmap, docs, ADRs for design decisions, source citations for algorithms/formulas, update docs/sources.md, guides and examples for new API surface, verify recipe version in zugot
11. Version check — VERSION, cyrius.toml, recipe (in zugot) all in sync
12. Return to step 1

### Build & Test Commands

```sh
# Set compiler
CC="${CC:-$HOME/.cyrius/bin/cc2}"

# Build test binary
cat src/main.cyr | "$CC" > build/agnostik_test && chmod +x build/agnostik_test

# Run tests
./build/agnostik_test

# Build and run benchmarks
cat benches/bench.bcyr | "$CC" > build/agnostik_bench && chmod +x build/agnostik_bench
./build/agnostik_bench

# Build library (for consumers)
cat src/lib.cyr | "$CC" > build/agnostik_lib

# Cleanliness checks
cat src/main.cyr | "$CC" > /dev/null 2>&1    # compile clean (no warnings)
grep -rn 'syscall(59' src/ | grep -v "# "     # security: no raw execve
```

### Task Sizing

- **Low/Medium effort**: Batch freely — multiple items per work loop cycle
- **Large effort**: Small bites only — break into sub-tasks, verify each before moving to the next. Never batch large items together
- **If unsure**: Treat it as large. Smaller bites are always safer than overcommitting

### Refactoring

- Refactor when the code tells you to — duplication, unclear boundaries, performance bottlenecks
- Never refactor speculatively. Wait for the third instance before extracting an abstraction
- Refactoring is part of the work loop, not a separate phase. If a review (step 5) reveals structural issues, refactor before moving to step 6
- Every refactor must pass the same cleanliness + benchmark gates as new code

### Key Principles

- Never skip benchmarks
- Own the stack — agnostik IS the stack's type vocabulary
- All public enums should have `*_name()` functions for string representation
- Every serializable struct must have a `*_to_json(ptr, sb)` function
- Consumers include modules via `include "src/lib.cyr"` or individual files
- Zero panic in library code — use Result (Ok/Err) for fallible operations
- All parse functions must have roundtrip tests

### Cyrius Conventions

- All struct fields are 8 bytes (i64), accessed via `load64`/`store64` with offset
- Heap allocation via `alloc()` (bump allocator) — no individual free
- Lazy initialization pattern: `_lazy_vec(ptr)` and `_lazy_map(ptr)` for deferred collection creation
- Tagged unions via `tagged_new(tag, value)` for enums with data
- Trait objects via vtable dispatch: `trait_obj_new(vtable, data)`
- Function pointers via `fncall0`/`fncall1`/`fncall2` (inline asm)
- `#derive(Serialize)` generates correct `_to_json` since Cyrius v3.2.3 — integers as bare numbers, `: Str` fields as quoted strings. Manual `*_to_json` implementations can be replaced but are kept for backwards compatibility
- Compiler limit: 2048 functions per compilation unit (expanded from 1024 in Cyrius 3.2.2)

## DO NOT

- **Do not commit or push** — the user handles all git operations
- **NEVER use `gh` CLI** — use `curl` to GitHub API only
- Do not add unnecessary dependencies
- Do not break backward compatibility without a major version bump
- Do not skip benchmarks before claiming performance improvements

## Documentation Structure

```
Root files (required):
  README.md, CHANGELOG.md, CLAUDE.md, CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md, LICENSE

docs/ (required):
  architecture/overview.md — module map, data flow, consumers
  development/roadmap.md — completed, backlog, future, v1.0 criteria

docs/ (when earned):
  adr/ — architectural decision records
  guides/ — usage guides, integration patterns
  examples/ — worked examples
  standards/ — external spec conformance
  compliance/ — regulatory, audit, security compliance
  sources.md — source citations for algorithms/formulas (required for science/math crates)
```

## CHANGELOG Format

Follow [Keep a Changelog](https://keepachangelog.com/). Performance claims MUST include benchmark numbers. Breaking changes get a **Breaking** section with migration guide.
