# Agnostik Roadmap

## Status

**v1.12.0** is the most recent stable release, on Cyrius `6.7.9`. It has 12
modules plus `src/proto.cyr` (the protobuf writer behind the OTLP encoders),
1,122 public fns (every fn documented), and 2,307 test assertions across 24
`.tcyr` files, passing on x86_64-linux and on aarch64 under `qemu-aarch64`.
There are 31 benchmarks and no external dependencies.
Five repos declare `[deps.agnostik]` (aegis, anuenue, ark, kybernet, mela), and
`scripts/consumer-sweep.sh` builds and tests all of them against the working
tree before each release. CI runs the suite on x86_64 and on aarch64 (under
`qemu-user`), on a signature-verified toolchain.

Live numbers are in [`state.md`](state.md). Release history is in
[`../../CHANGELOG.md`](../../CHANGELOG.md). The most recent audit is
[`../audit/2026-10-14-audit.md`](../audit/2026-10-14-audit.md) (the 1.6.5 and v1.7.0 through v1.12.0 passes).

Every item below is pinned to a release. Shipped work leaves this file for the
CHANGELOG: if work is worth doing, it has a slot, and once it ships it is no
longer here.

| Slot | Kind | Theme |
|---|---|---|
| [v2.0.0](#v200--breaking-changes-next-major) | major | Accessor rename, dead-helper removal (`_stik_json_int` Result may move earlier) |

---

## v2.0.0 — Breaking changes (next major)

Breaking changes are batched here so every consumer absorbs migration once.

### `_stik_json_int` Result return

🔧 `_stik_json_int(src: Str, key: Str)` (`_json_int` before v1.7.0) returns
`i64` and conflates "missing key" with a literal `0`. F-003 hardened the
overflow path but left that ambiguity. Return a value-form `Result` instead, so
callers can tell missing from zero. Every caller moves to
`var n_t, n = _stik_json_int(s, k);` or `_stik_json_int(s, k)?`.

Note from v1.7.0: the helper is private. It is not in
`docs/api-surface.snapshot`, and no consumer calls it (checked across all five
`[deps.agnostik]` repos for the v1.7.0 rename). Its callers are all inside
agnostik, so this change no longer needs a major and could move to any minor.

### `#derive(accessors)` migration with prefix rename

🔧 Cyrius's `#derive(accessors)` generates `<Struct>_<field>` getters and
setters. agnostik's convention is `<prefix>_<field>` (`amsg_*`, `aentry_*`,
`secctx_*`). About 470 hand-written one-line accessors could become derive
markers, either by renaming to the derive shape (a consumer-visible break) or
by upstream derive-with-prefix support. v2.0.0 absorbs the rename.

### Remove dead and vestigial public helpers

🔧 The v1.3.0 review found public helpers with no consumer. Removing them is
breaking, and was gated on the cross-consumer sweep proving that no consumer
uses them. v1.7.0 delivers the sweep; this slot acts on its census:

- `security.cyr`: `seccomp_errno` / `seccomp_trace`, the `SeccompArg`
  cluster, `id_mapping_*`, `network_policy_*`
- `llm.cyr`: `stream_usage`
- `agent.cyr`: `AgentInfo_from_json`. Its round-trip was fixed in v1.6.3 and
  it now has a test, so it is a keep-or-remove call rather than dead weight.

Context: [`../audit/2026-06-01-audit.md`](../audit/2026-06-01-audit.md)
§Deferred.

---

## Working agreement

- **Default shape**: small fast-follows in patch slots, bundled minors, and
  breaking changes batched at majors. Documentation-only work never gets a
  release of its own; it rides the next minor. Public API additions ride minors. Each
  minor cut runs the security audit pass per CLAUDE.md.
- **Adding items**: draft a proposal under `docs/proposals/`, citing the
  trigger and the slot. An item without a slot stays a proposal until it earns
  one.
- **Removing items**: when a slot's work ships, its entry is deleted, because
  the CHANGELOG owns shipped history. When an item is abandoned, an ADR records
  why before the entry is deleted.
- **Scope**: the roadmap carries work that changes this repository. Toolchain
  defects get an agnostik-side mitigation here, and a record under
  `docs/development/issues/` when one is needed, but not "to file upstream"
  items ([ADR-003](../adr/003-roadmap-tracks-agnostik-work-only.md)).
