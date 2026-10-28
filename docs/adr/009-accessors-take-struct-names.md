# ADR-009: Accessors are named `<Struct>_<field>`; only uniquely named, all-plain structs are derived

**Status:** Accepted
**Date:** 2026-10-28
**Slot:** v2.0.0

## Context

Through 1.12.x agnostik's 706 field accessors used short per-type prefixes: `amsg_content`, `secctx_run_as_user`,
`cglim_set_memory_max`. Cyrius's `#derive(accessors)` generates `<Struct>_<field>(p)` and
`<Struct>_set_<field>(p, v)`. The roadmap carried a rename to that shape for v2.0.0, so accessors could be derived
instead of hand-written. Three facts shaped how:

- **Consumer reach is small.** Excluding vendored copies, the five `[deps.agnostik]` consumers call the accessors
  in a handful of files: kybernet in 7, anuenue and mela in 2 each, aegis and ark in none.
- **Not every accessor is a plain load or store.** Some clamp (InjectionScores, AcceleratorFlags since 1.12.3),
  validate and return a `Result` (`cresult_set_level`), create a collection on first use (`hval_bounds`), or read
  a presence bit (`hval_has_sum`). A derive cannot express those.
- **Struct names are not unique across AGNOS.**
  - kavach derives `SandboxConfig` and stiva derives `AuditEntry`.
  - argonaut, stiva and agnoshi declare `HealthCheck`, `ResourceLimits`, `SecurityContext` and `NetworkPolicy`.
  - In a unit that vendors two of them, same-named hand-written fns silently replace each other, and two derives
    with different layouts are a compile error.
  - A struct declared twice keeps its first layout: kybernet already builds argonaut's 32-byte `ResourceLimits`
    with agnostik's 48-byte layout, and warns. That is safe only because agnostik's is the larger and comes first.
  - Derived accessors are only width-correct for narrow fields from cyrius 6.6.7 on, and consumers compile the
    bundle on their own pins (6.6.2 today).

## Options considered

- **Keep the prefixes.** No churn and no new clash risk, but it leaves the roadmap item undone.
- **Namespace every struct name** (`StikSandboxConfig_*`). No clash is possible, but the names are long and
  differ from the type names every document uses.
- **Plain struct names, derive only where safe.** This gives the derive's names everywhere.
  - Accessors whose struct name is unique in the ecosystem, and whose fields are all 8-byte plain slots, can be
    derived. Every derive version since 3.7.1 handles 8-byte slots alike; verified on 6.6.2, 6.6.20 and 6.7.18.
  - The rest stay hand-written under the same names, with explicit offsets that do not depend on which layout the
    compiler kept.

## Decision

Plain struct names, decided with the maintainer on 2026-10-28.

- **Naming:** every accessor, a fn that reads or writes a field slot, is named `<Struct>_<field>` /
  `<Struct>_set_<field>`, whether it is plain, lazy, clamping, validating or a presence bit.
  - Constructors (`*_new`), codecs (`*_name`, `*_parse`, `*_to_*`, `*_from_*`) and operations (`asum_add_device`,
    `tctx_child`, `agent_id_eq`) keep their names.
- **Field names:** a field takes the struct's field name where the old accessor differed. For the seven value
  structs that is the JSON key: `rusage_memory` → `ResourceUsage_mem_used`, `aflags_cuda` →
  `AcceleratorFlags_cuda_available`.
- **One name per slot:** where a getter and setter disagreed, one name wins: `span_set_parent` →
  `Span_set_parent_id`.
- **Deriving (v2.0.0 step 3b):**
  - Only structs whose name no other AGNOS library declares, and whose fields are all plain 8-byte slots, may be
    derived.
  - Clash-prone names stay hand-written: `SandboxConfig`, `AuditEntry`, `HealthCheck`, `ResourceLimits`,
    `SecurityContext`, `NetworkPolicy`, `Resource`.
  - Deriving adds a setter for each getter-only field; that addition is accepted.

## Consequences

- **The public surface keeps its 1,122 fns:** 706 renamed, nothing added or removed. The live surface equals the
  1.12.3 snapshot pushed through the map.
- **Consumers migrate mechanically.**
  - `docs/migration/2.0.0-renames.tsv` is the complete old → new map. `scripts/migrate-2.0.0.sh DIR...` applies it
    as whole-word replacements.
  - All five consumers, migrated by the script, build and pass every test against the renamed tree.
- **Collision risk remains.** A future unit that combines agnostik with kavach or stiva gets duplicate accessor
  names for the clash-prone structs. The consumer sweep's duplicate scan and cyrius's "duplicate fn" warning are
  the guards. The lasting fix is those libraries using agnostik's types, which is agnostik's purpose.
- **History keeps the old names:** the CHANGELOG, audits and earlier ADRs are not rewritten.

## Step 3b outcome (2026-10-28)

- **Derived:** 61 structs, replacing 250 getters and 118 setters and adding 132 setters. 36 stay hand-written.
- **Field names that had to change:**
  - Serialized structs take their JSON keys: `AgentInfo_agent_type`, `TelemetryConfig_export_endpoint`.
  - `default` is a reserved word: `FleetConfig_default_profile`.
- **Pre-cut changes:** the map and the migration script carry these renames. They changed 3a's names before any
  release, so no consumer saw the earlier ones.
- **The layout test** checks every derived field against the hand-written offsets it replaced.

## Verification

- `scripts/api-surface.sh`: 1,122 = 1,122; the map applied to the old snapshot equals the new one exactly.
- The suite: 25 files, 2,429 assertions, plus the three allocation probes.
- The consumer sweep over copies migrated with `scripts/migrate-2.0.0.sh`: 5 of 5 green on their own pins, 0
  agnostik duplicates. The sweep also caught a bug in the script's first version, which skipped kybernet's
  `src/lib/`.
