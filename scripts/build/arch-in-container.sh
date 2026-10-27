#!/usr/bin/env bash
set -euo pipefail

platform="${1:?platform required}"
version="${2:?version required}"
arch="${3:?arch required}"

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo"

export RUSTFLAGS="-C linker=gcc -C link-arg=-fuse-ld=mold"

PACMAN_PACKAGES="base-devel rust alsa-lib cmake clang lld mold git libarchive zstd"

retry() {
    local what="$1"; shift
    local attempt
    for attempt in 1 2 3 4 5; do
        if "$@"; then
            return 0
        fi
        if [ "$attempt" -lt 5 ]; then
            echo "::warning::$what failed (attempt ${attempt}/5); retrying in 10s" >&2
            sleep 10
        fi
    done
    echo "::error::$what failed after 5 attempts" >&2
    return 1
}

retry "pacman database sync" pacman -Syu --noconfirm
# shellcheck disable=SC2086
retry "pacman install" pacman -S --noconfirm --needed $PACMAN_PACKAGES

cargo build --release

./scripts/build/manpages.sh artifacts
./scripts/build/completions.sh "$(pwd)/artifacts"

root="gtw-arch-${arch}"
mkdir -p release-assets "$root/bin" "$root/man/man1" "$root/completions"
cp target/release/gtw "$root/bin/"
cp artifacts/man/*.1 "$root/man/man1/"
cp artifacts/completions/* "$root/completions/"
cp LICENSE "$root/"
cp install.sh uninstall.sh "$root/"
tar czf "release-assets/gtw-arch-${arch}.tar.gz" "$root"

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

bash "$repo/scripts/build/arch-pkg.sh" "$version" "$arch" stage \
  "release-assets/gtw-${version}-${arch}.pkg.tar.zst"

echo "arch build complete ($platform): $(ls -1 release-assets)"
