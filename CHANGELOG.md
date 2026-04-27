# Changelog

## [1.0.1] - 2026-04-26

Documentation cleanup pass after the 1.0.0 release. Pure docs — no
code, manifest, or test changes; `dist/agnostik.cyr` is byte-identical
to 1.0.0.

### Changed

- **`CONTRIBUTING.md`** rewritten. Pre-port content (Cyrius `v3.2.1+`,
  `cc2`, `cyrius.toml`, `benches/` path, `cat src/main.cyr | cc2 > out`
  build invocation) replaced with the current toolchain story:
  manifest-driven `cyrius build` / `cyrius deps` / `cyrius distlib`,
  per-test `cyrius test`, format/lint sweeps, and a code-standards
  bullet calling out the F-011 lesson (do not stack `#derive(Serialize)`
  alongside hand-written serde adapters).
- **`SECURITY.md`** supported-versions table refreshed: 1.0.x now
  marked supported; pre-1.0 marked unsupported (upgrade path). Scope
  section pulled in pointers to F-001 (CSPRNG) and the CI security
  scan as the source of truth for what's in/out of scope.
- **`docs/development/roadmap.md`** Status block updated from v0.97.0
  / Cyrius v3.2.4 / 613 assertions to v1.0.0 / Cyrius 5.7.12 / 653
  assertions. The previously-empty `## v1.0.0` section now lists the
  shipped scope as a completed checklist; future considerations
  (`\uXXXX` decoder, `_json_int` Result-return ABI) parked with
  rationale.
- **`docs/architecture/overview.md`** fixed: two U+FFFD replacement
  glyphs on the `src/llm.cyr` row of the module map; consumer list
  rewritten to match `state.md` (added agnoshi, sigil, ark, stiva,
  nein, yukti; removed stale `secureyeoman` reference).
- **`docs/development/issues/cyrius-audit-missing-check-script-2026-04-26.md`**
  re-stamped: removed "post-1.0" framing now that 1.0.0 has shipped;
  relinked from the deleted `docs/audit/2026-04-26-audit-5712.md` to
  the canonical `docs/audit/2026-04-26-audit.md` (where F-008..F-011
  are merged).
- **`tests/tcyr/test_audit_5712.tcyr`** header comment + run banner
  re-framed: was "post-1.0 toolchain-5.7.12 audit"; now
  "F-008/F-009/F-010 regression for the 1.0.0 audit (surfaced by the
  mid-pass toolchain bump)".

### Moved

- **`benchmark-rustvcyrius.md`** → **`docs/benchmarks/benchmark-rust-v-cyrius-legacy.md`**.
  The file is the Rust → Cyrius port-boundary comparison (Cyrius
  v0.95.0 / cc2 vs Rust v0.90.0 / Criterion). Numbers are no longer
  current — the Cyrius side has moved through several toolchain revs
  and the 1.0.0 audit's CSPRNG migration alone moved `agent_id_new`
  from ~35 ns to ~600 ns. Preserved as legacy reference for the
  migration cost-benefit story; new top-of-file note marks it as
  historical and points to `history.csv` for live data.

### Removed

- **`bench-history.log`** — Rust-era Criterion output (April 6, before
  the Cyrius port). Superseded by the structured CSV at
  `docs/benchmarks/history.csv`. The 0.97.0 CHANGELOG entry already
  declared this removed; reality now matches.

### Fixed (CI)

- 3 lint warnings for lines exceeding 120 bytes (`cyrius lint` counts
  UTF-8 bytes, not glyphs — box-drawing `─` is 3 bytes each):
  `tests/tcyr/agnostik.tcyr:490` (long `integrity_fields_new(...)` call,
  wrapped to multi-line); `tests/tcyr/test_audit_2026_04_26.tcyr:25`
  and `tests/tcyr/test_audit_5712.tcyr:86` (F-001 / F-010 section
  headers, trailing `─` decoration trimmed).
- 2 `cyrfmt --check` drift hits absorbed by `cyrfmt --write`:
  `src/main.cyr` (multi-arg call continuation indent normalized 8→4)
  and `tests/tcyr/test_audit_5712.tcyr` (F-009 test body re-indented
  via comment-relocation to dodge a cyrfmt formatter quirk on
  multi-line in-body comments — issue worth filing upstream once
  reproducible).

### `.gitignore`

- Removed `/dist/` to match the yukti/vidya gold-standard pattern.
  `dist/agnostik.cyr` is the canonical bundle consumers fetch via
  `[deps].modules = ["dist/agnostik.cyr"]` from the git tag, so it
  must be tracked. CI's "Verify dist bundle is in sync with src/"
  step now actually verifies — pre-fix it ran vacuously because
  `git diff` reported nothing for an ignored path.

## [1.0.0] - 2026-04-26

First stable release. Toolchain refresh to Cyrius **5.7.12**, P(-1)
scaffold hardening pass, security audit with external CVE / 0-day
research (11 findings closed, 1 resolved upstream, 1 new upstream
issue filed), and layout alignment with the vidya / yukti
gold-standard project shape.

### Security

- **F-001 (HIGH) — UUID PRNG hardened** (`src/types.cyr`,
  `src/telemetry.cyr`). Pre-1.0 the PRNG seeded a 64-bit xorshift state
  once from 8 bytes of `/dev/urandom` and silently fell through to
  `clock_gettime(CLOCK_MONOTONIC)` and ultimately a hardcoded constant
  on urandom failure. Same shape as **CVE-2025-66630** (gofiber UUID
  silent fallback). Replaced with a per-call `_fill_random` helper that
  reads 16 bytes via `getrandom(2)` (syscall 318), falls back to
  `/dev/urandom` open+read with short-read retry, and aborts the
  process on stderr if both fail. Identifier generation cost rose from
  ~65 ns to ~600 ns per UUID — accepted to remove the predictable-ID
  failure mode in sandboxed deployments.
