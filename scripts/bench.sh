#!/bin/sh
set -e
CC="${CC:-$HOME/.cyrius/bin/cc2}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
mkdir -p "$ROOT/build"

echo "Building benchmarks..."
cat "$ROOT/benches/bench.cyr" | "$CC" > "$ROOT/build/agnostik_bench"
chmod +x "$ROOT/build/agnostik_bench"
echo "Built build/agnostik_bench ($(wc -c < "$ROOT/build/agnostik_bench") bytes)"
echo ""

"$ROOT/build/agnostik_bench"
