# Security Policy

## Supported Versions

| Version  | Supported |
|----------|-----------|
| 1.12.x   | Yes (current) |
| < 1.12.0 | No (upgrade to the current 1.x release) |

Security fixes ship in a new release on the current line. There are no
backport branches: every 1.x release keeps the public API (additions only,
no breaks before 2.0.0), so moving to the current release is the upgrade
path for any 1.x consumer. Feature work targets the current minor only. The
table moves to the new line at each minor cut.

## Scope

Agnostik is a type-vocabulary library with zero external dependencies (stdlib
only). Primary attack surfaces:

1. **Deserialization of untrusted input** via the JSON parser primitives
   (`_stik_json_int`, `_stik_json_str`, `_stik_json_find_value`; named
   without the `_stik_` prefix before 1.7.0) and the public
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
   There is no `/dev/urandom` fallback: F-023 (v1.6.3) removed it, since
   its raw syscall numbers were wrong on agnos. A Linux kernel older than
   3.17, or a sandbox that denies `getrandom`, therefore exits rather than
   generating IDs.
3. **OTLP wire-format encoding** (v1.2.0; completed in v1.10.0) in
   `src/proto.cyr` and the `*_to_otlp_proto` encoders. Output is tested byte
   for byte against protoc's encoding of the official schema, and fuzzed
   through a schema walker. The encoders read caller-built structs, not
   external bytes. Attribute values are typed `AnyValue` boxes; an unknown
   type, nesting 16 levels deep or a cycle encodes as an empty value, not a
   crash or unbounded recursion. Since v1.12.0 every protobuf `string` field
   is valid UTF-8 (ADR-006): an attribute value or log body that is not goes
   out as `bytes_value`, unchanged, and other strings get U+FFFD for each
   ill-formed subpart. So a strict decoder never rejects a message over a
   caller's bytes.
4. **Build toolchain supply chain** (v1.7.1). CI and the release workflow
   install the pinned Cyrius toolchain through `scripts/ci-install-cyrius.sh`.
   It checks the release's `SHA256SUMS.sig` against the cyrius Ed25519
   release key pinned in that script, then the tarball against `SHA256SUMS`,
   and only then unpacks. It fails closed: a tampered, re-signed or unsigned
   toolchain does not build agnostik's release artifacts. The library itself
   reaches the kernel only through the stdlib `sys_*` wrappers, and the CI
   scan rejects a raw `syscall(` in `src/`.

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
| 2026-09-23 | v1.6.2 cross-target syscall review (cyrius 6.6.2 → 6.6.6) | F-022 (MEDIUM) closed in 1.6.2; F-023 and F-024 (LOW) closed in 1.6.3 | [`docs/audit/2026-09-23-audit.md`](docs/audit/2026-09-23-audit.md) |
| 2026-10-14 | v1.6.5 toolchain refresh (cyrius 6.6.6 → 6.6.20): advisories CYRIUS-2026-0009..0035 against agnostik's reach | No findings; CYRIUS-2026-0028 reaches `version_to_str` but was never exploitable there | [`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md) |
| 2026-10-14 | v1.7.0 minor-cut pass (same report, second section) | F-025 (LOW): unnamespaced private helpers could be replaced by another library's in a consumer's unit; fixed in 1.7.0 (`_stik_` prefix + CI gate) | [`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md) |
| 2026-10-14 | v1.7.1 release pass (same report, third section): raw-syscall gate, verified toolchain install, aarch64 CI | No findings | [`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md) |
| 2026-10-14 | v1.8.0 minor-cut pass (same report, fourth section): cyrius 6.6.20 → 6.7.9, 50 enum name/parse fns, cross-library symbol scan | No findings | [`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md) |
| 2026-10-14 | v1.9.0 minor-cut pass (same report, fifth section): `RestartPolicy` pair, every fn documented, Unix-time stamps, span-id byte order | F-026 (LOW): two JSON string fields (`export_endpoint`, `AgentInfo` `name`) serialized unescaped; fixed in 1.9.0 | [`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md) |
| 2026-10-14 | v1.10.0 minor-cut pass (same report, sixth section): OTLP completion, the in-place protobuf buffer, the Status field fix | F-027 (LOW, defense in depth): the OTLP encoders could write out of bounds on a negative-length `Str` or read address 0 on null span fields; fixed in 1.10.0 | [`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md) |
| 2026-10-14 | v1.11.0 minor-cut pass (same report, seventh section): checked trait dispatch, argument-passing hand-offs, the UUID separator check, 91 setters, newer Landlock rights | No findings | [`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md) |
| 2026-10-14 | v1.12.0 minor-cut pass (same report, eighth section): the UTF-8 writer, Resource / Scope encoders, export requests, histograms, exemplars | No findings | [`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md) |

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
