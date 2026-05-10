# `#derive(Serialize)` codegen corrupted by `#`-comments inside struct body

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
