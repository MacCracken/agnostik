# ADR-011: Audit entries hash a length-prefixed binary encoding, and the chain owns its hashes

**Status:** Accepted
**Date:** 2026-10-28
**Slot:** v2.0.0

## Context

An `AuditEntry` carries `IntegrityFields`: a version, a signature, and `prev_hash`, the hash of the entry before it.
Through 1.12.x agnostik gave no way to produce the bytes that hash covers. Every sink had to choose its own,
and the v1.12.3 P(-1) pass ([`../audit/2026-10-28-audit.md`](../audit/2026-10-28-audit.md) §"Deferred to v2.0.0")
recorded three gaps:
- **No canonical encoding.** Two sinks hashing the same entry could disagree. An ad hoc encoding, such as fields
  joined with a separator, can also collide: `id "ab", action "c"` and `id "a", action "bc"` join to the same text.
- **`prev_hash` was aliased.** `integrity_fields_new` stored the caller's Str. A sink that computes each hash into
  one reused buffer rewrote every earlier link whenever it hashed the next entry.
- **No `AuditEntry_set_timestamp`.** The constructor stamps the current time, so a sink rebuilding entries from
  storage could not restore the time they were recorded.

The bytes have to be reproducible outside agnostik: a verifier in another language must re-derive them from a
stored entry.

## Options considered

- **JSON.** Readable, but not canonical without a canonicalization scheme (key order, number forms, escapes).
  agnostik's JSON writer also passes invalid UTF-8 through (fixed in 2.0.0's step 5f), so the text is not always valid
  JSON.
- **Protobuf through the existing OTLP writer.** Compact, and the writer is already here. But protobuf elides
  default values, so 0 and "absent" encode alike, and its canonical form is not specified.
- **A fixed-order, length-prefixed binary encoding.** Each field is self-delimiting, with a presence byte, an
  8-byte big-endian length, then the bytes. It takes a few lines to implement in any language and has no
  ambiguity.

## Decision

Use the length-prefixed binary encoding, **format 1**, written by `AuditEntry_to_canonical(e, sb)`.
- **Header:** the 16 ASCII bytes `agnostik:audit:1`. They separate these hashes from any other use of the same
  key, and name the format.
- **Fields, in order:** id, correlation_id, timestamp, agent_id, action, severity, result, details, user_id,
  source_ip, target_resource, duration_ms, tags, then the integrity fields' version and prev_hash.
- **Field encodings:**
  - integers: 8 bytes big-endian, two's complement;
  - Str: `0x00` for none, else `0x01`, the length, the bytes, so an empty Str and no Str differ;
  - agent id: `0x00`, or `0x01` and its 16 bytes;
  - tags: `0x00`, or `0x01`, a count, and each tag as a Str;
  - version: `0x00`, or `0x01`, major, minor, patch, prerelease Str, build Str;
  - the integrity fields: `0x00`, or `0x01` followed by the version and prev_hash.
- **The signature is excluded:** it is computed over these bytes.
- **Malformed input:** a Str with a negative length is an invalid-argument Err, and nothing is appended.

The chain owns its hashes: `integrity_fields_new`, `integrity_fields_genesis` and the `IntegrityFields` Str setters
copy the signature and `prev_hash`. That takes `IntegrityFields` off `#derive(accessors)`, whose setters can only
store. `AuditEntry_set_timestamp` restores a stored time.

## Consequences

- **Injective:** every field encoding is prefix-free and the order is fixed, so two entries encode alike only when
  every encoded field is equal.
- **Agreement:** sinks that hash `AuditEntry_to_canonical`'s output agree with one another, and with any verifier
  that implements this ADR.
- **The format is frozen.** A field added to `AuditEntry` later, or any other change, needs format 2 with a new
  header. Format 1 then stays decodable, so existing chains still verify.
- **What agnostik leaves to the sink:** it computes no hash and checks no signature; the hash function belongs to
  the sink (sigil's, for one). The genesis hash stays the 64-character `GENESIS_HASH` text, and `prev_hash` is
  whatever Str the sink stores, hex or raw.
- **Cost:** copying a signature and `prev_hash` costs two allocations for each chained entry. `audit_entry_new`'s
  genesis fields skip the copy, since its empty signature is already the entry's own.
- **Layout test:** `IntegrityFields` leaves the derived set, which is now 60 structs; the layout test drops its 6
  checks.

## Verification

- **Golden bytes:** `tests/tcyr/test_v200_contract.tcyr` checks a 288-byte golden entry. The expected bytes came
  from a separate Python implementation written from this ADR's text; every kind of field is set and one Str is
  left unset.
- **Collisions:** the same file checks that each of 12 single-field changes alters the encoding, that the
  adjacent-field, empty-versus-none and `[a, b]`-versus-`[ab]` cases differ, and that the signature is not encoded.
- **Errors:** a negative length in any of four places is an Err that leaves the builder unchanged.
- **Copies:** links survive the caller rewriting its hash buffer, through the constructors and through both
  setters.
