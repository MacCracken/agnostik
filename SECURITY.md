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
   `trace_id_new` use a `getrandom(2)` → `/dev/urandom` retry loop with
   loud-fail on both unavailable (see audit F-001).
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
