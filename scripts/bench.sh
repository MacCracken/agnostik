#!/bin/sh
# Run the agnostik benchmark suite.
# Use scripts/bench-history.sh to also append to history CSV.
set -e
cyrius bench tests/bcyr/agnostik.bcyr
