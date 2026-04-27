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

## Future Considerations

- **JSON `\uXXXX` Unicode escape decoder** (`src/types.cyr:_json_str`) — documented limitation as of 1.0.0; defer until a consumer actually surfaces the need (no AGNOS type today carries non-ASCII text through the serde boundary).
- **`_json_int` Result return signature** — current i64 return cannot disambiguate "missing key" from "literal zero" (see audit F-003). Reserved for 1.1 if a consumer needs it; today every consumer treats `_json_int` as integer-typed and the bounded ambiguity is cheaper than the API churn.
