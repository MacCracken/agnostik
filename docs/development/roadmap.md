# Agnostik Roadmap

## Status

**v2.0.0** is the most recent stable release, on Cyrius `6.7.18`: the major that renamed every accessor to
`<Struct>_<field>` (ADR-009, mapped one-to-one in [`../migration/2.0.0-renames.tsv`](../migration/2.0.0-renames.tsv))
and closed the seven items deferred from 1.12.3. It has 12 modules plus `src/proto.cyr` (the protobuf writer
behind the OTLP encoders, and the UTF-8 rule the JSON writers share), 1,295 public fns (every fn documented), and
2,582 test assertions across 27 `.tcyr` files, passing on x86_64-linux and on aarch64 under `qemu-aarch64`.
There are 31 benchmarks and no external dependencies.
Five repos declare `[deps.agnostik]` (aegis, anuenue, ark, kybernet, mela), and
`scripts/consumer-sweep.sh` builds and tests all of them against the working
tree before each release. CI runs the suite on x86_64 and on aarch64 (under
`qemu-user`), on a signature-verified toolchain, on the `ubuntu-26.04` runner.

Live numbers are in [`state.md`](state.md). Release history is in
[`../../CHANGELOG.md`](../../CHANGELOG.md). The most recent audit is
[`../audit/2026-10-28-audit.md`](../audit/2026-10-28-audit.md) (the v1.12.1 toolchain refresh, the v1.12.2 CI
pass, the v1.12.3 P(-1) pass and the v2.0.0 release pass); the one before it,
[`../audit/2026-10-14-audit.md`](../audit/2026-10-14-audit.md), holds the 1.6.5 and v1.7.0 through v1.12.0 passes.

Every item below is pinned to a release. Shipped work leaves this file for the
CHANGELOG: if work is worth doing, it has a slot, and once it ships it is no
longer here.

**No slot is pinned.** v2.0.0 shipped the last one, and its decisions and steps are in the CHANGELOG's `[2.0.0]`
entry and ADR-009 to ADR-011. New work gets a slot through a proposal, as the working agreement below describes.

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
