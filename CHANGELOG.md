# Changelog

## [Unreleased]

## [1.10.0] - 2026-10-14

**Minor:** the OTLP wire format, completed. Spans gain attributes, events and links; log records, number data
points and metrics get encoders; attribute values get a type, `AnyValue`. Every encoder is checked byte for byte
against protoc's encoding of the official schema. That check found v1.2.0 writing the span status code to a
reserved field, and the minor-cut audit found the encoders faulting on malformed values (F-027, LOW); both are
fixed.

Public API 979 → **994** (+15, no removals, no signature or layout change). 22 test files, **2,148 assertions, 0
failures**, on x86_64 and aarch64. All five `[deps.agnostik]` consumers build and pass their tests against it.

### Fixed — `Span_to_otlp_proto` wrote the status code to a reserved field

Since v1.2.0 the encoder wrote `Status.code` as field 1 (`7a 02 08 02` for ERROR). `trace.proto` reserves field 1
(it was the deprecated code) and puts the code in **field 3**. protoc decodes the old bytes as `status { 1: 2 }`:
an unknown field and no code, so a collector read every exported span as UNSET, and ERROR spans never showed as
errors. The code is now field 3 (`7a 02 18 02`). A status slot holding a value outside `SpanStatus` (for example
a `span_status_error` box stored there) now writes no status, instead of the box's address as a code.

v1.2.0's test pinned the wrong bytes because its reference was computed by hand from the `.proto` files.
`test_v120_otlp.tcyr` now expects field 3, and every vector in the new test is protoc's own encoding (see Tests).
No consumer calls `Span_to_otlp_proto`.

### Added — `AnyValue`: typed attribute values

The attribute maps (`span_attributes`, `sevt_attributes`, `slink_attributes`, `logrec_attributes`,
`mdp_attributes`) held untyped words, and an encoder cannot tell a `Str` pointer from an integer. Their values are
now **AnyValue boxes**. No consumer or test stored anything in these maps, so the contract is new rather than
changed ([ADR-004](docs/adr/004-otlp-attributes-are-anyvalue-boxes-encoded-canonically.md)).
- `enum AnyValueType`: `ANY_STRING` = 1, `ANY_BOOL`, `ANY_INT`, `ANY_DOUBLE`, `ANY_ARRAY`, `ANY_KVLIST`,
  `ANY_BYTES` = 7. The values are the field numbers of OTLP's `AnyValue` oneof, so 0 is never a type.
- `any_value_type_name` / `any_value_type_parse`, spelled as the OTLP/JSON keys (`"stringValue"` …
  `"bytesValue"`).
- Constructors `any_string`, `any_bool` (normalized to 0 / 1), `any_int`, `any_double` (raw IEEE-754 bits, as
  `metric_float`), `any_array` (a vec of boxes), `any_kvlist` (a cstr- or Str-keyed map of boxes) and `any_bytes`.
  Readers `any_value_type` / `any_value_val`.
- `AnyValue_to_otlp_proto(v, sb)`: the body of an `AnyValue` message.

### Added — OTLP encoders for spans, logs and metrics

All follow opentelemetry-proto v1.7.0 and write **canonical** bytes, identical to protoc's:
- fields in ascending number order;
- proto3 defaults omitted, but oneof members always written, even a zero `int_value` or `as_int`;
- minimal varints;
- attribute keys sorted bytewise, so equal attributes give equal bytes whatever the insertion order or the
  per-process hash seed.

The encoders:
- **`Span_to_otlp_proto`** gains `attributes` (9), `events` (11, `Span.Event`: time, name, attributes) and `links`
  (13, `Span.Link`: trace and span ids, attributes).
- **`LogRecord_to_otlp_proto(lr, sb)`** writes:
  - `time_unix_nano`;
  - `severity_number`, for 1..24, since `LogSeverity` values are OTLP's;
  - `severity_text` for the six named severities;
  - `body`, as a string `AnyValue`;
  - `attributes`, `trace_id` and `span_id`.
- **`MetricDataPoint_to_otlp_proto(dp, sb)`** writes a `NumberDataPoint`:
  - `time_unix_nano`;
  - `as_int` (sfixed64) for `metric_int`, or `as_double` for `metric_float`;
  - `attributes`;
  - `flags` = 1 (`NO_RECORDED_VALUE`) when the point holds neither.
- **`metric_to_otlp_proto(desc, points, sb)`** writes a `Metric`: name, description and unit from an
  `InstrumentDescriptor`, and the points inside the member its kind selects.
  - `METRIC_GAUGE` → `gauge`.
  - `METRIC_COUNTER` → `sum`, `is_monotonic` true.
  - `METRIC_UP_DOWN_COUNTER` → `sum`, non-monotonic.
  - A sum's temporality is mapped to OTLP's numbering: agnostik's `TEMPORALITY_CUMULATIVE` = 0 becomes OTLP's 2,
    and `DELTA` 1 stays 1.
  - It returns an invalid-argument `Err`, having written nothing, for:
    - `METRIC_HISTOGRAM` or an unknown kind (agnostik has no histogram data point);
    - a sum with an unknown temporality;
    - a sum whose points disagree on temporality (OTLP carries one per `Sum`).

**Bounds.** A 0 box or an unknown type encodes as an empty `AnyValue`. So do arrays and kvlists nested 16 levels
deep, and a box that already encloses itself (a cycle): never a crash or unbounded recursion.

**Not written** (no slot holds them):
- trace state and flags;
- `observed_time_unix_nano` and `start_time_unix_nano`;
- exemplars;
- the status message;
- histogram points;
- the `Resource` / `InstrumentationScope` envelopes.

Strings are written as given, and protobuf requires UTF-8 in `string` fields. The roadmap carries both.

### Changed — `src/proto.cyr` writes into a private buffer

The first draft built each nested message in a child `str_builder`. Measured, `str_builder_putc` costs about 33 ns
a byte, and each child builder about 200 ns plus two copies; a full span took 14.3 µs. The private `_stik_proto_*`
helpers are replaced by `_stik_pb_*`. A field reserves its worst case once and stores bytes directly. A nested
message is written in place behind a one-byte length that is patched when it closes, and only a body of 128 bytes
or more is shifted to fit a longer length. The result reaches the caller's `str_builder` in one copy. These
helpers were private, so the public signatures are unchanged.

The shift is a hand-written backward copy, because the stdlib has no `memmove`. A cyrius proposal for one, with
an ecosystem census of 14 hand-rolled copies and chunked-copy measurements, is filed at
`cyrius/docs/development/proposals/2026-10-14-memmove.md`.

### Tests — `tests/tcyr/test_v1100_otlp.tcyr` (103 assertions)

- **32 golden vectors**, each the bytes protoc (libprotoc 35.0) encodes from the same message in text format
  against the official `.proto` files. They cover:
  - every AnyValue member, including zero values, null payloads and an unknown tag;
  - nested arrays and kvlists, and Str-keyed maps;
  - a span with every new field, and a minimal OK span;
  - four log records, five data points and four metrics.
- **Length widening:** 25 sizes across the 1→2 and 2→3 byte varint boundaries, at every nesting level, against
  a reference encoder written in the test. Two sizes were also matched against protoc.
- **Bounds:** self-containing arrays and kvlists are cut at the cycle, a shared leaf is written twice, and 20
  nested arrays stop at 16.
- **Determinism:** reversed insertion orders encode identically; an empty key is omitted.
- **Errors:** histogram, unknown kind, mixed temporality and an unknown temporality each return `Err` and write
  nothing. A gauge ignores temporality.
- **Malformed values (F-027):** a negative-length or null-data `Str` encodes as empty, a later entry is intact, and
  a span with a 0 name and 0 ids encodes without a fault.
- **Fuzz:** 200 random spans, 200 log records and 200 metric sets, with random attribute trees, unknown tags and
  arbitrary bytes. A schema walker checks each encoding:
  - every field exists in the schema with its wire type;
  - lengths stay in bounds and fields ascend;
  - an AnyValue holds at most one member;
  - the KeyValue count equals the attribute entries.

  Each encoding must also be stable when repeated, and `Err` must come back exactly where documented. The walker
  is itself tested: it rejects the old field-1 status, truncation, two AnyValue members and a wrong KeyValue
  count.

Mutation-checked: each of 19 deliberate breaks fails the file. They include:
- the status written to field 1;
- unsorted keys, no cycle cut, an elided `int_value` zero;
- an ordinal temporality, wrong event or link field numbers, `as_int` written as a double;
- a dropped no-value flag, an unchecked severity range, mixed temporality accepted, an empty key written;
- three in the buffer: no shift on close, a forward shift, a buffer that never grows;
- each F-027 guard removed. Two of these crash the test binary; the other two fail assertions.

`test_v120_otlp.tcyr`'s primitive tests are ported to `_stik_pb_*`.

### Security

