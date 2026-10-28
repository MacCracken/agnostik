#!/usr/bin/env bash
# Paired, interleaved benchmark gate: this tree against the last release,
# both measured on the SAME host in alternating rounds. This is CI's bench gate.
#
# Why not scripts/bench-regression.sh in CI: that script compares one run
# against docs/benchmarks/history.csv, whose rows are taken on the release
# host. A GitHub runner is a different CPU, and which one a job gets varies,
# so every row moves by the runner's speed rather than the code's. The 1.12.1
# release run read 7 "regressions" (compute rows ~2x, syscall-bound rows flat,
# one row 38% faster) on a commit whose CI run had passed minutes earlier. A
# paired ratio cancels the host: both trees run on one machine, in alternating
# order, and each row's verdict is the median of its per-round ratios.
# bench-regression.sh stays the local release gate, on the host that records
# history.csv.
#
# What is compared:
#   base: the tree at BASE (default: the most recent X.Y.Z tag before HEAD),
#         built on ITS OWN cyrius pin, so a pin move is measured too;
#   head: the working tree, on its pin.
# Each tree's tests/bcyr/agnostik.bcyr is compiled once (CYRIUS_DCE=0, as
# `cyrius bench` builds it). The two binaries then run ROUNDS times, swapping
# which goes first each round. Each run contributes each row's `min=`: the
# fastest window resists a preempted window on a shared runner better than
# the mean does.
#
# A row regresses when its median paired delta exceeds THRESHOLD percent AND
# its median absolute slowdown exceeds FLOOR ns (tiny rows amplify jitter
# into percent). Rows new in head are listed, not gated. When there is nothing
# to compare, the gate passes with a notice: no release tag before HEAD in a
# full clone, or src/, the bench, cyrius.cyml and cyrius.lock unchanged since
# BASE. A shallow clone is an error, since its missing tags would otherwise
# pass silently.
#
# Usage: scripts/bench-paired.sh [BASE_REF]
# Environment:
#   BENCH_PAIRED_ROUNDS     rounds (default 9)
#   BENCH_PAIRED_THRESHOLD  percent (default 50)
#   BENCH_PAIRED_FLOOR_NS   absolute floor in ns (default 50)
#   CYRIUS_HOME             toolchain home (default ~/.cyrius)
# A base pin that is not installed is fetched, verified and installed
# alongside the active toolchain by scripts/ci-install-cyrius.sh --alongside.
# Skipped when the HEAD commit message carries [bench-regression-ack].
# Exit: 0 pass or nothing to compare, 1 regression, 2 setup error.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

ROUNDS="${BENCH_PAIRED_ROUNDS:-9}"
THRESHOLD="${BENCH_PAIRED_THRESHOLD:-50}"
FLOOR_NS="${BENCH_PAIRED_FLOOR_NS:-50}"
BENCH=tests/bcyr/agnostik.bcyr
export CYRIUS_HOME="${CYRIUS_HOME:-$HOME/.cyrius}"
export CYRIUS_NO_WARN_SHADOW_LIB=1
export LC_ALL=C

die() { echo "bench-paired: $*" >&2; exit 2; }

if git log -1 --format=%B 2>/dev/null | grep -qE '\[bench-regression-ack\]'; then
    echo "bench-paired: skipped (HEAD carries [bench-regression-ack])"
    exit 0
fi

BASE="${1:-}"
if [ -z "$BASE" ]; then
    [ "$(git rev-parse --is-shallow-repository)" = "false" ] ||
        die "shallow clone: the release tags are missing (CI checks out with fetch-depth: 0)"
    BASE=$(git describe --tags --abbrev=0 --match '[0-9]*.[0-9]*.[0-9]*' HEAD^ 2>/dev/null || true)
    if [ -z "$BASE" ]; then
        echo "bench-paired: no release tag before HEAD; nothing to compare"
        exit 0
    fi
fi
git rev-parse --verify --quiet "$BASE^{commit}" > /dev/null || die "'$BASE' is not a commit"

if git diff --quiet "$BASE" -- src tests/bcyr cyrius.cyml cyrius.lock; then
    echo "bench-paired: src/, tests/bcyr/, cyrius.cyml and cyrius.lock unchanged since $BASE; nothing to compare"
    exit 0
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

pin_of() { grep -oP '(?<=^cyrius = ")[^"]+' "$1/cyrius.cyml"; }

# Build one tree's bench binary on its own pin, and check the wrapper really
# dispatched to that pin: a silent fallback to the active toolchain would turn
# a pin-move comparison into a same-pin one.
build() {  # tree out label
    local tree="$1" out="$2" label="$3" pin ver
    pin=$(pin_of "$tree") || die "$label: no cyrius pin in $tree/cyrius.cyml"
    ver=$(cd "$tree" && cyrius --version 2>/dev/null | awk 'NR == 1')
    [ "$ver" = "cyrius $pin" ] || die "$label pins cyrius $pin, but the wrapper runs '$ver'"
    if ! (cd "$tree" && CYRIUS_DCE=0 cyrius build "$BENCH" "$out") > "$WORK/build-$label.log" 2>&1 \
            || [ ! -x "$out" ]; then
        cat "$WORK/build-$label.log" >&2
        die "$label: the bench did not build"
    fi
    echo "  $label: cyrius $pin"
}

