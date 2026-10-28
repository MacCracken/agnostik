# `agnostik_err_new` writes through its allocation without checking it — OPEN

**Status:** 🟡 **OPEN** — filed by a consumer; confirmed by reading the source at tag `1.12.0`.
Not reproduced by running it: the window is narrow (below).
**Placement:** unplaced — for the user to place.
**Source:** anuenue (`/home/macro/Repos/anuenue`), found by the code review in its v1.4.4 closeout
(`docs/audit/2026-10-14-v14x-delta.md`).
**Discovered:** 2026-10-14.
**Severity:** Low — a null write, but only when the heap fails between two allocations a few bytes apart.
**Affects:** `src/error.cyr` `agnostik_err_new`, line 32 at tag `1.12.0` (`dist/agnostik.cyr` carries the same
code). Every consumer that builds an error on a path where memory may already be short — which is the path
where errors get built.

## Summary

```cyrius
fn agnostik_err_new(kind, message: Str) {
    var e = alloc(24);
    store64(e, kind);
```

`alloc` returns 0 when the allocator cannot map a fresh chunk; it does not abort. With `e == 0` the
`store64` (and the stores after it) write to absolute addresses 0x00–0x17 — a segfault rather than an error
report, on the one path whose job is to report.

## Why a consumer hits it

anuenue reports every failure through `anuenue_fail(kind, msg)`, which does:

1. `str_new(msg, len)` — the `Str` agnostik's API needs;
2. `agnostik_err_new(kind, m)`;
3. `agnostik_err_print(e)`.

anuenue checks step 1 and falls back to a bare stderr write when it fails — its allocation-failure probe
(`tests/probes/allocfail-probe.cyr`, run under `prlimit --as`) drives that fallback under a genuinely
exhausted heap. But if step 1 succeeds and step 2's 24-byte allocation is the one that fails — the heap
running out between two adjacent small allocations — the write in step 2 is unguarded. The probe does not
reach that window: once its heap is drained, `str_new` fails first.

## Reproduction

None run. By inspection: any caller of `agnostik_err_new` when `alloc(24)` returns 0. A test could reach it
the way anuenue's probe does — exhaust the heap under `prlimit --as=400MiB`, then call `agnostik_err_new`
directly.

## Root cause (if known)

The missing `if (e == 0)` after `alloc(24)`, line 32.

## Proposed fix

None — just surfacing. (One shape: return 0 on allocation failure and document that `agnostik_err_print`,
`agnostik_err_code` and friends accept a 0 error — or a static "out of memory" error the function can return
without allocating.)

## Consumer-side workaround (if any)

None shipped. anuenue keeps calling `agnostik_err_new` after a successful `str_new`; the window is two
adjacent small allocations, and anuenue's existing fallback covers the case where the first one fails.
