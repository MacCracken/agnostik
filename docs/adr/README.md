# Architecture Decision Records

ADRs document **why** the project chose X over Y. They capture decisions that
shape the architecture or code conventions and that a reader cannot derive by
reading the current source. Decisions that *can* be derived (the current
struct layout, the current accessor naming) belong in
[`../architecture/overview.md`](../architecture/overview.md), not here.

## When to write one

- Choosing between two or more reasonable approaches and the choice is
  load-bearing for downstream work (e.g. derive vs hand-written serde).
- Establishing or changing a code convention that future contributors will
  ask "why?" about.
- Deciding NOT to do something that looks tempting (e.g. why we don't use
  `#derive(accessors)` despite cyrius supporting it).
- Reversing a prior ADR — supersede the old one with a new ADR; keep the old
  file with a `## Status: superseded by ADR-NNN` header.

## Numbering and naming

- Filename: `NNN-kebab-case-title.md` (e.g. `001-revive-derive-serialize.md`).
- Numbers are monotonic — never renumber. Superseded ADRs keep their original
  number; superseding ADRs get the next free number.
- Title is short, decision-focused (`revive #derive(Serialize)`, not `serde
  decisions`).

## Template

```markdown
# ADR-NNN: <decision in active voice>

**Status:** Proposed | Accepted | Superseded by ADR-MMM | Withdrawn
**Date:** YYYY-MM-DD
**Slot:** vX.Y.Z (the release the work pins to)

## Context

What problem are we solving? What are the constraints, the existing
surface, the consumer impact?

## Options considered

- **Option A** — pros / cons
- **Option B** — pros / cons
- **Option C** — pros / cons

## Decision

Active-voice statement of the choice. Cite the deciding factor.

## Consequences

What changes after we land this? What new constraints does it introduce?
What does it foreclose?

## Verification

How will we know it worked? Tests, benchmarks, audit findings cleared,
consumer sweeps green.
```

## Index

- [ADR-001 — Revive `#derive(Serialize)` (post-F-011 cyrius maturity)](001-revive-derive-serialize.md)
