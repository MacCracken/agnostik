# Agnostik Roadmap

## Status

**v1.0.0** — first stable release. 12 modules, 653 test assertions across 9 test files, 25 benchmarks, zero external dependencies, Cyrius 5.7.12. Full audit pass (F-001..F-011 closed; F-006 resolved upstream in cyrius 5.7.7). See [`state.md`](state.md) for the live snapshot, [`../audit/2026-04-26-audit.md`](../audit/2026-04-26-audit.md) for the audit report, and [`../../CHANGELOG.md`](../../CHANGELOG.md) for the release notes.

## v1.0.0 (completed)

- ✅ Toolchain refresh — Cyrius 3.2.5 → 5.7.12; build pipeline manifest-driven (`cyrius build` / `cyrius deps`).
- ✅ Manifest format — `cyrius.toml` → `cyrius.cyml`; version pulled from `VERSION` via `${file:VERSION}`.
- ✅ Layout aligned with the vidya / yukti gold standard — `tests/tcyr/`, `tests/bcyr/`, `dist/agnostik.cyr` tracked, CI / release workflows reusable.
- ✅ Security audit — 11 findings closed (CSPRNG, JSON escape/sign/overflow/string-boundary/null-probe/whitespace, segment validation, derive-collision dead-code, line length).
- ✅ Documentation set — CLAUDE.md durable rules; `docs/development/state.md` volatile state; ADR / architecture / audit / issues directories scaffolded.
- ✅ CI gates — fmt, lint, vet, dist-bundle sync, ELF verify, aarch64 cross-build (best-effort), test, bench, security scan, docs check.

## Engineering Backlog

(none currently active — open an issue or send a PR)

## v1.1.x — Cyrius 5.10+ modernization (pinned)

The cyrius v5.10.x type-system arc closes consumer-pain shapes that
agnostik works around today. Items below are pinned for v1.1.x; each
lists its trigger and the surface it eliminates.

### Revive `#derive(Serialize)` — eliminates 18 hand-written serde fns

- **Surface today**: 9 structs across `agent.cyr` (ResourceLimits,
  ResourceUsage, AgentInfo, AgentStats), `config.cyr`
  (EdgeResourceOverrides), `validation.cyr` (InjectionScores),
  `telemetry.cyr` (TelemetryConfig), `llm.cyr` (TokenUsage),
  `hardware.cyr` (AcceleratorFlags) carry hand-written
  `<Struct>_to_json` + `<Struct>_from_json` pairs. ~18 functions of
  rote field-by-field JSON wiring.
- **Why removed pre-1.0**: F-011 (audit 2026-04-26) — old compiler
  emitted dead-code derive stubs that shadowed the hand-written
  impls but bloated the binary. cyrius v5.9.30/.31/.36 fixed the
  typed-i64, narrow-int, and API-rename bugs; v5.9.39 closed the
  Mach-O ARM64 fn-pointer ASLR cascade end-to-end. Str-field
  positional-init landed at v5.10.7; the JSON escape fix (quote /
  backslash / control chars) shipped at v5.10.8.
- **Trigger**: **ready now** under opt-in `CYRIUS_TYPE_CHECK=1`
  (which 1.0.3 wires into CI). The originally-pinned trigger of a
  5.10.5 default-on flip got *attempted and reverted* upstream — a
  separate generic-i64-param false-positive shape blocks the flip
  indefinitely. That doesn't gate this work; the type-check that
  matters (Str-field codegen + JSON escape) is solid since 5.10.8.
- **Risk**: derive output must round-trip identically to current
  hand-written form (`{"id":42,"name":"alice"}` exact bytes incl.
  field order). Plan: capture a golden-corpus snapshot of every
  `<Struct>_to_json` output against the current hand-written impls,
  then swap to derive and diff. Hand-written impls retain F-002 /
  F-003 / F-008 audit fixes; verify derive emits equivalent
  bytes for the security-relevant cases (escaped quotes, negative
  ints, max-int boundaries).

