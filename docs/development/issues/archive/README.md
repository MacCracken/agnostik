# Archived issues

Resolved issues: upstream defects that agnostik audits or refreshes found, and
agnostik defects that consumers reported. Files move here once the bug is
fixed in a shipped release (Cyrius's or agnostik's) and verified live, or once
a residue is shown not to be a defect.
The original report is preserved for audit lineage; each file's header
carries the resolution version + verification note.

Active issues live in the parent directory.

| File | Resolution |
|---|---|
| [`cyrius-lint-ufcs-pascal-prefix-snake-case-2026-04-26.md`](cyrius-lint-ufcs-pascal-prefix-snake-case-2026-04-26.md) | Fixed upstream in cyrius 5.7.7 |
| [`cyrlint-char-literal-brace-bug-2026-05-09.md`](cyrlint-char-literal-brace-bug-2026-05-09.md) | Fixed upstream in cyrius 5.10.10 (cyrfmt at 5.10.6) |
| [`cyrius-audit-missing-check-script-2026-04-26.md`](cyrius-audit-missing-check-script-2026-04-26.md) | Fixed upstream at 6.2.24 (`check.sh`) and 6.4.73 (audit preamble); the `cyrius self` residue was closed as not a defect at agnostik 1.6.4 |
| [`anuenue-agnostik-err-new-unchecked-alloc-2026-10-14.md`](anuenue-agnostik-err-new-unchecked-alloc-2026-10-14.md) | Fixed in agnostik 1.12.3 (F-028, ADR-007): a static out-of-memory error; checked by `tests/probes/err_oom.cyr` under `prlimit` |
| [`cyrius-derive-comments-in-struct-body-2026-05-10.md`](cyrius-derive-comments-in-struct-body-2026-05-10.md) | Fixed upstream in cyrius 6.6.7; verified on the 6.6.20 pin at agnostik 1.6.5. The workaround (comments above the directive) stays while consumers pin below 6.6.7 |
