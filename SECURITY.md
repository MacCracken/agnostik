# Security Policy

## Supported Versions

| Version  | Supported |
|----------|-----------|
| 1.0.x    | Yes       |
| < 1.0.0  | No (pre-stable; upgrade to 1.0.x) |

## Scope

Agnostik is a type-vocabulary library with zero external dependencies (stdlib only). Primary attack surface: deserialization of untrusted input via `_from_json` and `_from_str` parse functions in `src/types.cyr` and consumers' adapters. All types validate on construction where applicable; all parse functions return `Result` with descriptive error codes.

Secondary attack surface: identifier generation. `agent_id_new` and `span_id_new` use a `getrandom(2)` → `/dev/urandom` retry loop with loud-fail on both unavailable (see audit F-001 in [`docs/audit/2026-04-26-audit.md`](docs/audit/2026-04-26-audit.md)).

Out of scope: process spawning, file write surface, network surface, shell interpolation — verified by `Security Scan` job in CI (`.github/workflows/ci.yml`).

## Reporting a Vulnerability

Report security issues to the maintainer via GitHub private vulnerability reporting on the [agnostik repository](https://github.com/MacCracken/agnostik). Do not open public issues for security vulnerabilities.

Include:
- Description of the vulnerability
- Steps to reproduce
- Affected version(s)
- Impact assessment

You should receive a response within 48 hours.
