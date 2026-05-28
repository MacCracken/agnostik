# Contributing to Agnostik

## Setup

1. Install the [Cyrius toolchain](https://github.com/MacCracken/cyrius) — version pinned in `cyrius.cyml` (`[package].cyrius`).
2. Ensure `cc5`, `cyrlint`, and `cyrfmt` are on your PATH (`$HOME/.cyrius/bin/`).
3. Clone the repo and verify:
   ```sh
   cyrius lib sync                              # copy version-pinned stdlib snapshot into lib/ (6.0.x)
   cyrius deps                                  # resolve git deps into lib/ (stdlib comes from lib sync)
   cyrius build src/main.cyr build/agnostik     # build the test harness
   ```

## Development Workflow

1. Fork and create a feature branch.
2. Make changes in Cyrius (`.cyr` files in `src/`).
3. Build:
   ```sh
   cyrius build src/main.cyr build/agnostik
   ```
4. Run tests (each file is its own harness):
   ```sh
   for t in tests/tcyr/*.tcyr; do cyrius test "$t"; done
   ```
5. Run benchmarks:
   ```sh
   cyrius bench tests/bcyr/agnostik.bcyr
   ```
6. Format (write canonical form back):
   ```sh
   for f in src/*.cyr tests/tcyr/*.tcyr tests/bcyr/*.bcyr; do cyrfmt --write "$f"; done
   ```
7. Lint (must be clean — CI fails on any warning):
   ```sh
   for f in src/*.cyr tests/tcyr/*.tcyr tests/bcyr/*.bcyr; do cyrlint "$f"; done
   ```
8. Type-check (call-site `: Str` annotations):
   ```sh
   CYRIUS_TYPE_CHECK=1 cyrius build src/main.cyr build/agnostik
   ```
9. API surface gate (catches unintended public-fn drift):
   ```sh
   scripts/api-surface.sh check       # if intentional: scripts/api-surface.sh update
   ```
10. Bench-regression gate (catches catastrophic perf regressions):
    ```sh
    scripts/bench-regression.sh        # ack via [bench-regression-ack] in commit message
    ```
11. Regenerate the dist bundle (CI verifies in-sync):
    ```sh
    cyrius distlib
    ```
12. Submit PR.

## Code Standards

- Zero panic in library code — use `Result` (`Ok` / `Err`) for fallible operations.
- All public enums must have `*_name(t)` (string representation). Roundtrip
  (`*_parse(s)`) where the value is parsed from external input.
- **Serde split** (per [ADR-002](docs/adr/002-derive-serialize-7-of-9.md)):
  - Trivial all-int structs use `#derive(Serialize)` — cyrius emits
    `<Struct>_to_json` + `<Struct>_from_json_str`. Add `: i64` (or narrower)
    annotations to every field.
  - Structs with custom shape (UUID stringification, enum-name lookup,
    null-Str handling) keep hand-written `_to_json`/`_from_json` impls — the
    cyrius derive codegen can't replicate those today.
  - Either way, JSON output is **compact** (`{"k":v,"k":v}` no spaces) for
    library uniformity.
- All parse functions must return `Result` and have roundtrip tests.
- Every parser added under `src/` must also be wired into the fuzz harness at
  `tests/tcyr/test_v112_fuzz.tcyr` (deterministic xorshift64; survival
  contract: no crash on any byte sequence).
- Performance claims must include benchmark numbers (before/after) — see
  `docs/benchmarks/history.csv`.
- Struct fields default to 8 bytes (`i64`), accessed via `load64` / `store64`
  with offset. **Sub-byte widths** (`i8`/`i16`/`i32`) are allowed when the
  value range fits — see `InjectionScores` and `AcceleratorFlags` for the
  pattern. When you switch a struct from `i64` to a narrow width, update
  every accessor + setter to use `load8`/`store8` (or the matching width)
  and search the codebase for direct `store64(s + N, v)` callers — those
  OOB-write the now-smaller alloc.
- Heap allocation: `alloc()` (bump) for long-lived data, `fl_alloc()` /
  `fl_free()` (freelist) for individual lifetimes.
- Comments inside `#derive(Serialize)` struct bodies break cyrius 5.10.x's
  codegen (see
  [`docs/development/issues/cyrius-derive-comments-in-struct-body-2026-05-10.md`](docs/development/issues/cyrius-derive-comments-in-struct-body-2026-05-10.md)).
  Keep comments above the directive until upstream ships the fix.

## Adding New Types

1. Add the module to `src/` and include it from `src/lib.cyr`.
2. Register the module in `cyrius.cyml` under `[lib].modules` (order matters — error first, then types, then dependents).
3. Add tests in a dedicated `tests/tcyr/<topic>.tcyr` file.
4. Add benchmarks for hot paths in `tests/bcyr/agnostik.bcyr`.
5. Update `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/).
6. Run `cyrius distlib` and commit the regenerated `dist/agnostik.cyr` alongside src changes.