The minor-cut pass is recorded in [`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md)
§"v1.10.0 minor-cut pass". **One finding, fixed in this release:**

- **F-027 (LOW, defense in depth): the OTLP encoders could write out of bounds or read address 0 on malformed
  values.**
  - **Negative length:** `_stik_pb_bytes` trusted a `Str`'s length. A negative one, such as
    `str_new(p, 0 - 1000)` in an attribute value, moved the buffer's write position below zero, so the next field
    landed before the buffer, on the heap.
  - **Null pointers:** a 0 span name, trace id or span id was read from address 0 (`SIGSEGV`). That is pre-existing
    since v1.2.0.

  Both need a caller-built malformed struct; nothing parses external bytes into these. **Fixed:**
  - a negative length or 0 data pointer writes an empty field;
  - `_stik_pb_string` treats a 0 `Str` as empty;
  - `Span_to_otlp_proto` omits 0 ids.

  `test_malformed_values` reproduces each fault and fails, or crashes, when its guard is removed.

The re-scan of `src/` is clean (no raw `syscall(`, no `sys_system`, no system-path literal, no 64 KB stack
buffer), and the private-helper gate passes. F-001..F-026 remain closed.

### Closeout pass (CLAUDE.md, minor cut)

- **Tests:** 22 files, 2,148 assertions, 0 failures, on x86_64 and on aarch64. `cyrius audit` exits 0.
- **Dead code:** no private fn in a shipped module is unreferenced. **Floor:** of 983 fn definitions, 901 public
  fns are not called by library code (887 at 1.9.0, by the same method). The 15 new entry points add to it, and
  `log_severity_name` leaves it, now that the log encoder calls it. The untested count falls 294 → **288**.
- **Refactor:** the protobuf helpers were replaced by the in-place buffer (above). That was measured, not
  speculative.
- **Code review:** each encoder's output was compared with protoc's. The review of the buffer found F-027. The
  doc comments the change made stale were rewritten: the Span field table and the attribute getters.
- **Cleanup:** the private `_stik_proto_*` helpers are gone, with no remaining references.
- **Downstream:** `scripts/consumer-sweep.sh` on the release tree. 5 of 5 green, 0 regressed, 0 agnostik-involved
  duplicate definitions.
- **Version verify:** `VERSION`, the CHANGELOG header and the `dist/agnostik.cyr` banner read 1.10.0.

### Performance

New benchmark tier 5 (`tests/bcyr/agnostik.bcyr`), x86_64, cyrius 6.7.9:

| Row | First draft (str_builder) | Buffer |
|---|---:|---:|
| `span_otlp_full` (3 attributes, 2 events, a link, a status) | 14.3 µs | **4.10 µs** |
| `span_otlp_min` | 1.36 µs | **552 ns** |
| `log_record_otlp` (2 attributes, ids) | 3.18 µs | **1.46 µs** |
| `metric_otlp_sum` (a counter, 2 points) | 6.72 µs | **1.62 µs** |

Against the released 1.9.0 encoder, a minimal span (OK status, 3 ms duration) went from a median **735 ns to
477 ns** (−35%, 7 interleaved rounds, faster in all 7). It now carries an empty attribute map and still costs
less.

**Release gate.** `scripts/bench-regression.sh` ran against the 1.9.0 baseline on the final tree, at load 1.23:
**25 checked, 4 new, 0 regressions**. The A/B column is 15 interleaved rounds of the 1.9.0 tag tree against this
tree, both on cyrius 6.7.9, and each row is the median of per-round paired ratios. The A/B ran before the F-027
guards, which touch only OTLP code that none of the 25 rows run.

| benchmark | 1.9.0 baseline | 1.10.0 gate | Δ gate | 1.9.0 → 1.10.0, paired median of 15 | rounds slower |
|---|---:|---:|---:|---:|---:|
| agent_stats_from_json | 283 | 257 | -9.2% | -1.8% | 0/15 |
| trace_context_new | 1141 | 1090 | -4.5% | -0.4% | 4/15 |
| security_context_full | 669 | 616 | -7.9% | -0.3% | 5/15 |
| token_usage_to_json | 874 | 819 | -6.3% | -0.2% | 5/15 |
| agent_id_to_str | 737 | 690 | -6.4% | -0.1% | 5/15 |
| resource_limits_to_json | 976 | 915 | -6.2% | -0.1% | 4/15 |
| agent_id_roundtrip | 1026 | 971 | -5.4% | +0.0% | 7/15 |
| agent_stats_to_json | 675 | 633 | -6.2% | +0.0% | 7/15 |
| inference_request_full | 372 | 351 | -5.6% | +0.0% | 7/15 |
| token_usage_update | 33 | 31 | -6.1% | +0.0% | 2/15 |
| version_to_str | 108 | 101 | -6.5% | +0.0% | 2/15 |
| audit_entry_full | 2267 | 2193 | -3.3% | +0.1% | 9/15 |
| accel_flags_to_json | 1642 | 1517 | -7.6% | +0.1% | 9/15 |
| agent_id_new | 556 | 523 | -5.9% | +0.2% | 8/15 |
| trace_context_child | 573 | 529 | -7.7% | +0.4% | 9/15 |
| traceparent_format | 1812 | 1673 | -7.7% | +0.5% | 11/15 |
| version_roundtrip | 257 | 247 | -3.9% | +0.8% | 14/15 |
| resource_limits_from_json | 447 | 414 | -7.4% | +1.1% | 13/15 |
| injection_scores_to_json | 770 | 732 | -4.9% | +1.8% | 14/15 |
| message_build_3turn | 372 | 344 | -7.5% | +2.7% | 14/15 |
| token_usage_from_json | 403 | 386 | -4.2% | +4.0% | 14/15 |
| accelerator_device_full | 98 | 95 | -3.1% | +4.2% | 13/15 |
| sandbox_config_default | 23 | 22 | -4.3% | +4.5% | 13/15 |
| injection_scores_from_json | 302 | 280 | -7.3% | +5.4% | 14/15 |
| accel_flags_from_json | 695 | 664 | -4.5% | +6.3% | 14/15 |
| span_otlp_full | new | 4038 | – | – | – |
| span_otlp_min | new | 543 | – | – | – |
| log_record_otlp | new | 1463 | – | – | – |
| metric_otlp_sum | new | 1612 | – | – | – |

**Neutral.** The paired median row is **+0.1%**, and 23 of 25 rows are within ±5%. Two drift up, both slower in 14
of 15 rounds:
- `accel_flags_from_json`: +6.3%, about 45 ns;
- `injection_scores_from_json`: +5.4%, about 16 ns.

Neither runs any code this release changed, so this is the code-placement effect in the non-DCE bench build that
1.8.0 and 1.9.0 recorded. `token_usage_from_json` and `accelerator_device_full`, the two rows 1.9.0 saw move
down, are back up by 4.0% and 4.2%.

The release run is appended to `docs/benchmarks/history.csv` (29 rows, commit column `0f4fd8d`, HEAD at the
time), taken at load 1.23.

Binaries (`CYRIUS_DCE=1`):
- x86_64: 134,456 → **134,728 B**;
- aarch64: 896,192 → **896,464 B**;
- agnos: 134,808 → **135,080 B**;
- Windows: 875,520 → **889,344 B** (the Windows growth was not investigated);
- plain x86_64: 744,760 → 753,224 B.

`src/main.cyr` calls none of the new encoders, so DCE drops them. `dist/agnostik.cyr` grows 279,466 → **300,423
B**.

### Docs

- [ADR-004](docs/adr/004-otlp-attributes-are-anyvalue-boxes-encoded-canonically.md) records the three decisions:
  AnyValue boxes, canonical bytes checked against protoc, and the private buffer.
- **SECURITY.md:**
  - Supported Versions names **1.9.x** as the current line. It still said 1.2.x.
  - It states that there are no backport branches, and that 1.x upgrades are API-compatible.
  - Scope item 3 describes the completed encoders and their bounds.
- **CLAUDE.md:** the closeout doc sync now includes SECURITY.md's Supported Versions, so the table moves with each
  minor.
- **README and architecture overview:** the module rows and the OTLP example are updated.

### Verification

- **Clean-room CI replay.** A fresh copy of the release tree, starting from an empty `lib/`, was committed into a
  throwaway repo:
  - every `ci.yml` step except the bench passed, 17 of 17;
  - `deps --verify` reported 32 verified, 0 failed;
  - the aarch64 step's script passed: 22 files, 2,148 assertions under `qemu-aarch64`;
  - `cyrius audit` exits 0.
- **Gates:**
  - api-surface 994 = snapshot;
  - `scripts/doc-debt.sh check`: every fn in `src/` is documented;
  - `cyrius distlib --check` fresh;
  - lint, fmt, vet and type-check clean;
  - DCE builds warning-free on x86_64, aarch64, agnos and Windows.
- **Downstream:** 5 of 5 green.

## [1.9.0] - 2026-10-14

**Minor:** the last enum gets its name/parse pair, every function is documented (`cyrius audit` now
exits 0), and three defects the documentation pass found are fixed:
- OTLP and every other construction-time timestamp were monotonic, not Unix time;
- span-id text was byte-reversed against the OTLP bytes;
- two JSON string fields were written unescaped.

Public API 977 → **979** (+2, no removals, no signature or layout change). 21 test files, **2,045
assertions, 0 failures**, on x86_64-linux and on aarch64. All five `[deps.agnostik]` consumers build
and pass their tests against it.

### Fixed — timestamps are Unix time (OTLP `start_time_unix_nano` was boot-relative)

Every construction-time stamp came from `clock_now_ns()`. That is the stdlib's monotonic clock:
`CLOCK_MONOTONIC` on Linux, uptime on agnos, QPC on Windows. `Span_to_otlp_proto` wrote it as
`start_time_unix_nano` (and `end_time_unix_nano`), so an OTLP collector placed every span a few
days after 1970. The same stamp sat in audit entries, agent and topic messages, and secret
metadata. There it crosses process and machine boundaries, sits in an audit chain, and lives beside
an `expires_at` meant in wall-clock time. All ten stamp sites now use **`clock_epoch_ns()`**, Unix
time in ns on every target (whole seconds on agnos):

| struct | accessor |
|---|---|
| Span | `span_started_at` |
| LogRecord | `logrec_timestamp` |
| CrashReport | `crash_timestamp` |
| SpanEvent | `sevt_timestamp` |
| Exemplar | `exem_timestamp` |
| MetricDataPoint | `mdp_timestamp` |
| AuditEntry | `aentry_timestamp` |
| AgentMessage | `amsg_timestamp` |
| TopicMessage | `tmsg_timestamp` |
| SecretMetadata | `smeta_created_at` |

Span durations are unaffected: they come from `span_set_duration_ms`, never from the clock. Nothing
in agnostik compared a stamp against `clock_now_ns()`. No consumer reads any of these fields, and
the tests only asserted `> 0`.

### Fixed — span-id text is the bytes in memory order, as OTLP and W3C use

`span_id_to_str` printed `load64(id)` most significant byte first, and `span_id_from_str` stored the
parsed value with `store64`. Both were self-consistent, so round-trips passed, but on a
little-endian target the 16 hex digits were the **reverse** of the 8 bytes `Span_to_otlp_proto`
emits. A span correlated by its traceparent `parent-id` showed a different id in the collector. Both
now work byte by byte, as `trace_id_to_str` / `trace_id_from_str` always did: digits 2i..2i+1 are
byte i.

- **Text round-trips are unchanged.** `tctx_from_traceparent` → `tctx_to_traceparent` is still
  byte-exact.
- **What changes** is the text printed for a given in-memory id, and the bytes a given text parses
  to. Both now agree with OTLP.
- **Consumer impact:** none of the five consumers calls the span-id text functions, the traceparent
  functions or `Span_to_otlp_proto`.

### Fixed — JSON string fields are escaped (F-026)

`TelemetryConfig_to_json` wrote `export_endpoint`, and `AgentInfo_to_json` wrote the agent `name`,
between hand-placed quotes with no escaping. A `"` ended the string early, and the rest was parsed
as JSON. A name like `x","status":"Failed` injected a key, and it came first, so even agnostik's own
`AgentInfo_from_json` read the injected status. Both now use the stdlib's
`str_builder_add_json_str` (escapes `"`, `\`, `\b \t \n \f \r`, and other control characters as
`\u00XX`). Output for ordinary strings is byte-identical, so `test_v110_serde_golden` is
unchanged. A sweep of every hand-written `_to_json` found no other raw string field; the remaining
raw adds are a hex UUID, internal search keys, protobuf bytes and traceparent hex. See Security.

### Tests — `tests/tcyr/test_v190_fixes.tcyr` (27 assertions)

- **All ten stamps are plausible Unix time.** Each is after 2020-09-13 and within 60 s of
  `clock_epoch_ns()`. The OTLP encoding carries the same value as field 7 (tag `0x39` followed by
  the 8 bytes).
- **Span id:** `"0102030405060708"` parses to bytes 01..08. A raw byte id prints in order, and
  uppercase input prints lowercase. The OTLP `span_id` field (`0x12 0x08` and the 8 bytes) matches
  the text, and a traceparent round-trips byte-exact.
- **Escaping:** an endpoint containing `"`, `\` and a newline serializes to the exact escaped JSON
  and round-trips. An injection-shaped agent name round-trips as one string, with the real status
  intact.

Mutation-checked. Restoring the monotonic span stamp fails one assertion, restoring the reversed
span-id printing fails four, and restoring either raw JSON write (the endpoint or the name) fails
two.

### Changed — every function is documented; `cyrius audit` passes for the first time

All **849** grandfathered undocumented fns in `src/` now carry a doc comment, as do the **195**
test and bench fns. **`cyrius audit` exits 0** — fmt, lint, docs, tests and bench all pass. Since
1.3.6 its docs phase had been the one reason it could not serve as a single-command gate.

| module | fns | module | fns |
|---|---:|---|---:|
| agent | 174 | hardware | 55 |
| security | 152 | audit | 44 |
| llm | 148 | types | 40 |
| telemetry | 126 | validation | 26 |
| error | 18 | main (test harness) | 23 |
| config | 22 | secrets | 21 |

Plus tests/ and benches: 195 across 21 files.

**Comments only, proven three ways:**
- Each file with its full-line `#` comments stripped is byte-identical to the pre-pass copy, for
  all 12 `src/` modules and all 21 test and bench files.
- The `CYRIUS_DCE=1` x86_64 and aarch64 binaries are byte-identical to the pre-pass builds. The
  build is deterministic, and an added comment line alone was shown to leave the binary unchanged
  first, so this check would catch any code change.
- Every test file's assertion count is unchanged.

**How the comments were written.** Constructors carry a block with the struct's size and its
`+offset field (default)` layout, read from the constructor body. Getters say what the field is:
its kind (`Str` vs cstr, vec of what, enum type, pointer to which struct) and its unit where the
code establishes one. Setters say so and that they return 0. Where the code does not settle a
meaning, the comment says only what is certain, e.g. "zeroed, no accessor" or "this module fixes
no unit". Units were taken from the pre-port Rust original only where the Cyrius defaults match it
exactly: the 1000 / 30000 / 60000 ms retry delays, and the 2000 = 2.0x backoff multipliers.

**The gate is now absolute.** `docs/undocumented.baseline` is deleted. `scripts/doc-debt.sh check`
fails on **any** undocumented fn in `src/`, naming it, and its `update` mode is gone. It passes on
this tree, and a planted `fn doc_gate_mutant` fails it. CI's step is renamed "Verify
documentation", and CLAUDE.md's rule and Quick Start, and README's gate list, describe the new
rule.

The audit counts differently from `cyrius doc --check`. It reported **585** for what the per-file
check counts as 195 test and bench fns, and documenting those 195 brought it to "docs complete".
The two counts were never comparable, as `state.md` had noted. Track the per-file figure.

### Found while documenting

Writing a comment for every fn meant reading every fn, and a few read wrong. The first three are
fixed above. The rest are slotted on the roadmap (v1.11.0):
- **OTLP timestamps.** Spans, log records, exemplars and data points are stamped with
  `clock_now_ns()`. That is `CLOCK_MONOTONIC` on Linux, uptime on agnos and QPC on Windows.
  `Span_to_otlp_proto` writes it as `start_time_unix_nano`, so a collector sees boot-relative
  times. Audit and secret timestamps have the same source.
- **Span ID text vs bytes.** `span_id_to_str` prints `load64(id)` most significant byte first,
  while `Span_to_otlp_proto` emits the 8 raw bytes. On little-endian targets the traceparent form
  and the OTLP form of one span ID differ. Text round-trips still work.
- **`TelemetryConfig_to_json`** writes `export_endpoint` between quotes without JSON escaping.
- **Getters with no setter** (the F-021 class), across the modules. Among them:
  - `samp_freq_penalty`, `samp_pres_penalty`, `ireq_prompt`, `RateLimitInfo`;
  - the UTS / IPC / cgroup / time namespace flags, `sandbox_isolate_net`, the device-rule
    major / minor numbers;
  - several config, validation and audit-retention fields.

  Some structs (SecurityContext, SandboxCapabilities, TokenPayload) have no setters at all. The
  AppArmor and SELinux setters have no getters.
- **Unset optional vtable slots.** The vtable constructors leave the optional slots at 0, and
  `as_verify_entry` and `as_seal` dispatch without checking, so they call address 0. The audit
  `query` and secret-store `rotate` / `search_by_tag` slots have no dispatcher, so callers face
  the same hazard.
- **Global hand-offs** (`textmap_set`, `propagator_inject`, `ss_put`) pass an argument through a
  global, so they are not reentrant.
- `agent_id_from_str` skips the four dash positions without checking that they hold a `-`, and
  does not check the UUID version and variant bits.
- Newer Landlock rights (`FS_IOCTL_DEV`, `FS_RESOLVE_UNIX`, `NET_BIND_UDP`,
  `NET_CONNECT_SEND_UDP`) are missing from the enums, and `RlimitType` has no mapping to
  `RLIMIT_*`. Both were found at 1.8.0.


### Added — `restart_policy_name` / `restart_policy_parse`: the last enum gets its pair

The 26th enum, held back from 1.8.0. `"Never"`, `"Always"`, `"OnFailure"`: the Kubernetes
`restartPolicy` spelling, by the same rule as the other 25. Exact match. argonaut's
systemd-style `"on-failure"` is rejected here, because it belongs to
`argonaut_restart_policy_parse`. Public API 977 → **979**. Every public enum with a member now
has both `*_name` and `*_parse`, as CLAUDE.md requires.

The hold-back's three steps landed in order:
1. **argonaut 1.15.4** made its `RestartPolicy` and `SeccompAction` value-identical to agnostik's
   and renamed its parser `argonaut_restart_policy_parse`.
2. **kybernet 1.7.9** took that release, renamed its one call, and added a test that pins the
   values its link uses.
3. This pair is the third step.

`scripts/consumer-sweep.sh` against kybernet 1.7.9, which links this tree and argonaut 1.15.4:
**882 of 882, 0 agnostik-involved duplicate definitions**. The 1.8.0-era sweep had reported
kybernet REGRESSED on `restart_policy_parse`. All five consumers are green.

`tests/tcyr/test_v180_enum_parse.tcyr` gains the generated-style roundtrip block for the three
members, the Kubernetes spellings pinned by hand, rejections of argonaut's `"on-failure"` and of
lowercase `"never"`, and the three values pinned as the argonaut lockstep: 427 → **444**
assertions. Suite: 20 files, **2,018 assertions, 0 failures**, on x86_64 and aarch64.

### Security

The minor-cut pass is recorded in [`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md)
§"v1.9.0 minor-cut pass". **One finding, fixed in this release:**

- **F-026 (LOW): two JSON string fields were serialized unescaped.** `TelemetryConfig_to_json`'s
  `export_endpoint` and `AgentInfo_to_json`'s `name` could break or inject JSON when they held a
  `"`. The `name` is caller-supplied, and the endpoint usually comes from config. Fixed by
  `str_builder_add_json_str` (above).

The other defects the documentation pass found are correctness and robustness issues, not security
findings: the timestamps and span-id order (both fixed), and the getter-only fields, unset
optional vtable slots, global hand-offs and `agent_id_from_str` validation (roadmap v1.11.0). The
re-scan of `src/` is clean (no raw `syscall(`, no `sys_system`, no system-path literal, no 64 KB
stack buffer), and the private-helper gate passes. F-001..F-025 remain closed.

### Closeout pass (CLAUDE.md, minor cut)

- **Tests:** 21 files, 2,045 assertions, 0 failures, on x86_64 and on aarch64. `cyrius audit` exits
  0.
- **Dead code:** no private fn in a shipped module is unreferenced. **Floor:** of 955 fn
  definitions, 887 public fns are not called by library code. That count now excludes comment
  mentions; 1.8.0's 790 included them. The untested count stays at **294**.
- **Refactor:** none; the fixes are local.
- **Code review:** the comments-only proofs for the documentation pass, and mutation checks for
  every fix. The doc comments the documentation pass wrote that the fixes made false were
  corrected:
  - "monotonic, not Unix time" on the stamp accessors;
  - "the 8 bytes are read as one 64-bit value" on `span_id_to_str`;
  - "written verbatim, not JSON-escaped" on `TelemetryConfig_to_json`.
- **Cleanup:** `docs/undocumented.baseline` removed with the gate change.
- **Downstream:** `scripts/consumer-sweep.sh` on the release tree. 5 of 5 green, 0 regressed, 0
  agnostik-involved duplicate definitions; kybernet 1.7.9 links this with argonaut 1.15.4.
- **Version verify:** `VERSION`, the CHANGELOG header and the `dist/agnostik.cyr` banner read
  1.9.0.

### Performance

`scripts/bench-regression.sh` against the 1.8.0 baseline, taken at load 1.2: **25 checked, 0 new,
0 regressions.** The A/B column is 15 interleaved rounds of the 1.8.0 tag tree against this tree,
both on cyrius 6.7.9. Each row is the median of per-round paired ratios.

| benchmark | 1.8.0 baseline | 1.9.0 gate | Δ gate | 1.8.0 → 1.9.0, paired median of 15 | rounds slower |
|---|---:|---:|---:|---:|---:|
| token_usage_from_json | 430 | 391 | -9.1% | -14.1% | 0/15 |
| accelerator_device_full | 101 | 95 | -5.9% | -10.9% | 0/15 |
| accel_flags_from_json | 671 | 666 | -0.7% | -3.3% | 1/15 |
| injection_scores_to_json | 721 | 735 | +1.9% | -0.9% | 2/15 |
| security_context_full | 617 | 662 | +7.3% | -0.5% | 5/15 |
| agent_stats_from_json | 262 | 273 | +4.2% | -0.4% | 2/15 |
| injection_scores_from_json | 281 | 292 | +3.9% | -0.3% | 3/15 |
| resource_limits_to_json | 905 | 930 | +2.8% | -0.3% | 3/15 |
| agent_id_roundtrip | 941 | 1006 | +6.9% | -0.3% | 4/15 |
| token_usage_to_json | 811 | 842 | +3.8% | -0.2% | 2/15 |
| resource_limits_from_json | 414 | 427 | +3.1% | -0.2% | 3/15 |
| agent_id_to_str | 679 | 708 | +4.3% | +0.0% | 7/15 |
| sandbox_config_default | 21 | 23 | +9.5% | +0.0% | 1/15 |
| token_usage_update | 31 | 32 | +3.2% | +0.0% | 0/15 |
| version_to_str | 99 | 104 | +5.1% | +0.0% | 4/15 |
| traceparent_format | 1704 | 1814 | +6.5% | +0.1% | 8/15 |
| audit_entry_full | 2165 | 2207 | +1.9% | +0.1% | 9/15 |
| agent_stats_to_json | 624 | 643 | +3.0% | +0.1% | 10/15 |
| accel_flags_to_json | 1537 | 1592 | +3.6% | +0.2% | 9/15 |
| agent_id_new | 547 | 546 | -0.2% | +0.4% | 8/15 |
| trace_context_child | 532 | 567 | +6.6% | +0.5% | 8/15 |
| version_roundtrip | 235 | 253 | +7.7% | +0.8% | 14/15 |
| trace_context_new | 1083 | 1132 | +4.5% | +0.8% | 11/15 |
| inference_request_full | 338 | 358 | +5.9% | +1.4% | 14/15 |
| message_build_3turn | 334 | 360 | +7.8% | +2.8% | 15/15 |

**Neutral.** The paired median row is **0.0%**, and 23 of 25 rows are within ±5%.
`traceparent_format`, which now runs the byte-order span-id path, is flat (+0.1%). The two rows 1.8.0
attributed to code placement in the non-DCE bench build moved back down:
`token_usage_from_json` −14.1% and `accelerator_device_full` −10.9%, faster in all 15 rounds. That
is consistent with the explanation: this release shifts the layout again. `message_build_3turn` is
+2.8% (slower in 15 of 15), about 10 ns.

The release run is appended to `docs/benchmarks/history.csv` (25 rows, commit column `61c875f`,
HEAD at the time), taken at load 1.85. The gate passes against it.

Binaries (`CYRIUS_DCE=1`): x86_64 134,408 → **134,456 B**, aarch64 896,144 → **896,192 B**, agnos
134,808 B, Windows 875,520 B. The doc comments add nothing to a binary (shown byte-identical
above); the bytes are the fixes and the two new fns. `dist/agnostik.cyr` grows 197,447 →
**279,466 B**, almost all of it the new comments.

### Verification

- **Clean-room CI replay.** A fresh copy of the release tree, starting from an empty `lib/`, was
  committed into a throwaway repo. Every `ci.yml` step except the bench passed, 17 of 17. So did
  the aarch64 step's script (21 files, 2,045 assertions under `qemu-aarch64`), and `cyrius audit`
  exits 0.
- **Gates:** api-surface 979 = snapshot; `scripts/doc-debt.sh check`: every fn in `src/`
  documented; `cyrius distlib --check` fresh; `deps --verify` 32 / 32; lint, fmt, vet and
  type-check clean.
- **Downstream:** 5 of 5 green.

## [1.8.0] - 2026-10-14

**Minor: cyrius 6.7.9, and `*_name` / `*_parse` for 25 more enums.** Public API 927 → **977**
fns (+50, no removals, no signature, layout or wire-format change). 20 test files, **2,001
assertions, 0 failures**, on x86_64-linux and on aarch64. All five `[deps.agnostik]` consumers
build and pass their tests against it.

Two items move to v1.9.0:
- **The documentation burn-down.** It rides a minor, not a release of its own.
- **The `RestartPolicy` pair.** It waits for kybernet to take argonaut 1.15.4, which is cut, so
  that agnostik's pair does not collide with argonaut's inside kybernet.

### Added — `*_name` / `*_parse` for 25 more enums (50 fns)

CLAUDE.md requires every public enum to have both. v1.4.0 (F-018) covered the 31 enums that
already had a `_name`, and v1.5.0 added `LinuxCapability`. This adds the pair to 25 of the last
26:

| Module | Enums |
|---|---|
| agent | `ProbeType`, `StopReason`, `CircuitBreakerState`, `LabelOperator` |
| error | `AgnostikErrorCode` (`agnostik_err_code_name` / `_parse`) |
| llm | `ContentBlockType`, `CacheControl`, `ToolChoice`, `ResponseFormat`, `SafetyCategory`, `SafetyProbability`, `StreamEventType`, `BatchStatus` |
| security | `ConditionOperator`, `SeccompMode`, `SystemFeature`, `LandlockFsAccess`, `LandlockNetAccess`, `LandlockScope`, `DeviceType`, `RlimitType` |
| telemetry | `EventType`, `MetricKind`, `AggregationTemporality`, `MetricValueType` |

The rules are F-018's:
- `_name` returns `"Unknown"` for a value outside the enum.
- `_parse` is an exact, case-sensitive match returning `Ok(member)` or an invalid-argument `Err`.
- No new enum has a member that owns `"Unknown"`, so `_parse("Unknown")` is an `Err`.

Every fn has a doc comment, so the documentation-debt baseline stays at 849. Public API 927 →
**977** (+50, no removals).

**Spelling.** Each pair follows the library's rule: an external standard's spelling where one
clearly exists, otherwise the Rust variant in PascalCase. The standards, each named in its doc
comment, are:
- **Kubernetes** label-selector operators: `In`, `NotIn`, `Exists`, `DoesNotExist`.
- **LLM APIs:** the lowercase wire spellings already used by `finish_reason_name` (`tool_use`,
  `json_object`, `in_progress`, `ephemeral`). Safety ratings use Gemini's names
  (`HARM_CATEGORY_HATE_SPEECH` …, `NEGLIGIBLE` … `HIGH`); agnostik's five categories are exactly
  Gemini's five, civic integrity included. A field read from a provider response therefore
  parses as-is.
- **Landlock:** the Rust `landlock` crate's names (`Execute` … `Truncate`, `BindTcp`,
  `AbstractUnixSocket`).
- **Devices:** the OCI runtime spec's `"c"` / `"b"` / `"a"`.
- **Resource limits:** the `limits.conf(5)` item names (`nofile`, `as`, `fsize` …). As with
  `capability_name`, a config written against Linux documentation parses without translation.
- **OpenTelemetry:** the instrument and data-point names (`UpDownCounter`, `ExponentialHistogram`).

**Kernel facts, checked against the kernel's uapi headers rather than memory:**
- The `Landlock*` members are the kernel's bit indices (`LL_EXECUTE` = bit 0 … `LL_TRUNCATE` =
  bit 14; `1 << v` is the flag). The roadmap's "bit sets" description was wrong: the values
  are ordinals.
- `SeccompMode`'s values are `SECCOMP_MODE_*` (0, 1, 2).
- `RlimitType`'s values are **not** `RLIMIT_*`: `RLIM_NOFILE` is 0, which is `RLIMIT_CPU`.
  `rlimit_type_name`'s doc comment says so. No repo passes these values to `setrlimit(2)` today.
- `AggregationTemporality`'s values are not OTLP's either; the v1.9.0 roadmap item notes this.

**`RestartPolicy` is held back.** argonaut defines its own `enum RestartPolicy`, with agnostik's
constant names in a different order (ALWAYS = 0 vs agnostik's NEVER = 0), and a one-argument
`restart_policy_parse` that reads systemd-style `"on-failure"`. kybernet compiles both into one
unit, so agnostik's new `restart_policy_parse` would have been silently replaced by argonaut's
("last definition wins"). That is F-025's class, and `scripts/consumer-sweep.sh` reported kybernet
**REGRESSED** on it. The plan, in order:
1. An argonaut patch, on its current pin, aligns `RestartPolicy` and `SeccompAction` with
   agnostik's values and renames its config parser. Done: **argonaut 1.15.4**, cut the same day.
2. kybernet picks up that argonaut release.
3. agnostik then adds the `RestartPolicy` pair (roadmap v1.9.0).

Step 1 is written and tested in argonaut's working tree (984 assertions, 0 failures) and awaits
an argonaut release. A scratch build of kybernet resolved both in-flight trees, with agnostik's
`RestartPolicy` pair added back:
- With kybernet's one call renamed, it builds with no duplicate warning and passes 876 of 876.
- With the call left unrenamed, it still compiles, but its config tests fail. So step 2 must
  rename that call in the same change as the argonaut bump.
- Against agnostik ≤ 1.8.x, without the pair, an unrenamed call fails the build loudly
  (`undefined function`). The silent case appears only once the pair lands.

A collision pre-check had reported none. It was vacuous: under zsh an unquoted `$names` did not
word-split, so the loop built one pattern that matched nothing. The sweep caught the collision.
A Python re-scan of every local checkout and the 6.7.9 stdlib found `restart_policy_parse`
(argonaut, vendored in kybernet), `restart_policy_name` (stiva) and `event_type_name` (szal).
stiva and szal do not depend on agnostik today.

**Tests.** `tests/tcyr/test_v180_enum_parse.tcyr` has **427 assertions**:
- **Generated half**, from the same table that generated the fns: `parse(name(v)) == v` for all
  132 members, plus a rejection, the `"Unknown"` rejection, and `-1` / out-of-range naming
  `"Unknown"` for each enum.
- **Hand-written half**, because a table that drives both sides would let a misspelling
  roundtrip: every external-standard spelling, the kernel numbers above, and exact-match
  rejections (lowercase, uppercase, `RLIMIT_NOFILE`, a leading space, an empty string).

Three mutations were each caught: `"as"` misspelled `"address_space"` (by the hand check), a
`_parse` arm returning the wrong member (by the roundtrip), and a deleted `_name` arm (three
assertions). With the pair held back, the sweep is **5 of 5 green**, with 0 agnostik-involved
duplicates. 20 test files, **2,001 assertions, 0 failures**, on x86_64 and aarch64.

### Changed — cyrius pin 6.6.20 → **6.7.9**; stdlib re-vendored

The first move onto the 6.7.x line, a language minor. 6.7.x adds real traits, `const` and
`const fn`, `bool` with `true` / `false`, if-expressions, `loop` / `do`, compound assignment,
tuples, default and named arguments, checked `dyn`, and an opt-in bounds-checked mode. **No source
change was needed**, and the cyrius 6.7.0–6.7.9 CHANGELOG does not mention agnostik.

**The changes that could have altered agnostik's behaviour silently were checked against its
source, not assumed.** agnostik uses none of the affected patterns:
- 6.7.8 made `*p` through a narrow `*T` read and write at the element's width; agnostik has no
  narrow typed pointers.
- 6.7.8 made a signed slice element read sign-extended; agnostik has no slice types.
- 6.7.8 stopped an untyped parameter from being treated as a slice.
- Before 6.7.3 the lexer silently dropped a lone `!`, so `!x` compiled as `x`; agnostik has no
  lone `!`.
- 6.7.x reserves `dyn` and adds `const`, `bool`, `true`, `false`, `do` and `loop`; none is an
  agnostik identifier.

The compile-time checks the line added (calls to later fns arity-checked, constant indexes
range-checked, `for` steps ending at their `)`) all pass: every target builds with zero warnings.

`lib/` re-vendored with `cyrius lib sync` + `cyrius deps`. All **32** files are byte-identical to
the 6.7.9 snapshot: 21 changed content (`alloc`, `alloc_agnos`, `atomic`, `bayan`, `boxed`,
`fmt`, `fnptr`, `hashmap`, `hashmap_fast`, `io`, `str`, `tagged`, `trait`, `vec`, and `syscalls`
with its six peers), and 11 did not. Nothing new, nothing gone.

**The pin move needed `cyrius deps --relock`.** It is the first pin move with a committed lock;
1.6.5 created the lock fresh. `lib sync` re-locks the 26 files it copies under the new pin.
`cyrius deps` then found the two transitive leaves it adds, `atomic` and `boxed`, with 6.6.20
hashes in a lock already stamped 6.7.9. It refused them as "cyrius.lock and the pinned stdlib
snapshot DISAGREE under an unchanged pin" and left the stale 6.6.20 copies in place, which
`deps --verify` still passed against those hashes. `cyrius deps --relock` accepted the new content.
A fresh checkout with an empty `lib/` then reproduces the committed lock byte for byte (32 / 32).
CLAUDE.md's refresh note now spells out the step.

**Regenerated outputs are unchanged.** `cyrius distlib` reproduces `dist/agnostik.cyr` and its
12-leaf sidecar byte for byte, and the api-surface snapshot stays at 927 fns. No consumer-facing
artifact moves.

**Binaries** (`CYRIUS_DCE=1`, identical source): x86_64 132,552 B, the same size as on 6.6.20,
but 57,335 bytes differ (code generation changed); aarch64 828,776 → 828,752 B; agnos 132,896 B;
Windows 848,896 → 849,408 B. Plain x86_64 build 718,280 B.

**Security in range.** 6.7.x published two advisories:
- The cyriusly `cmdtools` current-directory script: CI never runs `cyriusly`.
- The dropped-`!` lexer bug: agnostik has no lone `!`, so its compiled code never differed.

The 6.7.6 `lib/fnptr.cyr` fix for `fncall8`'s swapped arguments 7 and 8 does not reach agnostik,
which uses only `fncall2`. The 6.7.9 release verifies against the cyrius release key pinned in
`scripts/ci-install-cyrius.sh`. Its `lib/` and every binary match the local install, apart from
the bundled `ci.sh` helper.

**Verification:**
- Every `ci.yml` step except the bench passes on 6.7.9, 17 of 17.
- 19 test files, **1,574 assertions, 0 failures**, on x86_64 and under `qemu-aarch64`.
- `scripts/consumer-sweep.sh`: 5 of 5 green.

**Performance:** neutral; see the Performance section below. The first measurement, taken at
load 14 while the pin moved, was too noisy to use.

### Security

The minor-cut security pass is recorded in
[`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md) §"v1.8.0 minor-cut pass".
**No findings.**
- **Advisories in range.** cyrius 6.7.0–6.7.9 published two. Neither reaches agnostik: CI never
  runs `cyriusly`, and `src/` has no lone `!`. The `fncall8` argument-order fix does not reach it
  either, since agnostik uses only `fncall2`.
- **New code.** The 50 new fns compare a string against fixed literals or an integer against
  constants. They allocate only the error box on rejection, and read no input past `str_eq_cstr`.
- **Re-scan.** The CI security scan is clean: no raw `syscall(`, no `sys_system`, no `/etc` /
  `/bin` / `/sbin` literal, no stack buffer of 64 KB or more. The private-helper gate passes.
- **Integrity.** The 6.7.9 toolchain verifies against the pinned release key.

### Closeout pass (CLAUDE.md, minor cut)

- **Tests:** 20 files, 2,001 assertions, 0 failures, on x86_64 and on aarch64.
- **Dead code:** no private fn in a shipped module is unreferenced. **Floor:** of 953 textual fn
  definitions, 790 public fns are referenced by no library code (consumer entry points; 767 at
  1.7.0). The count is textual, and each new `_name` doc comment names its `_parse`, so it rises
  by less than 50. The untested count stays at **294**, because every new fn is tested.
- **Refactor:** none. The 50 fns come from one table, the same one their test is generated
  from, so there is no parallel code to consolidate.
- **Code review:** the hold-back of `RestartPolicy`, the generator's comment wrapping, and a
  vacuous collision pre-scan, which the sweep caught. All are described above.
- **Cleanup:** README's toolchain line read 6.6.20.
- **Downstream:** `scripts/consumer-sweep.sh` on the release tree. 5 of 5 green, 0 regressed,
  0 invalid, 0 agnostik-involved duplicate definitions.
- **Version verify:** `VERSION`, the CHANGELOG header and the `dist/agnostik.cyr` banner read
  1.8.0. The release workflow's version gate accepts tag `1.8.0`.
- **Full build from clean:** see Verification.

### Performance

`scripts/bench-regression.sh` against the 1.7.1 baseline: **25 checked, 0 new, 0 regressions**.
It was taken at load 0.6. The A/B column comes from 15 interleaved rounds of the 1.7.1 tag tree
on 6.6.20 against this tree on 6.7.9, with order alternating each round. Load climbed from 0.6 to
5.4 during the A/B, so each row is the median of **per-round paired ratios**: the two trees ran
back-to-back in each round, which cancels the load spikes that hit single rounds.

| benchmark | 1.7.1 baseline | 1.8.0 gate | Δ gate | 1.7.1 (6.6.20) → 1.8.0 (6.7.9), paired median of 15 | rounds slower |
|---|---:|---:|---:|---:|---:|
| resource_limits_from_json | 488 | 414 | -15.2% | -8.8% | 4/15 |
| injection_scores_from_json | 321 | 282 | -12.1% | -7.1% | 4/15 |
| agent_stats_from_json | 294 | 262 | -10.9% | -5.8% | 6/15 |
| message_build_3turn | 381 | 334 | -12.3% | -3.0% | 6/15 |
| accel_flags_from_json | 727 | 666 | -8.4% | -2.9% | 3/15 |
| version_roundtrip | 253 | 238 | -5.9% | -2.4% | 5/15 |
| trace_context_child | 571 | 525 | -8.1% | -2.1% | 5/15 |
| trace_context_new | 1132 | 1076 | -4.9% | -1.3% | 5/15 |
| injection_scores_to_json | 771 | 720 | -6.6% | -1.2% | 5/15 |
| inference_request_full | 359 | 341 | -5.0% | -1.1% | 6/15 |
| agent_id_to_str | 726 | 682 | -6.1% | -1.1% | 5/15 |
| security_context_full | 660 | 624 | -5.5% | -0.9% | 6/15 |
| traceparent_format | 1768 | 1677 | -5.1% | -0.4% | 5/15 |
| agent_id_new | 549 | 528 | -3.8% | -0.4% | 7/15 |
| resource_limits_to_json | 973 | 904 | -7.1% | -0.2% | 5/15 |
| audit_entry_full | 2214 | 2177 | -1.7% | -0.1% | 7/15 |
| accel_flags_to_json | 1605 | 1512 | -5.8% | -0.1% | 6/15 |
| sandbox_config_default | 23 | 21 | -8.7% | +0.0% | 5/15 |
| token_usage_update | 33 | 31 | -6.1% | +0.0% | 5/15 |
| version_to_str | 106 | 100 | -5.7% | +0.0% | 6/15 |
| agent_id_roundtrip | 1000 | 950 | -5.0% | +0.7% | 9/15 |
| token_usage_to_json | 851 | 814 | -4.3% | +1.0% | 11/15 |
| agent_stats_to_json | 653 | 632 | -3.2% | +1.5% | 12/15 |
| accelerator_device_full | 97 | 101 | +4.1% | +8.0% | 12/15 |
| token_usage_from_json | 415 | 435 | +4.8% | +12.1% | 14/15 |

**Net neutral.** The paired median row is **−0.4%**, and the gate's median row is −5.8%. Two rows
are consistently slower: `token_usage_from_json` (+12.1%, slower in 14 of 15 rounds) and
`accelerator_device_full` (+8.0%, 12 of 15). A second paired run on the same 6.7.9 toolchain,
part 1 alone (commit `f251f1d`) against this tree, attributes them:
- The toolchain is clean (median 0.0%).
- **Part 2's added code** moves exactly those two rows, by +10.1% and +8.1% (11 of 11 rounds
  each). No benchmark calls the new fns, so this is code placement in the non-DCE build
  `cyrius bench` measures.
- In `CYRIUS_DCE=1` builds of the bench, `token_usage_from_json` runs at 379 ns on both trees
  (best of 7).

All 25 rows resolve. The release run is appended to `docs/benchmarks/history.csv` (25 rows,
commit column `f251f1d`, HEAD at the time), taken at load 1.65. Its median row is −6.5% against
1.7.1's row, and the gate passes against it.

Binaries (`CYRIUS_DCE=1`):

| target | 1.7.1 | 1.8.0 | why |
|---|---:|---:|---|
| x86_64 | 132,552 B | **134,408 B** | the harness's reachable code grows with the new fns |
| aarch64 | 828,776 B | **896,144 B** | aarch64 DCE still NOP-fills instead of compacting, so the 50 fns add their full size |
| agnos | 132,896 B | 134,760 B | |
| Windows | 848,896 B | 875,008 B | |

`dist/agnostik.cyr` is 166,565 → 197,447 B.

### Verification

- **Clean-room CI replay.** A fresh copy of the release tree, starting from an empty `lib/`, was
  committed into a throwaway repo. Every `ci.yml` step except the bench passed, 17 of 17, and so
  did the aarch64 step's script (20 files, 2,001 assertions under `qemu-aarch64`).
- **Gates:**
  - api-surface 977 = snapshot;
  - doc-debt 849 = baseline;
  - `cyrius distlib --check` fresh, with the sidecar still 12 leaves;
  - `deps --verify` 32 / 32;
  - lint, fmt, vet and `CYRIUS_TYPE_CHECK=1` clean;
  - security scan and private-helper gate clean.
- **Downstream:** 5 of 5 green.
- **argonaut 1.15.4**, the other half of the `RestartPolicy` plan, is cut and verified on its
  own: 984 assertions on x86_64 and aarch64, a neutral bench gate, and kybernet downstream checks.

## [1.7.1] - 2026-10-14

**Patch: CI and release hardening.**
- No raw syscalls left in `src/`, with a CI gate to keep it that way.
- The test suite runs on aarch64 in CI.
- CI and release install a signature-verified cyrius toolchain.
- The orphan `lib/keccak.cyr` is removed.

No public API change (927 fns), no wire-format or layout change. 19 test files, **1,574
assertions, 0 failures**, on x86_64-linux and on aarch64. All five `[deps.agnostik]` consumers
build and pass their tests against it.

### Changed — no raw syscalls in `src/`; CI enforces it

`agnostik_err_print` (`src/error.cyr`) wrote to stderr with four raw `syscall(1, 2, …)` calls,
the last raw syscalls in `src/`. They were correct everywhere: 1 is `write` on x86_64-linux and
agnos, and aarch64 and Windows translate it. But the compiler diagnoses raw numbers only for
aarch64 and Windows. agnos numbers its syscalls differently and gets no warning, which is how
F-023 went unseen. They are now `sys_write(2, …)`, the stdlib wrapper every target defines (Linux,
aarch64 and macOS through `syscalls_linux_common.cyr`, plus the Windows and agnos peers).
`src/types.cyr` has used the same wrapper for its fail-loud path since 1.6.3.

**Same bytes out.** A probe calling `agnostik_err_print` and `result_print_agnostik_err` produces
byte-identical stderr against the 1.7.0 tag, natively and under `qemu-aarch64`. The x86_64,
aarch64 and agnos binaries keep their sizes; the Windows one shrinks 512 B.

CI's security scan replaces its three raw-number patterns (execve 59, fork 57, clone 56) with one
that fails on **any** raw `syscall(` in `src/`
([ADR-003](docs/adr/003-roadmap-tracks-agnostik-work-only.md)'s agnostik-side answer to the
missing agnos diagnostic). Run the way CI runs it (`bash -e`), it passes on this tree and fails
on a planted `syscall(39)`. Like every pattern in that scan, it skips comment lines but not a
trailing comment after code. CLAUDE.md's syscall rule now says `src/` has none.

### Added — the test suite runs on aarch64 in CI

CI cross-built aarch64 but never ran it, so a result wrong only on aarch64 could ship behind a
green build. F-022, a raw `getrandom` that is `-ENOSYS` there, was that class. A new step,
**"Test (aarch64 under qemu-user)"**, installs `qemu-user` and runs every `.tcyr` under
`qemu-aarch64`, which cyrius execs from `/usr/bin`. It is deliberately not best-effort: it fails
when the emulator or `cycc_aarch64` is missing, or when a file prints no passing verdict, because
a silently skipped aarch64 step is how F-022 went unseen through 1.6.1.

The step's script, run locally as written apart from the `apt-get` lines, reports **19 test
files, 1,574 assertions**. A planted failing assertion fails it with exit 1. The package install
on GitHub's runner is the one part that only the first CI run can exercise.

### Changed — CI and release install a signature-verified toolchain

Both workflows downloaded the cyrius tarball with `curl -sLO` and unpacked it unchecked. That
bypassed the guarantee cyrius's own installers give (CYRIUS-2026-0035, 6.6.20: refuse a release
with a stripped signature), and `-sL` without `-f` would also have saved an HTTP error page as
the tarball. Both inline install steps are now one script, **`scripts/ci-install-cyrius.sh`**. It
downloads the tarball, `SHA256SUMS` and `SHA256SUMS.sig`, and before unpacking anything it checks:

1. **The signature**: `SHA256SUMS.sig` must be a valid Ed25519 signature over `SHA256SUMS` under
   the cyrius release key **pinned in the script**, not the `cyrius-release.ed25519.pub` the
   release ships, which anyone able to swap the tarball could swap too. It uses stock
   `openssl pkeyutl -rawin` (OpenSSL 3.0+), so no prior `cyrsign` is needed.
2. **The hash**: the tarball must match its single line in that `SHA256SUMS`.

The roadmap asked only for the `.sha256` sidecar. A same-origin checksum catches corruption, not
substitution, so the script checks the signature too. The pinned key was byte-identical in three
places at pinning time: the 6.6.20 release asset, `keys/cyrius-release.ed25519.pub` in the cyrius
repo, and `CYRIUS_RELEASE_PUBKEY` in cyrius's `install.sh`. A key rotation fails closed, and the
script's header says how to re-pin.

Proven both ways:
- In a throwaway `HOME`, it installs 6.6.20 with `cycc`, `cyrius` and `cycc_aarch64`
  byte-identical to the local install. Every `ci.yml` step except the bench then passed on that
  toolchain, 17 of 17.
- A one-digit change to the pinned key, and a tarball corrupted after download, each exit 1
  before anything is installed.

The version still comes only from `cyrius.cyml`. CLAUDE.md's CI section and SECURITY.md (a new
"build toolchain supply chain" scope item) describe the check.

### Removed — the orphan `lib/keccak.cyr`

It was git-tracked only because the ecosystem template's `!lib/k*.cyr` ignore exception matched
its name. No module includes it, `[deps] stdlib` does not declare it, `lib sync` never touched it,
and since 1.6.5 `cyrius deps --verify` had to vouch for it. The file and the exception are gone,
and `cyrius.lock` is re-hashed: **33 → 32** entries, `deps --verify` 32 / 32. A fresh checkout with
an empty `lib/` reproduces the committed lock byte for byte through `lib sync` + `deps`, and
nothing recreates the file. CLAUDE.md's `.gitignore` note now says all of `lib/*.cyr` is ignored.

### Security

The release pass is recorded in [`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md)
§"v1.7.1 release pass". **No findings.** The release is itself hardening: the raw-syscall gate,
and the signature-verified toolchain install, whose input validation and fail-closed paths were
probed (malformed versions are refused before any network access, and a wrong key or a corrupted
tarball installs nothing). F-001..F-025 remain closed.

### Fixed — docs

SECURITY.md said F-023 was "unreleased"; it shipped in 1.6.3. `state.md` drops its `keccak.cyr`
and 33-file lock figures and records the aarch64 CI run and the verified install.

### Performance

`scripts/bench-regression.sh` against the 1.7.0 baseline in `history.csv`: **25 checked, 0 new, 0
regressions.** The A/B column is 15 rounds interleaving the 1.7.0 tag tree with the 1.7.1 tree,
both on cyrius 6.6.20, with order alternating each round, taken right after the gate at load 5 to 7.

| benchmark | 1.7.0 baseline | 1.7.1 gate | Δ gate | 1.7.0 tree, median of 15 | 1.7.1 tree, median of 15 | Δ A/B |
|---|---:|---:|---:|---:|---:|---:|
| token_usage_from_json | 465 | 404 | -13.1% | 424 | 417 | -1.7% |
| agent_id_roundtrip | 1118 | 994 | -11.1% | 1029 | 1025 | -0.4% |
| agent_id_new | 611 | 560 | -8.3% | 575 | 573 | -0.3% |
| token_usage_to_json | 955 | 844 | -11.6% | 874 | 872 | -0.2% |
| audit_entry_full | 2402 | 2221 | -7.5% | 2264 | 2259 | -0.2% |
| trace_context_child | 655 | 586 | -10.5% | 589 | 588 | -0.2% |
| agent_stats_to_json | 739 | 647 | -12.4% | 674 | 673 | -0.1% |
| security_context_full | 744 | 665 | -10.6% | 686 | 685 | -0.1% |
| injection_scores_to_json | 859 | 774 | -9.9% | 793 | 792 | -0.1% |
| traceparent_format | 1947 | 1757 | -9.8% | 1832 | 1830 | -0.1% |
| accelerator_device_full | 112 | 98 | -12.5% | 102 | 102 | +0.0% |
| agent_id_to_str | 797 | 722 | -9.4% | 746 | 746 | +0.0% |
| inference_request_full | 454 | 364 | -19.8% | 375 | 375 | +0.0% |
| sandbox_config_default | 25 | 23 | -8.0% | 23 | 23 | +0.0% |
| token_usage_update | 37 | 33 | -10.8% | 33 | 33 | +0.0% |
| version_roundtrip | 284 | 255 | -10.2% | 261 | 261 | +0.0% |
| trace_context_new | 1253 | 1155 | -7.8% | 1162 | 1163 | +0.1% |
| resource_limits_to_json | 1074 | 959 | -10.7% | 986 | 989 | +0.3% |
| accel_flags_to_json | 1781 | 1579 | -11.3% | 1644 | 1654 | +0.6% |
| message_build_3turn | 413 | 369 | -10.7% | 373 | 376 | +0.8% |
| version_to_str | 120 | 106 | -11.7% | 108 | 109 | +0.9% |
| agent_stats_from_json | 321 | 298 | -7.2% | 300 | 305 | +1.7% |
| accel_flags_from_json | 770 | 731 | -5.1% | 729 | 748 | +2.6% |
| injection_scores_from_json | 344 | 321 | -6.7% | 315 | 331 | +5.1% |
| resource_limits_from_json | 501 | 478 | -4.6% | 461 | 494 | +7.2% |

**No performance change.** The A/B's median row is **0.0%**, and 23 of 25 rows are within ±5%,
most within ±1%. The gate's median row reads **−10.5%** because the 1.7.0 baseline row was
load-inflated, as its own CHANGELOG entry warned. The unchanged 1.7.0 tree, measured as the
control, reads −8.0% against that row too.

**The two rows that move are code placement, not slower code.** `resource_limits_from_json`
(+7.2%) and `injection_scores_from_json` (+5.1%) move consistently, on medians and on
per-round minimums alike, and neither calls the one changed fn. `cyrius bench` builds without
DCE. In that build, `agnostik_err_print`'s `sys_write` calls resize code in `src/error.cyr`, the
first module in the bundle, so every later fn moves. The two non-DCE bench binaries are the same
size, but 164,822 bytes differ. With `CYRIUS_DCE=1`, the way release binaries are built, the bench
binary is **byte-identical** to 1.7.0's, so the shipped code is unchanged.

**The appended baseline is clean.** The release run appended to `docs/benchmarks/history.csv`
(25 rows; commit column `8fd28b7`, HEAD at the time) was taken at load ≈ 3. Its median row is
**+1.8%** of 1.6.5's quiet-host row, so it replaces 1.7.0's inflated row (+13.5%) as the next
gate's baseline. The gate passes against it, and all 25 rows resolve.

Binaries (`CYRIUS_DCE=1`): x86_64 132,552 B, aarch64 828,776 B and agnos 132,896 B, all
unchanged; Windows 849,408 → 848,896 B.

### Verification

- **Clean-room CI replay.** A fresh copy of the release tree, starting from an empty `lib/`, was
  committed into a throwaway repo. Every `ci.yml` step except the bench passed, 17 of 17. So did
  the security scan as CI runs it (`bash -e`), and the new aarch64 step's script (19 files,
  1,574 assertions under `qemu-aarch64`). Earlier, the same steps also passed on a toolchain that
  `scripts/ci-install-cyrius.sh` itself installed into a throwaway `HOME`.
- **Tests:** 19 files, **1,574 assertions, 0 failures** on x86_64-linux and on aarch64.
- **Gates:** api-surface 927 = snapshot, doc-debt 849 = baseline, `cyrius distlib --check`
  fresh, `deps --verify` 32 / 32, private-helper gate clean. Both workflow files parse.
- **Downstream:** `scripts/consumer-sweep.sh` on the release tree, 5 of 5 green, 0 regressed, 0
  agnostik-involved duplicate definitions. That includes aegis, which builds its own copy of
  `src/error.cyr` on its 6.6.2 pin.
- **Version verify:** `VERSION`, the CHANGELOG header and the `dist/agnostik.cyr` banner read
  1.7.1, and the release workflow's version gate accepts tag `1.7.1`.
- **Links:** every relative link in every Markdown file resolves.
- **Not verifiable locally:** the `apt-get install qemu-user` on GitHub's runner, and the
  installer running there. The first CI run on this release exercises both.

## [1.7.0] - 2026-10-14

**Minor: the consumer safety net.** A cross-consumer build sweep, `_stik_`-namespaced private
helpers (F-025), and setters for `ClassificationResult`. Public API 923 → **927** fns (+4, no
removals, no signature or wire-format change). `ClassificationResult` grows 48 → 56 B, with every
existing offset unchanged. 19 test files, **1,574 assertions, 0 failures**, on x86_64-linux and on
aarch64 (under `qemu-aarch64`). Every consumer that declares `[deps.agnostik]` builds and passes
its tests against this release on its own pin: 5 of 5.

### Added — `scripts/consumer-sweep.sh`: every consumer, built and tested against the working tree

The roadmap carried the cross-consumer sweep since v1.2.4. The script finds every sibling checkout
whose `cyrius.cyml` declares `[deps.agnostik]`; that is five today: aegis, anuenue, ark, kybernet
and mela. `--list` prints that census. It copies each one twice into a scratch directory and builds
and tests both copies under the consumer's **own** cyrius pin:

- **base** keeps the manifest untouched, so it resolves the agnostik tag the consumer pins.
- **new** has its `[deps.agnostik]` rewritten to `path = "../agnostik"`, with a copy of this working
  tree as that sibling. The consumer's own resolve then vendors the in-flight code, exactly as it
  would vendor a release.

Tests are the manifest's `[build] test` file plus every `tests/**/*.tcyr`, run one file at a time,
which works on every CLI from 6.6.2 to 6.7.9. A consumer **regresses** if it was green at base and
the new run fails to build, fails a test file, loses assertions, or gains a warning that involves
agnostik. The report also lists every duplicate-definition warning that involves an agnostik file.
The exit status is 0 or 1; the scratch directory is kept for inspection on failure. CLAUDE.md's
Quick Start and closeout step 8 now call it. One run takes about a minute.

**Swapping `lib/agnostik.cyr` by hand does not work.** `cyrius build` and `cyrius test` re-resolve
deps first and put the pinned tag's copy back from `~/.cyrius/deps`. A marker appended to the
vendored file was gone after one build in ark and aegis (6.6.2) and kybernet (6.6.6). So after the
new build, the sweep compares every vendored agnostik module with the in-flight source byte for
byte, and a mismatch marks the consumer **INVALID** rather than passing.

Proven both ways in a scratch copy:
- Renaming `capability_parse` and `stik_err_io` turned aegis and kybernet **REGRESSED** (builds
  fail, 0 assertions), while anuenue, which calls neither, stayed ok. Exit 1.
- A sweep with the manifest rewrite disabled reported kybernet **INVALID** instead of passing.
  Exit 1.

Result on this tree: **5 consumers, 0 regressed, 0 invalid**. Assertion counts are unchanged
(aegis 428, anuenue 402, ark 392, kybernet 876, mela 492).

### Added — `ClassificationResult` setters (closes the F-021-class gap found at v1.6.3)

`level`, `auto_level` and `confidence` had getters and no setters. Overriding the final level
apart from the detected one needed a raw `store64(r + 0, …)`, and `confidence` had no unit, so
nothing ever set it. Four fns, each documented:

- `cresult_set_level(r, level)` / `cresult_set_auto_level(r, level)` store a `ClassificationLevel`.
  Each touches only its own field: a policy override leaves the detected `auto_level` alone.
- `cresult_set_confidence(r, ppt)` stores a confidence in **parts per thousand**, 0 (none) to 1000
  (certain), the fixed-point form of the Rust original's optional 0.0–1.0 float.
- `cresult_has_confidence(r)` returns 1 once a confidence is set. 0 is a valid score, so this flag
  is what tells "set to 0" from "never set". `cresult_confidence` still reads 0 when unset, so
  nothing existing changes.

**These setters validate, unlike the library's other 139.** Those store and return 0. These
return `Ok(0)`, or an invalid-argument `Err` that leaves the result unchanged, for a level outside
`ClassificationLevel` or a confidence outside 0..1000. A classification level decides how data is
handled, so an unknown value is refused rather than stored. That follows the 1.5.0
`capability_parse` decision: errors, not silent defaults, where the value drives a security
decision.

Layout: `classification_result_new` allocates **56 B (was 48)**. The new set flag is at +48, and
every existing offset is unchanged, so accessor users and the defaults are unaffected. No local
consumer references `cresult_*`.

`tests/tcyr/test_v170_classification_setters.tcyr` (38 assertions): the defaults and the vec slots
after the layout change, each setter touching only its field, all four levels accepted by both
level setters, rejections (−1, one past the last level, `i64` max and min, 1001, 2000) with the
error kind and the unchanged state checked, and an invariant sweep showing exactly 0..1000 is
accepted over −50..1050. Mutation-checked: an off-by-one bound fails 4 assertions, and a
`set_level` that writes the `auto_level` slot fails 4. The first mutant at first **crashed** the
file instead of failing it, because the error-kind check dereferenced the payload of an
unexpected `Ok`. The kind is now read only from an `Err`, so a wrongly accepting setter fails
cleanly.

### Changed — private helpers are `_stik_`-prefixed; CI enforces it

Consumers compile agnostik into one unit together with the other libraries they vendor. A
same-named fn there either replaces the other silently (last definition wins), or fails the build
when the arities differ, a hard error since cyrius 6.6.2 and how 1.6.1's `health_check_new` broke
kybernet. The sweep's first run showed **`_hex_nibble` colliding with sigil's in four of five
consumers**:

- In aegis, ark and mela, sigil's definition won.
- In kybernet, agnostik's won.
- The two are identical today, so nothing misbehaved yet.

A scan of every local AGNOS checkout and the 6.6.20 / 6.7.9 stdlib snapshots found worse:

| agnostik helper | also defined by | their arity vs agnostik's |
|---|---|---|
| `_hex_nibble` | sigil (stdlib snapshot; a copy in 60+ repos) | 1 vs 1, same mapping |
| `_json_int`, `_json_str` | ai-hwaccel, itihas | 4 and 3 vs 2: writers vs agnostik's readers |
| `_hex_byte` | bote | 2 vs 3 |

daimon, hoosh, stiva, kavach and agnosai all vendor ai-hwaccel, so none of them could have
vendored agnostik as it stood.

All **21** `_`-prefixed fns in shipped modules are now `_stik_<name>`:
- `types`: `_stik_fill_random`, `_stik_hex_byte`, `_stik_hex_nibble`, `_stik_json_find_value`,
  `_stik_json_int`, `_stik_json_parse_u4`, `_stik_json_str`, `_stik_utf8_encode`
- `proto`: the eight `_stik_proto_*`
- `security`: `_stik_lazy_vec`, `_stik_lazy_map`
- `telemetry`: `_stik_all_zero_bytes`, `_stik_span_kind_to_otlp`
- `audit`: `_stik_genesis_hash_cached`

The test-harness helpers `_to_json` and `_json_has` stay as they are, since `src/main.cyr` and
`tests/` never ship. The rename is mechanical: word-boundary matched across `src/` and `tests/`,
no old name left, binary size unchanged. **Not an API change**: no `_` fn is in
`docs/api-surface.snapshot`, and none of the five consumers calls one.

CI's Security Scan job gains **"Private helpers are namespaced"**. It fails on any `_`-prefixed fn
in a shipped `src/` module that lacks `_stik_`. It passes on this tree, and a planted
`fn _mutant_helper` fails it. After the rename, the sweep reports **0** agnostik-involved
duplicate definitions in every consumer. CLAUDE.md's Cyrius conventions gain the rule, and its
lazy-init line names `_stik_lazy_vec` / `_stik_lazy_map`. README and the architecture overview
name the `_stik_json_*` primitives.

### Fixed — 1.6.5's downstream claim was vacuous; the sweep shows its conclusion holds

The 1.6.5 CHANGELOG Consumers note and `docs/audit/2026-10-14-audit.md` say a scratch copy of
kybernet, rebuilt with the 1.6.5 bundle swapped into `lib/agnostik.cyr`, produced the same binary
size. That build had already put kybernet's pinned 1.6.3 bundle back before compiling (see the
sweep entry above), so it never tested the 1.6.5 bundle. The conclusion still holds, now with
evidence. The sweep's first run, on the 1.6.5 tree, verified that each consumer really vendored
the in-flight bundle (`cmp` against `dist/agnostik.cyr`):
- kybernet: 1,616,776 B in both runs, 876 / 876 assertions.
- The other four: all green, assertion counts unchanged.

The audit's Verified-sound bullet carries a correction note. The released 1.6.5 entry is left as
shipped.

### Changed — docs

- **Roadmap**: the cyrius 6.6.20 → 6.7.9 pin move now rides v1.8.0, with the 1.6.5 refresh
  checklist plus the consumer sweep. The v1.7.0 items are marked done. The v2.0.0 `_json_int`
  item becomes `_stik_json_int`, noting that the helper is private and uncalled by any consumer,
  so its `Result` change no longer needs a major.
- **README**: the Consumers section still listed the eleven-name vocabulary that `state.md`
  dropped at 1.6.5. It now names the five `[deps.agnostik]` repos, and the gate-script list gains
  the sweep.

### Security

The minor-cut security pass is recorded in
[`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md) §"v1.7.0 minor-cut pass". One
finding, fixed in this release:

- **F-025 (LOW, defense in depth) — agnostik's parsers could run another library's code.** A
  consumer compiles agnostik into one unit together with the other libraries it vendors, and a
  same-named, same-arity fn there wins by definition order. So in aegis, ark and mela, sigil's
  `_hex_nibble` replaced agnostik's. That helper does the hex decoding for the UUID parser
  (`agent_id_from_str`), `trace_id_from_str`, `span_id_from_str`, `tctx_from_traceparent`, and the
  JSON `\u` escape decoder. The two
  definitions are identical today, so nothing parsed differently. A diverging same-arity copy
  would have silently changed how agnostik parses untrusted UUIDs, traceparents and JSON. The
  different-arity collisions (`_json_int` / `_json_str` against ai-hwaccel and itihas,
  `_hex_byte` against bote) fail the build instead, an availability problem rather than an
  integrity one. Fixed by the `_stik_` rename above and the CI gate that keeps it.

The re-scan of `src/` finds no `sys_system`, no `/etc` / `/bin` / `/sbin` literal and no stack
buffer of 64 KB or more. The only raw syscalls are `src/error.cyr`'s four `syscall(1, 2, …)`
stderr writes, unchanged and correct on every target; they move to `sys_write` at v1.7.1. The
new setters validate their input. `scripts/consumer-sweep.sh` runs consumer code by design: it is
a local release tool, not shipped code, and not in CI.

### Closeout pass (CLAUDE.md, minor cut)

- **Tests:** 19 files, 1,574 assertions, 0 failures, on x86_64 and on aarch64.
- **Dead code:** no private fn in a shipped module is unreferenced. **Floor recorded:** of 903
  textual fn definitions in shipped modules, 767 public fns are called by no library code. They
  are consumers' entry points, as expected for a type library, and 294 of them are referenced by
  no test or harness either. That untested-entry-point count is the figure to drive down.
- **Refactor:** none needed. The two level setters share one validation line, below CLAUDE.md's
  wait-for-the-third rule.
- **Code review** of the diff since 1.6.5. The setters test's crash on an unexpected `Ok`, found
  by mutation, was fixed before the release (see above). Two fixes to the new sweep script:
  - It now refuses to run when `mktemp` fails. An empty scratch path would have aimed its copies
    at `/base`, `/new` and `/logs`. A mutant with `mktemp` forced to fail exits 2 and creates
    nothing.
  - Its preflight now prints what `cyrius distlib --check` reported, instead of always claiming
    the bundle is stale. An unwritable `TMPDIR` produced exactly that misleading message.
- **Cleanup:**
  - `src/secrets.cyr` said `SecretMetadata.expires_at` / `.owner` "have getters but no setters",
    false since 1.4.0 added `smeta_set_expires_at` / `smeta_set_owner`.
  - SECURITY.md's attack-surface section named the pre-rename `_json_*` helpers.

  Both corrected.
- **Security re-scan:** as above.
- **Downstream:** `scripts/consumer-sweep.sh` on the release tree. 5 consumers, 0 regressed, 0
  invalid, 0 agnostik-involved duplicate definitions. Assertions unchanged: aegis 428,
  anuenue 402, ark 392, kybernet 876, mela 492.
- **Version verify:** `VERSION`, the CHANGELOG header and the `dist/agnostik.cyr` banner all read
  1.7.0, and `cyrius.cyml` reads `VERSION`. The release workflow's version gate accepts tag
  `1.7.0`.
- **Full build from clean:** see Verification.

### Performance

`scripts/bench-regression.sh` against the 1.6.5 baseline in `history.csv`: **25 checked, 0 new, 0
regressions.** The host was shared with other sessions' compilers for the whole cut (load 2.6 to
12), so the gate's Δ column is mostly host, not code. The A/B separates the two: 15 rounds
interleaving the 1.6.5 tag tree with the 1.7.0 tree, both on cyrius 6.6.20, with order alternating
each round, taken right after the gate.

| benchmark | 1.6.5 baseline | 1.7.0 gate | Δ gate | 1.6.5 tree, median of 15 | 1.7.0 tree, median of 15 | Δ A/B |
|---|---:|---:|---:|---:|---:|---:|
| token_usage_update | 32 | 34 | +6.2% | 38 | 37 | -2.6% |
| traceparent_format | 1759 | 1868 | +6.2% | 2068 | 2035 | -1.6% |
| inference_request_full | 363 | 379 | +4.4% | 427 | 421 | -1.4% |
| agent_stats_to_json | 644 | 689 | +7.0% | 762 | 753 | -1.2% |
| token_usage_to_json | 836 | 895 | +7.1% | 986 | 977 | -0.9% |
| resource_limits_from_json | 447 | 473 | +5.8% | 520 | 516 | -0.8% |
| agent_stats_from_json | 290 | 312 | +7.6% | 342 | 340 | -0.6% |
| resource_limits_to_json | 948 | 1013 | +6.9% | 1113 | 1107 | -0.5% |
| accel_flags_to_json | 1571 | 1684 | +7.2% | 1851 | 1842 | -0.5% |
| agent_id_roundtrip | 988 | 1054 | +6.7% | 1162 | 1161 | -0.1% |
| audit_entry_full | 2200 | 2302 | +4.6% | 2441 | 2442 | +0.0% |
| token_usage_from_json | 406 | 434 | +6.9% | 477 | 478 | +0.2% |
| message_build_3turn | 360 | 384 | +6.7% | 421 | 422 | +0.2% |
| version_roundtrip | 247 | 266 | +7.7% | 299 | 300 | +0.3% |
| agent_id_new | 583 | 581 | -0.3% | 618 | 623 | +0.8% |
| agent_id_to_str | 740 | 757 | +2.3% | 838 | 846 | +1.0% |
| security_context_full | 653 | 704 | +7.8% | 758 | 767 | +1.2% |
| injection_scores_from_json | 296 | 324 | +9.5% | 352 | 357 | +1.4% |
| accelerator_device_full | 95 | 105 | +10.5% | 113 | 115 | +1.8% |
| injection_scores_to_json | 743 | 811 | +9.2% | 876 | 893 | +1.9% |
| accel_flags_from_json | 685 | 745 | +8.8% | 804 | 820 | +2.0% |
| version_to_str | 104 | 110 | +5.8% | 122 | 125 | +2.5% |
| trace_context_child | 577 | 591 | +2.4% | 650 | 668 | +2.8% |
| trace_context_new | 1188 | 1189 | +0.1% | 1291 | 1332 | +3.2% |
| sandbox_config_default | 24 | 24 | +0.0% | 26 | 27 | +3.8% |

**1.7.0 is performance-neutral.** The A/B's median row is **+0.2%**, every one of the 25 rows is
within ±5%, and the largest move is `sandbox_config_default` at +3.8% (26 → 27 ns). None of the
benchmarks exercises the new setters, and the `_stik_` rename leaves the generated code unchanged.
The gate's median row, **+6.7%**, is load. The unchanged 1.6.5 tree, measured in the same window
as the control, read +17.5% against its own baseline, as the load rose during the A/B.

**The appended baseline is load-inflated.** The release run appended to
`docs/benchmarks/history.csv` (25 rows, commit column `0f5a9b7`, HEAD at the time) was taken at
load ≈ 5. It reads **+13.5%** on the median row over 1.6.5's quiet-host row (range +4.2% to
+25.1%), with code that the A/B shows is neutral. So the next gate compares against a baseline
about 13% lenient. That is small against the gate's 50% / 80% thresholds, but next release's
deltas should be read with it in mind, or the row re-captured on a quiet host. The gate passes
against the new row.

All 25 rows resolve (no `UNRESOLVED` lines). DCE binary 132,424 → **132,552 B** (+128 B, the
setters and the 56-byte allocation). Plain build 718,152 → 718,280 B; aarch64 828,648 →
828,776 B.

### Verification

- **Clean-room CI replay.** A fresh copy of the release tree was committed into a throwaway repo
  and built from an empty `build/`. Every `ci.yml` step except the bench passed, 17 of 17, from
  stdlib sync through the security scan and version check, with the dist check run against a real
  committed HEAD. The new "Private helpers are namespaced" step passed too. The bench gate ran
  separately (above).
- **Tests:** 19 files, **1,574 assertions, 0 failures** on x86_64-linux, and the same under
  `qemu-aarch64` 11.1.1.
- **Builds** (`CYRIUS_DCE=1`): x86_64 132,552 B, aarch64 828,776 B, agnos 132,896 B, Windows
  849,408 B. No warning comes from agnostik source; the Windows build's one warning is in the
  stdlib's `lib/syscalls_windows.cyr`, as at 1.6.5.
- **Gates:** api-surface 927 = snapshot; doc-debt 849 = baseline (all four new fns documented);
  `cyrius distlib --check` fresh; `deps --verify` 33 / 33; fmt, lint, type-check and vet clean.
- **Downstream:** `scripts/consumer-sweep.sh`, 5 of 5 green, 0 regressed, 0 invalid, 0
  agnostik-involved duplicate definitions.
- **Links:** every relative link in every Markdown file resolves (0 broken).

## [1.6.5] - 2026-10-14

**Toolchain refresh to cyrius 6.6.20, with the repairs that pin move needed, the derive-comments
record archived, and the roadmap rebuilt into pinned v1.7.0 → v2.0.0 slots.** No code change: the
only `src/` edits are two comments. No public API, struct-layout or wire-format change. The
api-surface snapshot moves 916 → 923 because cyrius 6.6.8 fixed its scanner, not because the API
changed (see below). 18 test files, **1,536 assertions, 0 failures**, on x86_64-linux and on
aarch64 (under `qemu-aarch64`).

### Changed — cyrius pin 6.6.6 → **6.6.20**; stdlib re-vendored

`cyrius lib sync` + `cyrius deps` against the 6.6.20 snapshot. `lib/` holds **33** files, every one
byte-identical to the snapshot:

- **29 changed content**: every vendored module and platform peer except the three below,
  including `fmt` (CYRIUS-2026-0028, see Security) and the git-tracked `keccak.cyr`. `lib sync`
  does not touch `keccak.cyr`, because no module includes it, so it was copied from the snapshot by
  hand. Its removal is pinned to v1.7.1.
- **3 did not**: `alloc_agnos`, `args_macos`, `fnptr`.
- **1 is new**: `poison.cyr` (6.6.18's allocator poison seam), which 6.6.20's `alloc.cyr` includes.

A fresh checkout gets 26 files from `lib sync` plus the tracked `keccak.cyr`. `cyrius deps` adds
the six transitive includes: `args_agnos`, `args_macos`, `atomic`, `boxed`, `hashseed` and
`poison`. There are still no `[deps.NAME]` git dependencies to move.

### Added — `cyrius.lock` is committed

Since cyrius 6.6.9, `cyrius deps` writes a stdlib-only project's first lock, and `deps --verify`
fails any `lib/*.cyr` the lock does not cover. The lock covers all 33 files. CI's
"Verify dep hashes" step skipped with a warning while no lock existed. It now enforces the lock:
`cyrius deps --verify` reports **33 verified, 0 failed**. On a fresh checkout, `lib sync` and
`deps` reproduce the committed lock byte for byte, which also settles the 6.6.17 note the roadmap
carried ("`lib sync` now checks the lock"). The release workflow already ships `cyrius.lock` when
one exists. CLAUDE.md's Quick Start gains `cyrius deps --verify`, and its refresh note now says to
commit the lock with a pin move.

### Changed — api-surface snapshot 916 → 923; no API change

cyrius 6.6.8 rebuilt `cyrius_api_surface`'s `#derive` scanner to mirror the compiler, and its
release notes named agnostik as needing a one-time `--update`. The 6.6.6 scanner listed each
derived `_to_json` at arity 1, although the compiler has emitted it as `(ptr, sb)` since 5.9.31.
It also never listed the derived `_from_json_str`. For the seven `#derive(Serialize)` structs
(`ResourceLimits`, `ResourceUsage`, `AgentStats`, `EdgeResourceOverrides`, `AcceleratorFlags`,
`TokenUsage`, `InjectionScores`):

- 7 entries move `_to_json/1` → `_to_json/2`;
- 7 `_from_json_str/1` entries appear.

None of these fns is new: `tests/tcyr/test_serde_roundtrip.tcyr` has called the `_from_json_str`
fns all along, and a probe built on 6.6.6 calls `TokenUsage_from_json_str` cleanly. Regenerated
with `scripts/api-surface.sh update`.

### Changed — `dist/agnostik.deps` is compile-verified; the bundle carries a requires block

cyrius 6.6.18 changed `cyrius distlib` (its "P4 option 2"). A bundle's `.deps` sidecar now lists
only the stdlib leaves that a compile of the bundle proves it needs. It is no longer the package's
`[deps] stdlib` union. Each bundle also opens with one `include "lib/<leaf>.cyr"` per sidecar leaf,
so it compiles when included on its own.

- `dist/agnostik.deps`: **15 → 12** leaves. It drops `tagged`, `fnptr`, `assert` and `io`, none
  of which the bundle calls (they stay in `[deps] stdlib` for agnostik's own builds and tests). It
  adds `boxed`, which defines the `tagged_new` / `boxed_tag` / `boxed_payload` the bundle's 19
  tagged boxes use.
- `dist/agnostik.cyr` gains the 12-line requires block (164,007 → 164,427 B).

**Consumers:** a consumer that used one of the four dropped leaves only because agnostik's
sidecar vendored it must declare that leaf in its own `[deps] stdlib`. None does today. Of the
five repos that declare `[deps.agnostik]` (aegis, anuenue, ark, kybernet, mela), only anuenue does
not declare `tagged` and `fnptr` itself, and its source calls neither. A scratch copy of kybernet,
rebuilt on its own 6.6.6 pin with the new bundle swapped in, produced the same 1,616,776 B binary
as before. Include-once is keyed on the literal path, so the requires block duplicates nothing in
a `cyrius deps` consumer.

### Changed — `cyrius-derive-comments-in-struct-body` archived: fixed upstream at 6.6.7

The `#derive(Serialize)` body-comment corruption, filed upstream at 1.6.4, was fixed at cyrius
6.6.7. That release's "bite 5" credits agnostik. The body is now walked the way the parser reads
it, with `#` comments as trivia. Re-verified here on the same probe built under both pins:

| cyrius | the record's `Bad` repro | a struct with four kinds of body comment, then a round-trip |
|---|---|---|
| 6.6.6 | `{"#":330325,"xss":32,…}` | `{"#":1,"#":2,"#":3}` → `{"#":3,"#":0,"#":0}` |
| 6.6.20 | `{"sql":85,"xss":10,"command":5,"path_traversal":0,"prompt_injection":0}` | `{"a":1,"b":2,"c":3}` → `{"a":1,"b":2,"c":3}` |

The record gains a Resolution section and moves to `docs/development/issues/archive/`. Its index
row, and the links in CONTRIBUTING.md, `state.md` and this file's 1.1.1 entry, now point there.
`docs/development/issues/` has no open records.

**The workaround stays.** Consumers compile `dist/agnostik.cyr` with their own compilers, and
aegis, ark and mela pin 6.6.2 and kybernet pins 6.6.6. A comment moved back inside a derive body
would reach those builds as the silent corruption above. The two comments in `src/validation.cyr`
and `src/hardware.cyr` that said "revisit when fixed" now give this reason. CONTRIBUTING.md says
the same. `tests/tcyr/test_v110_serde_golden.tcyr` still pins every derived `_to_json` byte for
byte.

### Changed — roadmap rebuilt into pinned v1.7.0 → v2.0.0 slots

`docs/development/roadmap.md` held shipped and resolved items struck through in place, and a
status block that retold releases since 1.3.x. Shipped work leaves it now, and every open item has
a release:

| Slot | Work |
|---|---|
| v1.7.0 (minor) | Cross-consumer build sweep (carried since v1.2.4, now five repos rather than eleven); private-helper namespacing, starting with `_hex_nibble`, which collides with sigil's in kybernet; `ClassificationResult` setters with a parts-per-thousand confidence |
| v1.7.1 (patch) | Raw-syscall CI gate (`error.cyr`'s four `syscall(1, 2, …)` → `sys_write`); aarch64 tests in CI under qemu-user; toolchain tarball checked against its `.sha256`; orphan `lib/keccak.cyr` removed |
| v1.7.2 – v1.9.3 (patches) | Documentation debt, module by module, smallest first: 849 → 0 |
| v1.8.0 (minor) | `*_name` / `*_parse` for the 26 enums that still lack them |
| v1.9.0 (minor) | OTLP `LogRecord`, `MetricDataPoint`, and `Span` attributes / events / links |
| v2.0.0 (major) | `_json_int` Result return, `#derive(accessors)` prefix rename, dead-helper removal (gated on the v1.7.0 sweep) |

One entry left without shipping: "Upstream (cyrius) papercuts to file". The roadmap's working
agreement asks for an ADR in that case, so
[ADR-003](docs/adr/003-roadmap-tracks-agnostik-work-only.md) records the rule: the roadmap carries
work that changes this repository. Each papercut keeps an agnostik-side answer. `cyrius update`
is already covered by CLAUDE.md, and the missing agnos raw-syscall diagnostic becomes the v1.7.1
gate.

### Fixed — `state.md`'s consumer list

`state.md` listed eleven consumers (daimon, hoosh, agnoshi, aegis, argonaut, sigil, ark, kavach,
stiva, nein, yukti). A census of every local AGNOS checkout finds five repos that declare
`[deps.agnostik]`: **aegis** (1.4.0, `src/types.cyr` + `src/error.cyr`, pin 6.6.2), **anuenue**
(1.6.0, pin 6.7.9), **ark** (1.6.0, pin 6.6.2), **kybernet** (1.6.3, pin 6.6.6) and **mela**
(1.6.0, pin 6.6.2). The last four vendor `dist/agnostik.cyr`. The section is now that table, with
a note on the old list.

### Security

The release security pass is recorded in
[`docs/audit/2026-10-14-audit.md`](docs/audit/2026-10-14-audit.md). **No findings.** It checks
the cyrius advisories CYRIUS-2026-0009 … 0035, published between 6.6.7 and 6.6.20, against what
agnostik reaches:

- **CYRIUS-2026-0028 (P1)** — `fmt_int_buf` (`lib/fmt.cyr`) wrote 24 bytes at `buf` whatever the
  number's length, so a `buf + pos` caller with less room overran. agnostik's `version_to_str`
  is such a caller, but it was never exposed. Its buffer is `alloc(128)`, and the third call
  starts at most at offset 42, so the widest scratch write ends at byte 66. A probe of the widest
  possible version (`i64` minimum and maximum, `-1`, a prerelease and a build) formats correctly,
  69 bytes, on both pins. The re-vendored `fmt.cyr` writes exactly `len + 1` bytes.
- **CYRIUS-2026-0034** (`file_write_atomic`): dead in agnostik's binary.
  **CYRIUS-2026-0031** (Windows munmap): agnostik allocates with bump `alloc()` and vendors
  neither affected fn. **CYRIUS-2026-0033** (aarch64 frames past 64 KiB): `src/` has no stack
  arrays. **CYRIUS-2026-0030 / 0032** (lock pins for git deps, CRLF locks): agnostik has no git
  deps and checks out LF. The rest are in modules agnostik does not vendor (TLS, HTTP, WebSocket,
  net, log, pam).
- **CYRIUS-2026-0035** (installers accepted a stripped signature): CI unpacks the release tarball
  by hand and checks nothing, so it bypasses those installers. Checking the tarball against its
  published `.sha256` is pinned to v1.7.1. For this release the tarball was checked by hand: it
  matches its sidecar.

Every target builds with `CYRIUS_DCE=1`: x86_64-linux, aarch64, agnos and Windows. No warning
comes from agnostik source. The Windows build's one warning is in the stdlib's
`lib/syscalls_windows.cyr`. The CI security scan, run verbatim, is clean.

### Performance

`scripts/bench-regression.sh` against the 1.6.4 baseline in `history.csv` (commit `07720f0`):
**25 checked, 0 new, 0 regressions.** The source is unchanged, so this table measures the toolchain
and the host. Two measurements separate them:

- **Median of 3** comes from three release-tree runs. They were interleaved with three runs of
  the untouched 1.6.4 tree on its own 6.6.6 pin, as a same-moment host control. The control read
  **−1.1%** (median row) against its own baseline, so the host matched the 1.6.4 capture.
- **A/B** is 15 rounds interleaving the 1.6.4 tree on 6.6.6 with this tree on 6.6.20, run
  earlier the same day. Order alternated each round. It isolates the toolchain from host drift.

| benchmark | 1.6.4 baseline | 1.6.5 gate | Δ gate | 1.6.5 median of 3 | Δ median | A/B 6.6.6 → 6.6.20, median of 15 | Δ A/B |
|---|---:|---:|---:|---:|---:|---:|---:|
| injection_scores_from_json | 331 | 294 | -11.2% | 306 | -7.6% | 339 → 316 | -6.8% |
| accelerator_device_full | 101 | 91 | -9.9% | 97 | -4.0% | 103 → 100 | -2.9% |
| token_usage_update | 34 | 31 | -8.8% | 33 | -2.9% | 34 → 34 | +0.0% |
| traceparent_format | 1838 | 1775 | -3.4% | 1803 | -1.9% | 1860 → 1861 | +0.1% |
| security_context_full | 686 | 632 | -7.9% | 673 | -1.9% | 696 → 695 | -0.1% |
| accel_flags_from_json | 709 | 664 | -6.3% | 696 | -1.8% | 725 → 720 | -0.7% |
| token_usage_to_json | 875 | 826 | -5.6% | 859 | -1.8% | 896 → 888 | -0.9% |
| trace_context_new | 1159 | 1126 | -2.8% | 1141 | -1.6% | 1176 → 1179 | +0.3% |
| agent_id_new | 576 | 561 | -2.6% | 568 | -1.4% | 583 → 584 | +0.2% |
| agent_id_roundtrip | 1020 | 1000 | -2.0% | 1010 | -1.0% | 1036 → 1042 | +0.6% |
| version_to_str | 108 | 105 | -2.8% | 107 | -0.9% | 111 → 111 | +0.0% |
| resource_limits_from_json | 459 | 451 | -1.7% | 455 | -0.9% | 468 → 468 | +0.0% |
| agent_stats_to_json | 669 | 632 | -5.5% | 664 | -0.7% | 686 → 684 | -0.3% |
| agent_id_to_str | 741 | 725 | -2.2% | 737 | -0.5% | 752 → 756 | +0.5% |
| resource_limits_to_json | 975 | 930 | -4.6% | 971 | -0.4% | 996 → 1005 | +0.9% |
| injection_scores_to_json | 770 | 725 | -5.8% | 767 | -0.4% | 788 → 788 | +0.0% |
| audit_entry_full | 2256 | 2180 | -3.4% | 2251 | -0.2% | 2239 → 2235 | -0.2% |
| accel_flags_to_json | 1618 | 1533 | -5.3% | 1618 | +0.0% | 1657 → 1678 | +1.3% |
| sandbox_config_default | 23 | 23 | +0.0% | 23 | +0.0% | 24 → 24 | +0.0% |
| trace_context_child | 574 | 564 | -1.7% | 575 | +0.2% | 595 → 592 | -0.5% |
| token_usage_from_json | 416 | 399 | -4.1% | 417 | +0.2% | 423 → 431 | +1.9% |
| message_build_3turn | 365 | 346 | -5.2% | 369 | +1.1% | 373 → 382 | +2.4% |
| inference_request_full | 362 | 355 | -1.9% | 369 | +1.9% | 376 → 384 | +2.1% |
| agent_stats_from_json | 287 | 289 | +0.7% | 298 | +3.8% | 289 → 307 | +6.2% |
| version_roundtrip | 244 | 253 | +3.7% | 256 | +4.9% | 252 → 263 | +4.4% |

**The toolchain is performance-neutral for agnostik.** The A/B's median row is **0.0%**, and 23 of
25 rows are within ±5%. Only two rows move consistently in both measurements, and both are small:

- `injection_scores_from_json` is faster: −6.8% in the A/B and −7.6% in the median.
- `version_roundtrip` is slightly slower: +4.4% / +4.9%, about 11 ns, below the gate's 50 ns
  ns-bracket floor.

`agent_stats_from_json` reads +6.2% in the A/B and +3.8% in the median, about 11–18 ns. The gate
column (median row −3.4%) is one sample and sits below the medians, as single runs do.

**Host load.** Two earlier gate runs, taken while other sessions' compilers held the 16-core host
at load 6–8, read +8% and +13.5% on the median row. Both still passed. Neither was recorded,
because the appended row becomes the next release's baseline, and a baseline inflated by load
would hide a regression. The release run appended to `docs/benchmarks/history.csv` (25 rows) was
taken after the host quieted (load ≈ 2.7) and re-checked: the gate passes against it, 25 checked,
0 regressions. Its commit column reads `07720f0`, HEAD at the time, because
`scripts/bench-history.sh` records HEAD and the 1.6.5 changes were not yet committed.

Binary size: x86_64 DCE **127,096 → 132,424 B** (+5,328 B, +4.2%) on identical source. That is
6.6.20's code generation and the re-vendored stdlib's reachable code (2,063 unreachable fns
removed, 586,347 B, against 1,986 / 553,519 B for the same source on 6.6.6). aarch64 **1,024,016 → 828,648 B** (−19.1%), from
6.6.18's compile-time syscall-translation fold. All 25 rows resolve (no `UNRESOLVED` lines).

### Verification

- **Every `ci.yml` step, clean-room.** In a throwaway `HOME`, the toolchain was unpacked from the
  published `cyrius-6.6.20-x86_64-linux.tar.gz` exactly as CI's install step does it. In a copy of
  the tree, all 19 steps passed, from stdlib sync through the bench-regression gate, the security
  scan and the version check. The finished tree, with the version bump and the two comment edits,
  passes the same steps locally. Its dist check was run as an idempotence check, because the
  regenerated bundle is not committed yet. The tarball matches its `.sha256` sidecar. Its `lib/` and every
  binary CI uses are byte-identical to the local install; only its bundled `bin/ci.sh` helper
  differs.
- **Tests:** 18 files, **1,536 assertions, 0 failures** on x86_64-linux, and the same under
  `qemu-aarch64` 11.1.1 (`cyrius tests --aarch64`).
- **Builds:** `CYRIUS_DCE=1` x86_64 **132,424 B** (127,096 B on 6.6.6, same source; plain build
  718,152 B), aarch64 **828,648 B** (1,024,016 B; 6.6.18 folds aarch64 syscall translation at
  compile time, and aarch64 DCE still NOP-fills), agnos 132,776 B, Windows 848,384 B.
- **Gates:** `deps --verify` 33 / 33; api-surface 923 = snapshot; doc-debt 849 = baseline; `cyrius
  distlib` idempotent (bundle and sidecar byte-identical on a second run); lint and fmt clean;
  `CYRIUS_TYPE_CHECK=1` no agnostik-side warnings; `cyrius vet` clean. `cyrius audit` passes fmt,
  lint, tests (18 / 18) and bench, and exits 1 only on its docs phase (1,425 over `src tests`,
  as at 1.6.4).
- **Downstream:** kybernet, the consumer on the newest agnostik tag, rebuilt on its own pin with
  the 1.6.5 bundle, to an unchanged binary size. The full five-repo sweep is the v1.7.0 slot.

## [1.6.4] - 2026-09-23

**Patch: a review of the two open toolchain-issue records. One is closed and archived; the other is
re-verified open and filed upstream.** Documentation only: no source, test, API
or wire-format change. 916 public fns, 1,536 assertions.

### Changed — `cyrius-audit-missing-check-script-2026-04-26` closed and archived

The record's two toolchain defects were fixed upstream long ago: the missing `check.sh` at cyrius
6.2.24, and the `tests`/`bench` preamble at 6.4.73. It stayed open for one residue, "`cyrius self`
still false-fails", re-measured at every pin since 1.3.6.

Reading the verb's implementation (`cbt/commands.cyr` at the 6.6.6 tag) closes it as **not a
defect**. `cyrius self` is the compiler's self-hosting fixpoint check:
1. It pipes the compiler source for the target (`src/main.cyr` on x86-64 Linux) into `cycc` raw,
   with no manifest and therefore no stdlib preamble.
2. It runs the result as a compiler on the same source.
3. It byte-compares the two builds.

In agnostik, `src/main.cyr` is the test harness, so the check can never pass. The skipped preamble
is where the two undefined symbols came from, and the harness is not a compiler.

- The file gains a **Status** header and a **Resolution** section, and moves to
  `docs/development/issues/archive/`. Its one relative link was fixed for the move.
- Every link to it now points at the archive: three in this CHANGELOG, one in the 2026-04-26 audit,
  and the ones in the roadmap and `state.md`. This follows the precedent set when the lint issue was
  archived.
- `state.md` no longer says to run `cyrius self` and expect a known failure. It now says not to run
  it here. `cyrius audit` is the gate, and its fmt, lint, tests and bench phases pass on 6.6.6.
- The roadmap's v1.3.6 backlog item is closed.
- The Resolution suggests an optional upstream papercut: `cmd_self` could refuse to run outside the
  cyrius source repo, which cbt already recognises through `_dep_is_cyrius_source_repo()`.

### Changed — `cyrius-derive-comments-in-struct-body-2026-05-10` re-verified: still open

The `#derive(Serialize)` body-comment corruption is unchanged on cyrius 6.6.0, 6.6.2 and 6.6.6. The
original repro prints `{"#":330325,…}`, and all four negative cases the record lists corrupt too.
The bug had **no record in the cyrius issue tracker**, so it is now filed there, with a self-proving
repro, as `2026-09-22-agnostik-derive-serialize-comment-in-struct-body.md`. The nearest upstream
issue, fixed at 6.6.6,
is a different lexer path: comments that start with an attribute name. That is the likely reason
the bug has survived since 5.10.14. The record gains a **Status** header and an update with these
findings.

agnostik stays protected. All 7 derive structs keep their comments above the directive. A
mutation check, one comment placed inside `InjectionScores`' body, makes `test_v110_serde_golden.tcyr`
fail (`FAIL: InjectionScores compact bytes`), so the corruption cannot ship silently.

### Fixed — archive links and index

- `archive/cyrlint-char-literal-brace-bug-2026-05-09.md` has linked the sibling `cyrius/CHANGELOG.md`
  with one `../` too few ever since its own move into `archive/`. Fixed.
- `archive/README.md` gains an index of the archived files and their resolutions. It also now
  allows a file to be archived when a residue is shown not to be a defect.

### Performance

`scripts/bench-regression.sh` against the 1.6.3 baseline in `history.csv` (commit `542a6ea`):
**25 checked, 0 new, 0 regressions.** The medians column is 3 further runs taken right after the
gate.

| benchmark | 1.6.3 baseline | 1.6.4 gate | Δ gate | 1.6.4 median of 3 | Δ median |
|---|---:|---:|---:|---:|---:|
| token_usage_update | 34 | 32 | -5.9% | 32 | -5.9% |
| agent_id_roundtrip | 1023 | 982 | -4.0% | 970 | -5.2% |
| resource_limits_to_json | 971 | 930 | -4.2% | 926 | -4.6% |
| token_usage_from_json | 412 | 397 | -3.6% | 393 | -4.6% |
| sandbox_config_default | 24 | 23 | -4.2% | 23 | -4.2% |
| resource_limits_from_json | 458 | 441 | -3.7% | 440 | -3.9% |
| agent_id_to_str | 739 | 711 | -3.8% | 713 | -3.5% |
| token_usage_to_json | 874 | 837 | -4.2% | 843 | -3.5% |
| agent_stats_from_json | 281 | 266 | -5.3% | 272 | -3.2% |
| security_context_full | 679 | 662 | -2.5% | 658 | -3.1% |
| inference_request_full | 362 | 359 | -0.8% | 351 | -3.0% |
| version_roundtrip | 245 | 238 | -2.9% | 238 | -2.9% |
| accel_flags_from_json | 710 | 703 | -1.0% | 690 | -2.8% |
| agent_stats_to_json | 671 | 654 | -2.5% | 652 | -2.8% |
| version_to_str | 109 | 106 | -2.8% | 106 | -2.8% |
| traceparent_format | 1820 | 1743 | -4.2% | 1775 | -2.5% |
| accel_flags_to_json | 1616 | 1618 | +0.1% | 1583 | -2.0% |
| accelerator_device_full | 101 | 99 | -2.0% | 99 | -2.0% |
| message_build_3turn | 364 | 353 | -3.0% | 357 | -1.9% |
| trace_context_child | 585 | 570 | -2.6% | 574 | -1.9% |
| injection_scores_to_json | 769 | 761 | -1.0% | 755 | -1.8% |
| audit_entry_full | 2242 | 2211 | -1.4% | 2220 | -1.0% |
| trace_context_new | 1139 | 1123 | -1.4% | 1128 | -1.0% |
| injection_scores_from_json | 329 | 331 | +0.6% | 328 | -0.3% |
| agent_id_new | 571 | 554 | -3.0% | 588 | +3.0% |

**No code changed**, so the whole table is host variance. `git diff 1.6.3` over `src/`, `tests/`,
`scripts/`, the CI workflows and `cyrius.cyml` is empty, and the DCE binary is byte-for-byte
1.6.3's size, 127,096 B. The median row moved −2.8%, and 24 of 25 rows moved down, which matches a
host a little less loaded than during the 1.6.3 capture. The one row up, `agent_id_new` +3.0%
(571 → 588 ns), is dominated by the `getrandom` syscall. Its four 1.6.4 samples spanned 554–624 ns,
with the gate's own sample at 554 ns (−3.0%), on byte-identical code. All 25 rows resolve (no `UNRESOLVED` lines, 3 of 3 runs).

The release run is appended to `docs/benchmarks/history.csv` (25 rows) as the next baseline. Its
commit column reads `3729b55`, the tagged 1.6.3 commit, because `scripts/bench-history.sh`
records HEAD and the 1.6.4 changes were not yet committed.

### Verification

- **Code unchanged** since the `1.6.3` tag, as above. `CYRIUS_DCE=1 cyrius build`: x86_64
  127,096 B and aarch64 1,024,016 B, both identical to 1.6.3.
- **Tests:** 18 files, **1,536 assertions, 0 failures**.
- **Every `ci.yml` step**, run in order on a clean copy of the tree with the archive move applied:
  19 of 19 green. That includes `scripts/doc-debt.sh check` (849, matching the baseline) and
  `scripts/api-surface.sh check` (916). The release workflow's `Verify version` step accepts
  tag `1.6.4`.
- **Links:** every relative link in every Markdown file in the repo resolves (0 broken). That
  covers the repointed references to the archived record.
- **The issue findings are measured, not read.**
  - The derive repro was rebuilt warning-free on 6.6.0, 6.6.2 and 6.6.6.
  - The golden-test guard was mutation-checked.
  - `cyrius audit` on 6.6.6 passes fmt, lint, tests (18 / 18) and bench; it exits 1 only on its
    docs phase (1,425 over `src tests`).
  - `cyrius self` prints `FAIL: cycc!=cycc`, as its implementation predicts for a
    non-compiler repo.


## [1.6.3] - 2026-09-23

**Patch: identifier generation has no `/dev/urandom` fallback any more (F-023), `AgentInfo`'s JSON
round-trips and no longer crashes on a missing id (F-024), and two roadmap items land — benchmark
windows that clear the 6.6.5 resolution bar, and a documentation-debt gate.** No public API change:
916 fns, matching `docs/api-surface.snapshot`. 18 test files, **1,536 assertions, 0 failures**, on
x86_64-linux and on aarch64 (under `qemu-aarch64`).

### Fixed — F-023: `_fill_random` no longer falls back to `/dev/urandom`

F-022 (1.6.2) moved `_fill_random`'s primary path to `sys_getrandom`, but its `/dev/urandom`
fallback and fail-loud exit still issued raw x86_64-linux syscall numbers: open 2, read 0, close 3,
write 1, exit 60. aarch64 and Windows route those. The agnos target does not, and there they mean
getpid, exit, spawn, write and winsize. The 1.6.2 `--agnos` binary confirms it: the old function
loads 2, 0, 3, 1 and 60 straight into `syscall`. The path ran only if `sys_getrandom` failed.

This takes the audit's option 2. The fallback is gone, and the fail-loud path goes through the
stdlib wrappers: if `sys_getrandom` does not return `n`, `_fill_random` writes
`agnostik: CSPRNG unavailable (getrandom failed)` to stderr with `sys_write` and exits 70 with
`sys_exit`. In the new `--agnos` binary `_fill_random` issues no syscall of its own; it reaches
getrandom #45, write #1 and exit #0 through the wrappers.

Option 1, routing the fallback through the wrappers plus an agnos arm for
`sys_open(name, namelen, flags)`, was not taken. `sys_getrandom` is defined on every target, and
neither agnos nor Windows has a `/dev/urandom` to open, so that arm could only ever fail. It would
also have put the first `#ifdef` into `src/` and `dist/agnostik.cyr`. `src/` now opens no files, and
its only raw syscalls are `error.cyr`'s four stderr writes, which the audit found correct on every
target.

⚠ **Behaviour change.** Two environments that used to get IDs from the fallback now exit 70 on the
first one:

- Linux kernels older than 3.17 (2014), which have no `getrandom(2)`.
- Sandboxes whose seccomp policy denies `getrandom` but still allows opening `/dev/urandom`.

This was run, not inferred. On x86_64, under a seccomp filter that fails `getrandom` with `ENOSYS`,
a probe calling `agent_id_new()` three times printed three IDs and exited 0 before the change (the
fallback served them). After it, the probe prints the message and exits 70; a `qemu-x86_64 -strace`
of that run shows `getrandom = -ENOSYS`, `write(2, …, 48)` and `exit(70)`, with no `open`. The
message text changed too; it read `(getrandom + /dev/urandom both failed)`.

**Unchanged:** the success path (the same `sys_getrandom` call and compare), exit code 70, and the
Linux exit semantics — `sys_exit` is `exit(2)`, as the raw 60 was. No public API, struct-layout or
wire-format change: 916 public fns, matching `docs/api-surface.snapshot`. `dist/agnostik.deps` is
unchanged, because `sys_write` and `sys_exit` come from the `syscalls` module `sys_getrandom`
already required.

The F-014 tests in `tests/tcyr/test_v137_hardening.tcyr` exercise a replica of the removed read
loop, not `src/`. They stay as a pin on the flag + `continue` loop exit and on signed comparison,
and the file header now says so.

### Fixed — `AgentInfo_from_json` reads what `AgentInfo_to_json` writes; F-024

`AgentInfo` was the one serializable struct whose `_from_json` could not read its own `_to_json`.
`AgentInfo_to_json` writes `agent_type` and `status` as name strings (`"Service"`, `"Failed"`), but
`AgentInfo_from_json` read integer `agent_type_id` / `status_id` keys. So every round trip came back
as type 0 / status 0 (`AGENT_TYPE_SYSTEM` / `AGENT_PENDING`). It also had no test: it was the only
one of the 9 `_from_json` functions that `test_serde_roundtrip.tcyr` did not cover, although the
file's header claimed all 9. This was a v1.3.0 roadmap backlog item.

`AgentInfo_from_json` now parses the names with `agent_type_parse` / `agent_status_parse` (1.4.0).
The integer keys stay as the fallback when a name is absent or unrecognised, so JSON in the old
shape parses exactly as before. When both are present, a recognised name wins.

**F-024 (LOW) — `AgentInfo_to_json` dereferenced a null id.** `AgentInfo_from_json` leaves the id
at 0 when the `id` key is missing or is not a UUID, and `AgentInfo_to_json` passed that 0 straight
to `agent_id_to_str`, which reads from address 0. So parsing any JSON without a valid id and
re-serializing the result crashed; a probe reproduced it (SIGSEGV, exit 139). An id of 0 is now
written as `"id":null`, which `AgentInfo_from_json` reads back as 0. A valid id serializes
byte-for-byte as before, pinned by a golden test. Latent: no consumer in the local ecosystem calls
either function. Written up in [`docs/audit/2026-09-23-audit.md`](docs/audit/2026-09-23-audit.md).

**Tests.** `test_serde_roundtrip.tcyr` gains 123 assertions (36 → 159):
- all 27 `AgentType` × `AgentStatus` pairs round-trip id, name, type and status;
- byte-exact output for a fixed id, with and without a null name;
- a null, missing and unparseable id;
- the legacy integer keys, name-over-legacy precedence, and the unknown-name fallback.

`test_v112_fuzz.tcyr` gains a 9th target: 200 random JSON-like inputs plus 6 seeds through
`AgentInfo_from_json` → `AgentInfo_to_json`. It checks the re-serialize step on purpose, because
a parse-only check passes on the old code. Against the unfixed `src/agent.cyr` (HEAD `3da9b65`),
both files die with SIGSEGV.

**No public API change** (916 fns). The only wire-format change is for an id of 0, which previously
crashed instead of serializing.

### Changed — benchmark windows sized to clear the 6.6.5 resolution bar

cyrius 6.6.5 reworked `lib/bench.cyr` so that a timed window counts toward a row's `min`/`max` only
if its net duration clears 100 × (clock floor + tick). That is ~225 µs on the reference host, and up
to ~455 µs on the Linux CI hosts `bench.cyr` documents. The harness's windows (batch × per-op time)
ranged from 25 µs to 1.1 ms. Seven or eight rows, depending on the run, never cleared the bar, so
their min/max just echoed the mean. Rows near it counted only their slow windows, so they could print a `min` above their `avg`:
`message_build_3turn` read `368ns avg (min=446ns …)`. This was a v1.6.2 roadmap backlog item.
Upstream tracks the instrument side as an open issue,
`2026-09-21-hisab-bench-min-above-mean-below-resolution-bar.md` in the cyrius repo, filed from
another consumer. Sizing the windows clear of the bar sidesteps it, and stays valid once it is
fixed.

`tests/bcyr/agnostik.bcyr` now sizes every row's window to about 1 ms or more. Batches range from
1,000 to 50,000 iterations, and rounds drop from 10 to 5. With 5 rounds peak RSS is 126 MB, against
32 MB before and 250 MB at 10 rounds, because the bump allocator never frees and the fastest rows
now run 250,000 iterations. The run takes 0.25 s. Every row now resolves: no `UNRESOLVED` lines and
no `avg` outside `[min, max]`, 3 of 3 runs.

**What the gate sees.** `scripts/bench-regression.sh` parses only `avg`. An interleaved A/B, 5
pairs of the old and new harness on the same tree, put 23 of 25 rows within ±5% and the median row
at ~0%. The two outliers are the two smallest ops: `sandbox_config_default` 26 → 23 ns and
`token_usage_update` 31 → 33 ns. Those are 2–3 ns moves, and the longer windows make them less
sensitive to clock-floor calibration. The 1.6.3 baseline row therefore comes from the new sizes.

### Added — documentation-debt gate; `src/classification.cyr` fully documented

This is the v1.3.6 roadmap backlog item on undocumented public fns. That item was the sole reason
`cyrius audit` exits non-zero, and it planned for incremental work: "document a module per cycle,
gate new fns".

**The gate.** `scripts/doc-debt.sh check` now runs in CI next to the API-surface check. It lists
every fn in `src/` that `cyrius doc --check` reports as undocumented, as `module::fn`. `cyrius doc`
counts a fn as documented when a `#` comment sits directly above it. The gate diffs that list
against the committed `docs/undocumented.baseline` and fails in two cases:

- **A new undocumented fn.** So from here on every new public fn ships with a doc comment, while
  existing debt is grandfathered.
- **A listed fn that has since been documented.** So the committed count moves only when someone
  records the progress, with `scripts/doc-debt.sh update`.

Both directions were exercised: a probe fn added to `src/secrets.cyr` failed with
`+ secrets::tmp_probe_fn`, and documenting two listed fns failed until the update.

**This cycle's module.** `src/classification.cyr` goes from 2 of 11 documented to 11 of 11. The
`ClassificationResult` field meanings come from the original Rust definition
(`rust-old/src/classification.rs` in history): `level` is the final level, which policy may
override, and `auto_level` is what detection produced. `AgentInfo_to_json` and
`AgentInfo_from_json` are also documented, since their contracts changed above. The undocumented
count over `src/*.cyr` goes **860 → 849**, and the baseline records 849.

Writing the docs surfaced a contract gap of the F-021 kind, recorded on the roadmap rather than
fixed in a patch, because a fix would add API: `ClassificationResult` has getters but no setters
for `level`, `auto_level` and `confidence`. So the policy override the original modeled cannot be
expressed without raw `store64` offsets. `confidence` also has no defined unit; the Rust original
was an optional 0.0–1.0 float.

### Performance

`scripts/bench-regression.sh` against the 1.6.2 baseline in `history.csv` (commit `5e1b9ed`):
**25 checked, 0 new, 0 regressions** (thresholds 50% ns-bracket / 80% µs-bracket). The medians
column is 3 further runs taken right after the gate.

| benchmark | 1.6.2 baseline | 1.6.3 gate | Δ gate | 1.6.3 median of 3 | Δ median |
|---|---:|---:|---:|---:|---:|
| accel_flags_from_json | 743 | 666 | -10.4% | 674 | -9.3% |
| sandbox_config_default | 25 | 22 | -12.0% | 23 | -8.0% |
| token_usage_from_json | 413 | 420 | +1.7% | 385 | -6.8% |
| injection_scores_from_json | 338 | 308 | -8.9% | 319 | -5.6% |
| agent_stats_from_json | 278 | 292 | +5.0% | 265 | -4.7% |
| injection_scores_to_json | 765 | 718 | -6.1% | 730 | -4.6% |
| accel_flags_to_json | 1595 | 1588 | -0.4% | 1534 | -3.8% |
| accelerator_device_full | 100 | 96 | -4.0% | 97 | -3.0% |
| agent_stats_to_json | 661 | 629 | -4.8% | 643 | -2.7% |
| trace_context_new | 1091 | 1099 | +0.7% | 1068 | -2.1% |
| token_usage_to_json | 854 | 858 | +0.5% | 844 | -1.2% |
| version_roundtrip | 240 | 239 | -0.4% | 237 | -1.2% |
| message_build_3turn | 350 | 342 | -2.3% | 350 | +0.0% |
| resource_limits_to_json | 930 | 916 | -1.5% | 930 | +0.0% |
| security_context_full | 643 | 638 | -0.8% | 644 | +0.2% |
| inference_request_full | 338 | 335 | -0.9% | 339 | +0.3% |
| resource_limits_from_json | 441 | 440 | -0.2% | 445 | +0.9% |
| traceparent_format | 1713 | 1740 | +1.6% | 1748 | +2.0% |
| audit_entry_full | 2142 | 2192 | +2.3% | 2201 | +2.8% |
| version_to_str | 100 | 110 | +10.0% | 103 | +3.0% |
| agent_id_new | 524 | 533 | +1.7% | 541 | +3.2% |
| agent_id_to_str | 681 | 711 | +4.4% | 711 | +4.4% |
| trace_context_child | 535 | 559 | +4.5% | 562 | +5.0% |
| agent_id_roundtrip | 926 | 985 | +6.4% | 974 | +5.2% |
| token_usage_update | 29 | 31 | +6.9% | 31 | +6.9% |

Medians land between −9.3% and +6.9% of the baseline. Two kinds of change could move these rows, and
each was isolated with an interleaved A/B on this host:

- **The harness change** (windows resized, 10 → 5 rounds). Old vs new harness on the same tree,
  5 pairs: 23 of 25 rows within ±5%. The two outliers are this table's two smallest rows, and they
  moved the same way here: `sandbox_config_default` −3 ns, `token_usage_update` +2 ns.
- **The code changes** (F-023's fallback removal; the `AgentInfo` fix, which no benchmark reaches).
  The 1.6.2 tag's `src/` vs this release's `src/`, both built with the new harness, 3 pairs:
  **25 of 25 rows within ±5%**, 13 within ±2%, with mixed signs. The ID-generating rows, which
  run through `_fill_random`, moved −2.0% to +3.1% (`agent_id_new` +3.1%, `trace_context_new`
  −2.0%).

So the code costs nothing measurable. The ID-generating rows' +3–5% against the baseline is host
variance: those rows are dominated by the `getrandom` syscall, and `agent_id_new` alone has read
516–572 ns across this release's runs. The largest win, `accel_flags_from_json` −9.3%, is mostly a
high baseline sample: the 1.6.2 medians for that row were 703–713 ns, and the harness A/B put it
at −0.3%. The earlier F-023-only A/B (the unchanged tree `b2fe277` vs the F-023 tree) agrees: the 8
ID-generating rows moved −0.8% to +1.9%.

The release run is appended to `docs/benchmarks/history.csv` (25 rows) as the next gate's baseline,
the first measured with the resized windows. Its commit column reads `542a6ea`, because
`scripts/bench-history.sh` records HEAD and the version bump was not yet committed.

### Verification

- `CYRIUS_DCE=1 cyrius build`: x86_64-linux **127,096 B** (1.6.2: 127,128), aarch64
  **1,024,016 B** (1,024,048), agnos **126,944 B** (126,976), Windows **790,016 B** (790,528). All
  four build with 0 warnings except Windows' informational routed-raw-syscall note, which `error.cyr`'s
  four stderr writes produce on 1.6.2 too. Plain x86_64 build: 684,152 B.
- `cyrius test`: **18 files, 1,536 assertions, 0 failures**, on x86_64-linux and on aarch64
  (`cyrius test --aarch64`, under `qemu-aarch64`). 1.6.2 had 1,412; the +124 are the 123 new
  `AgentInfo` assertions and the 9th fuzz target.
- `cyrius fmt --check` and `cyrius lint`: 34 files clean, 0 warnings. `CYRIUS_TYPE_CHECK=1`: 0
  agnostik-side warnings. `cyrius vet`: 24 deps, 0 untrusted, 0 missing.
- `scripts/api-surface.sh check`: 916 fns, matching the snapshot. `scripts/doc-debt.sh check`: 849
  undocumented fns, matching the baseline.
- `cyrius distlib`: `dist/agnostik.cyr` 161,663 → **164,007 B**, carrying the `_fill_random`,
  `AgentInfo` and doc-comment changes plus the banner. `dist/agnostik.deps` unchanged.
- `qemu-aarch64 -strace` on an ID probe: one `getrandom(…,16,0) = 16` per ID. On `--agnos`,
  disassembled rather than run, `_fill_random` issues no syscall of its own; its callees load 45, 1
  and 0.
- Every `ci.yml` step, run in order on a clean copy of the tree: 19 of 19 green.
- CI security scan (run locally, verbatim): clean.

## [1.6.2] - 2026-09-23

**Toolchain refresh to cyrius 6.6.6, plus the `_fill_random` fix that pin's release notes asked
agnostik for.** No public API, struct-layout or wire-format change — 916 public fns, matching
`docs/api-surface.snapshot`. 18 test files, **1,412 assertions, 0 failures**, on x86_64-linux and
on aarch64 (under `qemu-aarch64`).

### Changed — cyrius pin 6.6.2 → **6.6.6**; stdlib re-vendored

`cyrius lib sync` + `cyrius deps` against the 6.6.6 snapshot. `lib/` now holds **32** files, every
one byte-identical to the snapshot:

- **17 changed content**: `alloc`, `assert`, `bayan`, `chrono`, `fmt`, `fnptr`, `hashseed`, `io`,
  `string`, `vec`, and `syscalls` with its six platform peers.
- **13 did not**, including the git-tracked `keccak.cyr`.
- **2 are new**: `alloc_cx.cyr`, a new `alloc` peer that `lib sync` takes along with every
  `alloc_*`; and `args_agnos.cyr`, which 6.6.6's `io.cyr` now includes.

A fresh checkout gets 27 files from `lib sync` (26, plus the tracked `keccak.cyr`). `cyrius deps`
adds the five transitive includes: `args_agnos`, `args_macos`, `atomic`, `boxed` and `hashseed`.

There were no other deps to update. `cyrius.cyml` declares no `[deps.NAME]` git dependencies, so
`cyrius deps` and `cyrius update` have no tags to move, and no `cyrius.lock` is written (since 6.6.4
a stdlib-only project gets one only from `deps --lock`). `dist/agnostik.deps` is unchanged at 15
stdlib leaves.

⚠ **`cyrius update` is not the refresh command for this repo.** With no git deps it copied the
*whole* snapshot into `lib/`: 111 files, including an untracked `lib/unicode/` that the `lib/*.cyr`
ignore rule does not cover. That was reverted; `lib sync` + `deps` is the right pair.

Builds still read the stdlib from the pinned snapshot, not from `./lib/`. The 6.6.6 binary is
byte-identical whether built against the stale 6.6.2 `lib/` or the re-synced one.

### Fixed — F-022: `_fill_random` reached `getrandom` by its raw x86_64 number

`syscall(318, buf, n, 0)` → `sys_getrandom(buf, n, 0)` in `src/types.cyr`. 318 is `getrandom` on
x86_64-linux and nowhere else:

| target | raw 318 | effect on every `agent_id_new` / `user_id_new` / trace-id call |
|---|---|---|
| aarch64-linux | unassigned, `-ENOSYS` | fell to `/dev/urandom`: openat + read + close, 4 syscalls instead of 1. In a sandbox that denies the open, the fail-loud path exits 70 |
| agnos | no syscall 318 (its `getrandom` is #45) | per the stdlib's agnos peer, an unknown number falls through the kernel dispatch and the caller reads the result as data |
| Windows | unrouted, `-ENOSYS` | no `/dev/urandom` either, so the first ID exits 70 |
| macOS | rewritten to `getentropy`, which returns `0` on success | read as a failure, so the fallback ran on every ID |

The aarch64 row was executed: `qemu-aarch64 -strace` showed `Unknown syscall 318` and then the
fallback, once per ID. After the fix it shows `getrandom(…,16,0) = 16`. The IDs were random all
along, since the fallback reads the kernel CSPRNG, so on aarch64 this was cost and availability,
not entropy. The agnos, Windows and macOS rows come from the stdlib tables and were not executed.

⭐ **`sys_getrandom`, not the `SYS_GETRANDOM` spelling cyrius 6.6.4's notes suggested for this
site.** The constant is absent from the Windows peer, so `SYS_GETRANDOM` fails `--win` builds with
`undefined variable 'SYS_GETRANDOM'`. It would also leave macOS reading getentropy's `0` as a
failure. The wrapper is the stdlib's portable CSPRNG primitive and is defined in every target's
peer.

**Consumer impact: none on x86_64-linux.** It is the same syscall with the same arguments; only the
wrapper call is new, and the wrapper's `r == 0 → len` branch cannot fire on Linux for `n > 0`. On
aarch64, agnos, Windows and macOS the primary path now works. `dist/agnostik.cyr` now references
`sys_getrandom`, which every 6.6.x stdlib peer defines, and agnostik 1.6.x already requires
cyrius ≥ 6.6.2.

**Still open — F-023 (LOW).** The `/dev/urandom` fallback and the fail-loud exit keep raw x86_64
numbers (2 / 0 / 3 / 1 / 60). aarch64 and Windows route them. On agnos they mean getpid / exit /
spawn / write / winsize. After F-022 that path runs only if `sys_getrandom` fails. The fix is a
design choice (an agnos-specific open, or dropping the fallback), so it is deferred. Both findings
are written up in [`docs/audit/2026-09-23-audit.md`](docs/audit/2026-09-23-audit.md).

### Fixed — CI never ran the aarch64 cross-build

`ci.yml` and `release.yml` probed `$HOME/.cyrius/bin/cc5_aarch64`. cyrius renamed `cc5` → `cycc` at
6.0.0. The `cc5_*` compat symlinks came only from its `install.sh` and were dropped at 6.1.1, and no
6.x release tarball — the thing CI unpacks — ships the old name (checked 6.0.14, 6.0.26 and 6.6.6).

So on every pin since 1.2.3, the first 6.x one, CI skipped the aarch64 build with a `::warning::`
and exited 0, and every release silently dropped its `agnostik-<tag>-aarch64-linux` artifact. Both
workflows now probe `cycc_aarch64`. The skipped build is where F-022 would have surfaced: the
compiler flagged `src/types.cyr:8` on aarch64 at the old 6.6.2 pin too.

**Release artifact change:** from 1.6.2 on, releases again carry `agnostik-<tag>-aarch64-linux`
(1,024,048 B), listed in `SHA256SUMS`.

### Toolchain notes — what 6.6.3 → 6.6.6 changed for agnostik

- **DCE has removed code for real since cyrius 6.5.72**, and 1.6.0 crossed that release without
  recording it; the `675,824 B` in its entry is the plain build. The `CYRIUS_DCE=1` artifact that CI
  and release ship went **629,032 B** (1.3.6–1.5.x on 6.5.35) → **126,960 B** (the 1.6.1 tree on
  6.6.2) → **127,128 B** (1.6.2 on 6.6.6, with 1,989 unreachable fns / 553,555 B eliminated). The
  plain build went 675,824 → 684,184 B. aarch64 DCE still NOP-fills (1,024,048 B).
- **`lib/bench.cyr` accounting was reworked in 6.6.5.** It measures the timer floor (~1.3 µs per
  clock read on this host) and nets it per window. min/max now count only windows that clear a
  100 × (floor + tick) resolution bar. Rows whose window sits near that bar can print `avg < min`:
  `message_build_3turn` and `resource_limits_from_json` did in 2 of 3 runs. `avg`, the only field
  the gate parses, counts every window and is unaffected.
- **cyrdoc counts more in 6.6.5**: `pub fn`, `public fn` and `#inline fn`, and whole files rather
  than the first 64 KB. The per-file `cyrius doc --check` sum over `src/*.cyr` is **860** (853 at
  6.5.35), plus 187 over the test and bench files. `cyrius audit`'s docs phase reports **1,421**
  over its `src tests` scope, which is not the sum of those per-file counts (1,047). That phase is
  still the only reason `audit` exits 1; fmt, lint, tests (18/18) and bench pass.
- **`cyrfmt --check` really checks now (6.6.5).** It used to print to stdout and exit 0. All 34
  source, test and bench files are clean.
- `cyrius self` still false-fails exactly as recorded at 1.3.6 (`bayan_json_get` / `clock_now_ns`,
  `FAIL: cycc!=cycc`).

### Performance

`scripts/bench-regression.sh`: **25 checked, 0 new, 0 regressions**. The baseline is the 1.5.0 run
on cyrius 6.5.35, because 1.5.1, 1.6.0 and 1.6.1 appended none. The deltas therefore span two pin
steps, 6.5.35 → 6.6.2 → 6.6.6. To split them, the table adds medians of 3 runs of the *same* source
(the 1.6.1 tree) under each pin, taken back to back.

| benchmark | 1.5.0 baseline (6.5.35) | 1.6.2 gate | Δ vs baseline | 6.6.2 median | 6.6.6 median | Δ pin only |
|---|---:|---:|---:|---:|---:|---:|
| token_usage_update | 27 | 31 | +14.8% | 28 | 31 | +10.7% |
| injection_scores_from_json | 286 | 323 | +12.9% | 291 | 316 | +8.6% |
| token_usage_from_json | 378 | 418 | +10.6% | 392 | 396 | +1.0% |
| agent_stats_from_json | 256 | 283 | +10.5% | 275 | 267 | -2.9% |
| accel_flags_from_json | 674 | 727 | +7.9% | 682 | 703 | +3.1% |
| trace_context_child | 524 | 565 | +7.8% | 554 | 544 | -1.8% |
| trace_context_new | 1048 | 1126 | +7.4% | 1095 | 1022 | -6.7% |
| resource_limits_from_json | 432 | 459 | +6.2% | 435 | 433 | -0.5% |
| message_build_3turn | 346 | 359 | +3.8% | 348 | 368 | +5.7% |
| audit_entry_full | 2150 | 2223 | +3.4% | 2172 | 2147 | -1.2% |
| traceparent_format | 1673 | 1718 | +2.7% | 1668 | 1676 | +0.5% |
| resource_limits_to_json | 923 | 941 | +2.0% | 912 | 918 | +0.7% |
| agent_id_new | 550 | 559 | +1.6% | 532 | 516 | -3.0% |
| agent_id_roundtrip | 944 | 954 | +1.1% | 972 | 960 | -1.2% |
| accel_flags_to_json | 1547 | 1563 | +1.0% | 1507 | 1544 | +2.5% |
| version_roundtrip | 255 | 256 | +0.4% | 233 | 236 | +1.3% |
| token_usage_to_json | 829 | 829 | 0.0% | 800 | 807 | +0.9% |
| version_to_str | 106 | 104 | -1.9% | 101 | 105 | +4.0% |
| agent_stats_to_json | 677 | 661 | -2.4% | 639 | 648 | +1.4% |
| security_context_full | 657 | 639 | -2.7% | 610 | 650 | +6.6% |
| inference_request_full | 366 | 356 | -2.7% | 352 | 338 | -4.0% |
| agent_id_to_str | 713 | 689 | -3.4% | 703 | 698 | -0.7% |
| injection_scores_to_json | 777 | 744 | -4.2% | 750 | 750 | 0.0% |
| accelerator_device_full | 119 | 104 | -12.6% | 102 | 99 | -2.9% |
| sandbox_config_default | 34 | 27 | -20.6% | 25 | 25 | 0.0% |

Every row is inside its threshold (50% ns-bracket / 80% µs-bracket). The notable movers, read
against all three 6.5.35-era history runs rather than the single baseline row:

- **Win: `accelerator_device_full`.** Every 6.6.x sample (94–107 ns) is below every 6.5.35 sample
  (111–119 ns), about −10%. It arrived in the 6.5.35 → 6.6.2 step, and 6.6.6 leaves it flat.
- **Not a win: `sandbox_config_default`, despite reading −20.6%.** Its 34 ns baseline is the middle
  of a noisy 27 / 34 / 42 ns trio. 6.6.x sits at 24–29 ns, the low end of that range.
- **Drift: `injection_scores_from_json`** is the one consistent mover across the 6.6.6 step. All
  four 6.6.6 samples (309–347 ns) are above all three 6.6.2 samples (290–303 ns), which are
  themselves above the 6.5.35 range (276–289 ns): about +8% / +25 ns at 6.6.6.
- **Drift: `token_usage_update`**, +2–4 ns on a 27 ns op (6.5.35: 27 / 27 / 27; 6.6.6: 29–32).
- **Noise: `token_usage_from_json` and `agent_stats_from_json`.** Both read +10% against the
  baseline row, but they sit within or just above their 6.5.35 ranges (378–406 and 256–270 ns) and
  moved −3% to +1% across the 6.6.6 step itself.

**F-022's cost.** Five interleaved A/B pairs on 6.6.6 (raw 318 vs `sys_getrandom`, same host
state): 22 of 25 rows within ±2%. The three outliers at +4.7% to +5.4% are `agent_id_new` 535 → 563,
`injection_scores_from_json` and `accel_flags_from_json`, and the last two never call
`_fill_random`. The composites that do call it moved −1.5% to +1.6% (`agent_id_roundtrip` −1.4%,
`security_context_full` −1.5%, `trace_context_new` +1.6%). So there is no consistent per-call cost;
a call and a compare are a few ns against a ~500 ns syscall.

The release run is appended to `docs/benchmarks/history.csv` as the next baseline (25 rows). Its
commit column reads `5e1b9ed`, because the script records HEAD and the 1.6.2 tree was uncommitted
when it ran.

### Verification

- `CYRIUS_DCE=1 cyrius build`: x86_64-linux 127,128 B, aarch64 1,024,048 B, agnos 126,976 B,
  Windows 790,528 B. All build, with 0 warnings except Windows' informational
  routed-raw-syscall note.
- `cyrius test`: **18 files, 1,412 assertions, 0 failures**, on x86_64-linux and on aarch64
  (`cyrius test --aarch64`, under `qemu-aarch64`).
- `cyrius fmt --check` and `cyrius lint`: 34 files clean, 0 warnings, 0 untracked deferrals.
  `CYRIUS_TYPE_CHECK=1`: 0 agnostik-side warnings. `cyrius vet`: 24 deps, 0 untrusted, 0 missing.
- `scripts/api-surface.sh check`: 916 fns, matching the snapshot. `cyrius distlib --check`:
  current. The `dist/agnostik.cyr` diff is the version banner plus the `_fill_random` change
  (161,077 → 161,433 B).
- CI security scan (run locally, verbatim): clean.
- The published `cyrius-6.6.6-x86_64-linux.tar.gz` matches its `.sha256` sidecar, and its `lib/`
  and toolchain binaries are byte-identical to the local install.

### Correction

The 1.6.0 entry's "18 test files, 788 assertions" does not reproduce. The same 18 files, unchanged
since 1.5.1, report **1,412** on both 6.6.2 and 6.6.6.


## [1.6.1] - 2026-09-10

**A name collision that had been live and silent since 1.3.5.**

### Fixed — `health_check_new` renamed to `agnostik_health_check_new`

agnostik and **argonaut** both exported `health_check_new`, at different arities for different
types: agnostik's is a zero-argument default constructor for a 40-byte probe descriptor
(`hc_probe_type` / `hc_interval` / …); argonaut's is
`health_check_new(check_type, target, port, interval_ms, timeout_ms, retries)` over its own
`HealthCheck`.

**kybernet** vendors both, and got "last definition wins". argonaut's loaded second and took the
name, so agnostik's own zero-argument call landed on the six-argument body — reading six garbage
registers and storing them as a probe descriptor. No diagnostic, at any cyrius version before
6.6.2.

kybernet is the only affected consumer. (stiva also has a `health_check_new`, but it is a third,
independent one — arity 1, declared in its own `src/ansamblu.cyr`, and stiva depends on neither
agnostik nor argonaut. Three definitions of the name exist across the ecosystem; only the two that
get vendored together ever collided.)

cyrius **6.6.2**'s arity-disagreement diagnostic is what surfaced it — it makes a same-name
different-arity duplicate a hard error rather than a `last definition wins` warning. The
collision itself dates to agnostik 1.3.5 (2026-08-24) and was live in every release since.

⭐ **agnostik's side moved because it is the cheap one**: one caller (`src/main.cyr`) and no
external consumers — nothing outside this repo ever called the zero-argument form. argonaut's
name stays put, with its ~27 call sites across argonaut and kybernet unchanged.

**Consumer impact: none unless you also vendor argonaut.** No agnostik consumer called the
renamed function, so 1.6.0 pins that do not also depend on argonaut need no change.

### Fixed — `scripts/version-bump.sh` advised a tag name that does not resolve

The script's closing instructions said `git tag v${NEW_VERSION}`. Every one of agnostik's 20+
tags is **bare** (`1.6.0`, not `v1.6.0`), and every consumer pins `tag = "1.6.0"`. Following the
script's advice would have published a tag no `[deps.agnostik]` entry could resolve.


## [1.6.0] - 2026-09-10

**Migrated to the cyrius 6.6.x value form; `tagged_new` boxes now read through `boxed_*`.**
A minor rather than a patch: one public function changed arity. 18 test files, **788 assertions,
0 failures**.

### Changed — cyrius pin 6.5.35 → **6.6.2**

cyrius 6.6.0 flipped `Result`/`Option`/`Either` to a **value form** — a payload variant returns a
`(tag, payload)` REGISTER PAIR and allocates zero bytes — and deleted `payload()`. That release
also deleted `tagged_new()`, which agnostik uses **19 times**, on a survey scoped to the twelve
fold-table stdlibs but written down as "nothing in the ecosystem". agnostik is a *domain* library
and was never in that scope. cyrius **6.6.2** restores the boxed primitives and this release
migrates to them.

⭐ **All 19 `tagged_new` call sites are unchanged.** It came back under its own name because its
meaning never changed — it still builds the same 16-byte box, tag at +0, payload at +8. Only the
**reads** moved:

| before | after | why |
|---|---|---|
| `tag(b)` | `boxed_tag(b)` | `tag()` was deleted at 6.6.2 — 6.6.0 had kept the NAME and redefined the body, so on a box it silently returned the **pointer** |
| `payload(b)` | `boxed_payload(b)` | `payload()` stays deleted; the boxed reader is now explicit |

Migrated: content blocks (`src/llm.cyr`), stream events, metric values (`src/telemetry.cyr`),
stop reasons (`src/agent.cyr`), seccomp actions (`src/security.cyr`).

### ⚠ BREAKING — `result_print_agnostik_err` takes the tag

    result_print_agnostik_err(res)        ->  result_print_agnostik_err(res_t, res)

Forced, not chosen: under the value form `rdx` never reaches a parameter, so **no** one-argument
function can receive a Result and read its payload. `docs/api-surface.snapshot` regenerated —
**916 public fns, and this is the only delta** (`/1` → `/2`; nothing added, nothing removed).

⚖️ **Measured blast radius: zero call sites.** Six repos vendor the definition
(aegis, aethersafha, anuenue, ark, kybernet, mela) and **none of them call it**. So the change
breaks no consumer code today — but the public surface did change incompatibly, which is why this
is 1.6.0 and not 1.5.2.

### Fixed — the vendored stdlib was incoherent on disk

`lib/` held the post-flip `tagged.cyr` (no `tagged_new`) under a **6.5.35** pin, so the tree did
not build at all: 9 × "returns two values" plus undefined `payload` and `tagged_new`. Re-vendored
against 6.6.2 and verified file-by-file against the installed snapshot — 29 of 30 byte-identical.

⚠ The 30th, `lib/hashmap_fast.cyr`, is an **orphan**: nothing in `src/` or `tests/` references it,
it is not in `[deps].stdlib`, and it therefore is never refreshed by `cyrius deps`. Dated
2026-08-23. Left in place and recorded rather than silently deleted.

### Verification

- `cyrius build` OK (675,824 B) · `cyrius test` **18 files, 788 assertions, 0 failures**
- **Zero mixed-return warnings.** cyrius 6.6.0 added a diagnostic for a fn returning a pair on one
  path and a single value on another — the shape that produced **19 silent defects in yukti**
  during the 6.6.0 migration, where a propagated `Err(77)` arrived as `tag=77, is_err=0`: an error
  that reads as SUCCESS. agnostik's six Result-returning parsers were audited end-to-end and
  return a pair on every path.
- Every `boxed_*` site traced to a `tagged_new` producer; no `boxed_*` applied to a Result and no
  pair form applied to a box. A wrong classification here is silent in both directions.


## [1.5.1] - 2026-08-24

**`cgroup_limits` gains the three missing setters.** Additive; no layout
change. 223 → 233 assertions, 0 failures.

### Added — `cglim_set_memory_high` / `cglim_set_cpu_max` / `cglim_set_cpu_weight`

`cgroup_limits` has five fields and five accessors, but shipped setters for
only two of them (`memory_max`, `pids_max`). A consumer populating the
struct from config had to reach for raw `store64(c + 8, v)` — which puts
agnostik's field offsets into the consumer's source and turns any future
reordering into a silent wrong-field write. That is precisely the hazard
the setters exist to prevent, and the offsets here are not obvious: there
is an unused gap at +24 between `cpu_max` (+16) and `cpu_weight` (+32).

Surfaced by kybernet 1.5.5, which parses per-service limits from
`/etc/kybernet/config.json` into this struct and needs all five.

The new tests set all five fields to distinct values and read all five
back, so a wrong offset shows up as a collision rather than passing; they
also pin `cgroup_limits_new()` as all-zero, since 0 is the "unset" sentinel
every consumer keys on and a non-zero default would silently apply a limit
nobody configured.

### Performance

`scripts/bench-regression.sh`: 25 checked, 0 new, **0 regressions**. No
benchmark touches the new setters; movement is run-to-run noise, the
largest being `trace_context_child` +10.1% and `sandbox_config_default`
−11.8%, both well inside their thresholds.


## [1.5.0] - 2026-08-25

**`LinuxCapability` values are kernel capability numbers again, and the enum
gains name/parse.** Security fix plus new public surface — a minor, not a
patch: agnostik ships API additions in minors only (1.4.0 was the 31
`*_parse` inverses), and every patch in the 1.3.x line is fixes-only.

### Fixed — the enum disagreed with the kernel above CAP_LEASE

These are not ordinals. Consumers build masks with `1 << cap` and hand them
to `capset(2)` / `PR_CAPBSET_DROP`, so a value that disagrees with
`include/uapi/linux/capability.h` silently operates on a **different
capability than the operator named**.

Two defects, compounding:

- **`CAP_MAC_OVERRIDE` (32) and `CAP_MAC_ADMIN` (33) were absent entirely**,
  shifting every capability above 31 down by two. `CAP_SYSLOG` resolved to
  33 — the kernel's `CAP_MAC_ADMIN`. `CAP_BPF` resolved to 36
  (`CAP_BLOCK_SUSPEND`). `CAP_CHECKPOINT_RESTORE` to 37 (`CAP_AUDIT_READ`).
- **`CAP_AUDIT_READ` and `CAP_AUDIT_CONTROL` were transposed**, so both were
  wrong even below the shift.

"Drop everything except `CAP_SYSLOG`" really retained `CAP_MAC_ADMIN`.

Values are now explicit rather than implicit ordinals, so a future insertion
cannot renumber the tail again, and `CAP_LAST_CAP` is exported.

This is a **behaviour change for any consumer that stored or persisted the
old numbers**. Nothing in-tree does; the values were only ever correct by
accident below `CAP_LEASE` (28), and wrong above it.

Found from kybernet, whose 1.5.2 makes per-service capability policy real —
its privilege drop feeds these numbers straight to `capset(2)`. argonaut
defines an enum with the same name and member names, so a consumer linking
both got whichever came last; argonaut's was also wrong and is corrected at
its 1.11.0 to the same kernel values, making the duplicate definition
value-identical and harmless.

### Added

- **`capability_name(c)` / `capability_parse(s)`** — the lowercase `cap_`
  spelling `capsh(1)`, `getcap(8)` and `capability(7)` use, so a config file
  written against Linux documentation parses without translation. Exact
  match; an unrecognised name errors rather than defaulting, because the
  failure mode of a silent default here is retaining a capability the
  operator asked to drop. Completes the F-018 contract for this enum.

### Performance

Benchmark gate: **25 checked, 0 new, 0 regressions** (`scripts/bench-regression.sh`,
thresholds 50% ns-bracket / 80% us-bracket). Baseline appended to
`docs/benchmarks/history.csv`.

This release touches one enum's values and adds two pure string-comparison
functions, so no benchmark should move for a structural reason — and none
does. The whole table drifts a few percent upward, which is the single-run
bounce CLAUDE.md documents (±20–30% on load), not a change in the code
under test.

| benchmark | baseline | current | delta |
|---|---:|---:|---:|
| accelerator_device_full | 111 | 143 | +28.8% |
| message_build_3turn | 344 | 406 | +18.0% |
| sandbox_config_default | 27 | 31 | +14.8% |
| security_context_full | 628 | 714 | +13.7% |
| injection_scores_from_json | 276 | 300 | +8.7% |
| trace_context_new | 1064 | 1144 | +7.5% |
| token_usage_update | 27 | 29 | +7.4% |
| agent_stats_from_json | 259 | 278 | +7.3% |
| agent_id_to_str | 671 | 719 | +7.2% |
| resource_limits_from_json | 417 | 447 | +7.2% |
| agent_id_new | 525 | 561 | +6.9% |
| injection_scores_to_json | 732 | 780 | +6.6% |
| resource_limits_to_json | 938 | 993 | +5.9% |
| agent_stats_to_json | 650 | 685 | +5.4% |
| audit_entry_full | 2171 | 2284 | +5.2% |
| agent_id_roundtrip | 953 | 1002 | +5.1% |
| trace_context_child | 538 | 562 | +4.5% |
| token_usage_from_json | 388 | 405 | +4.4% |
| inference_request_full | 388 | 403 | +3.9% |
| accel_flags_from_json | 657 | 679 | +3.3% |
| accel_flags_to_json | 1534 | 1584 | +3.3% |
| token_usage_to_json | 836 | 861 | +3.0% |
| version_roundtrip | 263 | 271 | +3.0% |
| traceparent_format | 1761 | 1799 | +2.2% |
| version_to_str | 113 | 113 | 0.0% |

The two largest movers are the two smallest absolute figures in the set —
`accelerator_device_full` at 111 ns and `sandbox_config_default` at 27 ns —
where a few nanoseconds of jitter is a double-digit percentage. Neither is
on a path this release modified.

### Tests

`tests/tcyr/test_v150_capability_numbers.tcyr` (35 assertions): every
boundary value against the kernel table, explicit assertions on the tail
that was wrong, distinctness checks for the shift class, and a
`parse(name(v)) == v` roundtrip across all 41 members.

---

## [1.4.0] - 2026-08-24

### Added

- **F-018 — every enum carrying a `*_name()` now has a `*_parse()`
  inverse.** 31 new functions, one per enum, covering **204 members**.
  Before this release the library had 31 enum `_name` functions and
  **zero** enum `_parse`, so the CLAUDE.md contract — *"All public enums
  must have `*_name()` (string representation) and `*_parse(s)`
  (roundtrip)"* — held for no enum at all, and each of the eleven
  consumers had to hand-roll its own string→enum mapping. That is the
  exact duplication agnostik exists to prevent, and divergent hand-rolled
  mappings are how two components come to disagree about what
  `"restricted"` means.

  Design decisions worth knowing:
  - **Exact match, no coercion.** Each `*_parse` accepts precisely what
    its `*_name` emits — no case-folding, no whitespace trimming, no
    aliases. An unrecognised string returns
    `Err(STIK_ERR_INVALID_ARGUMENT)` rather than defaulting to a sentinel
    variant, so a typo in a config file surfaces instead of silently
    becoming `Unknown`.
  - **Seven members are named by their `_name` *fallback*, not an
    explicit arm** — `STIK_ERR_UNKNOWN`, `HEALTH_UNKNOWN`,
    `VENDOR_CUSTOM`, `LLM_CUSTOM`, `MEM_UNKNOWN`, `PII_CUSTOM`,
    `STATUS_UNKNOWN`. For those enums the fallback string maps back to
    that member, which makes the roundtrip **total**. The other 24 enums
    *reject* their fallback string, because no member owns it —
    `span_kind_parse("Unknown")` is an error, not `SPAN_INTERNAL`.
  - Returns `Result`, per the zero-panic rule.

- **F-021 — nine previously unsettable fields got setters.**
  `smeta_set_expires_at` / `smeta_set_owner` on `SecretMetadata`, and the
  seven original `mcap_set_supports_*` flags on `ModelCapabilities`
  (`vision`, `tools`, `structured`, `streaming`, `thinking`, `audio`,
  `citations`). All nine had getters with no setter and were never written
  after construction, so every one of them read `0` in every release up to
  and including 1.3.7.

  The `SecretMetadata` half is the security-relevant one: a consumer
  implementing rotation or expiry enforcement read `smeta_expires_at()`,
  correctly interpreted `0` as "no expiry set", and concluded the secret
  never expires — for every secret, permanently. **If you built expiry
  handling against `smeta_expires_at` before 1.4.0, it was a no-op and
  needs re-checking.** The seven `mcap_supports_*` flags are the benign
  end of the same defect class; they are fixed together so the library
  gives one consistent answer on setter-less getters rather than two.

  Additive only — no constructor signature changed.

### Testing

- New `tests/tcyr/test_v140_enum_parse.tcyr` (+481 assertions, 886 →
  **1,367** across 17 files). This is deliberately exhaustive and
  mechanically derived rather than a set of spot checks: it asserts
  `parse(name(v)) == v` for **every member of every one of the 31 enums**,
  plus a rejection case per enum, plus a fallback-string case per enum
  proving the 24-vs-7 split above. Adding an enum member without
  extending its `_name`/`_parse` fails here.

  This matters as much as the 31 functions. The reason a 31-enum,
  zero-parser gap survived eight releases is that **nothing checked it** —
  so the invariant is now pinned by a test instead of by discipline.

### Changed

- **Public API surface 871 → 911 fns** (+40: 31 `*_parse` + 9 setters),
  zero removals. `docs/api-surface.snapshot` regenerated and committed
  alongside. Additive only, hence a minor rather than a major — no
  existing signature, wire format or struct layout changed.

## [1.3.7] - 2026-08-24

### Security

- **F-014 (MEDIUM) — `_fill_random`'s error sentinel neither exited its
  loop nor stayed in bounds.** The `/dev/urandom` fallback in
  `src/types.cyr` set `off = 0 - 1` on a failed `read(2)`, intending to
  abandon the loop. Cyrius compares **signed** (verified: `(0 - 1) < 16`
  is true), so `off < n` stayed true and the loop retried with
  destination `buf + (0 - 1)` — one byte *before* the buffer — and length
  `n - (0 - 1)` = `n + 1`. Two failure modes: a successful retry wrote
  `n + 1` bytes from `buf - 1`, corrupting the preceding byte, shifting
  every random byte by one position, and then reporting **success**
  (`off` reached `n`), handing the caller a silently-misaligned
  identifier with no error; or, if reads kept failing, `off` stayed `-1`
  forever and the loop never terminated, so the deliberate fail-loud path
  (`stderr` + `exit(70)`) was unreachable and the library call hung.
  Reachable from `agent_id_new()` / `user_id_new()` / `span_id_new()`,
  but only when `getrandom(2)` does not return exactly `n` (old kernel,
  `ENOSYS`, seccomp, signal) *and* `/dev/urandom` then fails — not
  attacker-triggerable, which is why it is MEDIUM and not HIGH. The
  fallback exists for degraded environments, however, which is exactly
  where it would have run. Fixed by exiting on an explicit flag and
  reporting success off that flag rather than off the offset (`break` is
  unreliable in a Cyrius `while` carrying `var` declarations, so
  flag + `continue` is the mandated idiom).

### Fixed

- **F-015 (MEDIUM) — traceparent accepted an all-zero trace-id and an
  all-zero parent-id.** W3C Trace Context declares both invalid and
  requires implementations to ignore such a header; both parsed cleanly.
  A peer that sent `00-000…0-00f067aa0ba902b7-01` had its context
  accepted and propagated, collapsing distinct traces onto a reserved
  sentinel id across all eleven consumers, since agnostik is the shared
  implementation. Now rejected via a new private `_all_zero_bytes`
  helper. The rule is "all bytes zero", not "leading zeros" — a trace-id
  of `0…01` stays valid and is covered by a test.

- **F-016 (MEDIUM) — the traceparent version field was never
  validated.** `tctx_from_traceparent` checked the three `-` delimiters
  at offsets 2/35/52 but never inspected the 2-char version at 0–1, so
  `zz-…` parsed and version `ff` — explicitly forbidden by the spec —
  was accepted. Both are now rejected. A valid hex version above `00`
  still parses: the spec requires forward compatibility for the fields
  an implementation understands, so `01-…` must be accepted, and that
  direction is tested too.

- **F-017 (LOW) — uppercase hex was accepted throughout the
  traceparent.** The spec mandates lowercase. The shared `_hex_nibble`
  accepts `A`–`F`, which is *correct* for its other caller — JSON's
  four-hex-digit `\u` escapes, where RFC 8259 permits either case. This
  was a latent consequence of the v1.3.0 de-duplication that merged
  `_json_hex_digit` into `_hex_nibble`, coupling two callers with
  different case rules. Fixed at the W3C boundary rather than in the
  shared helper, so conforming JSON is unaffected; a test asserts JSON
  `\u` escapes stay case-insensitive to stop a future "simplification"
  from pushing the check back down.

- **F-019 (LOW) — `user_id_from_str` had no roundtrip test**, breaking
  the rule that every parse function carries one. It is a thin
  delegation to `agent_id_from_str`, so the risk was low — but the
  delegation is precisely what an accidental edit to the `user_id_*`
  alias block would break silently. Roundtrip and reject-garbage tests
  added.

- **F-020 (INFO) — `secret_metadata_new` over-allocated by 16 bytes.**
  Confirmed from the v1.3.0 deferred list: it allocated 72 B / 9 slots
  and wrote all nine, but only offsets 0/8/16/32/40/48 have accessors —
  24, 56 and 64 were zeroed and then unreachable. Shrunk to 56 B, the
  smallest size still covering the highest accessed field (`owner` at
  48), and the two now-out-of-range stores removed. Offset 24 stays
  zeroed as an interior hole; removing it would mean renumbering every
  accessor for no gain. Internal only — no caller writes these records by
  raw offset.

### Changed

- **`cyrius lint` is now fully clean, including its deferral tracker.**
  Previously 9 untracked-deferral reports across `src/types.cyr` (4) and
  `tests/tcyr/test_v107_unicode_pii.tcyr` (5) — every one a false
  positive from prose mentioning JSON's `\uXXXX` escape form, which
  cyrlint reads as an untracked `XXX` marker. Reworded to "four-hex-digit
  `\u` escape" (comments and one banner string only — no test data or
  assertion semantics touched, confirmed by diff). CI's lint gate only
  fails on `warn` lines so it was already passing, but the noise made the
  deferral tracker useless for its actual purpose. `cyrius lint` now
  reports **0 warnings and 0 untracked deferrals across all 32 files**.

### Documentation

- **Audit report** at
  [`docs/audit/2026-08-24-audit.md`](docs/audit/2026-08-24-audit.md).
  This was a full source sweep rather than a diff review: v1.3.5 and
  v1.3.6 were toolchain-only, so the last audit of the *source* was
  2026-06-01 (v1.3.0). It records six new findings, confirms two
  long-standing backlog items, and — deliberately — writes down what was
  checked and found **sound**, so the next pass does not re-derive it:
  the v1.1.1 sub-byte struct narrowing (`InjectionScores` 5 B,
  `AcceleratorFlags` 9 B — accessor widths and offsets all correct, no
  raw `store64` bypass); `_proto_varint`, whose suspicious-looking
  post-shift mask turns out to emulate a logical shift exactly and
  produce byte-identical output to canonical protobuf across the full
  i64 range including negatives; all 10 syscall sites; and the continued
  absence of `sys_system`, path traversal, stack buffers, and any
  unbounded `strchr`/`strlen` over a non-NUL-terminated `Str`.

- **Two contract gaps quantified and pinned to v1.4.0** in
  [`docs/development/roadmap.md`](docs/development/roadmap.md) rather
  than repaired here, because both need *additive public API* and that
  does not belong in a patch:
  - **F-018 (MEDIUM)** — 58 enums, 54 `*_name()` functions, and
    **zero** enum `*_parse()`. The only `_parse` in the library is the
    private `_json_parse`. So the CLAUDE.md roundtrip guarantee holds for
    no enum at all, and each of the eleven consumers must hand-roll its
    own string→enum mapping — the exact duplication agnostik exists to
    prevent.
  - **F-021 (MEDIUM)** — `SecretMetadata` has six getters and no
    setters, and the constructor zeroes `expires_at` and `owner`, so
    `smeta_expires_at()` returns `0` for every secret in every released
    version. A consumer building expiry enforcement on it concludes
    nothing ever expires. Interim guidance: do not treat it as
    authoritative.

### Testing

- New `tests/tcyr/test_v137_hardening.tcyr` (+28 assertions, 858 → 886
  across 16 files). Covers F-015/F-016/F-017 against the W3C spec
  examples, the F-019 roundtrip, and F-014's control flow through a
  faithful replica with a scripted `read` result — forcing a real
  `getrandom(2)` failure from a test is impractical. It also pins the
  signed-comparison behaviour F-014 depended on, so a future toolchain
  flipping `<` to unsigned trips the guard instead of silently making the
  removed sentinel pattern safe again.

### Performance

Benchmark gate vs the committed v1.3.6 baseline: **25 checked, 0
regressions.** The added traceparent validation is a 55-byte scan plus
two all-zero checks and is not on a benchmarked path (`traceparent_format`
exercises the formatter, not the parser). Most ops measured slightly
faster: `sandbox_config_default` 42 → 31 ns (−26.2%),
`accelerator_device_full` 118 → 101 ns (−14.4%), `agent_id_new`
609 → 525 ns (−13.8%), `agent_id_to_str` 772 → 700 ns (−9.3%),
`trace_context_child` 612 → 553 ns (−9.6%).

An earlier pre/post comparison in this pass showed a uniform +3…12% rise
across *every* benchmark, including ones the diff cannot touch. That was
host contention from a concurrent build, not a regression: untouched
control benchmarks moved as much (+7.4%) as or more than benches near
changed code (+3.2%), and the shift shrank to +2.7% when samples went
from 3 to 5 runs. Noted because this host cannot produce trustworthy
sub-10% comparisons while another project is compiling.

## [1.3.6] - 2026-08-24

### Toolchain

- **Cyrius toolchain pin: `6.5.27` → `6.5.35`.** No agnostik-side logic
  changes. The public API surface (**871 fns**) and every wire format are
  byte-for-byte unchanged, so this is a drop-in patch for every consumer.
  `lib/` re-synced from the 6.5.35 snapshot: `cyrius lib sync` resolves the
  **15** declared `[deps].stdlib` modules to **25 files** (the 15 plus 10
  platform peers — `syscalls_{macos,windows,aarch64_linux,linux_common,
  x86_64_linux,x86_64_agnos}`, `alloc_{agnos,macos,windows}`,
  `hashmap_fast`). `lib/` holds **27** because two files sit outside
  `lib sync`: `atomic.cyr`, written by `cyrius deps`, and the vendored,
  git-tracked `keccak.cyr`. All 27 byte-match the snapshot. The
  stdlib **module set is unchanged at 101 modules**: no `json`→`bayan`-style
  reorg this cycle, and `[deps].stdlib` needs no edit. Three vendored
  modules changed content:

  - **`bayan`** `215,481` → `641,083` B (431 → **799** fns, +368). 6.5.3x
    folds a complete PDF parse/encode subsystem into the bundled `bayan`
    distribution module: **361 PDF fns**, of which **209** are private
    `_pdf*` internals across eleven families (`_pdfc`, `_pdfe`, `_pdff`,
    `_pdfg`, `_pdfl`, `_pdfo`, `_pdfp`, `_pdfu`, `_pdfw`, `_pdfx`, `_pdfz`)
    and **152** are public `bayan_pdf_*` entry points. 6.5.27 has zero of
    either. agnostik calls none of it: its JSON is the hand-rolled `_json_*`
    family in `src/types.cyr`, and the only bayan symbol the build touches is
    `bayan_json_get` from the auto-resolved preamble. All 361 land in the
    binary **NOPed, not stripped**, which is the entire cause of the size
    delta below.
  - **`fmt`** `11,428` → `12,844` B — upstream `fmt_float_buf` carry fix: a
    rounded fraction that reached a full unit (`frac == 10^decimals`) was
    emitted verbatim as the fraction field instead of carrying into the
    integer part, so `3 - 1e-7` printed `2.1000000`. Not user-visible in
    agnostik, which formats no floats (`metric_float` in `src/telemetry.cyr`
    is a tagged-union constructor over an integer payload, not a formatter).
  - **`syscalls_windows`** `16,361` → `18,078` B — non-x86_64-linux path,
    unexercised by this project's gates.

- **Build warnings back to 0.** The 1.3.5 pin emitted three
  `assigning non-pointer to typed pointer` warnings from
  `lib/bayan.cyr` (`_toml_parse_multiline_q` ×2, `_toml_parse_str`);
  all three are fixed upstream in 6.5.35.

- **`cyrius audit` is usable — and already was on the previous pin.** This
  release runs it for the first time in a while, which is how the harness
  bug below was found, but **none of that is a 6.5.35 change** and the
  earlier drafts of this entry got the attribution wrong. Bisecting the
  installed toolchains against this exact tree:
  - The original missing-`check.sh` breakage ended at **6.2.24** — 6.2.23
    still exits `error: script not found: ~/.cyrius/bin/check.sh`, 6.2.24
    runs `fmt / lint / docs / tests / bench` inline. (The issue file's
    2026-07-13 update recorded this as "resolved at 6.4.62"; 6.4.62 was
    simply the next version agnostik happened to re-test.)
  - The follow-on defect — `tests`/`bench` compiling without resolving the
    manifest `[deps] stdlib` preamble, false-failing on
    `undefined function 'clock_now_ns'` / `'bayan_json_get'` — ended at
    **6.4.73**: 6.4.72 reports `10 passed, 5 failed`, 6.4.73 reports
    `15 passed, 0 failed`. So it was **already fixed in 6.5.27**, the pin
    this release bumps *from*: running `~/.cyrius/versions/6.5.27/bin/cyrius
    audit` against this tree produces the same clean sweep as 6.5.35.

  What actually surfaced the bench-harness bug was **running the gate at
  all** — v1.3.5 skipped it — not the toolchain bump.

  Still open, and the reason
  [`cyrius-audit-missing-check-script-2026-04-26`](docs/development/issues/archive/cyrius-audit-missing-check-script-2026-04-26.md)
  is not archived: **`cyrius self` still false-fails**, standalone, with the
  same two undefined symbols and `FAIL: cycc!=cycc`. Verified on 6.5.27,
  6.5.30 and 6.5.35, so it is not a 6.5.35 regression — it is the original
  preamble defect, still live on the one command that triggers it. `audit`
  does not cover it and has not since **6.2.24**, when the self-host phase
  left its phase list; `cyrius --help` read "full check: self-host, test,
  fmt, lint" from 6.0.1 through **6.2.10**, "local item suite (check.sh:
  fmt/lint/format/tests)" from 6.2.11, and "project sweep:
  fmt/lint/docs/tests/bench" since 6.2.24.

- **`cyrius audit` exits 1 on its `docs` phase — 853 undocumented public
  fns.** Unchanged by this bump (6.5.27 reports the identical 853) and
  agnostik's own gap, not a toolchain bug; recorded as a roadmap backlog
  item rather than papered over. Note on the tooling: `cyrius doc --check`
  takes a **single file** and exits with *that file's* undocumented count —
  `src/main.cyr` → 23, `src/agent.cyr` → 177, `src/lib.cyr` → 0 — so 853 is
  the sum across the 15 `src/*.cyr` files, not any one command's exit code.
  Read `audit`'s per-phase verdicts, not its exit code, and keep running
  `cyrius self` separately in the knowledge that it false-fails.

- **Binary `413,512` → `629,032` B (+215,520 B, +52.1%).** Entirely the
  NOPed `bayan` PDF subsystem above — nothing new is reachable, and
  unreachable-fn accounting moves in lockstep (1,503 fns / 291,041 B →
  1,871 fns / 445,949 B). Both figures measured on this host by building
  each pin end-to-end (its own `cycc` **and** its own stdlib snapshot), so
  the delta isolates the toolchain change. Note `CYRIUS_DCE=1` and a plain
  build now yield the **identical** byte count at both pins (413,512 at
  6.5.27, 629,032 at 6.5.35): DCE NOP-fills unreachable code in place rather
  than removing it, so it does not shrink the artifact — it only guarantees
  dead code cannot execute. This is not new and not a 6.5.x change: a probe
  binary with 3,000 unreferenced fns measures byte-identical under plain and
  `CYRIUS_DCE=1` builds on **6.0.1**, the oldest 6.x on this host, as on
  every version since. The v1.3.5 CHANGELOG's
  `354,112 -> 417,608 bytes` line does not reproduce under either
  measurement and is superseded by these figures.

### Fixed


- **CI's format gate reported drift for every file, on every run.** The
  `Format check` step tested formatting with
  `diff -q <(cyrius fmt "$f" 2>/dev/null) "$f"`, which assumes
  `cyrius fmt <file>` is a stdout filter that prints the formatted source.
  **That stopped being true at cyrius `6.5.27`**: through `6.5.26` bare
  `fmt` printed the formatted source to stdout (4,329 B for
  `src/error.cyr`), and from `6.5.27` onward it is an in-place rewriter that
  prints **nothing**. So the `diff` compared an *empty* stream against each
  file and failed unconditionally — all 31 files, regardless of their actual
  formatting. Bisected across every installed toolchain: `6.5.20`/`6.5.24`/
  `6.5.25`/`6.5.26` → stdout filter; `6.5.27`/`6.5.28`/…/`6.5.35` → in-place.

  The breakage therefore entered with the **v1.3.5** pin bump to `6.5.27`
  and lay dormant because that release skipped its gates; v1.3.6 is the
  first CI run on a `≥6.5.27` toolchain. It is not caused by the
  `6.5.27 → 6.5.35` bump, and the source files were never mis-formatted —
  `cyrius fmt --check` reports all 31 clean, as it did at the v1.3.6 cut.

  The step now uses `cyrius fmt --check "$f"`, which signals drift purely
  through its exit code (0 = canonical) and never rewrites the file, and it
  echoes cyrfmt's message so a real failure names the offending line.
  Verified both directions locally under `bash`: clean tree → `rc 0`,
  "fmt: clean (31 files)"; with a deliberately non-canonical continuation
  indent appended to `src/validation.cyr` → `rc 1`, `needs fmt:
  src/validation.cyr` plus `cyrfmt: src/validation.cyr:78: not canonically
  formatted`. Note the prior history here: the step originally read
  `diff <(cyrius fmt "$f" --check)`, and commit `f8fdcee` ("ci fmt issue")
  moved it to bare `fmt` — correctly diagnosing that `--check` emits no
  output, but landing on a form that emits none either. The fix is to stop
  diffing stdout altogether and use the exit code.

  CI-only; no library source, public API, or wire-format change.

- **`tests/bcyr/agnostik.bcyr` was missing `include "src/proto.cyr"`.**
  The same defect v1.3.4 fixed in `tests/tcyr/agnostik.tcyr`, present in the
  bench harness and missed by that sweep. The file includes
  `src/telemetry.cyr` — whose `Span_to_otlp_proto` calls the `_proto_*` wire
  helpers — but never pulled in `src/proto.cyr`, leaving 5 undefined-function
  references (`_proto_string` / `_proto_bytes` / `_proto_int64` /
  `_proto_fixed64` / `_proto_message`). Benign in effect: no benchmark calls
  `Span_to_otlp_proto`, so the references are unreachable and the harness
  linked and ran correctly. Latent since `proto.cyr` landed in v1.2.0 — and
  **not** hidden by tooling: a plain `cyrius bench tests/bcyr/agnostik.bcyr`
  prints all five warnings, and does so on 6.4.62 and 6.5.27 as well as
  6.5.35. It was overlooked in bench output for four releases, then caught
  here by `cyrius audit`'s bench phase. Nothing about 6.5.35 revealed it;
  running the gates that v1.3.5 skipped did. Include added after `types`
  mirroring
  `src/lib.cyr` / `agnostik.tcyr` order. Bench-harness warnings back to 0;
  the recorded baseline and every figure in Performance below come from the
  fixed harness.

- **Three files were not canonically formatted.** `cyrius fmt --check`
  failed on `src/main.cyr`, `tests/tcyr/agnostik.tcyr`, and
  `tests/tcyr/test_v110_serde_golden.tcyr` — multi-line call arguments were
  indented flush with the call (4 spaces) instead of the canonical 2-spaces-
  per-open-paren continuation. Pre-existing debt, **not** introduced by this
  bump: the 6.5.27 and 6.5.30 formatters reject the same three files
  identically, so it entered at or before the 1.3.5 cut and was missed
  because that release skipped its fmt gate. Canonicalized with
  `cyrius fmt`; the change is whitespace-only across 16 lines (`git diff -w`
  is empty) and all 858 assertions pass unchanged afterward.

- **The v1.3.5 release skipped two mandatory gates.** Per CLAUDE.md the
  benchmark gate is required on *every* version bump including
  toolchain-only patches, and the closeout requires a doc-sync pass; 1.3.5
  ran neither. Consequences visible in-tree: no 1.3.5 row was appended to
  `docs/benchmarks/history.csv` (so this release's baseline is the
  2026-07-13 / 1.3.4-line run, and its deltas span `6.4.62 → 6.5.27 →
  6.5.35` rather than one pin step), `cyrius.cyml` was left pinned at
  `6.5.27` against a `6.5.35` wrapper (toolchain-drift warning on every
  build), and `README.md`, `docs/development/state.md`,
  `docs/development/roadmap.md`, and `docs/doc-health.md` still described
  **1.3.4 / cyrius 6.4.62**. All of the above are reconciled in this
  release; 1.3.5 itself is left as shipped.

- **Benchmark harness silently dropped fractional-µs averages.** Both
  `scripts/bench-history.sh` and `scripts/bench-regression.sh` parsed
  bench output with an integer-only value capture (`([0-9]+)(ns|us|ms)`),
  so lines like `trace_context_new: 1.294us avg` or `traceparent_format:
  2.336us avg` failed to match — the parser consumed `1`, expected a unit,
  hit `.`, and skipped the whole line. Benches that render ≥1µs with a
  decimal were captured only on the occasional run where they rounded to a
  whole unit, so a single run parsed ~15–20 of the 25 tracked benches:
  `history.csv` release-baseline appends missed ~5–10 benches and the
  regression gate reported "20 checked" instead of 25, leaving those
  benches only intermittently regression-checked. The value capture now
  accepts an optional fractional part (`[0-9]+(\.[0-9]+)?`) and normalizes
  to ns with floating-point math (`awk`, since bash integer arithmetic
  can't compute `1.294 * 1000`), preserving the ns/us/ms unit scaling. A
  single `bench-history.sh` run now appends all 25 rows and the regression
  gate reports "25 checked". The finer precision introduces no false
  positives — the us-bracket floor (>80% **and** ≥2000ns absolute) still
  clears the largest observed mover (`audit_entry_full`, +39%/+788ns) by a
  wide margin. Tooling-only; no library source or wire-format change.

### Performance

Benchmark gate (`scripts/bench-regression.sh`): **25 checked, 0 new,
0 regressions** — no `[bench-regression-ack]` needed. Baseline is the most
recent committed row per bench in `docs/benchmarks/history.csv`. Because
v1.3.5 never appended one the baseline is split, and **no row here is a
single-pin-step delta**: 20 of 25 benches baseline to the 2026-07-13
(v1.3.4-line) run and span `6.4.62 → 6.5.27 → 6.5.35`, while the five that
the fractional-µs parser bug dropped from that run — `accel_flags_to_json`,
`audit_entry_full`, `resource_limits_to_json`, `trace_context_new`,
`traceparent_format` — fall back to the 2026-06-20 run, captured under
cyrius **6.2.11**, and span `6.2.11 → 6.5.35`. The "baseline from" column
marks which is which. Current values are **medians of 3 runs** (per CLAUDE.md,
single-run benches bounce ±20–30%); a separate single-pass gate run also
reported 0 regressions.

| benchmark | baseline (ns) | 1.3.6 median (ns) | delta | baseline from |
|---|---:|---:|---:|---|
| `accel_flags_from_json` | 683 | 675 | -1.2% | 2026-07-13 |
| `accel_flags_to_json` | 2,000 | 1,540 | -23.0% | 2026-06-20 |
| `accelerator_device_full` | 154 | 110 | -28.6% | 2026-07-13 |
| `agent_id_new` | 579 | 545 | -5.9% | 2026-07-13 |
| `agent_id_roundtrip` | 994 | 945 | -4.9% | 2026-07-13 |
| `agent_id_to_str` | 723 | 677 | -6.4% | 2026-07-13 |
| `agent_stats_from_json` | 279 | 264 | -5.4% | 2026-07-13 |
| `agent_stats_to_json` | 772 | 660 | -14.5% | 2026-07-13 |
| `audit_entry_full` | 2,000 | 2,168 | +8.4% | 2026-06-20 |
| `inference_request_full` | 684 | 378 | -44.7% | 2026-07-13 |
| `injection_scores_from_json` | 300 | 291 | -3.0% | 2026-07-13 |
| `injection_scores_to_json` | 858 | 768 | -10.5% | 2026-07-13 |
| `message_build_3turn` | 592 | 360 | -39.2% | 2026-07-13 |
| `resource_limits_from_json` | 437 | 424 | -3.0% | 2026-07-13 |
| `resource_limits_to_json` | 1,000 | 924 | -7.6% | 2026-06-20 |
| `sandbox_config_default` | 51 | 30 | -41.2% | 2026-07-13 |
| `security_context_full` | 729 | 621 | -14.8% | 2026-07-13 |
| `token_usage_from_json` | 428 | 381 | -11.0% | 2026-07-13 |
| `token_usage_to_json` | 948 | 824 | -13.1% | 2026-07-13 |
| `token_usage_update` | 41 | 27 | -34.1% | 2026-07-13 |
| `trace_context_child` | 549 | 548 | -0.2% | 2026-07-13 |
| `trace_context_new` | 1,000 | 1,080 | +8.0% | 2026-06-20 |
| `traceparent_format` | 2,000 | 1,659 | -17.1% | 2026-06-20 |
| `version_roundtrip` | 360 | 267 | -25.8% | 2026-07-13 |
| `version_to_str` | 164 | 111 | -32.3% | 2026-07-13 |

**23 of 25 ops improved.** agnostik's source is unchanged, so every
movement here is toolchain-side. Notable wins:
`inference_request_full` 684 → 378 ns (**-44.7%**), `sandbox_config_default` 51 → 30 ns (**-41.2%**), `message_build_3turn` 592 → 360 ns (**-39.2%**), `token_usage_update` 41 → 27 ns (**-34.1%**), `version_to_str` 164 → 111 ns (**-32.3%**), `accelerator_device_full` 154 → 110 ns (**-28.6%**), `version_roundtrip` 360 → 267 ns (**-25.8%**).

Above baseline: `audit_entry_full` +8.4%, `trace_context_new` +8.0% — both
well inside the 80% us-bracket threshold, and both compare against
**whole-µs-truncated** 2026-06-20 baselines, so a `2,000` ns baseline
encodes anything from 2,000–2,999 ns. The cause is the bench formatter, not
the parser fix below: pre-6.2.15 `_fmt_time` rendered the µs branch as
`major = ns / 1000` — integer division, no fraction, no rounding term — and
fractional rendering arrived in 6.2.15. Measured against a real value
anywhere in that bucket, both figures sit inside noise rather than
representing measured slowdowns.

## [1.3.5] - 2026-08-17

### Changed

- cyrius pin 6.4.62 -> **6.5.27**, matching the rest of the AGNOS desktop stack. Build 354,112 -> 417,608 bytes; 15/15 green.

## [1.3.4] - 2026-07-13

### Toolchain

- **Cyrius toolchain pin: 6.3.15 → 6.4.62.** No agnostik-side source
  changes; the public API surface and all wire formats are byte-for-byte
  unchanged. `lib/` wiped and re-synced from the 6.4.62 snapshot
  (`cyrius lib sync`). Note the 6.4.x `lib sync` default changed — it now
  copies only the declared `[deps].stdlib` subset (25 `.cyr`) rather than
  the whole snapshot; pass `--full` for the complete set. `bayan` still
  ships in the 6.4.62 stdlib (no `json`→`bayan`-style reorg this cycle).
  `dist/agnostik.cyr` re-generated byte-identical (version banner
  unchanged pending a release cut). All 858 assertions across the 15
  `.tcyr` files pass on the new stack; lint/fmt clean.

### Fixed

- **`tests/tcyr/agnostik.tcyr` was missing `include "src/proto.cyr"`.**
  The harness file includes `src/telemetry.cyr` (whose
  `Span_to_otlp_proto` calls the `_proto_*` wire helpers) but never
  pulled in `src/proto.cyr`, leaving 5 undefined-function references
  (`_proto_string`/`_proto_bytes`/`_proto_int64`/`_proto_fixed64`/
  `_proto_message`). Benign — the proto fns are unreachable in that
  compilation unit, so it linked and passed — but latent since proto.cyr
  landed in v1.2.0 and surfaced as 5 build warnings under 6.4.62's
  link diagnostics. Include added mirroring `main.cyr`/`lib.cyr` order
  (proto after types). Build/test warnings back to 0.

### Performance

Bench-regression gate (`scripts/bench-regression.sh`) vs the committed
baseline in `docs/benchmarks/history.csv`: **20 checked, 0 regressions.**
6.4.62 codegen is uniformly faster — **every** benchmarked op improved,
several by 40–53%. Headline movers confirmed as medians of 3 runs
(not one lucky sample): `token_usage_update` 88→~42ns (−52%),
`sandbox_config_default` 99→~50ns (−49%), `version_roundtrip`
666→~350ns (−47%), `accelerator_device_full` 296→~159ns (−46%),
`version_to_str` 292→~170ns (−42%), `message_build_3turn` 1000→~618ns
(−38%), `inference_request_full` 1000→~715ns (−29%). Full gate table
(baseline → current, single-run):

| benchmark | baseline | current | delta% |
|---|---|---|---|
| accel_flags_from_json | 803 | 668 | −16.8% |
| accelerator_device_full | 296 | 154 | −48.0% |
| agent_id_new | 617 | 550 | −10.9% |
| agent_id_roundtrip | 1000 | 975 | −2.5% |
| agent_id_to_str | 843 | 707 | −16.1% |
| agent_stats_from_json | 329 | 291 | −11.6% |
| agent_stats_to_json | 1000 | 792 | −20.8% |
| inference_request_full | 1000 | 699 | −30.1% |
| injection_scores_from_json | 376 | 300 | −20.2% |
| injection_scores_to_json | 1000 | 855 | −14.5% |
| message_build_3turn | 1000 | 606 | −39.4% |
| resource_limits_from_json | 503 | 437 | −13.1% |
| sandbox_config_default | 99 | 51 | −48.5% |
| security_context_full | 985 | 729 | −26.0% |
| token_usage_from_json | 496 | 401 | −19.2% |
| token_usage_to_json | 1000 | 937 | −6.3% |
| token_usage_update | 88 | 41 | −53.4% |
| trace_context_child | 634 | 569 | −10.3% |
| version_roundtrip | 666 | 348 | −47.7% |
| version_to_str | 292 | 172 | −41.1% |

DCE binary (`CYRIUS_DCE=1 cyrius build`): **350,016 B**, down ~43 KB
from 392,840 B at the last recorded 6.2.11 baseline (6.4.62's DCE
NOP-fill is leaner). `history.csv` baseline unchanged — it refreshes on
the release tag, not while work sits under `[Unreleased]`.

## [1.3.3] - 2026-07-03

### Changed

- **Namespaced the error family** to end symbol collisions when this bundle is
  co-included with sibling libs that define their own `ERR_*` (notably agnodrm):
  `ERR_*` → `STIK_ERR_*`, `err_*` → `stik_err_*`, `syserr_*` → `stik_syserr_*`.
  `dist/agnostik.cyr` regenerated. Downstream consumers must migrate their
  references (known consumer: aegis).

## [1.3.2] - 2026-06-30

Leaf step of the coordinated base-security-stack migration to cyrius
**6.3.15** (agnostik is the shared-types leaf under aegis). The public
API surface and all wire formats are byte-for-byte unchanged. All 810
assertions across the 15 test files pass on the new stack; the benchmark
regression gate is green (0 regressions).

### Changed

- **Cyrius toolchain pin: 6.2.11 → 6.3.15.** `dist/agnostik.cyr`
  regenerated at v1.3.2.

### Performance

- **`sandbox_config_new` unrolled** (`src/security.cyr`). Replaced the
  11-iteration `while` zero-loop (which zeroed all 11 u64 slots, then
  overwrote two) with 11 direct `store64`s — the two live fields with
  their values, the other nine zeroed inline. Drops the per-slot loop
  counter/compare/branch from a hot constructor: **~101ns → ~90ns**
  (−10%) on the `sandbox_config_default` bench. The 6.3.x codegen shift
  had left that bench hugging the regression-gate floor (a noisy CI
  runner spiked it to 165ns / +66%, tripping the 50%/50ns threshold);
  the unroll restores real headroom so the bench sits at/below its 99ns
  baseline. No behavior change.

## [1.3.1] - 2026-06-15

Toolchain-refresh patch on top of 1.3.0. Cyrius pin `6.0.26` →
`6.2.11`. No agnostik-side source changes; the public API surface
(871 fns) and all wire formats are byte-for-byte unchanged. The one
project-visible change is a stdlib-layout migration: 6.2.x folds the
former standalone `json` module (along with `base64`/`csv`/`toml`)
into the bundled `bayan` distribution module, so `[deps] stdlib`
swaps `json` → `bayan`. agnostik never called stdlib json directly
(its JSON parsing is the hand-rolled `_json_*` family in
`src/types.cyr`), but the 6.2.11 build references `bayan_json_get`
from the auto-resolved stdlib preamble, so `bayan` must be declared
to keep the build warning-free. `lib/` was wiped and re-synced from
the 6.2.11 snapshot (`cyrius lib sync`, 97 `.cyr` files).

### Toolchain

- **`cyrius.cyml`** `[package].cyrius` pinned `6.0.26` → `6.2.11`.
- **`cyrius.cyml`** `[deps] stdlib` `json` → `bayan` (6.2.x stdlib
  reorg; standalone `json.cyr` no longer ships). All 858 test
  assertions across 15 `.tcyr` files pass under the new pin; lint
  clean across 15 source files (0 warnings); fmt clean; api-surface
  unchanged at 871 public fns; `dist/agnostik.cyr` re-bundled for the
  version banner only.

### Performance

Bench-regression gate vs the committed v1.2.0 baseline in
`docs/benchmarks/history.csv`: **3 consistent regressions** on an
unloaded runner (medians of 6 local runs), rising to **6 under CI
load** as more sub-µs/single-bucket ops cross threshold — single-run
benches bounce ±20–30% on a loaded github-runner, exactly the jitter
the gate header warns about. The 6.2.11 codegen is a strongly
net-positive trade-off — the multi-µs JSON-decode hot paths collapsed
by 67–87%, while a handful of sub-300ns construction/format ops gained
~100ns each. All regressions are toolchain codegen, not source changes
(source is unchanged), and are **ack'd with `[bench-regression-ack]`**
in the release commit (a whole-run gate skip) — there is nothing to
optimize agnostik-side and the net is overwhelmingly favorable.

Wins (the ops that matter for wire decode at scale):

| benchmark                    | baseline (ns) | current (ns) | delta% |
|------------------------------|--------------:|-------------:|-------:|
| accel_flags_from_json        |          6000 |         ~810 | −86.5% |
| resource_limits_from_json    |          3000 |         ~500 | −83.5% |
| injection_scores_from_json   |          2000 |         ~360 | −82.4% |
| token_usage_from_json        |          2000 |         ~485 | −75.8% |
| agent_stats_from_json        |          1000 |         ~330 | −67.6% |

Regressions (all ack'd — small constructor/format ops, ~100ns
absolute). The first three fire consistently (real codegen drift);
the last three fire only under CI load (sub-µs ops below the gate's
50ns/2µs absolute floors on an unloaded runner, inflated past
threshold by runner contention):

| benchmark                    | baseline (ns) | unloaded | under load | nature |
|------------------------------|--------------:|---------:|-----------:|--------|
| version_roundtrip            |           371 |     ~668 |       ~939 | codegen |
| accelerator_device_full      |           177 |     ~284 |       ~397 | codegen |
| version_to_str               |           191 |     ~296 |       ~374 | codegen |
| traceparent_format           |          1000 |    ~2000 |       3000 | load jitter (µs rounding) |
| sandbox_config_default       |            61 |     ~100 |        145 | load jitter (sub-µs) |
| token_usage_update           |            42 |      ~87 |        120 | load jitter (sub-µs) |

DCE binary `311,264 B` → `392,840 B` (+81 KB). Two compounding causes:
(1) 6.2.11's DCE **NOPs** unreachable functions in place rather than
stripping them (`257,630 bytes NOPed`, vs eliminated under 6.0.x); and
(2) the `bayan` bundle (base64+json+csv+toml) adds ~119 KB of
now-NOPed dead code that the leaner standalone `json.cyr` did not.
None of it is reachable — all 858 tests pass and the type vocabulary
consumers depend on is unchanged.

## [1.3.0] - 2026-06-01

Minor release on top of 1.2.3: a toolchain refresh paired with a
refactoring / optimization closeout review. The cyrius pin advances
within 6.x; the public API and all wire formats are unchanged (no
breaking changes). The review applied four internal improvements
(OTLP encode hot path, audit hot path, a hex-decode consolidation,
and a buffer-safety hardening) plus one new regression test. The
type vocabulary consumers depend on is byte-for-byte compatible.

### Toolchain

- **`cyrius.cyml`** `[package].cyrius` pinned `6.0.14` → `6.0.26`. All
  858 test assertions across 15 `.tcyr` files pass under the new pin;
  lint clean across 15 source files (0 warnings); fmt clean; `cyrius
  vet` clean (24 deps, 0 untrusted, 0 missing); `dist/agnostik.cyr`
  re-bundled.

### Changed

- **`src/proto.cyr`** — `_proto_string`, `_proto_bytes`, and
  `_proto_message` replaced their per-byte `str_builder_putc` copy
  loops (one `_sb_grow` bounds-check per byte) with a single
  `str_builder_add` (one grow + one `memcpy`). This is the OTLP encode
  hot path — every attribute string, trace_id/span_id, and nested
  message on the wire. Byte-exact OTLP output unchanged (66 byte-exact
  assertions in `test_v120_otlp.tcyr` still pass).
- **`src/audit.cyr`** — the 64-char `GENESIS_HASH` Str is now built
  once and cached (`_genesis_hash_cached`) instead of being re-wrapped
  via `str_from` on every `audit_entry_new` and every
  `integrity_is_genesis`. Removes a per-audit-event `strlen` + 16-byte
  alloc on the per-event hot path.

### Refactored

- **Hex-nibble decoding consolidated.** `src/telemetry.cyr`'s
  `_hex_nibble` and `src/types.cyr`'s byte-identical `_json_hex_digit`
  were merged into a single `_hex_nibble` (in `types.cyr`), and the
  five open-coded `if (c>=48 && c<=57) ... elif ...` hex ladders in
  `agent_id_from_str`, `trace_id_from_str` (×2 nibbles),
  `span_id_from_str`, and the JSON `\uXXXX` decoder now all route
  through it. One security-sensitive parse, one copy. No behavior
  change — fuzz survival (`test_v112_fuzz.tcyr`) and all roundtrips
  hold.

### Security

- **F-013 (LOW) — `version_from_str` unbounded separator scan.** The
  prerelease (`-`) / build (`+`) separator was located with `strchr`,
  which walks to a NUL terminator with no `slen` bound. On a `Str`
  that is a slice of a larger buffer (`str_new` over heap, no NUL at
  `slen`), this over-read past the logical end and could pick up a
  `-`/`+` from adjacent memory, corrupting the patch-segment length.
  Replaced with a single bounded forward scan over `[dot2+1, slen)`
  (which also subsumes a redundant double-scan). The fuzz harness
  never caught it — `fuzz_random_bytes` always NUL-terminates at
  `buf+len`, masking the over-read. Findings in
  [`docs/audit/2026-06-01-audit.md`](docs/audit/2026-06-01-audit.md).

### Tests

- **`tests/tcyr/test_v130_slice_safety.tcyr`** (+7 assertions) — F-013
  regression: constructs genuine non-NUL-terminated version slices over
  a larger buffer (all-digit filler + a stray `-` past `slen`) to
  exercise the bounded scan directly, where the fuzz harness could not.
  Test count `851 → 858`.

### Performance

- Bench-regression gate clean: **25/25 checked, 0 regressions** against
  the committed baseline (`docs/benchmarks/history.csv`, v1.2.0 commit
  `8479082`). The `_from_json` codegen wins from the 6.x line held
  (`accel_flags_from_json` 6000ns→937ns, `injection_scores_from_json`
  2000ns→342ns, `resource_limits_from_json` 3000ns→620ns,
  `token_usage_from_json` 2000ns→456ns vs the v1.2.0 baseline). Single-
  run bench jitter bounces several ops ±20–40% within the noise floor;
  no op crossed threshold. The proto.cyr OTLP-encode win is structural
  (per-byte grow-check → memcpy) and not covered by a dedicated bench.

### Stats

- DCE binary `313,344 B` → `311,264 B` (−2,080 B / ~−2 KB) from the
  removed per-byte copy loops and collapsed hex ladders.
- Test count `851` → `858` (+7 from `test_v130_slice_safety.tcyr`).
- Public API unchanged; no breaking changes.

## [1.2.3] - 2026-05-28

Major-toolchain-refresh patch on top of 1.2.2. No agnostik-side
source changes; the cyrius pin crosses the 5.x → 6.x boundary. The
only project-visible change is the stdlib resolution workflow (see
Toolchain) — the type vocabulary, wire formats, and public API are
byte-for-byte unchanged.

### Toolchain

- **`cyrius.cyml`** `[package].cyrius` pinned `5.10.44` → `6.0.14`
  (first 6.x pin). All 851 test assertions across 14 `.tcyr` files
  pass under the new pin; lint clean across 15 source files (0
  warnings); fmt clean (no drift); `cyrius vet` clean (24 deps, 0
  untrusted, 0 missing); `dist/agnostik.cyr` re-bundled for the
  version banner only.
- **Stdlib resolution moved `cyrius deps` → `cyrius lib sync`.** Under
  5.x, `cyrius deps` copied the version-pinned stdlib snapshot into
  `./lib/`. Under 6.0.x that job belongs to the new `cyrius lib sync`
  command; `cyrius deps` now resolves only `[deps.NAME]` git deps and
  treats the `[deps] stdlib` array as a presence-check against `./lib/`.
  A fresh checkout must run `cyrius lib sync` before `cyrius deps`.
  `cyrius build`/`test`/`bench` still resolve stdlib directly from the
  snapshot, so they work with no `./lib/` at all; the sync exists for
  the `deps` presence-check, lock-hash reproducibility, and offline
  builds. The `[deps] stdlib` list itself is retained unchanged —
  `cyrius distlib` confirms consumers still rely on it to supply stdlib
  to the bundle.

### CI / Docs

- **`ci.yml` / `release.yml`** gained a `cyrius lib sync` step before
  `cyrius deps` so fresh runners populate `./lib/` under 6.0.x.
- **`CLAUDE.md`** Quick Start updated: `cyrius lib sync` now precedes
  `cyrius deps`.

### Performance

- Bench-regression gate clean: 25/25 checked, **0 regressions**
  against the most recent committed baseline (`docs/benchmarks/history.csv`,
  v1.2.0 commit `8479082`). The v1.2.1/1.2.2 hot-path codegen wins
  held through the major-version pin (`resource_limits_from_json`
  457ns, `token_usage_from_json` 403ns, `accel_flags_from_json` 699ns,
  `injection_scores_from_json` 291ns — all still well below the v1.2.0
  baselines).
- `accelerator_device_full` continued to drift down: `155ns → 133ns`
  vs the v1.2.2 figure.

### Stats

- DCE binary `~305 KB` → `~306 KB` (313,344 bytes; +1 KB nominal
  codegen drift across the major-version boundary).
- Test count unchanged: 851 → 851 (toolchain refresh, no test
  additions).
- Public API unchanged: 871 → 871 fns; no breaking changes.

### Deferred

- **Cross-consumer build sweep automation**: re-pinned v1.2.3 →
  v1.2.4. Re-pinned across every patch since v1.2.0; still the next
  ecosystem pin once a slot opens.
- **Span repeated-field encoders + LogRecord/MetricDataPoint**:
  re-pinned v1.2.4 → v1.2.5. Trigger unchanged: a consumer (likely
  `stiva`) surfaces the need.

## [1.2.2] - 2026-05-11

Toolchain-refresh patch on top of 1.2.1. No agnostik-side source
changes; pure upstream codegen pin bump.

### Toolchain

- **`cyrius.cyml`** `[package].cyrius` pinned `5.10.34` → `5.10.44`
  (10 upstream slots picked up). All 851 test assertions across 14
  `.tcyr` files pass under the new pin; lint clean across 15 source
  files (0 warnings); `dist/agnostik.cyr` re-bundled for the version
  banner only.

### Performance

- Bench-regression gate clean: 25/25 checked, **0 regressions**
  against the most recent committed baseline (`docs/benchmarks/history.csv`,
  v1.2.0 commit `8479082`). The v1.2.1 hot-path codegen wins held
  through the new pin (`resource_limits_from_json` 493ns,
  `token_usage_from_json` 476ns, `accel_flags_from_json` 754ns —
  all still well below the v1.2.0 baselines).
- One notable additional improvement vs the v1.2.1 numbers:
  `accelerator_device_full` `177ns → 155ns` (**−12.4%**) — this was
  the bench that needed a `[bench-regression-ack]` at v1.2.1 cut
  due to CI jitter; the new pin pulls it cleanly clear of the noise
  band.

### Stats

- DCE binary `~304 KB` → `~305 KB` (+1 KB nominal codegen drift).
- Test count unchanged: 851 → 851 (toolchain refresh, no test
  additions).
- Public API unchanged: 871 → 871 fns; no breaking changes.

### Deferred

- **Cross-consumer build sweep automation**: re-pinned v1.2.2 →
  v1.2.3. Originally bundled with v1.2.0 (OTLP took the slot), then
  re-pinned across v1.2.1 (toolchain) and v1.2.2 (toolchain again).
  Still the next ecosystem pin once a slot opens.
- **Span repeated-field encoders + LogRecord/MetricDataPoint**:
  re-pinned v1.2.3 → v1.2.4. Trigger unchanged: a consumer (likely
  `stiva`) surfaces the need.

## [1.2.1] - 2026-05-10

Toolchain-refresh patch on top of 1.2.0. No agnostik-side source
changes; the gain is entirely upstream codegen.

### Toolchain

- **`cyrius.cyml`** `[package].cyrius` pinned `5.10.20` → `5.10.34`
  (14 upstream slots picked up). All 851 test assertions across 14
  `.tcyr` files pass under the new pin; lint clean across 15 source
  files; `cyrius_api_surface` snapshot matches (871 agnostik public
  fns, no drift); `dist/agnostik.cyr` re-bundled for the version
  banner only.

### Performance

- Bench-regression gate clean: 25/25 checked, **0 regressions**.
  Several hot-path serde reads improved materially under the new
  codegen — top movers (5.10.20 baseline → 5.10.34):
  - `resource_limits_from_json` 3000ns → 476ns (**−84.1%**)
  - `token_usage_from_json` 2000ns → 422ns (**−78.9%**)
  - `agent_stats_from_json` ~ → 297ns (gate at 80% bracket)
  - `version_to_str` 191ns → 166ns (−13.1%)
  - `trace_context_child` 583ns → 548ns (−6.0%)
  No agnostik-side perf work; entirely upstream codegen.

### Stats

- DCE binary `~311 KB` → `~304 KB` (−7 KB nominal codegen shrink).
- Test count unchanged: 851 → 851 (toolchain refresh, no test
  additions).
- Public API unchanged: 871 → 871 fns; no breaking changes.

### Deferred

- **Cross-consumer build sweep automation**: re-pinned v1.2.1 →
  v1.2.2. Originally bundled in v1.2.0 (OTLP took the slot), then
  re-pinned to v1.2.1 (this slot took the toolchain refresh). Still
  the next ecosystem pin once a slot opens.
- **Span repeated-field encoders + LogRecord/MetricDataPoint**:
  re-pinned v1.2.2 → v1.2.3. Trigger unchanged: a consumer (likely
  `stiva`) surfaces the need.

## [1.2.0] - 2026-05-10

First v1.2.x minor — OTLP wire-format primitives + toolchain refresh.

### Toolchain

- **`cyrius.cyml`** `[package].cyrius` pinned `5.10.14` → `5.10.20`.
  Picks up six upstream slots:
  - 5.10.15 — shadow-lib compile note (held from v5.10.10)
  - 5.10.16 — SIMD cross-arch close + api-surface brace-desync fix
  - 5.10.17 — keystone SIMD primitives for hisab
  - 5.10.18 — hotfix: lib/process.cyr O_WRONLY
  - 5.10.19 — `cyrius deps` transitive include resolution
  - 5.10.20 — P(-1) project-hardening sweep
  - DCE binary nominal codegen drift (~307 KB → ~311 KB).

### Added

- **`src/proto.cyr`** — minimal protobuf wire-format helpers
  (~120 LoC). Foundation for OTLP encoders. Not a general-purpose
  protobuf library; just what OTLP types need: `_proto_varint`,
  `_proto_tag`, `_proto_int64`, `_proto_fixed64`, `_proto_string`,
  `_proto_bytes`, `_proto_message`. Wire-type constants exposed for
  callers building custom messages.
- **`Span_to_otlp_proto(ptr, sb)`** in `src/telemetry.cyr` — maps
  agnostik's `Span` struct to `opentelemetry.proto.trace.v1.Span`
  on the wire. Covers all scalar fields:
  - `trace_id` (bytes, 16) · `span_id` (bytes, 8)
  - `parent_span_id` (bytes, 8 — only if non-zero)
  - `name` (string)
  - `kind` (varint, with SpanKind enum re-mapping; default
    `INTERNAL` elided per canonical OTLP encoders)
  - `start_time_unix_nano` (fixed64) · `end_time_unix_nano`
    (fixed64; computed as `start + duration_ms × 1e6`)
  - `dropped_attributes_count` / `dropped_events_count` /
    `dropped_links_count` (varint; emitted only if non-zero)
  - `status` (sub-message; emitted only if non-`UNSET`)
- **66 byte-exact assertions** in `tests/tcyr/test_v120_otlp.tcyr`
  covering both the primitives (varint, tag, fixed64 LE,
  length-delimited string) and the Span encoder under
  representative shapes (minimal / kind+status / parent / dropped
  counts). Reference bytes hand-computed from the OTLP proto
  definitions and the canonical wire spec.

### Deferred

- **Span repeated-field encoders** (attributes / events / links):
  field 9 KeyValue, field 11 Event, field 13 Link. Nested
  sub-message encoders are a substantial follow-on; pinned for
  v1.2.2 when a consumer surfaces the pin.
- **`LogRecord_to_otlp_proto`** + **`MetricDataPoint_to_otlp_proto`**:
  pinned for v1.2.2 alongside the Span repeated fields.
- **Cross-consumer build sweep automation**: re-pinned from v1.2.0
  to v1.2.1 (OTLP took the v1.2.0 slot).

Test count 785 → 851 (+66); api-surface 870 → 871 fns (intentional
addition: `Span_to_otlp_proto`); `CYRIUS_TYPE_CHECK=1` clean;
bench-regression no regressions.

## [1.1.2] - 2026-05-10

### Added (Hardening)

- **Fuzz harness** for the 8 parser entry points handling untrusted
  input: `_json_int`, `_json_str`, `_json_find_value`,
  `version_from_str`, `agent_id_from_str`, `trace_id_from_str`,
  `span_id_from_str`, `tctx_from_traceparent`. Per the v1.1.2
  roadmap pin (deferred from v1.1.0).
  - **Survival contract** — each parser must accept any byte
    sequence and return without crashing, OOB-reading, or
    OOB-writing. Doesn't assert correctness for random inputs;
    only that the parser doesn't fault.
  - **Per-parser**: 200 deterministic fuzz iterations + audit-
    finding regression seeds (F-002 escape, F-003 negatives + i64
    overflow, F-004 string-boundary, F-005 segment validation,
    F-008 19-digit boundary, F-009 truncated null, F-010 RFC 7159
    whitespace, v1.0.7 `\uXXXX` malformed-escape paths). ~1680
    calls per run total.
  - **Deterministic xorshift64 PRNG** seeded with a memorable fixed
    constant per parser (CAFEBABE, DEADBEEF, etc.) — any failure
    reproduces from the seed; no wall-clock or CSPRNG seeding.
  - **Future-proofing**: when a fuzz seed surfaces a real bug, the
    workflow is to add the offending input as an audit-finding
    seed *before* fixing the parser, so the regression is locked
    in forever (same pattern as F-NNN regression rows in the
    audit test files).
  - File: `tests/tcyr/test_v112_fuzz.tcyr` (~290 LoC). Test count
    777 → 785 (+8 survival counters); ~milliseconds wall-clock.

## [1.1.1] - 2026-05-10

### Optimization

- **Sub-byte field widths** for the two highest-payoff structs
  (per v1.1.1 roadmap pin, deferred from v1.1.0):
  - **`InjectionScores`** — 5 fields `i64` → `i8`. Per-instance
    heap 40 B → 5 B (87.5% reduction). Range 0..100 fits in i7.
  - **`AcceleratorFlags`** — 9 fields `i64` → `i8`. Per-instance
    heap 72 B → 9 B (87.5% reduction). Each field is a 0/1 boolean.
  - Wire format unchanged — cyrius derive emits the same
    `{"k":N,"k":N}` shape regardless of underlying width.
  - 5 new explicit `iscore_set_*` setters added (InjectionScores
    didn't have them pre-v1.1.1; only direct memory access).
  - 42 new test assertions in `tests/tcyr/test_v111_subbyte_widths.tcyr`
    exercising setters, field-independence, i8 boundaries, and
    derive roundtrip at i8 widths.

### Removed (breaking)

- **Direct memory writes via `store64(is + N, v)` no longer safe**
  for `InjectionScores` instances — the alloc shrank from 40 B to
  5 B, so an 8-byte store at offset 8/16/24/32 OOB-writes adjacent
  heap bytes. Callers must use the new `iscore_set_*` setters
  (5 fns: sql, xss, command, path_traversal, prompt_injection).
  AcceleratorFlags's existing `aflags_set_*` setters preserve the
  same signature; only the storage width shrank, transparently
  to callers using the setters.

### Issues

- **Filed**: `docs/development/issues/archive/cyrius-derive-comments-in-struct-body-2026-05-10.md`
  — cyrius 5.10.14's `#derive(Serialize)` codegen treats `#`
  inside the struct body as a field-name disruptor, producing
  silent-garbage JSON output. Workaround: keep all comments
  ABOVE the `#derive(Serialize)` directive, never inside the
  struct's `{ ... }` block. Applied to InjectionScores and
  AcceleratorFlags definitions.

## [1.1.0] - 2026-05-10

First minor release on top of the 1.0.x line. Modernization + features
bundle.

### Modernization

- **Revived `#derive(Serialize)`** for the 7 trivial all-int structs:
  `ResourceLimits`, `ResourceUsage`, `AgentStats`, `EdgeResourceOverrides`,
  `InjectionScores`, `TokenUsage`, `AcceleratorFlags`. Cyrius 5.10.14
  emits byte-equivalent (modulo compact format) JSON output and
  roundtrip parses correctly through `<Struct>_from_json_str`.
  **14 hand-written serde fns deleted (~280 LoC).** Each derive
  struct gained explicit `: i64` field type annotations so the
  prim-int codegen path emits correctly.
- **`AgentInfo` + `TelemetryConfig` retain hand-written impls** —
  custom serialization shape (UUID stringification, enum-name lookup,
  null-Str fallback) that cyrius 5.10.14 derive can't replicate. See
  [`docs/adr/002-derive-serialize-7-of-9.md`](docs/adr/002-derive-serialize-7-of-9.md)
  for the upstream gap analysis. Both rewritten to compact byte
  format so library output is uniform across all 9 structs.
- **ADR-001 superseded by ADR-002.** ADR-001's "replace all 9"
  decision proved infeasible against actual cyrius 5.10.14
  capabilities; ADR-002 documents the discovered gaps and the
  7-of-9 split.

### Added

- **5 new `LlmProvider` variants** for the 2026-Q1/Q2 ecosystem
  wave. Appended after `LLM_CUSTOM` to preserve wire-format tag
  values 0..12 for existing consumers: `LLM_TOGETHER`,
  `LLM_FIREWORKS`, `LLM_BEDROCK`, `LLM_VERTEX`, `LLM_COHERE`.
- **3 new `ModelCapabilities` flags** (`mcap_supports_video_input`,
  `mcap_supports_caching`, `mcap_supports_parallel_tool_calls`)
  with matching setters. Struct grew 96 → 120 bytes; existing
  offsets 0..88 unchanged.
- **`tests/tcyr/test_v110_serde_golden.tcyr`** — byte-exact golden
  corpus (34 assertions) locking in the v1.1.0 compact JSON format
  across all 9 struct serializers + tests for the new LLM additions.

### Changed

- **JSON serialization byte format** is now compact
  (`{"k":v,"k":v}`, no spaces) for all 9 struct serializers. Prior
  format was `{"k": v, "k": v}` (spaces). RFC 7159 makes inter-token
  whitespace optional; consumer parsers (incl. agnostik's own
  `_json_*` family) handle both forms unchanged.

### Removed (breaking)

- **`<Struct>_from_json(s: Str)` helpers removed** for the 7 derive
  structs. Cyrius derive emits `<Struct>_from_json_str(s: cstr)`
  natively; consumers should call `<Struct>_from_json_str(str_data(j))`
  directly. Affected: `ResourceLimits`, `ResourceUsage`, `AgentStats`,
  `EdgeResourceOverrides`, `InjectionScores`, `TokenUsage`,
  `AcceleratorFlags`. `AgentInfo_from_json` and
  `TelemetryConfig_from_json` retain the original `(s: Str)`
  signature (hand-written).

### Roadmap

- Sub-byte field widths **deferred to v1.1.1** (originally bundled
  with v1.1.0's derive revival; landed on a narrower scope than
  ADR-001 envisioned, so a separate slot is appropriate).
- Fuzz harness for parsers **deferred to v1.1.2** — keeps v1.1.0
  focused on the derive transition.

735/735 tests pass (was 701; +34 from the new golden corpus);
`CYRIUS_TYPE_CHECK=1` clean; api-surface gate matches (859 → 865
fns, intentional); bench-regression no regressions.

## [1.0.8] - 2026-05-10

### Security

- **2026-05-10 audit pass.** First audit since the pre-1.0.0
  pass at 2026-04-26 — covers the cumulative 1.0.1 through 1.0.7
  diff (7 patch releases). Findings recorded in
  [`docs/audit/2026-05-10-audit.md`](docs/audit/2026-05-10-audit.md):
  - **F-012** (INFO): `_fill_random` fatal-message stderr write
    off-by-one. The literal at `src/types.cyr:27` is 68 bytes; the
    syscall passed 67 — trailing `\n` silently dropped. Cosmetic;
    no memory-safety issue, no info leak. Fixed in this slot
    via `strlen()`-based length computation so the byte count
    tracks the literal automatically.
  - All previously-closed findings (F-001..F-011) re-verified
    closed across the 1.0.x line.
  - The v1.0.7 `\uXXXX` decoder was the only meaningful new
    parser surface; verified clean across input validation, buffer
    safety, syscall review, and pointer validation.

  **Established cadence:** audit at every minor cut + on demand
  for CVE/0-day surfaces. Next pinned audit: v1.1.0.

## [1.0.7] - 2026-05-10

### Added

- **JSON `\uXXXX` Unicode escape decoder** in `src/types.cyr:_json_str`
  closes the F-002/F-004 follow-up note ("`\uXXXX` is left as a known
  limitation"). Consumers carrying non-ASCII text through serde now
  get correct UTF-8 output instead of the literal 6-byte passthrough.
  - BMP codepoints (`A` → `A`, `é` → `é`, `中` → `中`)
    encode to 1/2/3 UTF-8 bytes per RFC 3629.
  - Surrogate pairs (`😀` → `😀`) combine to U+10000+ and
    encode to 4 UTF-8 bytes.
  - Malformed escapes (truncated, invalid hex, orphan high or low
    surrogate, high+non-low) emit U+FFFD REPLACEMENT CHARACTER —
    common JSON-parser convention; never crashes.
  - New private helpers in `types.cyr`: `_json_hex_digit`,
    `_json_parse_u4`, `_utf8_encode`. All bounds-checked (same
    discipline as F-009).
  - 38 new test assertions in `tests/tcyr/test_v107_unicode_pii.tcyr`.

- **3 new `PiiKind` variants** for emerging regulatory-attention
  categories. Appended after `PII_CUSTOM` to preserve wire-format
  tag values for existing consumers (tags 0..15 unchanged; new
  variants take 16..18; `PII_CUSTOM` stays at tag 15 as the
  fall-through catch-all).
  - `PII_GENETIC` — GDPR Art 9 §1 "genetic data"; CCPA SPI category.
  - `PII_BIOMETRIC_TEMPLATE` — algorithmic representation, distinct
    from raw `PII_BIOMETRIC_DATA`. Different threat model:
    templates are typically irreversible by design.
  - `PII_PRECISE_GEOLOCATION` — CCPA / CO / CT / VA precise-location
    category (typically <1850 ft / radius-based definition).
  - `pii_kind_name` extended with explicit cases returning the
    PascalCase form ("Genetic", "BiometricTemplate",
    "PreciseGeolocation").
  - 10 new test assertions in `tests/tcyr/test_v107_unicode_pii.tcyr`.

Test counts: 701 total (was 653; +48 from the v1.0.7 features file).

## [1.0.6] - 2026-05-09

### Added

- **Bench-regression CI gate** (`scripts/bench-regression.sh`).
  Runs benches, compares each per-op average against the most
  recent committed baseline in `docs/benchmarks/history.csv`,
  fails on slowdown beyond the threshold:
  - **50%** on ns-bracket (sub-µs ops; ns-precision allows tighter
    detection)
  - **80%** on us-bracket (cyrius rounds to whole µs and CI runners
    have ±20-30% jitter; threshold accounts for both)
  Intentional perf trade-offs ack'd via `[bench-regression-ack]`
  in the HEAD commit message. Baseline updates ride
  `history.csv` on each release tag (via the existing
  `scripts/bench-history.sh`).
- **Compile-time profile pass** documented at
  `docs/development/compile-profile-2026-05-09.md`. Per-phase
  distribution snapshot under `CYRIUS_PROF=1`: lex 68%, fixup 18%,
  gvar 10%, others <1%. Slot closes with no agnostik-side action —
  the >30% phase is upstream-bound (lex performance is a cyrius
  toolchain concern; agnostik can't shrink its public surface
  without losing functionality). Future compile-time wins ride
  cyrius slot bumps, not agnostik refactors.

### Fixed

- **`scripts/bench-history.sh` parser** was capturing only
  ns-bracket benches via `([0-9]+)ns`; the 12 us-bracket entries
  (and any future ms-bracket) silently dropped from
  `history.csv`. Now matches `(ns|us|ms) avg` and normalizes to
  ns. Lossy by construction (cyrius rounds; 1µs = 1000ns ±
  sub-µs jitter) but no longer drops rows.
- **API surface snapshot was not portable** between build
  environments. The 1.0.5 snapshot included 146 stdlib
  platform-specific peer fns (`alloc_macos`, `alloc_windows`,
  `syscalls_aarch64_linux`, `syscalls_x86_64_linux`) that
  `cyrius_api_surface` picks up locally whenever the install path
  holds those peers. CI on Linux x86_64 only resolves what the
  dispatcher needs, so the gate fired as
  `BREAKING: 146 removed`. Filter the snapshot to **agnostik
  namespaces only** (859 fns: `agent`, `audit`, `classification`,
  `config`, `error`, `hardware`, `lib`, `llm`, `main`, `secrets`,
  `security`, `telemetry`, `types`, `validation`) — that's the
  actual API contract; stdlib churn is tracked by the toolchain
  pin, not by the API gate.
- **API surface snapshot ordering** depended on host locale.
  Arch defaults to dictionary order (case-folded `Aa`); ubuntu CI
  defaults to byte order (uppercase first). Same content,
  different line order — gate fired "BREAKING" on every CI run.
  Pin `LC_ALL=C sort` in both `update` and `check` paths for
  portable byte-order ordering.

### Changed

- **`scripts/api-surface.sh`** wrapper for `cyrius_api_surface`
  added as part of these fixes — `check` (CI gate: generate live
  → filter to agnostik → diff) and `update` (regenerate the
  committed snapshot the same way). Replaces the direct
  `cyrius_api_surface` call in CI; the agnostik module list lives
  in the script as the single source of truth (any new
  `src/<name>.cyr` adds `<name>` to the regex).

## [1.0.5] - 2026-05-09

### Added

- **API surface snapshot gate** in CI (`docs/api-surface.snapshot`,
  1317 public fns at the 1.0.5 baseline). The new
  `Verify API surface snapshot` step runs `cyrius_api_surface`,
  which diffs the live surface against the committed snapshot and
  fails on unexplained additions or removals. Catches the failure
  mode where a public fn gets renamed / removed between releases
  and downstream consumers only find out on rebuild. Workflow for
  intentional API bumps: `cyrius_api_surface --update` then commit
  the regenerated snapshot alongside the API change in the same PR.

### Changed

- **F-005 + F-010 audit-regression files adopt `lib/test.cyr`'s
  `test_each` helper.** Two clearly-homogeneous case clusters
  collapse to one driver fn + one case-builder per cluster:
  - **F-010** (`test_audit_5712.tcyr`): 4 single-purpose
    `test_f010_*` fns → one `test_f010_whitespace_table` of 5 rows
    through `_check_f010`. Adding a new whitespace edge case is
    now one `vec_push(_f010_case(...))` line.
  - **F-005** (`test_audit_2026_04_26.tcyr`): 3 of 4 fns were
    `is_ok(version_from_str(...))` accept/reject — collapsed to a
    4-row table through `_check_f005`. The fourth case's detailed
    `version_major/minor/patch` extraction moved to its own
    `test_f005_canonical_parse` (heterogeneous shape).

  Heterogeneous clusters (F-001 / F-002 / F-003 / F-004 / F-008 /
  F-009) stay as direct test fns; their assertions don't fit a
  homogeneous case-row shape. Test counts unchanged (30 in 04_26;
  10 in 5712).

## [1.0.4] - 2026-05-09

### Added

- **`docs/adr/` directory + ADR-001** documenting the v1.1.0
  `#derive(Serialize)` revival decision. ADR-001 reviews the F-011
  history (pre-1.0 audit dropped 9 derive markers because the old
  cyrius compiler emitted dead-code stubs that shadowed hand-written
  adapters), summarizes the cyrius codegen maturation arc closing
  every consumer-pain shape (v5.9.30 typed-i64; v5.9.36 narrow-int;
  v5.9.39 Mach-O ARM64 ASLR; v5.10.7 Str-field positional init;
  v5.10.8 JSON escape; v5.10.14 multi-stack derive directives),
  considers three options (status quo / replace outright / hybrid),
  picks **Option B** gated by a four-step verification plan (golden
  corpus, boundary cases, roundtrip, consumer sweep), and explicitly
  documents what the choice forecloses (custom per-struct JSON
  shapes; per-field default-value control).
- **`docs/adr/README.md`** establishing the convention — when to
  write an ADR, monotonic numbering rule, template, and an index
  pointing at ADR-001.
- **`docs/proposals/`** scaffold (per the v1.0.4 working agreement
  in `docs/development/roadmap.md` — new items draft as proposals
  before earning a roadmap slot).

### Changed

- **Pointer-to-struct dot syntax** adopted in 8 parsers across
  `src/types.cyr` (agent_id_from_str, _json_find_value, _json_int,
  _json_str, version_from_str) and `src/telemetry.cyr`
  (trace_id_from_str, span_id_from_str, tctx_from_traceparent).
  Pattern: `var data = str_data(s); var slen = str_len(s);` →
  `var data = s.data; var slen = s.len;`. Pure substitution; zero
  behavior change. Cyrius v5.8.17 (dot syntax on `: Str` params)
  + v5.10.4 (type inference through `var x = f(...)`) make the
  shape clean for both annotated params and inferred Str locals.
  Not wholesale — selective to the parser cluster where readability
  benefits most; other `str_data` / `str_len` call sites stay as-is
  and future slots pick up the convention opportunistically.

### Fixed

- **`baggage_set` / `baggage_get` / `textmap_set` / `textmap_get`
  over-annotation** in `src/telemetry.cyr`. The 1.0.2 `: Str`
  annotation pass tagged the key/value params on these four
  pass-through helpers, but the function bodies forward the values
  straight into the stdlib hashmap (`map_set` / `map_get`) and
  trait dispatch (`trait_call1`) which take untyped 8-byte slots.
  cyrius 5.10.14's call-site type-checker now flags the mismatch.
  Drop the annotations — these helpers don't inspect the bytes,
  and the stdlib slot semantics are opaque-pointer, not Str. (The
  surface was hidden until 1.0.3 wired `CYRIUS_TYPE_CHECK=1` into
  CI.)
- **CI `Type-check` step filter pattern**: stdlib self-flags format
  the path as `lib/<file>.cyr` (no leading slash); the prior
  `grep -v "/lib/"` missed them. Switch to `^warning:lib/` and
  route output through a tempfile to avoid the `$()` substitution
  dropping null bytes from the cyrius mixed-stream output.

## [1.0.3] - 2026-05-09

### Toolchain

- **`cyrius.cyml`** `[package].cyrius` pinned `5.10.3` → `5.10.14`.
  Picks up the v5.10.x type-system arc closures (5.10.5: agnosys
  1.1.12 verbatim repro CLOSE + extended overload dispatch +
  diagnostic hint catalog), the cyrfmt char-literal brace fix
  (5.10.6 — the bug that drove the 1.0.2 `'}'` → `125` workaround),
  the `#derive(Serialize)` Str-field positional-init fix (5.10.7),
  the JSON escape fix for derive (5.10.8), the version-pinned
  lib path that kills cross-version snapshot contamination
  (5.10.9), the cyrlint char-literal brace fix (5.10.10 — filed
  by agnostik's 5.10.9 refresh; closes the workaround that was
  holding `125` at 8 putc sites), the `lib/net.cyr` non-blocking
  connect primitive (5.10.11), the defensive `lib/fnptr.cyr`
  rewrite (5.10.12), TLS surface tightening (5.10.13), and the
  multi-stack `#derive(...)` directive support (5.10.14). DCE
  binary `273 KB` → `274 KB`.
- **`CYRIUS_TYPE_CHECK=1`** build is now warning-free including
  on stdlib — 5.10.4's param-side `: Str` annotation pass closed
  the two `lib/str.cyr:268/276` self-flags that 1.0.2 had to
  filter past.

### Changed

- **Reverted the 1.0.2 `'}'` putc workaround** at 8 call sites
  across `agent.cyr`, `config.cyr`, `hardware.cyr`, `llm.cyr`,
  `telemetry.cyr`, `validation.cyr`. cyrlint 5.10.10 handles the
  char-literal brace token correctly; `str_builder_putc(sb, '}')`
  is the readable form again.

### Added

- **`CYRIUS_TYPE_CHECK=1` step in CI** (`.github/workflows/ci.yml`).
  Opts the build into the v5.10.1 call-site type-checker so the
  v1.0.2 `: Str` annotation pass doesn't drift. Filters `/lib/`
  self-flags (currently none on 5.10.14, but the filter is durable
  across stdlib churn). The originally-pinned upstream default-on
  flip was attempted at 5.10.5 and reverted — opt-in CI is now the
  durable form rather than a stopgap.

### Docs

- **Resolved-issue archive**: introduced
  `docs/development/issues/archive/` with a short README explaining
  the convention. Two issues moved in:
  - `cyrlint-char-literal-brace-bug-2026-05-09.md` — fixed upstream
    in cyrius 5.10.10 (this slot).
  - `cyrius-lint-ufcs-pascal-prefix-snake-case-2026-04-26.md` —
    fixed upstream in cyrius 5.7.7 (was already marked resolved
    in-file but had stayed under `issues/`).
  Each archived file carries a resolution-status header pointing
  at the fixing release; original reports preserved for audit
  lineage. Inbound references in `docs/audit/2026-04-26-audit.md`
  repointed.

### Fixed

- **`cyrius.cyml [package].repository`** typo:
  `https://github.com/MacCracken/2mp/agnostik` →
  `https://github.com/MacCracken/agnostik`. Stray `2mp/` segment
  removed; the repo has always lived at the un-prefixed path.

## [1.0.2] - 2026-05-09

Cyrius 5.10.3 modernization pass on top of 1.0.1. No public API
changes; the surface change is in *how* internals are written, not
*what* they expose. All 653 assertions across 9 `.tcyr` files pass;
`CYRIUS_TYPE_CHECK=1` clean (zero warnings from agnostik code; the
two remaining warnings are stdlib `lib/str.cyr` self-flags upstream
will close in 5.10.4's already-shipped param-side annotation pass);
DCE binary `261 KB` → `273 KB` from the chrono dependency pulling
in extra compile-only surface that DCE didn't fully eliminate.

### Toolchain

- **Test boilerplate dropped** across 9 `.tcyr` files + 1 `.bcyr`
  + `src/main.cyr`. cyrius 5.10.x auto-injects the `main()` caller
  in `cyrius build` / `test` / `bench` and lazy-inits the heap on
  first `alloc()` (since v5.8.37). Removed `alloc_init();` and the
  trailing `var r = main(); syscall(SYS_EXIT, r);` from every test
  + harness. CLAUDE.md's matching durable rule was updated.
- **`result` + `chrono` added to `[deps] stdlib`** in `cyrius.cyml`.
  `chrono.cyr` provides `clock_now_ns()` (CLOCK_MONOTONIC, id 1);
  the inlined `now_ns()` that 9 files previously carried (using
  CLOCK_MONOTONIC_RAW, id 4) is removed. 10 call sites in
  `audit.cyr`, `agent.cyr`, `secrets.cyr`, `telemetry.cyr` now
  call `clock_now_ns()`. Functionally equivalent for elapsed-time
  measurement; CLOCK_MONOTONIC is NTP-slewed where _RAW isn't —
  acceptable for span/audit/secret timestamps.

### Changed

- **`?` operator adopted in `tctx_from_traceparent`**
  (`src/telemetry.cyr`). The only parse function that was manually
  propagating sub-parse errors via `is_ok(...) == 0 { return ... }`.
  Now reads `var tid = trace_id_from_str(...)?;` —
  cyrius v5.8.29's Result-propagation operator binds the Ok payload
  on success, returns Err on failure.
- **`: Str` annotation pass across 12 source files.** Every public
  function whose param or return is unconditionally `Str`
  (`str_data` / `str_len` consumers; `str_new` / `str_from` /
  `str_builder_build` producers) now carries the type tag. ~120
  annotations across `types.cyr`, `error.cyr`, `validation.cyr`,
  `secrets.cyr`, `config.cyr`, `audit.cyr`, `agent.cyr`,
  `security.cyr`, `telemetry.cyr`, `llm.cyr`, `hardware.cyr`,
  `main.cyr`. Skips slots that may be `0` (unset prerelease /
  build / metadata / message), `*_name()` fns that return cstr
  literals, and raw byte buffers. Verified clean under
  `CYRIUS_TYPE_CHECK=1` build.
- **Single-char `str_builder_add_cstr` → `str_builder_putc`** at
  15 call sites across `agent.cyr`, `config.cyr`, `hardware.cyr`,
  `llm.cyr`, `validation.cyr`, `telemetry.cyr`, `types.cyr`. The
  v5.10.x putc API takes a byte directly (via the char-literal
  token `'X'`), avoiding the strlen() loop add_cstr does for every
  cstr write. Touches `}`, `"`, and `-` byte emits in JSON
  serialization and traceparent formatting.

## [1.0.1] - 2026-05-08

Documentation cleanup + toolchain refresh on top of 1.0.0. Manifest
pin moved from Cyrius `5.7.12` to `5.10.3` (the latest released
version) and the bench banner stripped of its hardcoded toolchain
literal. Public API unchanged; all 653 assertions across 9 `.tcyr`
files pass; stdlib dependencies (`syscalls`, `string`, `alloc`, `str`,
`fmt`, `vec`, `hashmap`, `tagged`, `fnptr`, `trait`, `assert`, `io`,
`json`) re-resolved against 5.10.3 via `cyrius deps`.

### Toolchain

- **`cyrius.cyml`** `[package].cyrius` pinned `5.7.12` → `5.10.3`.
  Locally installed `cyrius` may report a newer dev build (e.g.,
  `5.10.4`); the manifest always pins to the latest **released**
  version so CI and external contributors get a reproducible
  toolchain.
- **`tests/bcyr/agnostik.bcyr`** banner literal stripped of the
  toolchain version (`=== agnostik benchmarks (Cyrius v5.7.12) ===`
  → `=== agnostik benchmarks ===`) so it no longer drifts from the
  manifest pin on every refresh.
- **DCE binary size** floor moved from `214 KB` (5.7.12) to `261 KB`
  (5.10.3). Pure codegen difference — no new code in `src/`.
- **`cyrius audit`** is still broken on 5.10.x (`check.sh` missing
  from the toolchain install — same upstream issue carried forward
  from 5.7.12, tracked in
  [`docs/development/issues/archive/cyrius-audit-missing-check-script-2026-04-26.md`](docs/development/issues/archive/cyrius-audit-missing-check-script-2026-04-26.md)).
  Workaround unchanged: run `cyrius {test,fmt --check,lint}`
  individually.

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
  [`docs/development/issues/archive/cyrius-lint-ufcs-pascal-prefix-snake-case-2026-04-26.md`](docs/development/issues/archive/cyrius-lint-ufcs-pascal-prefix-snake-case-2026-04-26.md)
  re-stamped with resolution status; stays local for audit lineage.

### Filed upstream — new

- **`cyrius audit` invokes `~/.cyrius/bin/check.sh` but the install
  never ships it.** Verified on a freshly-`cyriusly install`'d
  5.7.12: `cmd_audit` in `cbt/commands.cyr:395-398` expects
  `check.sh` next to the cyrius binary, but Cyrius's release
  manifest `scripts` array omits it. `cyrius audit` exits 127 on
  every fresh install of 5.7.x. Workaround: run `cyrius self /
  test / fmt --check / lint` individually. Filed locally:
  [`docs/development/issues/archive/cyrius-audit-missing-check-script-2026-04-26.md`](docs/development/issues/archive/cyrius-audit-missing-check-script-2026-04-26.md).

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
