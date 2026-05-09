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

# Parse "name: X(ns|us|ms) avg" lines, normalize to ns, append to CSV.
# cyrius bench emits whole-unit averages — `1us` means 1000ns ± rounding;
# precision is lost on sub-µs values that fall in the [1, 1.999) µs band.
# Regression detection accommodates this by widening the threshold for
# us-bracketed entries (see scripts/bench-regression.sh).
echo "$BENCH_OUTPUT" | while IFS= read -r line; do
    if [[ "$line" =~ ^[[:space:]]*([a-zA-Z_/0-9]+):[[:space:]]*([0-9]+)(ns|us|ms)[[:space:]]+avg ]]; then
        val="${BASH_REMATCH[2]}"
        case "${BASH_REMATCH[3]}" in
            ns) ;;
            us) val=$((val * 1000)) ;;
            ms) val=$((val * 1000000)) ;;
        esac
        echo "${TIMESTAMP},${COMMIT},${BRANCH},${BASH_REMATCH[1]},${val}" \
            >> "$HISTORY_FILE"
    fi
done

echo
echo "appended to $HISTORY_FILE"
