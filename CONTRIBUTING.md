# Contributing to Agnostik

## Setup

1. Install the [Cyrius toolchain](https://github.com/MacCracken/cyrius) — version pinned in `cyrius.cyml` (`[package].cyrius`).
2. Ensure `cc5`, `cyrlint`, and `cyrfmt` are on your PATH (`$HOME/.cyrius/bin/`).
3. Clone the repo and verify:
   ```sh
   cyrius deps                                  # resolve stdlib + git deps into lib/
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
   for f in src/*.cyr tests/tcyr/*.tcyr tests/bcyr/*.bcyr; do cyrius lint "$f"; done
   ```
8. Regenerate the dist bundle (CI verifies in-sync):
   ```sh
   cyrius distlib
   ```
9. Submit PR.

## Code Standards

- Zero panic in library code — use `Result` (`Ok` / `Err`) for fallible operations.
- All public enums must have `*_name(t)` (string representation) and `*_parse(s)` (roundtrip) functions.
- All serializable structs must have hand-written `<Type>_to_json(ptr, sb)` and `<Type>_from_json(src)` adapters. Do **not** add `#derive(Serialize)` alongside hand-written adapters — last-define-wins makes the derive output silent dead code (see audit F-011).
- All parse functions must return `Result` and have roundtrip tests.
- Performance claims must include benchmark numbers (before/after) — see `docs/benchmarks/history.csv`.
- All struct fields are 8 bytes (`i64`), accessed via `load64` / `store64` with offset.
- Heap allocation: `alloc()` (bump) for long-lived data, `fl_alloc()` / `fl_free()` (freelist) for individual lifetimes.

## Adding New Types

1. Add the module to `src/` and include it from `src/lib.cyr`.
2. Register the module in `cyrius.cyml` under `[lib].modules` (order matters — error first, then types, then dependents).
3. Add tests in a dedicated `tests/tcyr/<topic>.tcyr` file.
4. Add benchmarks for hot paths in `tests/bcyr/agnostik.bcyr`.
5. Update `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/).
6. Run `cyrius distlib` and commit the regenerated `dist/agnostik.cyr` alongside src changes.
