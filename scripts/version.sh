#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <new-version>" >&2
  exit 1
fi

NEW="$1"
if ! [[ "$NEW" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: version must be X.Y.Z (got '$NEW')" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

sed -i -E '0,/^version = "[0-9.]+"$/s//version = "'"$NEW"'"/' "$ROOT/Cargo.toml"
sed -i -E 's/(Version: )[0-9]+\.[0-9]+\.[0-9]+/\1'"$NEW"'/' "$ROOT/dist/termux/gtw.yml"
sed -i -E 's/(gtw_)[0-9]+\.[0-9]+\.[0-9]+(_)/\1'"$NEW"'\2/g' "$ROOT/README.md"
sed -i -E 's/(gtw-)[0-9]+\.[0-9]+\.[0-9]+/\1'"$NEW"'/g' "$ROOT/README.md"

# Keep Cargo.lock in sync without a full rebuild.
cargo metadata --format-version=1 >/dev/null 2>&1 || true

echo "Bumped version to $NEW in:"
echo "  Cargo.toml"
echo "  dist/termux/gtw.yml"
echo "  README.md package examples"
echo "Review the diff, then commit."
