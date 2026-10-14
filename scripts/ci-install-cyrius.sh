#!/usr/bin/env bash
# Install the pinned Cyrius toolchain from its GitHub release, VERIFIED, into
# the version-pinned layout cyrius resolves:
#   ~/.cyrius/versions/<V>/{bin,lib}, with ~/.cyrius/{bin,lib} symlinked to it
# Used by .github/workflows/ci.yml and release.yml; runs the same locally.
#
# Verification, fail-closed, before anything is unpacked:
#   1. SHA256SUMS.sig must be a valid Ed25519 signature over SHA256SUMS under
#      the release key PINNED below — not the key the release ships, which
#      anyone able to replace the tarball could replace too. Checked with
#      stock `openssl pkeyutl -rawin` (OpenSSL 3.0+), so no cyrsign is needed.
#   2. The tarball's SHA-256 must match its line in that SHA256SUMS.
# A same-origin `.sha256` sidecar alone only detects corruption. cyrius has
# signed SHA256SUMS since 6.2.31, and its own installers refuse a stripped
# signature since 6.6.20 (CYRIUS-2026-0035); this keeps CI to that bar.
#
# Usage: scripts/ci-install-cyrius.sh [VERSION]
#   VERSION defaults to $CYRIUS_VERSION, else the `cyrius = "X.Y.Z"` pin in
#   cyrius.cyml (the single source of truth — never hardcode it in YAML).
# Environment: CYRIUS_HOME (default ~/.cyrius). Under GitHub Actions the bin
# dir is appended to $GITHUB_PATH and CYRIUS_HOME to $GITHUB_ENV.

set -euo pipefail

# The cyrius release-signing key (Ed25519, raw 32 bytes as hex). Provenance:
# byte-identical in three places at pinning time (2026-10-14, cyrius 6.6.20):
# the release asset cyrius-release.ed25519.pub, keys/cyrius-release.ed25519.pub
# in the cyrius repo, and CYRIUS_RELEASE_PUBKEY in cyrius's scripts/install.sh.
# If cyrius rotates its key, this check fails closed: confirm the new key out
# of band (the cyrius repo's keys/ and install.sh), then update it here.
CYRIUS_RELEASE_PUBKEY="adbde6b11ccf8d86dc760387fa7f4dfbe3942fa318e459fb6e62d1536e254008"
REPO="MacCracken/cyrius"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${1:-${CYRIUS_VERSION:-$(grep -oP '(?<=^cyrius = ")[^"]+' "$REPO_ROOT/cyrius.cyml")}}"
echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$' || {
    echo "ci-install-cyrius: '$VERSION' is not a X.Y.Z cyrius version" >&2; exit 2; }
CYRIUS_HOME="${CYRIUS_HOME:-$HOME/.cyrius}"
TARBALL="cyrius-$VERSION-x86_64-linux.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION"

for tool in curl openssl sha256sum xxd tar; do
    command -v "$tool" > /dev/null || { echo "ci-install-cyrius: needs $tool" >&2; exit 2; }
done

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cd "$WORK"

echo "Installing Cyrius $VERSION"
for f in "$TARBALL" SHA256SUMS SHA256SUMS.sig; do
    curl -fsSL --retry 3 -o "$f" "$URL/$f" || {
        echo "ci-install-cyrius: could not fetch $URL/$f" >&2; exit 1; }
done

# 1. Signature over SHA256SUMS, against the pinned key. The key is wrapped in
#    the fixed SubjectPublicKeyInfo DER prefix for Ed25519 (RFC 8410).
printf '%s' "302a300506032b6570032100$CYRIUS_RELEASE_PUBKEY" | xxd -r -p > release.pub.der
openssl pkey -pubin -inform DER -in release.pub.der -out release.pub.pem
if ! openssl pkeyutl -verify -pubin -inkey release.pub.pem -rawin \
        -in SHA256SUMS -sigfile SHA256SUMS.sig > /dev/null 2>&1; then
    echo "ci-install-cyrius: SHA256SUMS.sig does not verify against the pinned cyrius release key." >&2
    echo "  Either the release was tampered with, or cyrius rotated its key (see this script's header)." >&2
    exit 1
fi
echo "signature: SHA256SUMS verified against the pinned release key"

# 2. The tarball against its line in the now-trusted SHA256SUMS. Exactly one
#    line must name it; `sha256sum -c` then checks the bytes.
lines="$(awk -v f="$TARBALL" '$2 == f || $2 == "*" f' SHA256SUMS)"
if [ "$(printf '%s\n' "$lines" | grep -c .)" -ne 1 ]; then
    echo "ci-install-cyrius: SHA256SUMS has no single entry for $TARBALL" >&2; exit 1
fi
printf '%s\n' "$lines" | sha256sum -c --strict - || {
    echo "ci-install-cyrius: $TARBALL does not match the signed SHA256SUMS" >&2; exit 1; }

# Unpack and lay out the version-pinned toolchain.
tar xzf "$TARBALL"
SRC="$WORK/cyrius-$VERSION-x86_64-linux"
[ -d "$SRC/bin" ] && [ -d "$SRC/lib" ] || {
    echo "ci-install-cyrius: $TARBALL has no bin/ and lib/" >&2; exit 1; }
VERSIONED="$CYRIUS_HOME/versions/$VERSION"
mkdir -p "$VERSIONED/bin" "$VERSIONED/lib"
cp "$SRC/bin/"* "$VERSIONED/bin/"
cp -r "$SRC/lib/"* "$VERSIONED/lib/"
chmod +x "$VERSIONED/bin/"*
ln -sfn "$VERSIONED/bin" "$CYRIUS_HOME/bin"
ln -sfn "$VERSIONED/lib" "$CYRIUS_HOME/lib"
echo "$VERSION" > "$CYRIUS_HOME/current"

if [ -n "${GITHUB_PATH:-}" ]; then echo "$CYRIUS_HOME/bin" >> "$GITHUB_PATH"; fi
if [ -n "${GITHUB_ENV:-}" ]; then echo "CYRIUS_HOME=$CYRIUS_HOME" >> "$GITHUB_ENV"; fi
echo "installed: $VERSIONED ($(ls "$VERSIONED/bin" | wc -l) tools)"
