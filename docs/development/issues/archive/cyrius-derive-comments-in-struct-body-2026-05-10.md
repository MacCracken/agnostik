# `#derive(Serialize)` codegen corrupted by `#`-comments inside struct body

**Status:** ✅ **Resolved upstream at cyrius 6.6.7** (bite 5: `#derive` now walks
the struct body the way the parser reads it, so `#` comments are trivia).
Verified live on agnostik 2026-10-14 under the 6.6.20 pin: the repro below and
all four negative cases emit the right field names and round-trip. agnostik
**keeps its workaround** (comments above the directive), because consumers
compile `dist/agnostik.cyr` under their own pins and any pin below 6.6.7 still
corrupts. See **Resolution — 2026-10-14** at the end. Archived at agnostik 1.6.5.

**Discovered:** 2026-05-10 during agnostik v1.1.1 sub-byte field widths work.
**Severity:** Medium (silent codegen corruption — produces malformed JSON
with garbage field names and wrong byte values; no compile-time error).
**Affects:** Cyrius toolchain 5.10.14 (probably earlier; not bisected).
**Filed by:** agnostik (1.1.1 sub-byte widths refactor).

## Summary

Putting a `#`-style comment inside a struct body that's preceded by a
`#derive(Serialize)` directive corrupts the derive codegen. Output is
silent garbage: the first field name becomes `"#"`, subsequent values
are read at wrong offsets / widths.

## Repro

```cyr
#derive(Serialize)
struct Bad {
    # any comment here breaks codegen
    sql: i8; xss: i8; command: i8; path_traversal: i8; prompt_injection: i8;
}

fn main() {
    var s = alloc(5);
    store8(s, 85); store8(s + 1, 10); store8(s + 2, 5);
    var sb = str_builder_new();
    Bad_to_json(s, sb);
    var j = str_builder_build(sb);
    syscall(1, 1, str_data(j), str_len(j));
    syscall(1, 1, "\n", 1);
    return 0;
}
```

Expected output:
```
{"sql":85,"xss":10,"command":5,"path_traversal":0,"prompt_injection":0}
```

Actual output (cyrius 5.10.14):
```
{"#":330325,"xss":237,"command":50,"path_traversal":64,"prompt_injection":0}
```

The first field name is the literal `#`; values are garbage bytes read
from out-of-bounds offsets relative to the (now-tiny) i8 struct.

## Diagnosis (best guess)

The `PP_DERIVE_SERIALIZE` codegen walks the struct body to collect field
names + types. The lex-or-parse pass that builds the field-name table
appears to treat `#` as a token rather than skipping the comment through
to end-of-line. The `#` becomes the first "field name"; subsequent comment
text gets interpreted as a width spec or offset advance, corrupting the
rest of the table.

Bug is silent because:
1. The struct body still parses (syntactically valid).
2. Codegen emits the right number of fields.
3. Field-name table is just wrong — first entry is `#`, others shifted.
4. Width-detection at struct-decl time also gets confused, leading to
   wrong load widths in `_to_json` (i8 fields read as i16 / i32 / i64
   depending on the corrupted table state).

Standalone struct (no `#`-comment inside the body) emits correctly.
Multiline struct decl without comment emits correctly. Adding even a
single `# anything` line inside the braces triggers the bug.

## Workaround in agnostik

Move all comments OUTSIDE the struct body — above the
`#derive(Serialize)` directive. Applied at `src/validation.cyr`
(`InjectionScores`) and `src/hardware.cyr` (`AcceleratorFlags`) in
v1.1.1.

## Suggested upstream fix

