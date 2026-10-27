# gtw development and packaging targets.

PREFIX ?= /usr/local
exec_prefix ?= $(PREFIX)
datarootdir ?= $(PREFIX)/share
datadir ?= $(datarootdir)
DESTDIR ?=
BINDIR ?= $(exec_prefix)/bin
MANDIR ?= $(datarootdir)/man
COMPLETIONSDIR ?= $(datadir)/bash-completion/completions
ZSH_COMPLETIONSDIR ?= $(datadir)/zsh/site-functions
FISH_COMPLETIONSDIR ?= $(datadir)/fish/vendor_completions.d
ELVISH_COMPLETIONSDIR ?= $(datadir)/elvish/lib
POWERSHELL_COMPLETIONSDIR ?= $(datadir)/powershell/Modules
ANDROID_API ?= 27

VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TARGET_DIR := target/release

.PHONY: all release test check clean man completions install stage deb rpm \
	termux termux-elf termux-clean deb-termux

all:
	cargo build

release:
	cargo build --release

test:
	cargo test --workspace

check:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --workspace
	cargo deny check

clean:
	cargo clean

man: release
	./scripts/build/manpages.sh artifacts

completions: release
	./scripts/build/completions.sh "$(CURDIR)/artifacts"

install: release man completions
	install -Dm 0755 $(TARGET_DIR)/gtw $(DESTDIR)$(BINDIR)/gtw
	install -Dm 0644 artifacts/man/gtw.1 $(DESTDIR)$(MANDIR)/man1/gtw.1
	install -Dm 0644 artifacts/completions/gtw.bash $(DESTDIR)$(COMPLETIONSDIR)/gtw
	install -Dm 0644 artifacts/completions/_gtw $(DESTDIR)$(ZSH_COMPLETIONSDIR)/_gtw
	install -Dm 0644 artifacts/completions/gtw.fish $(DESTDIR)$(FISH_COMPLETIONSDIR)/gtw.fish
	install -Dm 0644 artifacts/completions/gtw.elv $(DESTDIR)$(ELVISH_COMPLETIONSDIR)/gtw.elv
	install -Dm 0644 artifacts/completions/gtw.ps1 $(DESTDIR)$(POWERSHELL_COMPLETIONSDIR)/gtw.ps1
	install -Dm 0644 LICENSE $(DESTDIR)$(datadir)/licenses/gtw/LICENSE

stage: release man completions
	rm -rf stage
	mkdir -p stage/usr/bin stage/usr/share/man/man1 \
		stage/usr/share/bash-completion/completions \
		stage/usr/share/zsh/site-functions \
		stage/usr/share/fish/vendor_completions.d \
		stage/usr/share/elvish/lib \
		stage/usr/share/powershell/Modules \
		stage/usr/share/licenses/gtw
	cp $(TARGET_DIR)/gtw stage/usr/bin/
	cp artifacts/man/gtw.1 stage/usr/share/man/man1/
	cp artifacts/completions/gtw.bash stage/usr/share/bash-completion/completions/gtw
	cp artifacts/completions/_gtw stage/usr/share/zsh/site-functions/_gtw
	cp artifacts/completions/gtw.fish stage/usr/share/fish/vendor_completions.d/
	cp artifacts/completions/gtw.elv stage/usr/share/elvish/lib/
	cp artifacts/completions/gtw.ps1 stage/usr/share/powershell/Modules/
	cp LICENSE stage/usr/share/licenses/gtw/

deb: stage
	@command -v dpkg-deb >/dev/null 2>&1 || { echo "dpkg-deb not found"; exit 1; }
	./scripts/build/debian-deb.sh "$(VERSION)" "$$(dpkg --print-architecture)" stage \
		"dist/gtw_$(VERSION)_$$(dpkg --print-architecture).deb"

rpm: stage
	@command -v rpmbuild >/dev/null 2>&1 || { echo "rpmbuild not found"; exit 1; }
	rpmbuild -bb --define "_topdir $(CURDIR)/rpmbuild" \
		--define "gtw_version $(VERSION)" \
		--define "gtw_staging $(CURDIR)/stage" \
		--define "gtw_changelog_date $$(date -u +'%a %b %d %Y')" \
		dist/rpm/gtw.spec

termux:
	@command -v cargo-ndk >/dev/null 2>&1 || { echo "cargo-ndk not found: cargo install cargo-ndk"; exit 1; }
	CARGO_INCREMENTAL=0 cargo ndk -t arm64-v8a -P $(ANDROID_API) build --release

termux-elf:
	@command -v termux-elf-cleaner >/dev/null 2>&1 || \
		{ echo "termux-elf-cleaner not found: pip install termux-elf-cleaner"; exit 1; }
	termux-elf-cleaner target/aarch64-linux-android/release/gtw

termux-clean:
	rm -rf target/aarch64-linux-android

deb-termux: termux termux-elf
	@command -v termux-create-package >/dev/null 2>&1 || \
		{ echo "termux-create-package not found: pip install termux-create-package"; exit 1; }
	./scripts/build/termux-deb.sh
