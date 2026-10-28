#!/usr/bin/env bash
# Run every allocation-failure probe in tests/probes/ under an address-space
# limit, so the heap really runs out. Each probe drains the heap, then
# exercises an agnostik path under it. A probe passes when it exits with the
# status its `# expect-exit: N` header line names (default 0) and, if it has a
# `# expect-stderr: TEXT` line, its output contains TEXT. A signal never passes.
# They are not `.tcyr` files because without the limit the drain would map
# memory until the address space ran out.
#
# Usage: scripts/oom-probe.sh [LIMIT_MIB]   (default 512)
# Needs prlimit (util-linux). x86_64-linux only: it runs the native binary.
# Exit: 0 every probe passed, 1 a probe failed, 2 setup error.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"
LIMIT_MIB="${1:-512}"
command -v prlimit > /dev/null || { echo "oom-probe: needs prlimit (util-linux)" >&2; exit 2; }
export CYRIUS_NO_WARN_SHADOW_LIB=1

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
fail=0; n=0
for p in tests/probes/*.cyr; do
    [ -f "$p" ] || continue
    name=$(basename "$p" .cyr)
    n=$((n + 1))
    if ! CYRIUS_DCE=1 cyrius build "$p" "$WORK/$name" > "$WORK/$name.build" 2>&1; then
        cat "$WORK/$name.build" >&2; echo "FAIL: $name did not build"; fail=1; continue
    fi
    want_rc=$(sed -n 's/^# expect-exit: *\([0-9][0-9]*\) *$/\1/p' "$p" | head -1)
    want_rc="${want_rc:-0}"
    want_err=$(sed -n 's/^# expect-stderr: *//p' "$p" | head -1)
    rc=0
    prlimit --as=$((LIMIT_MIB * 1024 * 1024)) "$WORK/$name" > "$WORK/$name.out" 2>&1 || rc=$?
    sed 's/^/    /' "$WORK/$name.out"
    if [ "$rc" -ne "$want_rc" ]; then
        echo "FAIL: $name exited $rc$([ "$rc" -ge 128 ] && echo " (signal $((rc - 128)))"), expected $want_rc"
        fail=1
    elif [ -n "$want_err" ] && ! grep -qF -- "$want_err" "$WORK/$name.out"; then
        echo "FAIL: $name exited $rc as expected, but printed no '$want_err'"
        fail=1
    else
        echo "ok: $name (exit $rc)"
    fi
done
[ "$n" -gt 0 ] || { echo "oom-probe: no probes in tests/probes/" >&2; exit 2; }
echo "oom-probe: $n probe(s), limit ${LIMIT_MIB} MiB"
[ "$fail" -eq 0 ] || exit 1