### Adopt `#derive(accessors)` — eliminates ~470 rote getters/setters

- **Surface today**: every domain struct ships hand-written
  `<prefix>_<field>(s)` getters and `<prefix>_set_<field>(s, v)`
  setters. ~470 single-line `load64`/`store64` wrappers.
- **Why now**: cyrius generates these correctly for typed fields
  (8-byte default + i8/i16/i32 sub-byte widths). The agnostik
  prefix convention (`amsg_*`, `aentry_*`, `secctx_*`) doesn't match
  the derive default `<Struct>_<field>` shape — reviving requires
  either a rename (breaking — every consumer depends on the prefix
  names) or a derive variant that takes a custom prefix.
- **Trigger**: upstream feature — derive-with-prefix support, OR a
  consumer pin willing to absorb the rename cost. Hold until one
  surfaces. *Park, don't promote yet.*

### Sub-byte field widths — shrinks several hot structs

- **Targets**: `InjectionScores` (5×i64 score fields, each 0..100 →
  i8: 40 B → 5 B); `AcceleratorFlags` (9×i64 booleans → i8 each:
  72 B → 9 B); `ResourceUsage`, `TokenUsage` similar opportunities
  where a field's value range fits sub-i64.
- **Trigger**: paired with the derive revival above (cyrius
  generates width-correct emit for sub-byte fields per v5.9.36).
  Independent of derive otherwise — current hand-written serde
  fns can stay if widths are added directly to struct fields.
- **Risk**: ABI break if any consumer reads these structs by raw
  offset (none do today — every consumer goes through the public
  accessor functions). Verify via consumer-build sweep.

### Pointer-to-struct dot syntax — `s.data` / `s.len`

- **Surface today**: every parse / format function spells out
  `str_data(s)` and `str_len(s)`. v5.8.17's pointer-to-struct dot
  syntax (`s.data`, `s.len`) is now the idiomatic shape; v5.10.4
  closed type inference through `var x = f(...)` so the chain
  works without explicit annotation. Adopt selectively where
  readability wins — not a wholesale rewrite.
- **Trigger**: ready now (no upstream gate). Light-touch refactor;
  bundle with the next slot that already touches the affected file.

  > Note: the previously-bundled `println(s)` overload item moved
  > out of this section — the 1.0.2 boilerplate-drop already
  > cleared the raw `syscall(1, 1, ...)` write sites, and the
  > remaining `syscall(1, 2, ...)` calls in `types.cyr` /
  > `error.cyr` are intentional stderr writes that `println`
  > (stdout) wouldn't replace.

### `_json_int` Result return signature — disambiguate missing-vs-zero

- **Surface today**: `_json_int(src: Str, key: Str)` returns 0 for
  both "key missing" and "literal `0`". F-003 (audit 2026-04-26)
  hardened the integer-overflow path but left the missing-key
  ambiguity in place. Every consumer treats the int as load-bearing.
- **Trigger**: a consumer surfacing the ambiguity as a real bug.
  None today. Hold.

### JSON `\uXXXX` Unicode escape decoder

- **Surface today**: `src/types.cyr:_json_str` documented as ASCII
  + common escapes only (`\" \\ \/ \n \t \r \b \f`). `\uXXXX`
  passes through as the literal 6 bytes.
- **Trigger**: a consumer carrying non-ASCII text through serde.
  None today (every AGNOS type is ASCII identifier / version /
  enum / hash). Hold.

## Future Considerations

(items above this line are pinned; below are forward-looking).

- **Compile-time wins** — when cyrius 5.10.10+ ships its lex/fixup
  optimizations, profile agnostik's build with `CYRIUS_PROF=1` and
  see if any agnostik-side patterns (heavy include count, large
  init block) disproportionately load specific phases. Action only
  if a measurable bottleneck surfaces.
