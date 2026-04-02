#!/bin/bash
set -euo pipefail
cargo bench --all-features --bench benchmarks 2>&1 | grep -E 'time:|Benchmarking' | tee -a bench-history.log
