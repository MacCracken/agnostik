#!/usr/bin/env bash
# Documentation gate: every fn in src/ has a doc comment. `cyrius doc --check`
# counts a fn as documented when a `#` comment sits directly above it.
#
# From v1.6.3 to v1.9.0 this compared src/ against a grandfathered list of
# undocumented fns (docs/undocumented.baseline, 860 at its peak). The v1.9.0
# burn-down documented all of them, so the baseline is gone and the rule is
# absolute: one undocumented fn in src/ fails the gate.
#
# Usage:
#   scripts/doc-debt.sh [check]    # CI gate

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

case "${1:-check}" in
  check) ;;
  *) echo "Usage: $0 [check]"; exit 2 ;;
esac

# `cyrius doc --check` exits with the file's undocumented count, so its status
# is not an error here. LC_ALL=C keeps the order identical on every host.
missing=$(for f in src/*.cyr; do
    m=$(basename "$f" .cyr)
    { cyrius doc --check "$f" 2>&1 || true; } | sed -n "s/^  undocumented: \(.*\)\$/$m::\1/p"
done | LC_ALL=C sort)

if [ -z "$missing" ]; then
    echo "ok: every fn in src/ is documented"
    exit 0
fi
echo "FAIL: fns in src/ without a doc comment:"
printf '%s\n' "$missing" | sed 's/^/  + /'
echo "Document each with a # comment directly above the fn."
exit 1