- **F-002 (MEDIUM) — JSON string escape decoding**
  (`src/types.cyr:_json_str`). Pre-1.0 the extractor terminated at the
  first `"` byte regardless of preceding `\`, so a value like
  `"abc\"xyz"` was truncated to `abc\` and the parse cursor was
  desynchronised for every subsequent field. Pattern-related to
  **CVE-2025-27788** (Ruby JSON unescape OOB). Now decodes the
  standard escape set (`\" \\ \/ \n \t \r \b \f`); `\uXXXX` left as a
  documented limitation.
- **F-003 (MEDIUM) — JSON integer sign + overflow guard**
  (`src/types.cyr:_json_int`). Pre-1.0 negatives parsed as `0` and
  long digit strings silently wrapped i64. Pattern-related to
  **GHSA-72HV-8253-57QQ** (Jackson async unchecked numeric lengths).
  Now handles a leading `-` and stops accepting digits after the 19th.
- **F-004 (LOW) — JSON key search respects string boundaries**
  (`src/types.cyr:_json_find_value`). Pre-1.0 the lookup did a raw
  `strstr` for `"key":` which could match the same byte pattern when
  it appeared inside a string *value* (e.g. an attacker-controlled
  `"note":"max_memory:..."` poisoning a sibling `max_memory` lookup).
  The walker now tracks string boundaries with `\` escape awareness
  and only matches the needle outside of `"..."`.
- **F-005 (LOW) — `version_from_str` digit validation**
  (`src/types.cyr`). Pre-1.0 `version_from_str("abc.1.2")` produced
  `Version{0, 1, 2}` because `str_to_int` silently returns `0` on
  garbage. Each segment is now validated to be wholly digits before
  parsing; non-numeric or empty segments return `Err`.
- **F-007 (INFO) — long-line cleanup**. 16 single-line `enum`/`struct`
  declarations across `classification.cyr`, `hardware.cyr`, `llm.cyr`,
  `main.cyr`, and `security.cyr` rewrapped to one variant/field per
  line. Clears the entire `line exceeds 120 characters` lint set.
- **F-008 (LOW) — `_json_int` i64 overflow on 19-digit inputs**
  (`src/types.cyr`). F-003's fix capped the digit run at 19, but
  i64::MAX (`9223372036854775807`) is also 19 digits. Inputs in
  `[9223372036854775808, 9999999999999999999]` were accepted and
  silently wrapped — pre-fix `{"x":9999999999999999999}` returned
  `-8446744073709551617`, which would slip past a downstream
  `if (x > LIMIT)` non-negative guard. Replaced the digit cap with
  a pre-multiply overflow check
  (`val > 922337203685477580 || (val == 922337203685477580 &&
  d > 7)`); the magnitude check is symmetric for negatives so
  i64::MIN is unrepresentable by 1, accepted as a deliberate
  trade. Pattern shape: **GHSA-72HV-8253-57QQ**, CWE-190.
  Surfaced during the post-toolchain-bump re-scan.
- **F-009 (LOW) — `_json_str` OOB read on truncated `null` literal**
  (`src/types.cyr`). The 4-byte `memeq(data + vi, "null", 4)` probe
  ran without checking that 4 bytes were available, reading up to
  3 bytes past `slen` on truncated input like `{"x":nu}`. Bounded
  over-read; bump-allocator info-leak shape, freelist-allocator
  near-page-boundary SIGSEGV shape. Gated the probe on
  `vi + 4 <= slen`. Pattern shape: **CVE-2025-27788** family.
- **F-010 (INFO) — RFC 7159 whitespace handling**
  (`src/types.cyr:_json_find_value`). The post-colon value-skip
  loop only advanced past ASCII space (32). RFC 7159 §2 also
  includes tab (9), LF (10), CR (13). Any pretty-printed input
  (`python json.dumps(indent=2)`, `jq .`) silently failed to find
  values, returning 0 — indistinguishable from "value really was
  0". Replaced with the full RFC whitespace set.
- **F-011 (LOW) — silent dead-code: derive + hand-written serde
  collision** (9 sites across `src/agent.cyr` × 4, `src/config.cyr`,
  `src/hardware.cyr`, `src/llm.cyr`, `src/telemetry.cyr`,
  `src/validation.cyr`). Each affected struct carried both
  `#derive(Serialize)` and a hand-written `<Type>_to_json` /
  `_from_json`; cyrius last-define-wins meant the hand-written
  adapter shadowed the derive output, but the dead bytes still
  shipped in the binary. Surfaced by Cyrius v5.7.9's new
  `duplicate fn` warning. Dropped the 9 derive markers; the
  hand-written adapters are the canonical form (and incorporate
  F-002 / F-003 / F-008 fixes that derive doesn't).

### Filed upstream — resolved

- **F-006 (UPSTREAM) → resolved upstream in Cyrius v5.7.7.**
  `cyrius lint`'s snake_case rule was raising 28 false-positives
  against agnostik's hand-written `<PascalStructName>_<snake_verb>`
  UFCS-style serde adapters. Upstream landed a Pascal-prefix
  carve-out in 5.7.7 (`programs/cyrlint.cyr`); after the toolchain
  bump to 5.7.12, `cyrius lint src/*.cyr` returns 0 warnings.
  Local report at
  [`docs/development/issues/cyrius-lint-ufcs-pascal-prefix-snake-case-2026-04-26.md`](docs/development/issues/cyrius-lint-ufcs-pascal-prefix-snake-case-2026-04-26.md)
  re-stamped with resolution status; stays local for audit lineage.

### Filed upstream — new

- **`cyrius audit` invokes `~/.cyrius/bin/check.sh` but the install
  never ships it.** Verified on a freshly-`cyriusly install`'d
  5.7.12: `cmd_audit` in `cbt/commands.cyr:395-398` expects
  `check.sh` next to the cyrius binary, but Cyrius's release
  manifest `scripts` array omits it. `cyrius audit` exits 127 on
  every fresh install of 5.7.x. Workaround: run `cyrius self /
  test / fmt --check / lint` individually. Filed locally:
  [`docs/development/issues/cyrius-audit-missing-check-script-2026-04-26.md`](docs/development/issues/cyrius-audit-missing-check-script-2026-04-26.md).

Full audit report and verification: [`docs/audit/2026-04-26-audit.md`](docs/audit/2026-04-26-audit.md).

### Changed

- **Cyrius compiler target**: v3.2.5 → **v5.7.12**. Build pipeline
  moved from raw `cat src/main.cyr | cc2 > out` to the manifest-driven
  `cyrius build` CLI (`cc5` underneath). Stdlib + git dependencies are
  resolved by `cyrius deps` from `[deps]` in the manifest; source
  files no longer carry their own stdlib `include` lines. The
  closeout pass absorbed a mid-pass toolchain refresh from 5.7.6 →
  5.7.12 that brought the F-006 lint-rule fix upstream (5.7.7), the
  duplicate-fn warning that surfaced F-011 (5.7.9), and a
  consumer-transparent `input_buf` 512 KB → 1 MB bump (5.7.10).
- **Stdlib deps**: `[deps].stdlib` includes `"io"` so `lib/json.cyr`'s
  reference to `file_read_all` resolves. Pre-fix the build emitted
  `warning: undefined function 'file_read_all'` with a synthesised
  runtime trap; agnostik never calls `json_parse_file` so no runtime
  impact, but the warning would have masked a real one. DCE strips
  the unused fns; net binary +2,328 B for the unavoidable syscall
  wrappers.
- **Manifest format**: `cyrius.toml` → `cyrius.cyml`. Version now
  pulled from `VERSION` via `${file:VERSION}` so the file is the only
  source of truth. Toolchain pinned in `[package].cyrius`.
- **Project layout**: `tests/*.tcyr` → `tests/tcyr/`,
  `benches/*.bcyr` → `tests/bcyr/agnostik.bcyr`, `tests/test.sh` →
  `scripts/run-tests.sh`.
- **Build / bench / version-bump scripts** rewritten to call `cyrius
  {build,test,bench}` and to drop all Cargo.toml references inherited
  from the Rust era.
- **CI / release workflows** rewritten to the yukti / vidya pattern:
  toolchain version pulled from `cyrius.cyml`, dead-code-eliminated
  build (`CYRIUS_DCE=1`), `cyrius lint` / `cyrius fmt --check` /
  `cyrius vet` sweep, ELF magic verification, best-effort aarch64
  cross-build, dist-bundle synchronisation check.
- **`.gitignore`** updated to the gold-standard form: `lib/*.cyr`
  ignored (regenerated by `cyrius deps`), build / dist / toolchain
  artifacts excluded.
- **CLAUDE.md** rewritten to the agnosticos `example_claude.md` gold
  standard. Volatile state (versions, sizes, test counts, consumers,
  verification hosts) moved to `docs/development/state.md`; CLAUDE.md
  is now durable preferences, process, and procedures only.

### Added

- **`docs/development/state.md`** — live state snapshot, refreshed
  every release (mandated by the new CLAUDE.md template).
- **`docs/audit/2026-04-26-audit.md`** — security audit report with
  external CVE references and per-finding remediation notes.
- **`tests/tcyr/test_audit_2026_04_26.tcyr`** — 30 regression
  assertions covering F-001 / F-002 / F-003 / F-004 / F-005 with
  positive and negative cases per finding.
- **`tests/tcyr/test_audit_5712.tcyr`** — 10 regression assertions
  covering F-008 / F-009 / F-010 (post-toolchain-bump findings).
- **`docs/benchmarks/history.csv`** — bench history, appended by
  `scripts/bench-history.sh`.
- Canonical doc directories scaffolded: `docs/adr/`,
  `docs/architecture/`, `docs/guides/`, `docs/examples/`,
  `docs/audit/`.

### Removed

- **`cyrius.toml`** (replaced by `cyrius.cyml`).
- **`benches/`** (consolidated under `tests/bcyr/`).
- **`bench-history.log`**, **`benchmark-rustvcyrius.md`** — Rust-era
  bench artifacts superseded by `docs/benchmarks/history.csv`.
- Vendored stdlib copies in `lib/*.cyr` un-tracked (regenerated by
  `cyrius deps`).

### Testing

- **653 assertions, 0 failures** across 9 test files (up from 613
  across 7 — 40 new assertions: 30 covering F-001..F-005 in
  `test_audit_2026_04_26.tcyr`, 10 covering F-008..F-010 in
  `test_audit_5712.tcyr`).
- 25 benchmarks, all green. Identifier-generation tier shows the
  expected CSPRNG cost; serde / format / integration tiers within
  ±10 % of the pre-fix baseline. F-008's pre-multiply overflow
  check adds a single comparison per digit, undetectable in the
  bench.

### Stats (final closeout)

| Metric                | Value     | Note                                |
|-----------------------|-----------|-------------------------------------|
| Binary (DCE'd)        | 214,560 B | After F-011 dead-derive removal (-40 KB vs the pre-F-011 closeout build) |
| Test files            | 9         |                                     |
| Test assertions       | 653       |                                     |
| Build warnings        | 0         |                                     |
| Lint warnings         | 0         | (was 28 false positives pre-5.7.7; resolved upstream) |
| Duplicate-fn warnings | 0         | (was 18 silent dead-code pre-F-011) |

### Performance

| Tier            | Pre-fix (cc3)   | Post-fix (1.0.0)  | Note                  |
|-----------------|-----------------|-------------------|-----------------------|
| `agent_id_new`  | 65 ns           | 608 ns            | CSPRNG cost (F-001)   |
| `span_id_new`*  | (PRNG)          | (CSPRNG)          | F-001                 |
| `version_to_str`| 160 ns          | 155 ns            | unchanged             |
| `version_roundtrip` | 299 ns      | 311 ns            | +4 % (segment validation) |
| serde tier      | 700 ns – 8 µs   | 700 ns – 8 µs     | unchanged (F-008 overhead negligible) |

\* indirect (used by trace_context_*).

### Breaking

This is the 1.0.0 cut. Consumers should expect:

- Build invocation: `cat src/main.cyr | cc2 > build/agnostik` no longer
  works. Use `cyrius build src/main.cyr build/agnostik`.
- Manifest filename / format. If a consumer was reading
  `cyrius.toml`, switch to `cyrius.cyml`.
- `_json_int` now stops at the 19th digit and honours a leading `-`.
  Consumers serializing numbers > 19 decimal digits via `_to_json` and
  expecting the same wrapped value to roundtrip will see different
  output. (No agnostik field type today exceeds 19 digits.)
- `version_from_str` rejects non-numeric and empty segments.
  Consumers passing `"latest"` or `"main"` through this parser will
  now get `Err`.

## [0.97.1] - 2026-04-09

### Changed
- **Cyrius compiler target**: v3.2.4 → v3.2.5 (cc3 compiler, minimum version)
- **Zero-dependency lib build** — replaced stdlib syscall constants (`SYS_WRITE`, `SYS_OPEN`, `SYS_READ`, `SYS_CLOSE`) with raw syscall numbers in `error.cyr` and `types.cyr`; lib no longer requires consumers to provide `syscalls.cyr`

### Fixed
- **`cyrius.toml` bench entry** — `bench.cyr` → `bench.bcyr` (was silently producing a 136-byte stub binary)
- **Bench header version** — `v3.2.4` → `v3.2.5`

## [0.97.0] - 2026-04-09

### Added
- **`_from_json` deserialization** for all 9 serializable structs: ResourceLimits, ResourceUsage, AgentStats, TokenUsage, AcceleratorFlags, EdgeResourceOverrides, TelemetryConfig, InjectionScores, AgentInfo (with UUID parsing)
- **JSON field extractors** (`_json_find_value`, `_json_int`, `_json_str`) — uses `strstr`/`memeq` to avoid compiler nested loop codegen bug
- **7 `_name()` functions** for internal security enums: `fs_access_name`, `net_access_name`, `seccomp_action_name`, `seccomp_arg_op_name`, `seccomp_arch_name`, `mount_propagation_name`, `policy_effect_name`
- **10 serde benchmarks** (tier4): 5 `_to_json` + 5 `_from_json` covering AgentStats, InjectionScores, TokenUsage, ResourceLimits, AcceleratorFlags
- **330 new test assertions** across 4 new test files (test_coverage_1–4) covering: agent lifecycle/capabilities/scheduling/rate-limits/resource-grants, security contexts/policies/capabilities/sandbox/auth, LLM tools/sampling/streaming/content-blocks/model-capabilities, telemetry spans/metrics/logs/exemplars/baggage, audit entries/integrity/retention, secrets metadata, config profiles/fleet, validation warnings/injection-scores, classification results, hardware devices/flags/summary, extended name functions
- **36 serde roundtrip assertions** (test_serde_roundtrip) — serialize → JSON → deserialize → verify all fields for 8 struct types

### Changed
- **Cyrius compiler target**: v2.7.2 → v3.2.4 (function limit 1024→2048, `#derive(Serialize)` Str field support, `strstr`/`memeq` stdlib additions)
- **Stdlib vendored**: `string.cyr` updated from Cyrius 3.2.4 (adds `strstr`, `atoi`)
- **CI workflows** — Cyrius toolchain updated to 3.2.4, benchmark path fixed (`bench.cyr` → `bench.bcyr`, was silently skipped), added CLAUDE.md and CODE_OF_CONDUCT.md to required docs check
- **README.md** — rewritten for Cyrius (removed Rust examples, feature flags, `use` statements)
- **CONTRIBUTING.md** — rewritten for Cyrius (was referencing `#[non_exhaustive]`, `make check`, `unwrap()`)
- **SECURITY.md** — updated supported versions (was 0.1.x), added vulnerability reporting process
- **CLAUDE.md** — marked Rust conversion complete, fixed benchmark command, updated `#derive(Serialize)` and function limit notes
- **CHANGELOG** — renamed duplicate 0.95.0 entry to 0.95.1

### Testing
- 613 assertions (up from 223), 0 failures, 7 test files
- 25 benchmarks (up from 15), no regressions

### Performance
- Serialization: ~1us (3-field) to ~2us (9-field)
- Deserialization: ~1us (3-field) to ~9us (9-field) via `strstr`-based field extraction
- Core benchmarks unchanged: agent_id_new 36ns, trace_context_child 42ns, sandbox_config 64ns

## [0.96.0] - 2026-04-09

### Removed
- **rust-old/** — deleted 1.2 GB of Rust source, Cargo.lock, build artifacts, and criterion data. Rust→Cyrius port verified complete across all 12 modules. Rust benchmark reference numbers preserved in `benchmark-rustvcyrius.md`.

### Added
- **benchmark-rustvcyrius.md** — head-to-head Rust Criterion vs Cyrius bench comparison with analysis (Cyrius wins on agent_id_new 1.4x, trace_context_child 1.3x; Rust wins on sandbox_config_default 2.3x)

### Changed
- **CI workflows** — rewritten to use `cyrius build`/`cyrius test`/`cyrius bench` instead of raw `cat | cc2` pipes. Fixes `build/` directory pre-existence failures.
- **Release workflow** — uses `cyrius test` for gate instead of manual `.tcyr` loop
- **Source formatting** — all 6 files flagged by `cyrfmt` fixed (agent, config, hardware, llm, telemetry, validation)

## [0.95.1] - 2026-04-09

### Fixed
- **`version_to_str` buffer overflow** — increased buffer from 64 to 128 bytes with bounds checking on prerelease/build `memcpy` (was heap corruption on long prerelease strings)
- **`version_from_str` uninitialized fields** — major/minor/patch zeroed on alloc (was garbage on parse failure)
- **`secret_destroy` incomplete zeroize** — now zeros both the secret buffer AND the struct pointer/length fields
- **`_json_has` passed C string to `str_contains`** — fixed to pass Str (was always failing after str.cyr API update)
- **`accelerator_device_new` memory_type sentinel** — uses `MEM_UNKNOWN` enum instead of raw `-1`
- **`TelemetryConfig_to_json` null endpoint** — renders as `null` instead of `""` (ambiguous with empty string)
- **`AgentInfo_to_json` null name** — guarded with null check, renders as `null`

### Added
- **14 `_name()` functions** for consumer-facing enums (all use clean `elif` chains):
  - types: `message_type_name`, `system_status_name`
  - llm: `message_role_name`, `finish_reason_name`
  - telemetry: `span_status_name`, `span_kind_name`
  - hardware: `device_family_name`, `device_health_name`, `memory_type_name`
  - classification: `pii_kind_name` (16 PII variants)
  - validation: `validation_severity_name`
  - secrets: `secret_kind_name`
- **12 missing accessors/setters**: `scarg_value_two`, 6 AcceleratorFlags setters (metal, oneapi, tpu, sycl, openvino, directml), 2 TokenUsage cache setters, 3 content block factories (`content_document`, `content_audio`, `content_citation`)
- **sakshi.cyr** — vendored slim tracing/error profile (zero-alloc, stderr output, packed i64 errors)
- **Regression test** — `tests/string_shift_bug.tcyr` for compiler bug #30

### Changed
- **Stdlib synced to vidya 2.0** — alloc (arena allocator), assert (6 new helpers), fmt (f64 formatting), io (getenv), str (Str-based contains/ends_with, direct-buffer string builder), hashmap (refactored internals), process (pipefd fix), json (io.cyr dep), syscalls (threading/mmap/futex enums)
- **Str_ method wrappers removed** — reclaimed 16 function slots (unused by agnostik, consumers use `str_*` directly)
- **Syscalls trimmed** — removed admin/epoll/timer/signal/identity wrappers (not needed by a types library)
- **All existing `_name()` functions** converted from separate `if` blocks to `elif` chains
- **Cyrius compiler target**: v1.11.4 → v2.6.4
- **Test/bench file format**: `.tcyr`/`.bcyr` for `cyrius test`/`cyrius bench` auto-discovery
- **Build config**: `cyrb.toml` → `cyrius.toml`

### Testing
- 226 assertions (up from 198), 0 failures
- 15 benchmarks, no regressions
- Regression test for compiler bug #30 (str_data buffer overflow)

### Performance
- agent_id_new: 35ns, trace_context_child: 41ns, sandbox_config_default: 62ns
- version_to_str: 151ns (up from 124ns — bounds checking cost, acceptable)
- accelerator_device_full: 153ns, token_usage_update: 36ns

### Breaking
- `str_contains` and `str_ends_with` now take Str arguments instead of C strings. Callers must wrap C strings with `str_from()`.

## [0.91.0] - 2026-04-07

### Fixed
- **`#derive(Serialize)` no-op stubs** — compiler generates empty `_to_json` functions; added manual implementations for all 9 serializable structs (TokenUsage, AgentInfo, AgentStats, ResourceLimits, ResourceUsage, InjectionScores, AcceleratorFlags, EdgeResourceOverrides, TelemetryConfig)
- **SandboxConfig default** — `NET_NONE` → `NET_LOCALHOST_ONLY` (Rust parity)
- **`trace_id_from_str` rejected uppercase hex** — now accepts A-F (consistent with `agent_id_from_str`)
- **Stale version references** — CI and bench header updated from Cyrius v1.9.4 to v1.11.4
- **`file_read_all` undefined warning** — added `io.cyr` to test include chain
- **Unused `json.cyr` in bench** — removed (freed function slots, eliminated `file_read_all` warning)

### Added
- `span_id_from_str` — hex string parsing with roundtrip support
- `tctx_from_traceparent` — W3C traceparent header parse (reverse of `tctx_to_traceparent`)
- `CacheControl` enum (`CACHE_EPHEMERAL`) for Anthropic prompt caching
- `AcceleratorDevice` accessors: `temperature`, `driver_version`, `compute_capability`, `power_watts`, `memory_bandwidth_gbps`, `memory_type` (+ setters)
- `InferenceRequest` fields: `service_tier`, `metadata`, `reasoning_effort` (+ accessors)
- 84 new test assertions covering: version serde/prerelease/errors, error codes/names, sandbox defaults, RBAC roles, permissions, cgroup limits, trace context propagation, traceparent validation, log severity ordering/names, log records, crash reports, metric data points, agent dependency/manifest/pool/messages/topics, classification ordering, secret zeroize, env profiles, hardware extended fields, LLM new fields, audit integrity chain

### Changed
- **Cyrius compiler target**: v1.9.4 → v1.11.4
- **CLAUDE.md**: full rewrite for Cyrius tooling (build commands, conventions, compiler notes)
- **docs/architecture/overview.md**: rewritten from `.rs` module map to `.cyr`
- **docs/development/roadmap.md**: updated for current state, v1.0.0 criteria, backlog

### Testing
- 198 tests (up from 58 passing / 107 total), 0 failures
- 15 benchmarks, no regressions

### Performance
- agent_id_new: 35ns, trace_context_child: 43ns, sandbox_config_default: 62ns
- agent_id_to_str: 212ns, version_to_str: 123ns, token_usage_update: 36ns
- accelerator_device_full: 148ns, inference_request_full: 488ns

## [0.95.0] - 2026-04-07

### Changed
- **Ported from Rust to Cyrius** — complete rewrite from 7,121 lines of Rust to 2,624 lines of Cyrius. Zero external dependencies. 107 KB library binary (was 8.7 MB .rlib).

### Added
- 123 constructors, 57 enums, 785 functions across 12 modules
- 6 traits via vtable dispatch: SpanCollector, MetricSink, TextMapPropagator, TextMapCarrier, AuditSink, SecretStore
- `#derive(Serialize)` on 9 struct types — auto-generated `_to_json` functions
- xorshift64 PRNG replacing `/dev/urandom` syscalls — agent_id_new 28ns (was 3,000ns pre-PRNG, Rust 45ns)
- Lazy initialization for vec/map fields — sandbox_config_default 61ns (was 1,000ns, Rust 40ns)
- Hex lookup table for UUID formatting — agent_id_to_str 215ns (was 308ns)
- Direct buffer version_to_str — 122ns (was 477ns with str_builder)
- 3-tier benchmark suite: 15 benchmarks matching Rust Criterion baseline
- `cyrb.toml` with `[lib]` section and `[[bench]]` for dep consumption
- `src/lib.cyr` library entry point for downstream consumers
- CI/CD workflows updated from Cargo to Cyrius (cyrb check, fmt, lint)

### Performance
- Cyrius beats Rust on 6 of 9 comparable benchmarks
- agent_id_new: 28ns vs Rust 45ns (1.6x faster)
- trace_context_child: 40ns vs Rust 53ns (1.3x faster)
- accelerator_device_full: 148ns vs Rust 711ns (4.8x faster)
- token_usage_update: 38ns

### Testing
- 107 tests (58 functional + 49 serde serialization)
- 15 benchmarks across 3 tiers (core, format, integration)

## [0.90.0] - 2026-04-02

### Added

#### Telemetry — OTel Alignment
- **Resource** — service identity struct (service_name, service_version, service_instance_id, attributes) for OTel signal attribution
- **SpanKind** — Internal/Server/Client/Producer/Consumer (OTel span kind, added to `Span`)
- **SpanEvent** — timestamped annotations on spans (OTel span events, added to `Span.events`)
- **SpanLink** — cross-trace span relationships (OTel span links, added to `Span.links`)
- `TraceContext::to_traceparent()` / `from_traceparent()` — W3C `traceparent` header format
- **AggregationTemporality** — Cumulative/Delta for metric data points (OTel-aligned)
- `MetricDataPoint.temporality`, `is_monotonic` — metric temporality and monotonicity fields
- `SpanStatus::Unset` variant (OTel default status)

#### Security — OCI Runtime Spec Alignment
- **SeccompProfile** — complete seccomp filter profile with `default_action`, `architectures`, `flags`, and `syscalls`
- **SeccompArch** — 17 target architectures (x86, x86_64, aarch64, riscv64, etc.)
- **SeccompArg** + **SeccompArgOp** — syscall argument-level filtering with 7 comparison operators
- `SeccompAction::Kill`, `KillProcess`, `Errno(u32)`, `Trace(u32)`, `Log` variants
- `SandboxConfig.apparmor_profile` — explicit AppArmor profile field
- `SandboxConfig.selinux_label` — explicit SELinux process label field
- `SandboxConfig.seccomp` — full `SeccompProfile` field
- **MountPropagation** — Private/Shared/Slave/Unbindable for filesystem mount rules
- `FilesystemRule.readonly`, `noexec`, `nosuid`, `nodev`, `propagation` — mount option fields

#### Security — Linux Capabilities
- **LinuxCapability** expanded from 19 to 39 variants (full Linux kernel capability set including CapBpf, CapPerfmon, CapCheckpointRestore, etc.)

#### Agent — Lifecycle Management
- **RestartPolicy** — Never/Always/OnFailure for failed agent restart control
- **HealthCheck** — liveness/readiness probe configuration (interval, timeout, retries, initial delay)
- `AgentConfig.restart_policy`, `max_restarts`, `health_check`, `startup_timeout_secs`, `shutdown_timeout_secs`

#### LLM — Multimodal + Structured Output
- **ContentBlock::Image** — base64/URL image inputs with media type
- **ContentBlock::Document** — base64/URL document inputs (PDF, etc.)
- **ContentBlock::Thinking** — model reasoning/extended thinking blocks
- **ToolChoice** — Auto/None/Required/Tool(name) for tool selection control
- **ResponseFormat** — Text/JsonObject/JsonSchema for structured generation
- `TokenUsage.cache_creation_input_tokens`, `cache_read_input_tokens` — prompt caching fields
- `InferenceRequest.system` — top-level system prompt (Anthropic API pattern)
- `InferenceRequest.logprobs`, `top_logprobs` — log probability output control
- `InferenceResponse.id` — provider-assigned response ID

#### Audit — Forensics & Compliance
- **AuditResult** — Success/Failure/Denied outcome for audited actions
- `AuditEntry.result`, `source_ip`, `target_resource`, `duration_ms`, `tags` — enriched audit fields

#### Classification — Extended PII
- **PiiKind** expanded: FullName, StreetAddress, BankAccountNumber, TaxId, NationalId, MedicalRecordNumber, BiometricData
- `ClassificationResult.confidence` — classification confidence score (0.0–1.0)

#### Hardware — Device Diagnostics
- **DeviceHealth** — Ok/Degraded/Failed/Unknown health status per device
- **MemoryType** — Gddr5/Gddr6/Gddr6x/Hbm2/Hbm2e/Hbm3/Lpddr4/5/5x/Ddr4/5 memory technology
- `AcceleratorDevice.power_watts`, `memory_bandwidth_gbps`, `memory_type`, `health`
- `AcceleratorFlags.sycl_available`, `openvino_available`, `directml_available`
- `AcceleratorSummary::by_vendor()` — filter devices by vendor

#### Config — Environment Profiles
- `EnvironmentProfile::Testing`, `Canary` — CI/CD and gradual rollout profiles

#### Telemetry — Logging & Exemplars
- **LogSeverity** — Trace/Debug/Info/Warn/Error/Fatal (OTel severity levels)
- **LogRecord** — structured log type with severity, body, attributes, trace correlation, and resource
- **Exemplar** — links a metric data point to a specific trace (OTel exemplar)
- `MetricValue::Histogram.min`, `max` — OTel histogram min/max fields

#### Agent — Lifecycle Hooks & Resources
- **LifecycleHooks** — pre_start/post_start/pre_stop/post_stop with command + timeout
- `AgentConfig.lifecycle_hooks` — optional lifecycle hook configuration
- `ResourceLimits.max_disk_bytes`, `network_bandwidth_bps` — disk and network resource limits

#### Security — OCI Process Fields
- `SecurityContext.run_as_user`, `run_as_group` — UID/GID for sandboxed processes
- `SecurityContext.readonly_root_filesystem` — immutable root filesystem
- `SandboxConfig.masked_paths`, `readonly_paths` — OCI maskedPaths/readonlyPaths
- `NamespaceConfig.time` — time namespace (kernel 5.6+)

#### Validation — Injection Breakdown
- **InjectionScores** — per-category scores: sql, xss, command, path_traversal, prompt_injection

#### Error — Numeric Codes
- `AgnostikError::code()` — numeric error code (1001–1010) for API versioning and client routing

#### LLM — Provider Routing
- **ModelCapabilities** — model metadata for routing: context window, supported features, pricing
- **RateLimitInfo** — provider rate limit state: limits, remaining, reset timer

### Changed
- **Breaking**: `SpanStatus` changed from `Copy` enum `{Ok, Error, Cancelled}` to `{Unset, Ok, Error { message }}` (OTel-aligned, Error now carries optional message)
- **Breaking**: `SeccompRule.syscall: String` replaced with `names: Vec<String>` + `args: Vec<SeccompArg>`
- **Breaking**: `SeccompAction::Deny` removed — use `Kill`, `KillProcess`, or `Errno(1)` instead
- **Breaking**: `SandboxConfig.mac_profile` split into `apparmor_profile` + `selinux_label`
- **Breaking**: `SandboxConfig.seccomp_rules` replaced with `seccomp: Option<SeccompProfile>`
- **Breaking**: `AgentStatus` — added `Restarting` and `Terminated` variants (consumers must update match arms)
- **Breaking**: `SecretMetadata` — added `kind`, `tags`, `owner`, `last_accessed_at`, `last_rotated_at` fields
- **Breaking**: `AuditSink` trait — added `verify_entry()`, `query()`, `seal()` methods (default impls provided)
- **Breaking**: `SecretStore` trait — added `rotate()`, `search_by_tag()` methods (default impls provided)
- **Breaking**: `AgentId::from_str` and `UserId::from_str` now return `AgnostikError` instead of `uuid::Error` — consistent with all other `FromStr` impls in the crate
- Error message capitalization standardized: "I/O error" → "i/o error" (lowercase, matching all other variants)

### Fixed
- `AgentId::from_str` and `UserId::from_str` error messages now include expected format and underlying cause (e.g., `"invalid agent id: foo (expected UUID, invalid character)"`)
- `TraceId::from_str` and `SpanId::from_str` error messages now include expected format (e.g., `"expected 32 hex digits"`, `"expected 16 hex digits"`)
- `Version::from_str` parse error improved: `"invalid version part: x"` → `"invalid version component: x (expected unsigned integer)"`

### Testing
- Integration tests expanded from 4 to 26, covering all feature-gated modules
- 249 tests total (223 unit + 26 integration)

### Maintenance
- `deny.toml`: removed 6 unmatched license allowances (GPL-3.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-DFS-2016, Zlib)
- `scripts/bench-history.sh`: fixed broken `--output-format bencher` flag (not valid in Criterion 0.5)
- Dependencies updated: uuid 1.22→1.23, libc, zerocopy, wasm-bindgen

## [2026.3.26] - 2026-03-25

### Added

#### Classification Module (new)
- **ClassificationLevel** — Public/Internal/Confidential/Restricted (ordered by sensitivity)
- **PiiKind** — Email, Phone, SSN, CreditCard, IpAddress, Passport, DriversLicense, DateOfBirth, Custom
- **ClassificationResult** — level, auto_level, rules_triggered, pii_found, keywords_found

#### Validation Module (new)
- **ValidationSeverity** — Low/Medium/High (ordered)
- **ValidationWarning** — code, message, severity, position, pattern
- **ValidationResult** — valid, sanitized, warnings, blocked, block_reason, injection_score

#### Hardware Module (new)
- **DeviceFamily** — Gpu, Tpu, Npu, AiAsic, Cpu
- **DeviceVendor** — Nvidia, Amd, Intel, Apple, Google, Qualcomm, Habana, Aws, Custom
- **AcceleratorFlags** — cuda, rocm, metal, vulkan, oneapi, tpu availability
- **AcceleratorDevice** — full device descriptor with VRAM, utilization, temperature, driver, compute capability
- **AcceleratorSummary** — device list with `by_family()` filter

#### Security RBAC & Sandbox Capabilities
- **Role** — Admin, Operator, Auditor, Viewer, Service
- **ConditionOperator** — Eq/Neq/In/Nin/Gt/Gte/Lt/Lte for permission conditions
- **PermissionCondition**, **RolePermission** — resource-level RBAC with conditions
- **TokenPayload** — JWT claims (sub, role, permissions, iat, exp, jti, email, display_name)
- **AuthContext** — agent_id + role + permissions
- **SeccompMode** — Disabled/Strict/Filter/Unsupported
- **SandboxCapabilities** — seccomp, landlock ABI version, cgroup v2, namespace detection

#### Audit Integrity Chain
- **IntegrityFields** — version, HMAC-SHA256 signature, previous_entry_hash
- **GENESIS_HASH** constant for chain initialization
- **IntegrityFields::genesis()**, **is_genesis()** helpers
- **AuditEntry** restructured with id, correlation_id, user_id, integrity chain
- **AuditSink** trait — append, verify_chain

#### LLM Module Expansion
- **MessageRole**, **Message**, **ContentBlock** — structured multi-turn conversation types replacing bare `prompt: String`
- **ToolDefinition**, **ToolCall**, **ToolResult** — function/tool calling types
- **SamplingParams** — top_p, top_k, frequency_penalty, presence_penalty, stop_sequences, seed
- **StreamEvent** — Delta, ToolCallDelta, Usage, Done, Error variants for streaming responses
- **FinishReason::ToolUse** variant for tool-calling flows
- `InferenceRequest` now supports `messages`, `tools`, and `sampling` fields
- `InferenceResponse` now uses `Vec<ContentBlock>` and `tool_calls`

#### Telemetry v2
- **MetricKind** (Counter, UpDownCounter, Gauge, Histogram), **MetricValue**, **MetricDataPoint**, **InstrumentDescriptor** — OTel-aligned metric types
- **SpanCollector** trait — pluggable span export backend (export, flush, shutdown)
- **MetricSink** trait — pluggable metric export backend
- **SpanId::Display** — hex-formatted display impl
- `TraceContext.trace_flags` (u8) and `trace_state` (String) for W3C Trace Context compliance
- `TraceContext::is_sampled()` and `TRACE_FLAG_SAMPLED` constant
- Child spans now propagate trace_flags and trace_state

#### Security Module Expansion
- **CgroupLimits** — memory_max/high, cpu_max/period/weight, pids_max (cgroups v2)
- **NamespaceConfig** — pid, net, mount, user, uts, ipc, cgroup namespace flags
- **IdMapping** — UID/GID mapping for user namespaces
- **LandlockFsAccess** (15 variants), **LandlockFsRule**, **LandlockNetAccess**, **LandlockNetRule**, **LandlockRuleset** — fine-grained Landlock v3 types
- **LinuxCapability** (19 POSIX caps), **CapabilitySet** (effective, permitted, inheritable, bounding, ambient)
- **SystemFeature** — renamed from `Capability` to resolve naming collision with Linux capabilities (`Capability` preserved as type alias)

#### Core Improvements
- **AgentId**, **UserId** moved to always-available `types` module (no longer feature-gated)
- **FromStr** impls for AgentId (UUID), Version (SemVer), TraceId (hex), SpanId (hex)
- **Display**, **FromStr** added to UserId (UUID parsing/formatting)
- **From\<Uuid\>** for AgentId and UserId
- **From\<std::io::Error\>** for AgnostikError (new `Io` variant)
- **From\<serde_json::Error\>** for AgnostikError (into `Serialization` variant)
- **Hash** added to AgentType, AgentStatus, LlmProvider, FinishReason, SystemFeature
- **Eq** added to ResourceLimits, ResourceUsage, TokenUsage
- **PartialEq**, **Eq** added to Capabilities
- Crate-root re-exports for all feature-gated modules' key types (consumers can use `agnostik::AuditEntry` instead of `agnostik::audit::AuditEntry`)

### Fixed
- `Version::default()` was hardcoded to `2026.3.25` — now correctly defaults to `0.0.0`
- `AgentConfig` serde shape changed based on `security` feature flag — `agent` feature now depends on `security`, fields always present
- `AgentManifest.requested_permissions` was conditionally compiled — now always present
- `Secret` derived `Debug` which leaked values in logs — now uses custom redacted Debug impl
- `Secret::Drop` had redundant `#[cfg(feature = "secrets")]` — removed
- `logging::init()` panicked if subscriber already set — replaced with `try_init()` returning `Result`
- Unused imports in `agent.rs` (FilesystemRule, FsAccess, NetworkAccess)
- Derivable `Default` impls flagged by clippy (Version, EnvironmentProfile)
- Redundant closures in benchmarks
- SPDX license identifier `GPL-3.0` → `GPL-3.0-only`

### Changed
- **Breaking**: `AgentEvent.agent_id`, `AgentInfo.id`, `SecurityContext.agent_id`, `AuditEntry.agent_id`, `CrashReport.agent_id` changed from `String` to `AgentId`
- **Breaking**: `AuditEntry.timestamp`, `CrashReport.timestamp`, `SecretMetadata.created_at`, `SecretMetadata.expires_at` changed from `String` to `chrono::DateTime<Utc>`
- **Breaking**: `InferenceResponse.text` replaced with `content: Vec<ContentBlock>`
- **Breaking**: `InferenceRequest.temperature` moved into `SamplingParams`
- **Breaking**: `logging::init()` renamed to `logging::try_init()` with `Result` return
- **Breaking**: `Version` serde format changed from struct `{"major":1,"minor":0,"patch":0}` to string `"1.0.0"` (matches SemVer convention)
- **Breaking**: `AgentManifest.version` changed from `String` to `Version`
- **Breaking**: `AgentDependency.min_version` changed from `Option<String>` to `Option<Version>`
- **Breaking**: `AgentMessage.timestamp` changed from `Option<DateTime<Utc>>` to `DateTime<Utc>` (now required)
- **Breaking**: `AuditEntry.user_id` changed from `Option<String>` to `Option<UserId>`
- **Breaking**: `IntegrityFields.version` changed from `String` to `Version`
- **Breaking**: `Span.start_ms` renamed to `started_at` and changed from `u64` to `chrono::DateTime<Utc>`
- `agent` feature now implies `security` feature
- `audit` and `secrets` features now depend on `chrono`
- `Secret` Serialize/Deserialize omission documented in doc comments

### Performance
- Serde benchmarks added: AgentId (37/60 ns ser/de), TraceContext (166/320 ns), SandboxConfig (326/336 ns)
- New serde benchmarks: InferenceRequest (700 ns/1.22 µs), AuditEntry (800 ns/1.02 µs), AcceleratorDevice (483/480 ns)
- No regressions in existing benchmarks

### Testing
- 194 tests total (190 unit + 4 integration), up from 49
- Serde roundtrip tests for all public types

## [0.1.0] - 2026-03-25

Initial extraction from agnos-common as standalone shared types crate.

### Modules
- **error** — AgnostikError with 9 variants, retriable classification
- **types** — Version, Capabilities, MessageType, SystemStatus, ComponentConfig
- **agent** — AgentId, UserId, AgentConfig, AgentManifest, AgentStatus, ResourceLimits, ResourceUsage, AgentRateLimit, StopReason
- **security** — SandboxConfig, Permission, NetworkAccess, NetworkPolicy, FsAccess, SeccompAction, SecurityContext, SecurityPolicy, Capability
- **telemetry** — TraceContext (W3C), TraceId, SpanId, Span, SpanStatus, TelemetryConfig, CrashReport, EventType
- **audit** — AuditEntry, AuditSeverity
- **llm** — LlmProvider, InferenceRequest, InferenceResponse, TokenUsage, FinishReason
- **secrets** — Secret (zeroize-backed), SecretMetadata
- **config** — EnvironmentProfile, AgnosConfig