mkdir -p "$WORK/base"
git archive "$BASE" | tar -x -C "$WORK/base"
BASE_PIN=$(pin_of "$WORK/base") || die "$BASE has no cyrius pin"
if [ ! -x "$CYRIUS_HOME/versions/$BASE_PIN/bin/cycc" ]; then
    bash "$REPO_ROOT/scripts/ci-install-cyrius.sh" --alongside "$BASE_PIN" ||
        die "could not install cyrius $BASE_PIN for $BASE"
fi
# The base tree vendors its stdlib exactly as CI does on a fresh checkout.
(cd "$WORK/base" && cyrius lib sync && cyrius deps) > "$WORK/vendor-base.log" 2>&1 || {
    cat "$WORK/vendor-base.log" >&2; die "$BASE: lib sync / deps failed"; }

echo "bench-paired: building"
build "$WORK/base" "$WORK/bench-base" "base $BASE"
build "$REPO_ROOT" "$WORK/bench-head" "head (working tree)"

# Each row's min=, in ns, as "name ns".
parse() {
    awk '/^  [A-Za-z_0-9]+: [0-9.]+(ns|us|ms) avg \(min=[0-9.]+(ns|us|ms) / {
        name = $1; sub(/:$/, "", name)
        m = $0; sub(/.*\(min=/, "", m); sub(/ .*/, "", m)
        s = 1; if (m ~ /us$/) s = 1000; else if (m ~ /ms$/) s = 1000000
        sub(/(ns|us|ms)$/, "", m)
        printf "%s %.1f\n", name, m * s }'
}

run() {  # binary rows-file
    "$1" > "$WORK/out" 2>&1 || { cat "$WORK/out" >&2; die "$1 exited non-zero"; }
    parse < "$WORK/out" > "$2"
    [ -s "$2" ] || { cat "$WORK/out" >&2; die "$1 printed no benchmark rows"; }
}

echo "bench-paired: $ROUNDS interleaved rounds"
: > "$WORK/pairs"
for r in $(seq 1 "$ROUNDS"); do
    if [ $((r % 2)) -eq 1 ]; then
        run "$WORK/bench-base" "$WORK/base.txt"; run "$WORK/bench-head" "$WORK/head.txt"
    else
        run "$WORK/bench-head" "$WORK/head.txt"; run "$WORK/bench-base" "$WORK/base.txt"
    fi
    # "name ratio-1 base head" for every row both trees have.
    awk 'NR == FNR { b[$1] = $2; next } ($1 in b) && b[$1] > 0 { print $1, $2 / b[$1] - 1, b[$1], $2 }' \
        "$WORK/base.txt" "$WORK/head.txt" >> "$WORK/pairs"
done
[ -s "$WORK/pairs" ] || die "the two trees share no benchmark rows"

# Per-row median of column $1 of the pairs file, as "name median".
median() {
    sort -k1,1 -k"$1,$1"g "$WORK/pairs" | awk -v c="$1" '
        function flush() { if (n) print cur, (n % 2 ? v[(n + 1) / 2] : (v[n / 2] + v[n / 2 + 1]) / 2) }
        $1 != cur { flush(); cur = $1; n = 0 }
        { v[++n] = $c }
        END { flush() }'
}
median 2 > "$WORK/m-delta"
median 3 > "$WORK/m-base"
median 4 > "$WORK/m-head"
awk '{ n[$1]++; if ($2 > 0) s[$1]++ } END { for (k in n) print k, s[k] + 0, n[k] }' "$WORK/pairs" > "$WORK/slower"

awk -v t="$THRESHOLD" -v floor="$FLOOR_NS" '
    FNR == 1 { f++ }
    f == 1 { d[$1] = $2; next }
    f == 2 { b[$1] = $2; next }
    f == 3 { h[$1] = $2; next }
    f == 4 { s[$1] = $2 "/" $3; next }
    END {
        for (name in d) {
            pct = d[name] * 100
            fail = (pct > t && h[name] - b[name] > floor)
            printf "%+09.2f %-32s %10.0f %10.0f %+9.1f%% %7s%s\n", pct, name, b[name], h[name], pct, s[name], fail ? "  **FAIL**" : ""
        }
    }' "$WORK/m-delta" "$WORK/m-base" "$WORK/m-head" "$WORK/slower" |
    sort -g | cut -d' ' -f2- > "$WORK/table"
awk 'NR == FNR { b[$1] = 1; next } !($1 in b) { print $1 }' "$WORK/base.txt" "$WORK/head.txt" > "$WORK/new"

echo
echo "head (working tree) vs $BASE: per-row min, median of $ROUNDS paired rounds"
printf "%-32s %10s %10s %10s %7s\n" "benchmark" "base ns" "head ns" "paired Δ" "slower"
printf '%.0s-' {1..74}; echo
cat "$WORK/table"
while read -r n; do printf "%-32s %10s %10s %10s %7s\n" "$n" "(new)" "-" "-" "-"; done < "$WORK/new"

checked=$(wc -l < "$WORK/table")
new=$(wc -l < "$WORK/new")
fail=$(grep -c 'FAIL' "$WORK/table" || true)
echo
echo "summary: $checked checked, $new new, $fail regressions (threshold: median paired delta > +$THRESHOLD% and > $FLOOR_NS ns)"
if [ "$fail" -gt 0 ]; then
    echo
    echo "BREAKING: $fail benchmark(s) slower than $BASE beyond threshold, on this host, in paired rounds."
    echo "Either optimize, or ack with [bench-regression-ack] in the commit message."
    exit 1
fi
echo "ok: no regressions against $BASE"
