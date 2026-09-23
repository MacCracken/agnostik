# Security Policy

## Supported Versions

| Version  | Supported |
|----------|-----------|
| 1.2.x    | Yes (current) |
| 1.1.x    | Yes |
| 1.0.x    | Yes |
| < 1.0.0  | No (pre-stable; upgrade to 1.x) |

Backports for security fixes go to all supported lines. Feature work targets
the current minor only.

## Scope

Agnostik is a type-vocabulary library with zero external dependencies (stdlib
only). Primary attack surfaces:

1. **Deserialization of untrusted input** via the JSON parser primitives
   (`_json_int`, `_json_str`, `_json_find_value`) and the public
   `<Struct>_from_json_str` / `*_from_str` parse functions. All parsers are
   bounds-checked against the source length, return `Result` with
   descriptive error codes, and are exercised by:
   - **Roundtrip tests** in `tests/tcyr/test_serde_roundtrip.tcyr` and the
     v1.1.0 byte-exact golden corpus at
     `tests/tcyr/test_v110_serde_golden.tcyr`.
   - **Fuzz harness** at `tests/tcyr/test_v112_fuzz.tcyr` (v1.1.2) — 8
     parser entry points × 200 deterministic xorshift64 iterations + every
     audit finding's input shape as a regression seed; runs every CI build.
2. **Identifier generation** — `agent_id_new` / `span_id_new` /
   `trace_id_new` draw every ID from the kernel CSPRNG through the stdlib's
   portable `sys_getrandom` (F-022, v1.6.2), never a raw syscall number.
   If it cannot supply the bytes, they fail loudly — a stderr message and
   exit 70 — rather than degrade to a deterministic seed (see audit F-001).
   There is no `/dev/urandom` fallback: F-023 (unreleased) removed it, since
   its raw syscall numbers were wrong on agnos. A Linux kernel older than
   3.17, or a sandbox that denies `getrandom`, therefore exits rather than
   generating IDs.
3. **OTLP wire-format encoding** (v1.2.0) in `src/proto.cyr` and
   `Span_to_otlp_proto`. Output is byte-exact-tested; helpers don't read
   external input directly (callers pass `str_builder` and pre-validated
   struct pointers).

Out of scope: process spawning, file write surface, network surface,
shell interpolation — verified by the `Security Scan` job in
`.github/workflows/ci.yml`.

## Audits

| Date       | Scope | Findings | Report |
|------------|-------|----------|--------|
| 2026-04-26 | Pre-1.0.0 hardening pass | 11 closed (F-001..F-011) | [`docs/audit/2026-04-26-audit.md`](docs/audit/2026-04-26-audit.md) |
| 2026-05-10 | Post-1.0 cadence (1.0.1..1.0.7 cumulative diff) | 1 INFO closed (F-012); F-001..F-011 re-verified | [`docs/audit/2026-05-10-audit.md`](docs/audit/2026-05-10-audit.md) |
| 2026-06-01 | v1.3.0 closeout pass | 1 LOW closed (F-013, bounded `version_from_str` scan) | [`docs/audit/2026-06-01-audit.md`](docs/audit/2026-06-01-audit.md) |
| 2026-08-24 | v1.3.7 P(-1) full-source sweep | F-014..F-021: 6 closed in v1.3.7, 2 contract gaps closed in v1.4.0 | [`docs/audit/2026-08-24-audit.md`](docs/audit/2026-08-24-audit.md) |
| 2026-09-23 | v1.6.2 cross-target syscall review (cyrius 6.6.2 → 6.6.6) | F-022 (MEDIUM) closed in 1.6.2; F-023 (LOW) closed after the cut, unreleased | [`docs/audit/2026-09-23-audit.md`](docs/audit/2026-09-23-audit.md) |

**Cadence**: an audit pass runs at every minor cut and on demand if a
CVE/0-day pattern surfaces in agnostik's input-handling paths or the cyrius
toolchain's parser/serde dependencies.

## Reporting a Vulnerability

Report security issues to the maintainer via GitHub private vulnerability reporting on the [agnostik repository](https://github.com/MacCracken/agnostik). Do not open public issues for security vulnerabilities.

Include:
- Description of the vulnerability
- Steps to reproduce
- Affected version(s)
- Impact assessment

You should receive a response within 48 hours.