In `src/frontend/lex_pp.cyr`'s `PP_DERIVE_SERIALIZE` body-walking pass,
treat `#` as start-of-comment when it appears at the beginning of a
struct-body line (mirror the lexer's normal comment-handling). Skip to
end-of-line; resume field-name collection on the next line.

Negative test cases to validate the fix:
- `# comment` line inside struct body (the agnostik repro)
- `# comment` immediately before a field decl
- `# comment` immediately after the `{` opening brace
- Multiple consecutive `# `-style comments

## Removal

When upstream ships the fix, the workaround comments in
`src/validation.cyr` and `src/hardware.cyr` can move back inside the
struct body for locality, and this file moves to
`docs/development/issues/archive/`.

## Update 2026-09-23 — still open on cyrius 6.6.6 (agnostik 1.6.4)

**Re-verified, not fixed.** The repro above, built warning-free (stdlib
`syscalls string alloc fmt str vec result io bayan`), prints on 6.6.6:

```
{"#":330325,"xss":32,"command":0,"path_traversal":128,"prompt_injection":64}
```

6.6.0 and 6.6.2 print the same `"#":330325` first field. 330325 is 0x050A55,
i.e. the bytes 85 / 10 / 5 read as one wider integer. The later fields read
past the 5-byte struct and vary from run to run. The negative cases this file
listed all corrupt as well. A second struct carried a comment after the `{`,
two consecutive comment lines, a comment before a field, and a trailing
`# comment` after a field. It emitted `{"#":…,"#":0,"#":…}`: every field was
named `#`. `_from_json_str` of either output does not round-trip.

**Upstream has no record of it.** Nothing in the cyrius repo's
`docs/development/issues/`, open or archived, describes this. The nearest,
`archived/2026-09-19-lexer-attribute-prefix-swallows-comments.md` (fixed
6.6.6), is a different path: comments that *start* with an attribute name,
such as `#ioctl`, in the main lexer. Its fix does not reach the derive
body-walker. "Filed by: agnostik" above means it was recorded here, not in the
toolchain's tracker. It is now filed there: `cyrius/docs/development/issues/2026-09-22-agnostik-derive-serialize-comment-in-struct-body.md`.

**agnostik stays protected.** All 7 `#derive(Serialize)` structs keep their
comments above the directive: `ResourceLimits`, `ResourceUsage`, `AgentStats`,
`EdgeResourceOverrides`, `AcceleratorFlags`, `TokenUsage`, `InjectionScores`.
None has a `#` inside its body, checked by scanning `src/`. And
`tests/tcyr/test_v110_serde_golden.tcyr` pins every one of their `_to_json`
outputs byte for byte, so a comment added inside a derive body fails CI with
a `"#"` field name instead of shipping silently. Mutation-checked: one
`# comment` line inside `InjectionScores`' body makes the file report
`FAIL: InjectionScores compact bytes` (33 passed, 1 failed; exit 1).

## Resolution — 2026-10-14 (cyrius 6.6.20, agnostik 1.6.5)

**Fixed upstream at cyrius 6.6.7**, from the filing made at agnostik 1.6.4. The
cyrius `[6.6.7]` CHANGELOG entry ("`#derive` built its field table from a byte
scan of the struct / enum body …", bite 5, "filed by agnostik") names the root
cause: `PP_PARSE_STRUCT_DEF` in `src/frontend/lex_pp.cyr` began a field only
after a `{` or `;` byte and stopped names at a stop set that lacked `#`. The
body is now walked by `PP_DERIVE_FIELDS`, which treats `#`-to-end-of-line
comments and every blank as trivia, as the parser does. The same fix covers a
space before `:`, a field with no `;`, and `,`-separated enum members.

**Measured, not read.** The same probe was built on both pins:

| cyrius | `Bad` (the repro above) | `Many`: comment after `{`, two consecutive, before a field, trailing |
|---|---|---|
| 6.6.6 | `{"#":330325,"xss":32,"command":0,"path_traversal":128,"prompt_injection":118}` | `{"#":1,"#":2,"#":3}`, then `{"#":3,"#":0,"#":0}` after `_from_json_str` |
| 6.6.20 | `{"sql":85,"xss":10,"command":5,"path_traversal":0,"prompt_injection":0}` | `{"a":1,"b":2,"c":3}`, and the same after `_from_json_str` |

**The workaround stays.** `src/validation.cyr` (`InjectionScores`) and
`src/hardware.cyr` (`AcceleratorFlags`) keep their comments above the
`#derive(Serialize)` directive. This file's **Removal** section allowed moving
them back inside the bodies, but agnostik's bundle is compiled by each
consumer's own compiler, and of the five repos that declare `[deps.agnostik]`,
aegis, ark and mela pin 6.6.2 and kybernet pins 6.6.6. A comment inside a
derive body would reach those builds as the silent corruption above. The two
source comments now say this instead of "revisit when fixed". The guard is
unchanged too: `tests/tcyr/test_v110_serde_golden.tcyr` still pins every
derived `_to_json` byte for byte.
