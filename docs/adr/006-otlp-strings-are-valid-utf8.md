# ADR-006: OTLP `string` fields are always valid UTF-8 (invalid AnyValue strings go out as bytes)

**Status:** Accepted
**Date:** 2026-10-14
**Slot:** v1.12.0

## Context

Protobuf `string` fields must hold valid UTF-8. Through v1.11.0 agnostik's OTLP encoders wrote every string as
given, and a `Str` is just bytes: an attribute value read from an HTTP path, a span name built from a file name,
or a log body copied from a process's output may hold invalid UTF-8. A strict decoder may reject the whole
message. Python's upb does, and Go's `proto.Unmarshal` validates proto3 strings. A permissive one passes the
bytes on to a backend that may fail later.

Two OpenTelemetry texts speak to it:
- **The OTLP specification**, `docs/specification.md` §"UTF-8 String Handling" in opentelemetry-proto:
  > Encoders SHOULD ensure `string` fields contain valid UTF-8. Producing invalid UTF-8 is not specification
  > compliant and is considered a bug within OpenTelemetry.

  It asks decoders to replace invalid sequences with U+FFFD.
- **The attribute type mapping**, `specification/common/attribute-type-mapping.md` §"String Values":
  > String values which are not valid Unicode sequences SHOULD be converted to AnyValue's bytes_value with the
  > bytes representing the string in the original order and format of the source.

The encoders cannot return an error without changing their signatures: `Span_to_otlp_proto` and the other
`<Struct>_to_otlp_proto` fns return 0, not a `Result`.

## Options considered

- **Validate and return `Err`.** That breaks every encoder's signature and drops the whole span or record over
  one bad byte. A telemetry pipeline should degrade, not lose data.
- **Replace invalid sequences with U+FFFD everywhere.** The output is always valid, but it is lossy where the spec
  offers a lossless path, AnyValue's `bytes_value`.
- **`bytes_value` for AnyValue strings, U+FFFD for every other `string` field.** This follows the attribute
  mapping where it applies: attribute values and the log body are AnyValues. Everywhere else it applies the
  replacement the OTLP spec gives decoders, so a decoder sees exactly what it would have produced itself: keys,
  span / event / metric names, descriptions, units, scope name and version, and schema URLs. Those fields have no
  bytes alternative.
- **Write as given (status quo).** That is a spec "bug", in the spec's own word.

## Decision

The third option.
- **Every protobuf `string` field goes through `_stik_pb_text`.** Valid UTF-8 is written unchanged.
- **Otherwise each maximal ill-formed subpart becomes U+FFFD.** That is Unicode §3.9, "U+FFFD Substitution of
  Maximal Subparts", the same rule as WHATWG Encoding and Python's `errors="replace"`. Overlongs, surrogates
  (`ED A0..BF`) and code points past U+10FFFF are ill formed.
- **AnyValue strings go through `_stik_pb_any_string`:** `string_value` when they are valid, else `bytes_value`
  with the original bytes. That covers attribute values, nested array and kvlist values, and the log body.

## Consequences

- Output always decodes under a strict decoder. Valid strings, the overwhelming case, are byte-identical to
  before, so every v1.10.0 golden vector is unchanged.
- An attribute value with invalid UTF-8 changes type on the wire, from string to bytes. A backend sees bytes, as
  the spec intends, rather than a corrupted string.
- A name or key with invalid UTF-8 is altered, so two distinct invalid keys can collapse to the same replacement.
  That is the decoder-side behaviour anyway; agnostik now applies it before the message leaves.
- Every string is validated as it is copied, in one pass: 8 ASCII bytes at a time (`load64` / `store64`), one
  sequence at a time otherwise. On the first ill-formed sequence the writer rewinds to the field's start and takes
  the sanitizing path. Measured in v1.12.0 work (paired A/B against 1.11.0, 15 rounds):
  - **Separate scan, then a byte-by-byte copy (first draft):** the OTLP rows were +1.9% to +13.2% slower, and a
    run with the scan bypassed put them back within noise, so the scan was the whole cost.
  - **Single pass:** the span row is −3.2% (the word copies beat the old byte-by-byte `memcpy`), and the others are
    within noise, at +0.8% to +3.6%.

## Verification

- **Differential test against Python,** comparing each string sanitized by `_stik_pb_text` with
  `bytes.decode("utf-8", "replace").encode()`, length prefix included. 0 mismatches in either set:
  - 20,000 short strings biased toward UTF-8 edge cases (lead and continuation bytes, overlongs, surrogates,
    truncations, values past U+10FFFF), 18,058 of them invalid;
  - 20,000 longer, ASCII-heavy strings with invalid bytes mixed in, 12,336 of which reach the 8-byte path.
- `tests/tcyr/test_v1120_otlp_envelopes.tcyr` pins the edge cases byte for byte, along with the `bytes_value`
  fallback for attribute values and the log body.

## Later change (v2.0.0)

The JSON writers (`AgentInfo_to_json`'s name, `TelemetryConfig_to_json`'s endpoint) apply the same rule through
`_stik_json_text`. The replacement loop moved out of `_stik_pb_text` into `_stik_utf8_fix_into`, so OTLP and JSON
share one implementation.
