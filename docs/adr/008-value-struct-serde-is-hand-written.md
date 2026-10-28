# ADR-008: The seven value structs' serde is hand-written, bounded and range-checked

**Status:** Accepted
**Date:** 2026-10-28
**Slot:** v1.12.3
**Supersedes:** [ADR-002](002-derive-serialize-7-of-9.md) for ResourceLimits, ResourceUsage, AgentStats,
EdgeResourceOverrides, InjectionScores, TokenUsage and AcceleratorFlags

## Context

ADR-002 put `#derive(Serialize)` on seven structs of integer fields. The generated `T_to_json`, `T_from_json`
(from bayan pairs) and `T_from_json_str` (from a C string) were expanded by each consumer's own compiler from the
dist bundle. The P(-1) review for 1.12.3 found two defects in the generated decoders. Both live in the toolchain's
codegen (`serialize.cyr`), not in agnostik:

- **Unbounded key scan.** `T_from_json_str` looks for a key's closing quote with
  `while (load8(json + _p) != 34)`, which has no stop at the NUL.
  - Truncated text (`{"pro`) runs off the end of its buffer and segfaults.
  - Text followed by other bytes in memory decodes values from past its terminator.
  - Rated MEDIUM: decoders are fed external input.
- **Wrapping numbers.** Both decoders wrap integers outside i64 (`18446744073709551615` reads -1, which gets
  past a `> LIMIT` quota check), keep a numeric prefix (`12abc` reads 12), and store an i8 field's low byte (a
  score of 256 reads 0). That reopens the F-003 / F-008 class.

Constraints: no API change, so the 21 fns keep their names, arity and modules (the API snapshot lists them), and
`T_to_json` keeps its exact bytes. And consumers compile the bundle on cyrius pins from 6.6.2 to 6.7.18.

## Options considered

- **Keep the derive and document "complete, NUL-terminated input only".** It leaves a MEDIUM input-driven
  over-read in the last v1 release.
- **Wait for an upstream fix.** It would only reach consumers who move their pin, and nothing is filed.
- **Hand-write the 21 fns.** Same names and arity, same `to_json` bytes, decoders on agnostik's own bounded,
  checked JSON reader.

## Decision

Hand-write them, generated from a field table so the seven follow one pattern.

- **`T_to_json`** emits one literal per member and `str_builder_add_int`, exactly as the derive did. The i8
  structs write through their getters, so a score is 0..100 and a flag 0 or 1.
- **`T_from_json_str(json)`** checks that the text opens with `{` (else 0), allocates once, and reads in one pass
  with `_stik_json_read_fields`.
  - The walk stops at the NUL, and a backslash just before it stops it too.
  - Only top-level members match, and a repeated one gives its last value.
  - Numbers are read with F-008's overflow rule: outside i64 reads 0.
  - A field table, built once per struct, maps member names to slots; the search starts after the last hit, so an
    in-order document matches each member on the first comparison.
- **`T_from_json(pairs)`** walks the pairs with `bayan_json_key` / `bayan_json_value`, the two accessors present on
  every consumer pin; `bayan_json_get_by_str` arrived only at 6.7.17. A value must be wholly a decimal integer
  within i64, else it reads 0.

## Consequences

- No decoder reads past its input. Out-of-range and malformed numbers read 0, InjectionScores saturate to 0..100,
  and AcceleratorFlags normalise to 0/1.
- **Stricter input:**
  - text with junk before the `{` now reads 0 (the derived scanner skipped to the first `{`);
  - `12abc` reads 0 on the pairs path.
- **Byte-identical output** for in-range values: `test_v110_serde_golden` passes unchanged.
- **Costs,** measured by a paired 15-round A/B against 1.12.2 on cyrius 6.7.18:
  - the five decoder rows are +8.1% to +15.9% (bounds and range checks the derived code skipped);
  - `to_json` is within ±1.4% for i64 structs, +2.2% / +6.4% for the two i8 structs (their getters clamp).
- **Maintenance:** a new field on these structs means editing the generator's table, the field-table fn and the
  three serde fns. The CHANGELOG has the generator's shape.

## Verification

`tests/tcyr/test_v1123_hardening.tcyr`:
- **Truncation:** `{"pro` decodes to zeros without a crash.
- **NUL boundary:** a text with `":77,"total_tok":88}` written after its NUL reads total 0.
- **Ranges:** values out of i64 read 0 on both paths.
- **Repeats:** a repeated member gives its last value.
- **i8 fields:** they saturate.
- **Fuzz:** every prefix of 40 random TokenUsage texts, NUL-terminated with digits written past the NUL, never
  decodes a field above its original value.

The serde golden, round-trip and sub-byte test files pass unchanged apart from the documented 127 → 100
saturation. The consumer sweep builds the bundle on each consumer's pin.
