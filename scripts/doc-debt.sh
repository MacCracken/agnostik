#!/usr/bin/env bash
# Documentation-debt gate. `cyrius doc --check` counts a fn as documented
# when a `#` comment sits directly above it. The fns in src/ that lack one
# are recorded, one `module::fn` per line, in docs/undocumented.baseline.
# That list is grandfathered debt: `check` fails when an undocumented fn
# appears that is not on it, so every new public fn ships documented, and
# when a listed fn has since been documented, so the committed count only
# moves when someone records it.
#
# Usage:
#   scripts/doc-debt.sh check    # CI gate: live list must equal the baseline
#   scripts/doc-debt.sh update   # rewrite the baseline after documenting fns

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASELINE="$REPO_ROOT/docs/undocumented.baseline"
LIVE="$(mktemp)"
trap 'rm -f "$LIVE"' EXIT

cd "$REPO_ROOT"

# `cyrius doc --check` exits with the file's undocumented count, so its
# status is not an error here. LC_ALL=C keeps the sort order identical on
# every host (see scripts/api-surface.sh).
for f in src/*.cyr; do
    m=$(basename "$f" .cyr)
    { cyrius doc --check "$f" 2>&1 || true; } | sed -n "s/^  undocumented: \(.*\)\$/$m::\1/p"
done | LC_ALL=C sort > "$LIVE"

case "${1:-check}" in
  update)
    cp "$LIVE" "$BASELINE"
    echo "baseline updated: $(wc -l < "$BASELINE") undocumented fns"
    ;;
  check)
    if [ ! -f "$BASELINE" ]; then
      echo "doc-debt: no baseline at docs/undocumented.baseline — run: scripts/doc-debt.sh update"
      exit 2
    fi
    new=$(LC_ALL=C comm -13 "$BASELINE" "$LIVE")
    done_=$(LC_ALL=C comm -23 "$BASELINE" "$LIVE")
    if [ -z "$new" ] && [ -z "$done_" ]; then
      echo "ok: $(wc -l < "$LIVE") undocumented fns, matching the baseline"
      exit 0
    fi
    if [ -n "$new" ]; then
      echo "FAIL: fns in src/ without a doc comment, not in the baseline:"
      printf '%s\n' "$new" | sed 's/^/  + /'
      echo "Document each with a # comment directly above the fn."
    fi
    if [ -n "$done_" ]; then
      echo "FAIL: baseline lists fns that are now documented (or gone):"
      printf '%s\n' "$done_" | sed 's/^/  - /'
      echo "Record the progress: scripts/doc-debt.sh update"
    fi
    exit 1
    ;;
  *)
    echo "Usage: $0 {check|update}"
    exit 2
    ;;
esac
