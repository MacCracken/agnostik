# Agnostik — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures (durable);
> this file is **state** (volatile). Bumped via `scripts/version-bump.sh`.

## Version

**1.0.0** — first stable release. Toolchain refresh to Cyrius 5.7.6,
manifest migration `cyrius.toml` → `cyrius.cyml`, P(-1) scaffold
hardening + security audit pass, layout aligned with vidya/yukti
conventions. See [`docs/audit/2026-04-26-audit.md`](../audit/2026-04-26-audit.md)
for security findings and [`CHANGELOG.md`](../../CHANGELOG.md) for full
release notes.

## Toolchain

- **Cyrius**: `5.7.6` (pinned in `cyrius.cyml [package].cyrius`)
- **Compiler**: `cc5` — invoked via `cyrius {build,test,bench}`; raw
  `cat | cc5` is forbidden (manifest auto-resolves deps and prepends includes)

## Source layout

```
src/
  lib.cyr            — include orchestrator (consumed by main.cyr)
  main.cyr           — test harness entry
  error.cyr          — Result / Err / error kinds
  types.cyr          — version, UUID, timestamp, identifiers
  agent.cyr          — agent ID, capabilities, scheduling, rate limits
  security.cyr       — sandbox, capabilities, auth, policies
  telemetry.cyr      — spans, metrics, logs, exemplars, baggage
  audit.cyr          — entries, integrity, retention
  llm.cyr            — tools, sampling, streaming, content blocks
  secrets.cyr        — metadata, zeroize
  config.cyr         — profiles, fleet
  classification.cyr — classification results
  validation.cyr     — warnings, injection scores
  hardware.cyr       — devices, flags, summary
```

Tests at `tests/tcyr/agnostik.tcyr` + 4 coverage modules + serde
roundtrip. Benches at `tests/bcyr/agnostik.bcyr`.

## Stats

> Updated by the closeout pass. Never inline these in CLAUDE.md.

| Metric                | Value     | Notes                              |
|-----------------------|-----------|------------------------------------|
| Source LOC (src/)     | ~3,200    | down from 7,121 LOC Rust           |
| Module count          | 12        |                                    |
| Test files            | 7         | tests/tcyr/                        |
| Test assertions       | 613+      | re-verified at 1.0.0 closeout      |
| Benchmarks            | 25        | tests/bcyr/                        |
| Test binary (DCE)     | TBD       | filled in at 1.0.0 build           |
| Lib bundle (dist/)    | TBD       | `cyrius distlib` output            |

## Consumers

Every AGNOS component depends on agnostik for shared types:

- **daimon** — agent runtime
- **hoosh** — LLM grounding service
- **agnoshi** — shell
- **aegis** — security policy engine
- **argonaut** — agent orchestrator
- **sigil** — capability/auth issuer
- **ark** — packaging / distributable
- **kavach** — sandbox enforcement
- **stiva** — telemetry pipeline
- **nein** — refusal / safety layer
- **yukti** — device abstraction (telemetry types)

## Recent releases

See [`CHANGELOG.md`](../../CHANGELOG.md). Most recent stable: `1.0.0` (this release).

## Verification hosts

- Local: x86_64-linux (LTS kernel 6.18)
- CI: `ubuntu-latest` (GitHub Actions)
- Cross: aarch64 best-effort via `cc5_aarch64` when shipped in toolchain
