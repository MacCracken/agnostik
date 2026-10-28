# ADR-007: An allocation failure fails by name; the error constructor never fails

**Status:** Accepted
**Date:** 2026-10-28
**Slot:** v1.12.3

## Context

The stdlib's `alloc(size)` does not abort. It returns 0 when the heap cannot grow (an mmap refused, as under
`RLIMIT_AS` or strict overcommit), and also for any size ≤ 0 or above 2 GiB. Through 1.12.2 none of agnostik's
135 allocation sites checked for that: the next `store64` wrote through address 0 plus an offset.

- **Where it surfaced:** anuenue reported it in `agnostik_err_new`. A consumer's failure-reporting path builds an
  error just when memory may be short, and there it died of SIGSEGV instead of reporting.
- **How bad a null write is depends on the target.** On x86_64 and aarch64 Linux, and on Windows, the low 64 KiB
  are never mapped, so the write faults at once: a crash, nothing worse. agnostik also builds for agnos, where
  page 0 is not known to be unmapped.
- **Not only exhaustion:** the size can also fail. Three sites sized buffers from input: a JSON string field, a
  secret's copy, and the protobuf buffer's growth. A size ≤ 0 or above 2 GiB fails there with plenty of memory
  free.

Patch-level constraints: the 1,122 public fns keep their signatures. A constructor that returns a struct pointer
cannot start returning a `Result`.

## Options considered

- **Check every site and return 0.** Every constructor gains a 0 return that no caller checks; the crash moves to
  the caller's first access. Not better.
- **Make constructors fallible (`Result`).** Correct, and a signature break across most of the API. That is a
  v2.0.0 question, not a v1 repair.
- **Clamp a bad size to a small block.** It hides arithmetic errors: a negative size from a wrapped sum would get
  a 1-byte block, and the copy that follows would overflow the heap. Worse than the crash it replaces.
- **Fail by name.** Route every allocation through one private wrapper. On 0 it writes `agnostik: allocation
  failed (out of memory, or a size <= 0 or over 2 GiB)` to stderr and exits with status 71 (EX_OSERR). The stdlib
  already behaves this way when its first heap chunk is refused (`alloc_init` prints and exits), and so does
  agnostik's own CSPRNG failure (F-023, exit 70).
- **For `agnostik_err_new`:** a static out-of-memory error, so the reporting path never fails at all.

## Decision

- **Every allocation in the shipped modules goes through `_stik_alloc(n)`,** which fails by name and exits 71.
  It does not clamp sizes: a size ≤ 0 is a failure.
- **Callers that know a size is out of range before asking call `_stik_alloc_fail()` directly.** The protobuf
  buffer does this when a reservation would take it past 2 GiB.
- **`agnostik_err_new` is the one exception.** If its 24 bytes cannot be had, it returns a static error: kind
  `STIK_ERR_RESOURCE_EXHAUSTED`, code 1005, message "out of memory". It never returns 0, so code that reports a
  failure keeps working when memory runs out. The deciding factor is that reporting is the path that must not fail.

## Consequences

- No agnostik allocation writes through address 0, on any target.
- A library fn can now end the process. It already could, by SIGSEGV; now the failure is attributable and the exit
  status is fixed. For a PID 1 consumer (kybernet) both outcomes are a kernel panic.
- **The error constructor can lose information.** The caller's own kind and message are lost when the static error
  stands in, which is correct: what failed was the allocation.
- **Out of reach:** allocations made inside stdlib fns that agnostik calls (`str_from`, `vec_new`,
  `str_builder_*`) keep the stdlib's own behaviour.
- **Each allocation pays one more call.** Paired A/B against 1.12.2: the smallest rows (`token_usage_update`,
  `accelerator_device_full`) cost a few ns; most rows moved under ±3%. `#inline` was tried and refused (the
  compiler does not inline a body with control flow).
- **New private code must call `_stik_alloc`, not `alloc`.**

## Verification

- **`scripts/oom-probe.sh` (in CI)** runs `tests/probes/*.cyr` under `prlimit --as=512MiB`, each checked by exit
  status and message. On the 1.12.2 source all three died of SIGSEGV.
  - `err_oom` drains the heap and checks that `agnostik_err_new` and `stik_err_io` return the static error, then
    prints it.
  - `alloc_fail` drains the heap and checks that `version_new` exits 71 with the message.
  - `pb_huge_len` encodes a span whose name claims a length near i64 max and checks that the encoder exits 71.
- **`tests/tcyr/test_v1123_hardening.tcyr`** covers the static error and `_stik_alloc`'s normal path.
