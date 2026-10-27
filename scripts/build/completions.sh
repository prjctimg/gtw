#!/usr/bin/env bash
# Generate shell completions for `gtw` into <outdir>/completions.
set -euo pipefail

outdir="${1:?usage: completions.sh <outdir>}"
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo"

path="${GTW_BIN:-target/release/gtw}"
if [[ ! -x "$path" ]]; then
  echo "completions.sh: $path not built; run 'cargo build --release' first" >&2
  exit 1
fi

dest="$outdir/completions"
mkdir -p "$dest"

"$path" --completions bash >"$dest/gtw.bash"
"$path" --completions zsh >"$dest/_gtw"
"$path" --completions fish >"$dest/gtw.fish"
"$path" --completions elvish >"$dest/gtw.elv"
"$path" --completions powershell >"$dest/gtw.ps1"
