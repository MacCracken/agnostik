# Agnostik Roadmap

## Status

**v1.9.0** is the most recent stable release, on Cyrius `6.7.9`. It has 12
modules plus `src/proto.cyr` (OTLP wire helpers), 979 public fns (every fn
documented), and 2,045 test assertions across 21 `.tcyr` files, passing on x86_64-linux and on aarch64
under `qemu-aarch64`. There are 25 benchmarks and no external dependencies.
Five repos declare `[deps.agnostik]` (aegis, anuenue, ark, kybernet, mela), and
`scripts/consumer-sweep.sh` builds and tests all of them against the working
tree before each release. CI runs the suite on x86_64 and on aarch64 (under
`qemu-user`), on a signature-verified toolchain.

Live numbers are in [`state.md`](state.md). Release history is in
[`../../CHANGELOG.md`](../../CHANGELOG.md). The most recent audit is
[`../audit/2026-10-14-audit.md`](../audit/2026-10-14-audit.md) (the 1.6.5, v1.7.0, v1.7.1, v1.8.0 and v1.9.0 passes).

Every item below is pinned to a release. Shipped work leaves this file for the
CHANGELOG: if work is worth doing, it has a slot, and once it ships it is no
longer here.

| Slot | Kind | Theme |
|---|---|---|
| [v1.10.0](#v1100--otlp-wire-format-completion-minor) | minor | OTLP `LogRecord`, `MetricDataPoint`, `Span` attributes / events / links |
| [v1.11.0](#v1110--contract-gaps-from-the-documentation-pass-minor) | minor | Missing setters / getters, unset vtable slots, reentrancy, `agent_id_from_str` validation, newer Landlock rights, `RlimitType` → `RLIMIT_*` |
| [v2.0.0](#v200--breaking-changes-next-major) | major | Accessor rename, dead-helper removal (`_stik_json_int` Result may move earlier) |

---

## v1.10.0 — OTLP wire-format completion (minor)

✨ This extends the v1.2.0 `Span_to_otlp_proto` foundation to the rest of the
OpenTelemetry data plane, using the existing `src/proto.cyr` primitives:

- **`LogRecord_to_otlp_proto`** and **`MetricDataPoint_to_otlp_proto`**, the
  same shape as the Span encoder.
- **Span's repeated nested fields**, skipped in v1.2.0: field 9 `attributes`
  (repeated `KeyValue`), field 11 `events` (repeated `Event`) and field 13
  `links` (repeated `Link`). These need nested encoders for `KeyValue` (a string
  key plus the `AnyValue` union), `Event` and `Link`. The `KeyValue` /
  `AnyValue` cluster is about half the slot.

Golden-byte tests against reference protobuf encodings, plus a fuzz target per
encoder. Map enum values at the encoder, as `_stik_span_kind_to_otlp` does for
`SpanKind`: `AggregationTemporality`'s values are agnostik ordinals
(CUMULATIVE = 0, DELTA = 1), while OTLP has DELTA = 1 and CUMULATIVE = 2.
Since v1.9.0 every stamp is Unix time (`clock_epoch_ns()`), so the new
encoders can write `time_unix_nano` as stored. No consumer has asked for this
yet, and it does not depend on v1.11.0: either order works.

---

## v1.11.0 — Contract gaps from the documentation pass (minor)

The v1.9.0 documentation pass read every fn. It fixed the three defects it found, and recorded
these gaps, which are additive or local fixes. None breaks the public API.

### Missing accessors (the F-021 class)

🔧 Fields with a getter and no setter:
- **llm:** `samp_freq_penalty`, `samp_pres_penalty`, `ireq_prompt`, the `RateLimitInfo` fields.
- **security:** the UTS / IPC / cgroup / time namespace flags, `sandbox_isolate_net`, the
  device-rule major / minor numbers.
- **config, validation, audit, telemetry:** several fields, including `telcfg_endpoint` and the
  audit-retention fields.

Some structs (`SecurityContext`, `SandboxCapabilities`, `TokenPayload`) have no setters at all,
and the AppArmor and SELinux setters have no getters. Take the census from the source before
starting: list every field offset, its getter and its setter. Each new setter gets a round-trip
test, as v1.7.0's `ClassificationResult` setters did.

### Unset optional vtable slots

🔧 `audit_sink_vtable` and `secret_store_vtable` leave their optional slots at 0. `as_verify_entry`
and `as_seal` dispatch through theirs without checking, so a sink built without
`audit_sink_set_verify_entry` / `_set_seal` crashes on a call to address 0. The audit sink's
`query` slot and the secret store's `rotate` / `search_by_tag` slots have no dispatcher at all:
callers use `trait_call*` directly, with the same hazard. Check the slot in the two dispatchers,
returning an unsupported-operation `Err` (or the fn's documented "none" value), and add checked
dispatchers for the other three.

### Global hand-offs

🔧 `textmap_set`, `propagator_inject` and `ss_put` dispatch with `trait_call1`, which passes one
argument after the data pointer, so each hands its second argument (`value`, `carrier`, `value`)
to the callback through a module global. A nested or concurrent call overwrites it. The stdlib
has no `trait_call2`, so dispatch through the slot with `fncall3(fp, data, a, b)` instead, and the
callback receives the argument directly. That changes the callback
signature consumers implement, so check every consumer's implementations first. If any exist,
keep the global path working alongside, or move the change to v2.0.0.

### `agent_id_from_str` validation

🔧 It skips positions 8, 13, 18 and 23 without checking that they hold a `-`, so any byte is
accepted there. Check them. Whether to also require the v4 version and
variant bits is a decision: a consumer may parse ids from elsewhere, so check every consumer's
calls first, and record the choice in an ADR.

### Newer Landlock rights

✨ The kernel uapi header (`linux/landlock.h`) defines four rights the enums lack:
`LANDLOCK_ACCESS_FS_IOCTL_DEV` (bit 15), `LANDLOCK_ACCESS_FS_RESOLVE_UNIX` (bit 16),
`LANDLOCK_ACCESS_NET_BIND_UDP` (bit 2) and `LANDLOCK_ACCESS_NET_CONNECT_SEND_UDP` (bit 3).
Append them to `LandlockFsAccess` / `LandlockNetAccess` at those values, with names in the
`landlock` crate's spelling. Appending keeps every existing value.

### `RlimitType` → `RLIMIT_*`

✨ `RlimitType`'s values are agnostik ordinals, not the kernel's `RLIMIT_*` numbers (`RLIM_NOFILE`
is 0, which is `RLIMIT_CPU`). No repo passes them to `setrlimit(2)` today. Add a mapping fn that
returns the `RLIMIT_*` resource number. Renumbering the enum itself would be a break, so it stays
out of a minor.

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
