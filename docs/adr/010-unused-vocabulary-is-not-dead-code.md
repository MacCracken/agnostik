# ADR-010: Vocabulary no consumer calls yet is not dead code; the v1.3.0 removal list stays

**Status:** Accepted
**Date:** 2026-10-28
**Slot:** v2.0.0

## Context

The v1.3.0 review ([`../audit/2026-06-01-audit.md`](../audit/2026-06-01-audit.md) §Deferred) called a set of public
helpers "dead or vestigial", and the roadmap gated their removal on a cross-consumer census:
- `seccomp_errno` / `seccomp_trace`;
- the `SeccompArg` cluster (`seccomp_arg_new`, its accessors, `SeccompArgOp` with its name and parse);
- `id_mapping_new` with the `IdMapping` accessors;
- `network_policy_*`;
- `stream_usage`;
- `AgentInfo_from_json`.

v2.0.0 ran that census on 2026-10-28. No repo under `~/Repos` outside agnostik uses any of them.

Checked against agnostik's own model, though, none of them is dead:

| Helper | Its place in agnostik's model |
|---|---|
| `seccomp_errno`, `seccomp_trace` | The documented way to express seccomp's data-carrying actions, ERRNO(n) and TRACE(n). `SeccompRule_action` names them, and the plain `SeccompAction` members cannot carry a value. |
| The `SeccompArg` cluster | `SeccompRule_args` is "a vec of SeccompArg conditions"; without it a rule cannot gate on syscall arguments. |
| `stream_usage` | The constructor for the `STREAM_USAGE` member, beside `stream_delta`, `_done`, `_error` and `_ping`. Without it the member has no constructor. |
| `IdMapping` | The user-namespace uid/gid mapping. A standalone type in the Rust original too, never attached to `NamespaceConfig`. |
| `network_policy_*` | Kept by the v2.0.0 decision, and attached to SandboxConfig in step 5a (`SandboxConfig_network_policy`). |
| `AgentInfo_from_json` | The reading half of `AgentInfo`'s serde pair. The v1.3.0 objection (it could not read its own output) was fixed in 1.6.3, and it is tested and was hardened in 1.12.3. |

## Options considered

- **Remove them as the roadmap planned.** A clean census means nothing breaks today, but `SeccompRule` loses its
  argument conditions and its data-carrying actions. Every later consumer that needs them would redefine them,
  which agnostik exists to prevent.
- **Remove only `IdMapping`,** the one type nothing in agnostik references. It is still the vocabulary for a real
  kernel concept, and removing it saves a dozen lines.
- **Keep all of them** and close the item.

## Decision

Keep all of them (decided with the maintainer, 2026-10-28). agnostik is a vocabulary library: its job is to own the
types AGNOS components share. That is often before every component that will use a type exists. "No current
consumer calls it" therefore does not make a public type or fn dead.

A public helper is a removal candidate only when one of these holds:
- **Unreachable:** agnostik's own model cannot reach or use it. Nothing produces or accepts it, and it represents no
  concept a consumer could exchange.
- **Superseded:** a newer API replaces it.
- **Wrong:** it cannot be made correct.

A clean census is necessary for removal, not sufficient.

## Consequences

- The roadmap's "Remove dead and vestigial public helpers" item closes with nothing removed, and the v2.0.0 API keeps
  these fns.
- **Private dead code is unaffected:** the closeout's dead-code audit still removes unreferenced *private* helpers,
  which no consumer can call.
- **Future proposals** to remove public vocabulary name which of the three conditions applies.

## Verification

- **Census:** a whole-word search of every `.cyr` / `.tcyr` / `.bcyr` under `~/Repos` outside agnostik, excluding
  vendored copies, finds 0 uses of each helper under both its 1.12.x and 2.0.0 names.
- **Rust original:** read from git history (`rust-old/`, deleted in `032fe0b`). `IdMapping` was standalone there,
  and `SeccompAction::Errno` / `Trace` and `SeccompRule.args: Vec<SeccompArg>` were part of the model.
