#!/usr/bin/env bash
# Wraps cyrius_api_surface so the snapshot tracks only agnostik's
# public-fn surface (~859 fns) — not stdlib platform peers (alloc_macos,
# syscalls_aarch64_linux, etc.) that vary by build environment.
#
# Usage:
#   scripts/api-surface.sh check    # CI gate: generate live, filter, diff
#   scripts/api-surface.sh update   # Regenerate docs/api-surface.snapshot

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SNAPSHOT="$REPO_ROOT/docs/api-surface.snapshot"
LIVE="$(mktemp)"
trap 'rm -f "$LIVE" "$LIVE.agnostik"' EXIT

# Modules under src/ — the agnostik public surface. Any new src/<name>.cyr
# adds <name> to this list.
AGNOSTIK_MODULES='^(agent|audit|classification|config|error|hardware|lib|llm|main|secrets|security|telemetry|types|validation)::'

# `sort` collation differs by locale (Arch defaults to dictionary
# order; ubuntu-latest CI defaults to byte order); pin LC_ALL=C in
# both update + check paths so the snapshot is environment-portable.

cmd="${1:-check}"

case "$cmd" in
  update)
    cyrius_api_surface --update --snapshot="$LIVE"
    grep -E "$AGNOSTIK_MODULES" "$LIVE" | LC_ALL=C sort > "$SNAPSHOT"
    echo "snapshot updated: $(wc -l < "$SNAPSHOT") agnostik public fns"
    ;;
  check)
    cyrius_api_surface --update --snapshot="$LIVE" > /dev/null
    grep -E "$AGNOSTIK_MODULES" "$LIVE" | LC_ALL=C sort > "$LIVE.agnostik"
    if diff -q "$SNAPSHOT" "$LIVE.agnostik" > /dev/null; then
      echo "ok: $(wc -l < "$SNAPSHOT") agnostik public fns, surface matches snapshot"
    else
      echo "BREAKING: agnostik public-fn surface drifted from snapshot:"
      echo "(- = removed since snapshot; + = added since snapshot)"
      diff -u "$SNAPSHOT" "$LIVE.agnostik" | tail -n +3 | head -200
      echo ""
      echo "If intentional, regenerate:"
      echo "  scripts/api-surface.sh update"
      exit 1
    fi
    ;;
  *)
    echo "Usage: $0 {check|update}"
    exit 2
    ;;
esac
