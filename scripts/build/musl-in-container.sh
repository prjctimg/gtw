#!/usr/bin/env bash
set -euo pipefail

platform="${1:?platform required}"
version="${2:?version required}"
arch="${3:?arch required}"

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo"

export RUSTFLAGS="-C target-feature=-crt-static -C linker=gcc -C link-arg=-fuse-ld=mold"

apk add --no-cache \
  ca-certificates curl cmake musl-dev git \
  pkgconfig alsa-lib-dev \
  gcc g++ binutils build-base clang lld mold

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
export PATH="$HOME/.cargo/bin:$PATH"

cargo build --release

./scripts/build/manpages.sh artifacts
./scripts/build/completions.sh "$(pwd)/artifacts"

root="gtw-${platform}"
mkdir -p release-assets \
  "$root/bin" "$root/man/man1" "$root/completions"
cp target/release/gtw "$root/bin/"
cp artifacts/man/*.1 "$root/man/man1/"
cp artifacts/completions/* "$root/completions/"
cp LICENSE "$root/"
cp install.sh uninstall.sh "$root/"
tar czf "release-assets/gtw-${platform}.tar.gz" "$root"

mkdir -p stage/usr/bin \
  stage/usr/share/man/man1 \
  stage/usr/share/bash-completion/completions \
  stage/usr/share/zsh/site-functions \
  stage/usr/share/fish/vendor_completions.d \
  stage/usr/share/elvish/lib \
  stage/usr/share/powershell/Modules \
  stage/usr/share/licenses/gtw
cp target/release/gtw stage/usr/bin/
cp artifacts/man/*.1 stage/usr/share/man/man1/
cp artifacts/completions/gtw.bash stage/usr/share/bash-completion/completions/gtw
cp artifacts/completions/_gtw stage/usr/share/zsh/site-functions/_gtw
cp artifacts/completions/gtw.fish stage/usr/share/fish/vendor_completions.d/
cp artifacts/completions/gtw.elv stage/usr/share/elvish/lib/
cp artifacts/completions/gtw.ps1 stage/usr/share/powershell/Modules/
cp LICENSE stage/usr/share/licenses/gtw/

bash "$repo/scripts/build/alpine-apk.sh" "$version" "$arch" stage \
  "release-assets/gtw-${version}-r0-${arch}.apk"

echo "musl build complete: $(ls -1 release-assets)"
