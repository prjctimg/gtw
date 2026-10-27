#!/usr/bin/env bash
# Generate the gtw man page from clap, so it cannot drift from the CLI.
set -euo pipefail

outdir="${1:-.}"
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo"

path="${GTW_BIN:-target/release/gtw}"
if [[ ! -x "$path" ]]; then
  echo "manpages.sh: $path not built; run 'cargo build --release' first" >&2
  exit 1
fi

mkdir -p "$outdir/man"
"$path" --man >"$outdir/man/gtw.1"
echo "Generated $outdir/man/gtw.1"
