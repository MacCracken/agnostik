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

# Parse "name: Xns" lines and append to CSV.
echo "$BENCH_OUTPUT" | while IFS= read -r line; do
    if [[ "$line" =~ ^[[:space:]]*([a-zA-Z_/0-9]+):[[:space:]]*([0-9]+)ns ]]; then
        echo "${TIMESTAMP},${COMMIT},${BRANCH},${BASH_REMATCH[1]},${BASH_REMATCH[2]}" \
            >> "$HISTORY_FILE"
    fi
done

echo
echo "appended to $HISTORY_FILE"
