# ADR-003: The roadmap tracks agnostik work only; upstream papercuts are not carried

**Status:** Accepted
**Date:** 2026-10-14
**Slot:** v1.6.5

## Context

The roadmap's working agreement says an item leaves the roadmap one of two ways:
it ships, and the CHANGELOG owns it, or it is abandoned, and an ADR records why.
The 1.6.5 roadmap cleanup removes one item that did neither.

The v1.6.2 toolchain review left a backlog entry, "Upstream (cyrius) papercuts
to file". It named two toolchain behaviours to report to cyrius:

1. `cyrius update` on a project with no `[deps.NAME]` entries copies the whole
   stdlib snapshot into `./lib/` (111 files at 6.6.6, including an untracked
   `lib/unicode/`).
2. No raw-syscall diagnostic fires for `--agnos` builds, although agnos numbers
   its syscalls further from x86_64-linux than aarch64 does. The aarch64
   diagnostic caught F-022; nothing would have caught F-023.

Neither was filed in the three releases since, and both are still present at
the 6.6.20 pin. Neither is agnostik work: the first is a toolchain UX choice,
and the second is a compiler diagnostic. Filing them costs maintainer time and
fixes nothing in this repository.

## Options considered

- **Keep the entry until someone files them.** It keeps a note that has not
  moved in three releases, and it holds the roadmap open on work that belongs
  to another repository.
- **File both upstream now.** That is work in the cyrius tracker, not here, and
  the maintainer has asked that agnostik sessions not drift into toolchain issue
  filing.
- **Drop the upstream half and keep what agnostik can do itself.** Each
  papercut has an agnostik-side answer: the first is already a CLAUDE.md
  Quick Start rule, and the second can be a CI gate in this repository.

## Decision

Take the third option. The roadmap carries work that changes this repository:
its source, tests, gates, docs and releases. A toolchain defect that blocks or
endangers agnostik gets an agnostik-side mitigation on the roadmap and, when it
needs a reproducible record, a file under `docs/development/issues/`. Whether
to report it upstream is the maintainer's call, made outside the roadmap.

For the two papercuts:

1. `cyrius update` stays covered by CLAUDE.md, which says to refresh `lib/`
   with `cyrius lib sync` + `cyrius deps`, never `cyrius update`.
2. The missing agnos diagnostic becomes the v1.7.1 raw-syscall gate. CI fails
   on any raw `syscall(` in `src/`, and the four remaining raw sites in
   `src/error.cyr` move to the stdlib wrapper. That catches the F-023 class on
   every target, whatever the compiler diagnoses.

## Consequences

- The "Upstream (cyrius) papercuts to file" entry leaves the roadmap at 1.6.5
  without shipping.
- A future toolchain review records what it finds in its CHANGELOG entry and
  pins any agnostik-side fix to a slot. It does not add "to file" items.
- `docs/development/issues/` remains the place for upstream defect records
  that agnostik depends on, such as the archived derive-comments record.

## Verification

The v1.7.1 raw-syscall gate fails CI when a raw `syscall(` is added to
`src/`. Until then, the closeout security re-scan (CLAUDE.md, Closeout Pass
step 7) greps `src/` for raw syscall numbers.
