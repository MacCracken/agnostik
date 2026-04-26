#!/bin/sh
CC="${1:-./build/cc2}"
echo "=== agnostik tests ==="
cat src/main.cyr | "$CC" > /tmp/agnostik_test && chmod +x /tmp/agnostik_test && /tmp/agnostik_test
echo "exit: $?"
rm -f /tmp/agnostik_test
