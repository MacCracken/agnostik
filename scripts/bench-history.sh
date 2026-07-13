#!/usr/bin/env bash
# Run agnostik benchmarks, append to a CSV history file.
# Usage:
#   ./scripts/bench-history.sh                # default: docs/benchmarks/history.csv
#   ./scripts/bench-history.sh custom.csv     # custom output
set -euo pipefail

HISTORY_FILE="${1:-docs/benchmarks/history.csv}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")

mkdir -p "$(dirname "$HISTORY_FILE")"
[ ! -f "$HISTORY_FILE" ] && echo "timestamp,commit,branch,benchmark,estimate_ns" > "$HISTORY_FILE"

echo "agnostik benchmarks (commit $COMMIT branch $BRANCH @ $TIMESTAMP)"
echo

BENCH_OUTPUT=$(cyrius bench tests/bcyr/agnostik.bcyr 2>&1)
echo "$BENCH_OUTPUT"

# Parse "name: X[.frac](ns|us|ms) avg" lines, normalize to ns, append to
# CSV. cyrius bench emits fractional averages (e.g. `1.294us`); the
# optional decimal is captured and scaled to ns with floating-point math
# (bash integer arithmetic can't multiply `1.294 * 1000`). Regression
# detection still widens the threshold for us-bracketed entries to absorb
# single-run CI jitter (see scripts/bench-regression.sh).
echo "$BENCH_OUTPUT" | while IFS= read -r line; do
    if [[ "$line" =~ ^[[:space:]]*([a-zA-Z_/0-9]+):[[:space:]]*([0-9]+(\.[0-9]+)?)(ns|us|ms)[[:space:]]+avg ]]; then
        raw="${BASH_REMATCH[2]}"
        case "${BASH_REMATCH[4]}" in
            ns) scale=1 ;;
            us) scale=1000 ;;
            ms) scale=1000000 ;;
        esac
        val=$(awk -v v="$raw" -v s="$scale" 'BEGIN { printf "%.0f", v * s }')
        echo "${TIMESTAMP},${COMMIT},${BRANCH},${BASH_REMATCH[1]},${val}" \
            >> "$HISTORY_FILE"
    fi
done

echo
echo "appended to $HISTORY_FILE"
