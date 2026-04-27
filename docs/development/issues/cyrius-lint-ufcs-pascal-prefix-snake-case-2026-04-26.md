# `cyrius lint` flags hand-written `<PascalStruct>_<snake_verb>` fns as non-snake_case

**Discovered:** 2026-04-26 during agnostik 0.97.1 → 1.0.0 closeout pass under `cyrius` 5.7.6 (cc5 5.7.6)
**Severity:** Low (ergonomic — false-positive lint warnings; 100 % false-positive rate against agnostik's UFCS-style serde naming. No correctness or codegen impact.)
**Affects:** `cyrius lint` shipped in toolchain 5.7.6 (`cc5 --version` → `cc5 5.7.6`)
**Filed by:** agnostik (P(-1) audit, [`docs/audit/2026-04-26-audit.md`](../../audit/2026-04-26-audit.md) F-006)
**Supersedes:** an earlier draft of this report mis-attributed the warnings to `#derive(Serialize)` codegen. They are not codegen — see "what I got wrong" below.

## Summary

`cyrius lint` enforces `fn name should be snake_case` against any `fn` whose identifier is not a pure-snake_case run. This rule produces a false positive against any `fn` named `<PascalIdent>_<snake_run>` — the UFCS-style "method-on-type" convention. Agnostik (a Rust-port type-vocabulary library) uses this convention deliberately for its serde pairs:

```cyrius
struct ResourceLimits { max_memory; max_cpu_time; ... }
fn ResourceLimits_to_json(ptr, sb)   { ... }   # flagged
fn ResourceLimits_from_json(src)     { ... }   # flagged
```

The naming pairs structs to their serde adapters by struct name, so a reader scanning `ResourceLimits_to_json` immediately knows it operates on `ResourceLimits`. This convention also happens to match the form that `#derive(Serialize)` emits — agnostik mixes hand-written and derived adapters and keeps them naming-compatible so consumers don't have to remember which is which.

The lint rule fires 28 times across 6 modules in agnostik 1.0.0, **every hit a false positive** against this convention. The same pattern shows up wherever a project ports Rust struct-name conventions into Cyrius — likely most agnosticos consumers.

## Reproduction

Smallest repro (paste into `/tmp/repro.cyr`):

```cyrius
include "lib/syscalls.cyr"
include "lib/string.cyr"
include "lib/alloc.cyr"
include "lib/str.cyr"
include "lib/fmt.cyr"

struct ResourceLimits { max_memory; max_cpu_time; }

fn ResourceLimits_to_json(ptr, sb) {
    str_builder_add_cstr(sb, "{\"max_memory\":");
    str_builder_add_int(sb, load64(ptr));
    str_builder_add_cstr(sb, "}");
    return 0;
}

fn main() { return 0; }
main();
syscall(60, 0);
```

Run:

```
$ cyrius lint /tmp/repro.cyr
=== cyrlint: /tmp/repro.cyr ===
  warn line 9: fn name should be snake_case
1 warnings
```

The flagged function is hand-written, on the line cyrlint reports. There is no `#derive(...)` involved.

## What I got wrong in the first draft

The first version of this report claimed cyrlint was walking the post-`#derive` AST and flagging synthesised codegen names. **That's incorrect.** Verifying against `src/agent.cyr:47` — the line cyrlint actually points at — shows a real hand-written `fn ResourceLimits_to_json(ptr, sb) { ... }` declaration with a full body. The user caught this. cyrlint is doing exactly what it should — pointing at the hand-written source — but the rule is wrong about what counts as acceptable.

The narrow fix the first draft suggested (exempt derive-emitted nodes from the snake_case rule) **wouldn't clear any of agnostik's 28 warnings** because none of them are derive output. A broader fix is needed.

## Real-world signal (agnostik 1.0.0)

```
src/agent.cyr:        8 warnings  (Resource{Limits,Usage}, AgentInfo, AgentStats — to_json + from_json)
src/config.cyr:       2 warnings  (EdgeResourceOverrides — to_json + from_json)
src/hardware.cyr:     2 warnings  (AcceleratorFlags — to_json + from_json)
src/llm.cyr:          2 warnings  (TokenUsage — to_json + from_json)
src/telemetry.cyr:    2 warnings  (TelemetryConfig — to_json + from_json)
src/validation.cyr:   2 warnings  (InjectionScores — to_json + from_json)
```

100 % false-positive rate. The CI lint job currently has to either accept the warnings (defeating the purpose) or filter them with `grep -v 'should be snake_case'` (which would also hide a real new violation).

## Suggested fix (broader than the first draft)

Recognise `<PascalIdent>_<snake_lower_run>` as a legitimate UFCS-style "free function on a type" pattern and skip the snake_case rule for it.

A precise rule: if the identifier matches the pattern

```
^[A-Z][A-Za-z0-9]*_[a-z][a-z0-9_]*$
```

…and the leading PascalIdent is the name of an in-scope `struct` or `enum`, the fn is a UFCS-style method and snake_case does not apply. Two stacking conditions:

1. **Pattern**: leading PascalCase token, single underscore, then a snake-case body. This rules out arbitrary `BadName_thing` typos.
2. **Resolution**: the leading token resolves to a `struct` / `enum` declared in the same compilation unit. This rules out namespacing accidents and forces the convention to track real types — when the type is renamed or removed, stale fns surface as snake_case violations again.

Either condition alone catches most legitimate cases; both together is the strict form. Either is fine for agnostik.

Behaviour after the fix:

- `ResourceLimits_to_json` — accepted (Pascal prefix is a real struct, snake suffix).
- `Foo_to_json` where `Foo` doesn't exist — still warns (helps catch typos / dead types).
- `bar_to_json` — accepted (already pure snake).
- `BadName` — still warns (no underscore, no UFCS shape).
- Hand-written *and* derive-emitted alike — both pass without a special derive carve-out.

## Workaround in agnostik until the fix lands

CI tolerates the warnings. The audit doc explicitly catalogues the false positives (28 named sites) so a real new snake_case violation can be spotted by diff. No bulk suppression. When the upstream fix lands, agnostik's CI will go green automatically.

## Tracking

- Filed locally in agnostik under `docs/development/issues/`.
- Audit reference: [`docs/audit/2026-04-26-audit.md`](../../audit/2026-04-26-audit.md) §F-006.
- Stays local — not mirrored to `MacCracken/cyrius`, not POSTed as a GitHub issue. The user takes responsibility for distribution.
