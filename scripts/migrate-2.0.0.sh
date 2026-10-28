#!/usr/bin/env bash
# Migrate a consumer's sources to agnostik 2.0.0's accessor names.
#
# agnostik 2.0.0 renames its 706 field accessors from `<prefix>_<field>` to
# `<Struct>_<field>` / `<Struct>_set_<field>` (ADR-009), e.g.
# `amsg_content` -> `AgentMessage_content`, `secctx_set_run_as_user` ->
# `SecurityContext_set_run_as_user`. docs/migration/2.0.0-renames.tsv is the
# complete old -> new map; this script applies it as whole-word replacements.
#
# Usage: scripts/migrate-2.0.0.sh [--dry-run] DIR...
#   Rewrites *.cyr, *.tcyr and *.bcyr under each DIR in place, skipping DIR's
#   own top-level lib/ and build/, every .git/, and any agnostik.cyr
#   (`cyrius deps` re-vendors those). Sources under src/lib/ are migrated.
#   --dry-run prints how many replacements each file would get, writing nothing.
# Needs GNU sed (for \b). Review the diff afterwards: an identifier of your own
# that happens to equal an old accessor name is renamed too.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAP="$REPO_ROOT/docs/migration/2.0.0-renames.tsv"
[ -f "$MAP" ] || { echo "migrate-2.0.0: missing $MAP" >&2; exit 2; }
DRY=0
if [ "${1:-}" = "--dry-run" ]; then DRY=1; shift; fi
[ $# -ge 1 ] || { echo "usage: $0 [--dry-run] DIR..." >&2; exit 2; }

SED="$(mktemp)"
trap 'rm -f "$SED"' EXIT
# Longest names first, so no rule rewrites part of a longer old name.
awk -F'\t' 'NF == 2 { print length($1) "\t" $0 }' "$MAP" | sort -rn | cut -f2- |
    awk -F'\t' '{ printf "s/\\b%s\\b/%s/g\n", $1, $2 }' > "$SED"

total=0; files=0
# Only each DIR's own top-level lib/ (the vendored stdlib and deps) and build/
# are skipped: a project's sources may live under src/lib/.
find_sources() {
    local d
    for d in "$@"; do
        d="${d%/}"
        find "$d" \( -path "$d/lib" -o -path "$d/build" -o -name .git \) -prune -o \
             -type f \( -name '*.cyr' -o -name '*.tcyr' -o -name '*.bcyr' \) ! -name agnostik.cyr -print0
    done
}
while IFS= read -r -d '' f; do
    if [ "$DRY" = 1 ]; then
        n=$(sed -f "$SED" "$f" | diff "$f" - | grep -c '^>' || true)
        if [ "$n" -gt 0 ]; then echo "$n lines: $f"; total=$((total + n)); files=$((files + 1)); fi
    else
        before=$(md5sum < "$f")
        sed -i -f "$SED" "$f"
        if [ "$before" != "$(md5sum < "$f")" ]; then echo "migrated: $f"; files=$((files + 1)); fi
    fi
done < <(find_sources "$@")
if [ "$DRY" = 1 ]; then echo "migrate-2.0.0: $total changed line(s) in $files file(s) (dry run)"
else echo "migrate-2.0.0: $files file(s) rewritten; review the diff"; fi
