# ADR-004: OTLP attribute values are AnyValue boxes, encoded canonically through a private buffer

**Status:** Accepted
**Date:** 2026-10-14
**Slot:** v1.10.0

## Context

v1.10.0 completes the OTLP encoders: Span attributes, events and links, plus `LogRecord`, `NumberDataPoint` and
`Metric`. Three questions had no answer in the existing code.

1. **What an attribute value is.** Every telemetry struct carries a cstr-keyed map (`span_attributes`,
   `logrec_attributes`, `mdp_attributes` …) whose values were "stored as given": untyped 64-bit words. OTLP needs a
   type for each value (`KeyValue.value` is an `AnyValue` oneof of string, bool, int, double, array, kvlist and
   bytes). An encoder cannot tell a `Str` pointer from an integer, and dereferencing a guess crashes on an integer.
   No consumer and no test stored anything in these maps (checked across all five `[deps.agnostik]` repos), so the
   value contract could be set now without breaking anyone.
2. **What bytes are "right".** v1.2.0's Span encoder was tested against bytes computed by hand from the `.proto`
   files. That process put `Status.code` in field 1, which the schema reserves (the code is field 3), so for eight
   minors every exported span status read as UNSET in a collector. Hand-derived goldens check an encoder against
   its author's reading of the schema, not against the schema.
3. **How to write nested messages.** A protobuf length prefix precedes its body, so a nested message must be
   measured before it is written. v1.2.0 built each sub-message in a child `str_builder` and copied it in. Measured
   on the first v1.10.0 draft (cyrius 6.7.9, x86_64), `str_builder_putc` costs about 33 ns a byte, and each child
   builder about 200 ns plus two copies of its bytes. A span with three attributes, two events and a link took
   14.3 µs.

## Options considered

**Attribute values.**
- **Typed AnyValue boxes** (`any_string`, `any_int`, …: a `tagged_new` box whose tag is the OTLP oneof field
  number). The type travels with the value, nesting falls out (an array is a vec of boxes, a kvlist a map of
  boxes), and the same box serves as a log body or a resource attribute. It costs a 16-byte box per value.
- **Strings only.** Treat every value as a `Str`. This is simple, but it loses ints, bools and doubles, which OTel
  semantic conventions use heavily (`http.status_code` is an int), and an integer stored by mistake crashes.
- **A parallel type map** (map key → type). Two maps must stay in step, and nesting has nowhere to go.

**Correctness reference.**
- **protoc's own encoding** of the same message in text format, against the official schema
  (opentelemetry-proto v1.7.0). The encoders must then emit canonical bytes: fields in ascending number order,
  proto3 defaults omitted, oneof members always present, minimal varints.
- **A decoder round-trip.** agnostik has no protobuf decoder, and a test-side decoder written from the same reading
  of the schema shares the hand-derivation blind spot.

**Nested messages.**
- **A private in-place buffer.** Write the tag and a one-byte length placeholder, write the body in place, and patch
  the length on close. A body of 128 bytes or more is shifted right by the extra varint bytes. Each field reserves
  its worst case once and stores bytes directly.
- **Two passes** (size, then write), as protobuf libraries do. That needs a size function mirroring every encoder,
  which doubles the code that must agree with the schema.
- **Padded (non-minimal) length varints.** These are legal to decode but not canonical, so they could not be
  compared byte for byte with protoc.

## Decision

- **Attribute map values are AnyValue boxes.** `AnyValueType`'s members are the oneof field numbers (`ANY_STRING`
  = 1 … `ANY_BYTES` = 7), so 0 is never a valid type. `any_value_type_name` spells them as the OTLP/JSON keys.
  The encoders read map values only through `boxed_tag` / `boxed_payload`. A 0 value, an unknown tag, nesting
  16 levels deep or a cycle (a box already among its own ancestors) encodes as an empty `AnyValue`, never a crash
  or unbounded recursion.
- **Keys are sorted bytewise** before encoding. Equal attributes then give equal bytes whatever the insertion order
  or the stdlib's per-process hash seed. Both cstr-keyed (`map_new`) and Str-keyed (`map_new_str`) maps are
  accepted.
- **protoc is the reference.** Every golden vector in `tests/tcyr/test_v1100_otlp.tcyr` is protoc's encoding (libprotoc
  35.0) of the same message, and the encoders emit canonical bytes so the comparison is exact.
- **Encoders write through `_stik_pb_*`** (`src/proto.cyr`): a private growable buffer with in-place nested
  messages, flushed to the caller's `str_builder` in one copy. The public signatures keep v1.2.0's shape,
  `<Struct>_to_otlp_proto(ptr, sb)`.

## Consequences

- An attribute value that is not an AnyValue box is a caller error the encoders cannot detect. The getter comments
  and the encoder comments state the contract.
- String fields are written as given. Protobuf requires valid UTF-8 in `string` fields, and a strict decoder may
  reject a message carrying invalid UTF-8; validating or replacing it is roadmap work, not done here.
- Output is deterministic and canonical, so it can be compared, hashed or deduplicated byte for byte.
- `Span_to_otlp_proto`'s status bytes change (field 1 → 3). That is the fix, and no consumer calls the fn.
- The private `_stik_proto_*` str_builder helpers are gone, replaced by `_stik_pb_*`. They were never public API.
- The in-place shift is a hand-written backward copy, because the stdlib has no `memmove`. A cyrius proposal for
  one is filed (`cyrius/docs/development/proposals/2026-10-14-memmove.md`).

## Verification

- `tests/tcyr/test_v1100_otlp.tcyr`:
  - 32 protoc golden vectors across AnyValue, Span, LogRecord, NumberDataPoint and Metric;
  - length-widening across the 1→2 and 2→3 byte varint boundaries at every nesting level (two sizes also matched
    against protoc);
  - cycle and depth bounds, sorted-key determinism and both map key types;
  - 600 fuzz iterations checked by a schema walker.
- Mutation checks: 15 deliberate breaks across the encoders and the buffer (wrong field numbers, unsorted keys, no
  cycle cut, elided oneof zeros, a wrong temporality mapping, no or forward shifting on close, a buffer that never
  grows). Each fails the file.
- Benchmarks (x86_64, `tests/bcyr/agnostik.bcyr` tier 5), first draft → buffer:

  | Row | First draft | Buffer |
  |---|---:|---:|
  | full span | 14.3 µs | 4.1 µs |
  | minimal span | 1.36 µs | 0.55 µs |
  | log record | 3.2 µs | 1.5 µs |
  | counter metric | 6.7 µs | 1.6 µs |

  Against the released 1.9.0 encoder, the minimal span's median falls from 735 ns to 477 ns (7 interleaved rounds,
  faster in all 7).
