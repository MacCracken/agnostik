# Compile-time profile pass — 2026-05-09 (v1.0.6 slot)

Roadmap pin: v1.0.6 §"Compile-time profile pass". Action only if a
phase exceeds 30% of total compile time *and* the cause is something
agnostik can fix.

## Method

`CYRIUS_PROF=1 cyrius build src/main.cyr build/agnostik` on the
v1.0.5 baseline (cyrius 5.10.14, x86_64 Linux, kernel 6.18, local
host). Three configurations measured:

| Config              | Total | pp | lex | gvar | parse | fixup | emit | write |
|---------------------|------:|---:|----:|-----:|------:|------:|-----:|------:|
| plain               |  380  | 13 | 257 |   37 |     0 |    70 |    0 |     0 |
| `CYRIUS_DCE=1`      |  377  | 13 | 253 |   38 |     0 |    71 |    0 |     0 |
| `CYRIUS_TYPE_CHECK=1` | (no output captured — type-check phase folds into parse cost) |

(All times in ms. Single run; expect ±5% noise on github runners.)

## Distribution

| Phase  | ms  | %    | Action?            |
|--------|----:|-----:|--------------------|
| **lex** | 257 | **68%** | No — upstream |
| fixup   |  70 |  18% | No — upstream |
| gvar    |  37 |  10% | No — see below |
| pp      |  13 |   3% | n/a |
| parse   |   0 |  <1% | n/a |
| emit    |   0 |  <1% | n/a |
| write   |   0 |  <1% | n/a |

## Findings

**Lex (68%, exceeds the 30% threshold)** — bounded by source size.
Agnostik's expanded compilation unit is ~10-15K lines (3.2K LoC
src/ + ~12K LoC stdlib auto-prepended via `[deps]`). The
proportional skew (vs cc5 self-host's 59% lex) reflects agnostik
being mostly type declarations + thin parse/format helpers — token
cost dominates, parse cost is light. Agnostik can't shrink stdlib
includes without losing functionality (every dep is justified —
see `cyrius.cyml`); can't shrink src/ without losing surface
consumers depend on. **No action available on agnostik side.**

**Fixup (18%)** — second-biggest. Upstream-tracked; cyrius's
fixup phase has been a slot target for several minor cycles
(the ftype==3 ARM ASLR fix at v5.9.39 lives here). agnostik
just inherits whatever fixup performance the toolchain ships.
**No action available on agnostik side.**

**Gvar (10%)** — agnostik has many `var <NAME> = <const>` globals
(the `_hex_lut` cstr, `GENESIS_HASH`, `var SS_GET = 0` action
constants in secrets/audit, the `var TRACE_FLAG_SAMPLED = 1` etc.
in telemetry). Total ~30 globals across all modules. Below the
30% threshold; converting to enum variants where the value is
just a tag would reduce gvar pressure but the audit-mandated
constants (e.g. F-008 `922337203685477580` integer-overflow
boundary) need numeric literals where they live. **Below
threshold; no action.**

**Parse + emit + write (<1% each)** — efficient. agnostik's parse
density is low (declarations + thin helpers); emit + write benefit
from small output (DCE binary 274 KB).

## Conclusion

**Slot closed with no action.** Agnostik's compile-time profile is
dominated by upstream-tracked phases (lex + fixup = 86%). The 30%
threshold is exceeded by lex alone, but agnostik has no
shrink-the-source lever that wouldn't sacrifice public-surface
functionality. The slot's value was the measurement itself + this
finding: **future compile-time wins for agnostik come from cyrius
toolchain bumps, not from agnostik refactors.**

## Re-measurement schedule

When cyrius ships a notable lex / fixup optimization (track via
`cyrius/CHANGELOG.md`), re-run this profile pass and update the
table above. If the lex share drops below 50% via upstream
changes, the proportional skew shifts and gvar might warrant
re-examination at that point.

## References

- Cyrius v5.10.0 introduced `CYRIUS_PROF=1` per-phase profiling
  ([cyrius CHANGELOG.md §5.10.0](../../../cyrius/CHANGELOG.md)).
- Agnostik v1.0.6 roadmap pin
  ([../development/roadmap.md §v1.0.6](roadmap.md)).
- cc5 self-host baseline (cyrius v5.10.0): 984ms total, lex
  580ms = 59%. Agnostik's 68% lex share reflects the type-library
  vs compiler-self-host skew.
