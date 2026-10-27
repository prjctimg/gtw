#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="${1:-$(grep -m1 '^version = ' "$REPO_ROOT/Cargo.toml" | sed 's/^version = "\(.*\)"/\1/')}"
ARCH="aarch64"

BIN_DIR="$REPO_ROOT/target/aarch64-linux-android/release"
FILES_DIR="$REPO_ROOT/target/aarch64-linux-android/termux-pkg/files"

if [ ! -f "$BIN_DIR/gtw" ]; then
  echo "Error: $BIN_DIR/gtw not found."
  echo "Run 'make termux' first."
  exit 1
fi

rm -rf "$FILES_DIR"
mkdir -p "$FILES_DIR/bin" "$FILES_DIR/share/man/man1"

cp "$BIN_DIR/gtw" "$FILES_DIR/bin/"

if [ -d "$REPO_ROOT/artifacts/man" ]; then
  cp "$REPO_ROOT"/artifacts/man/gtw.1 "$FILES_DIR/share/man/man1/" 2>/dev/null || true
fi

cd "$REPO_ROOT"
termux-create-package \
  --pkg-version "$VERSION" \
  --pkg-arch "$ARCH" \
  dist/termux/gtw.yml

DEB="gtw_${VERSION}_${ARCH}.deb"
if [ -f "$DEB" ]; then
  mv "$DEB" "$(dirname "$DEB")/gtw-android-${VERSION}-${ARCH}.deb"
  echo "Built: $REPO_ROOT/gtw-android-${VERSION}-${ARCH}.deb"
else
  echo "Built: $(ls "$REPO_ROOT"/*.deb 2>/dev/null | head -1)"
fi
