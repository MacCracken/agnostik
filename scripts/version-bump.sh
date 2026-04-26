#!/usr/bin/env bash
# Bump the project version across all files.
# Usage: ./scripts/version-bump.sh <version>
#
# cyrius.cyml pulls version via ${file:VERSION}, so it doesn't need
# editing here. The CHANGELOG header is stamped with today's date.
set -euo pipefail

[ $# -ne 1 ] && echo "Usage: $0 <version>" && exit 1
NEW_VERSION="$1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# 1. VERSION file
echo "$NEW_VERSION" > "$REPO_ROOT/VERSION"

# 2. CHANGELOG.md — stamp [Unreleased] as new version with today's date
DATE=$(date -u +%Y-%m-%d)
if grep -q "^## \[Unreleased\]" "$REPO_ROOT/CHANGELOG.md"; then
    sed -i "s/^## \[Unreleased\]/## [Unreleased]\n\n## [${NEW_VERSION}] - ${DATE}/" \
        "$REPO_ROOT/CHANGELOG.md"
fi

echo "Bumped to ${NEW_VERSION} (${DATE})."
echo ""
echo "To release:"
echo "  git add -A && git commit -m 'release: v${NEW_VERSION}'"
echo "  git tag v${NEW_VERSION}"
echo "  git push origin main --tags"
