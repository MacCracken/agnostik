#!/bin/sh
set -e
CC="${CC:-$HOME/.cyrius/bin/cc2}"
mkdir -p build
cat src/main.cyr | "$CC" > build/agnostik_test
chmod +x build/agnostik_test
echo "=== Built build/agnostik_test ($(wc -c < build/agnostik_test) bytes) ==="
