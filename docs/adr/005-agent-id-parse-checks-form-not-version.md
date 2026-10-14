# ADR-005: `agent_id_from_str` checks the UUID form, not its version

**Status:** Accepted
**Date:** 2026-10-14
**Slot:** v1.11.0

## Context

`agent_id_from_str` (and `user_id_from_str`, which calls it) parses the 36-character text form of a UUID into a
16-byte id. Through v1.10.0 it checked the length and that 32 positions held hex digits, but it skipped positions
8, 13, 18 and 23 without reading them. So `0af76519x16cd…` parsed as if the `x` were a dash. The v1.9.0
documentation pass recorded this, and asked a second question: should the parser also require the version 4
and RFC variant bits that `agent_id_new` sets?

A census of every local repo, `[deps.agnostik]` consumers included, found no caller of `agent_id_from_str` or
`user_id_from_str` outside agnostik. Inside agnostik, `AgentInfo_from_json` uses it for the `id` field.

## Options considered

- **Check the separators only.** Positions 8, 13, 18 and 23 must hold `-`, and every other position a hex digit
  in either case. Any version and variant parses.
  - This is the RFC 9562 text form exactly.
  - Ids that other systems mint still parse: v1 and v6 (time-based), v7 (time-ordered, increasingly the default
    for database keys), and v8 (custom).
  - The nil and max UUIDs parse too, which callers use as sentinels.
- **Also require version 4 and the RFC variant.** That matches what `agent_id_new` produces. But it rejects a
  well-formed id because of how it was generated, which says nothing about whether it is a valid identifier. An
  AGNOS component that receives agent ids from a service using v7 keys would have to re-mint them.
- **Leave it as is.** That accepts strings no UUID library would emit, and a typo in a separator would still parse
  as a different-looking but "valid" id.

## Decision

Check the four separators; accept every version and variant. `agent_id_new` keeps generating v4.

## Consequences

- A string with any byte other than `-` at a separator position is now an invalid-argument `Err`. Through v1.10.0
  it parsed. No consumer calls the parser, so nothing outside agnostik can observe the change.
- In agnostik, `AgentInfo_from_json` reads such an `id` as 0, as it already reads any invalid field.
- The version bits remain informational. A consumer that needs v4 specifically can check byte 6's high nibble
  itself.

## Verification

`tests/tcyr/test_v1110_contract_gaps.tcyr` (`test_agent_id_dashes`) covers:
- each separator position rejecting a non-dash;
- a hex digit in a separator slot, and a dash outside one;
- v1, v4, v7, nil and max UUIDs parsing;
- upper-case hex parsing;
- `user_id_from_str` sharing the check;
- the `AgentInfo_from_json` behaviour.

The v1.1.2 fuzz target (`test_v112_fuzz.tcyr`) still runs against the parser.
