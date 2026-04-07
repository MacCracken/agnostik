#!/bin/sh
set -e
CC="${CC:-$HOME/.cyrius/bin/cc2}"
mkdir -p build
# Concatenate stdlib (via includes in a wrapper) + agnostik modules + main
(
  command cat lib/syscalls_min.cyr lib/string.cyr lib/alloc.cyr lib/fmt.cyr \
    lib/str.cyr lib/vec.cyr lib/hashmap.cyr lib/tagged.cyr lib/assert.cyr
  # Inline now_ns
  echo 'fn now_ns() { var ts[16]; syscall(228, 4, &ts); return load64(&ts) * 1000000000 + load64(&ts + 8); }'
  # Agnostik modules
  command cat src/error.cyr src/types.cyr src/agent.cyr src/security.cyr \
    src/telemetry.cyr src/audit.cyr src/llm.cyr src/secrets.cyr \
    src/config.cyr src/classification.cyr src/validation.cyr src/hardware.cyr
  # Test harness (everything after the includes in main.cyr)
  # Extract test functions from main.cyr (skip include lines and now_ns block)
  command sed '/^include /d;/^fn now_ns/,/^}/d;/^# agnostik/d;/^# Cyrius/d;/^# Inline/d' src/main.cyr
) | "$CC" > build/agnostik_test
chmod +x build/agnostik_test
echo "=== Built build/agnostik_test ($(wc -c < build/agnostik_test) bytes) ==="
