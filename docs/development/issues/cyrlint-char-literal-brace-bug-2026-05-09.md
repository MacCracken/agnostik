# `cyrlint` brace counter mistakes `'}'` char literal for closing brace

**Discovered:** 2026-05-09 during agnostik 5.10.3 → 5.10.9 toolchain refresh
**Severity:** Low (tooling — same family as the cyrfmt bug fixed in
v5.10.6; cyrlint emits false-positive `unmatched closing brace` warnings
on every `}` after a `str_builder_putc(sb, '}')` call. No correctness
impact; emitted bytes unchanged.)
**Affects:** Cyrius toolchain 5.10.x (verified on 5.10.9; the cyrfmt
sibling fix at v5.10.6 closed the same class of bug for cyrfmt but not
for cyrlint).
**Filed by:** agnostik (5.10.9 toolchain refresh — see [`CHANGELOG.md`](../../../CHANGELOG.md))

## Summary

cyrius v5.10.6 bundled a cyrfmt fix for the char-literal brace-counter
bug (see upstream
[`CHANGELOG.md` v5.10.6 § cyrfmt char-literal brace fix](../../../../cyrius/CHANGELOG.md)):

> cyrfmt's brace-depth counter at `programs/cyrfmt.cyr` v5.7.22 added
> skip for `#` comments + `"..."` string literals. v5.10.6 adds the
> same skip for `'...'` char literals.

The cyrlint tool has the same bug class — it counts `}` inside char
literals as a closing brace. Repro:

```cyr
fn foo(sb) {
    str_builder_putc(sb, '}');
    return 0;
}
```

cyrlint output (5.10.9):

```
warn line 4: unmatched closing brace
```

The `}` inside `'}'` decrements the brace depth; the real closing
`}` of the function body then becomes "unmatched". Every subsequent
brace in the file is also flagged. agnostik's serde paths hit this
8 times (one `'}'` putc per `<Struct>_to_json` writer); 5.10.9 lint
on the unmasked code emits 694 false-positive warnings across 6
files.

## Workaround in agnostik

Replace the char-literal form with the bare ASCII byte value:

```cyr
str_builder_putc(sb, 125);   # was: str_builder_putc(sb, '}');
```

Same emit; cyrlint clean. Matches the existing agnostik idiom for
byte-comparing parsers (`if (ch == 34)` for `"`, `46` for `.`,
`92` for `\`, etc. across `types.cyr` / `telemetry.cyr`).

`'"'` and `'-'` char literals do NOT trip cyrlint — verified by
`types.cyr` (which uses `'"'` and lints clean) and isolated probes
of `'-'`. Only `'}'` and presumably `'{'` are affected (they're the
brace tokens the counter is watching for).

## Upstream fix sketch

Apply the same skip cyrfmt got at v5.10.6 to cyrlint's brace-counting
pass — character literals (`'\''`, `'\\'`, `'X'`) should advance the
cursor past the closing `'` without consulting the byte for brace
semantics.

## Removal

Once cyrlint ships the fix, swap the 8 sites back to `'}'` and delete
this file. The 8 call sites are mechanical — `git grep -n "putc.*125"
src/` finds them all.
