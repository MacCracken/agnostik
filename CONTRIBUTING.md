# Contributing to Agnostik

## Setup

1. Install the [Cyrius toolchain](https://github.com/MacCracken/cyrius) (v3.2.1+)
2. Ensure `cc2` and `cyrfmt` are on your PATH (`$HOME/.cyrius/bin/`)
3. Clone the repo and verify: `cat src/main.cyr | cc2 > /dev/null`

## Development Workflow

1. Fork and create a feature branch
2. Make changes in Cyrius (`.cyr` files in `src/`)
3. Build and test:
   ```sh
   cat src/main.cyr | cc2 > build/agnostik_test && chmod +x build/agnostik_test
   ./build/agnostik_test
   ```
4. Run benchmarks:
   ```sh
   cat benches/bench.bcyr | cc2 > build/agnostik_bench && chmod +x build/agnostik_bench
   ./build/agnostik_bench
   ```
5. Format: `cyrfmt src/*.cyr`
6. Security check: `grep -rn 'syscall(59' src/ | grep -v "# "` (must be clean)
7. Submit PR

## Code Standards

- Zero panic in library code — use Result (Ok/Err) for fallible operations
- All public enums must have `*_name()` functions for string representation
- All serializable structs must have manual `*_to_json(ptr, sb)` implementations
- All parse functions must return Result and have roundtrip tests
- Performance claims must include benchmark numbers (before/after)
- All struct fields are 8 bytes (i64), accessed via `load64`/`store64` with offset
- Heap allocation via `alloc()` (bump allocator) — no individual free

## Adding New Types

1. Add the module to `src/` and include it in `src/lib.cyr`
2. Register the module in `cyrius.toml` under `[lib] modules`
3. Add tests in `src/main.cyr` or a dedicated `.tcyr` file in `tests/`
4. Add benchmarks for hot paths in `benches/bench.bcyr`
5. Update CHANGELOG.md following [Keep a Changelog](https://keepachangelog.com/)
