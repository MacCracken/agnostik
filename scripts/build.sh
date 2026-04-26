#!/bin/sh
# Build the agnostik test harness binary using the Cyrius CLI.
# The manifest (cyrius.cyml) resolves stdlib + dep includes; this
# script is just a convenience wrapper.
set -e
cyrius deps
mkdir -p build
CYRIUS_DCE=1 cyrius build src/main.cyr build/agnostik
echo "agnostik: $(wc -c < build/agnostik) bytes"
