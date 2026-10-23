# Dependencies, toolchain, and testing

Versions below were checked on 2026-10-23. Pin exact versions in
`Cargo.toml`/`Cargo.lock`; re-check before implementation if a newer stable
release exists. Do not adopt release candidates (`ratatui-image 12.0.0-rc` is
not the target).

## Toolchain

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = ["aarch64-unknown-linux-gnu"]
```

- Edition: `2024`.
- MSRV: **1.90** (edition 2024 requires 1.85; `image` 0.25.10 requires 1.88;
  ratatui 0.30.2 requires 1.88).
- Current development toolchain: 1.98.1.
- Primary target: `aarch64-unknown-linux-gnu`; CI also builds
  `x86_64-unknown-linux-gnu`.

## Workspace dependencies

```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.package]
edition = "2024"
rust-version = "1.90"
license = "GPL-3.0-or-later"
repository = "https://github.com/prjctimg/gtw"
```

### UI and rendering

```toml
ratatui = { version = "0.30.2", features = [
  "crossterm",
  "all-widgets",
  "unstable-rendered-line-info",
  "underline-color",
] }
crossterm = { version = "0.29", features = ["bracketed-paste", "event-stream"] }
tui-big-text = "0.8.10"
tachyonfx = "0.25.2"
ratatui-image = { version = "11.1.0", default-features = false, features = [
  "crossterm",
  "image-defaults",
] }
syntect = { version = "5.3", default-features = false, features = [
  "default-fancy",
  "default-syntaxes",
  "default-themes",
] }
textwrap = "0.16"
unicode-width = "0.2"
unicode-segmentation = "1.12"
```

`default-fancy` uses the pure-Rust regex engine; do not enable the `onig`
defaults.

### HTTP, parsing, data

```toml
tokio = { version = "1.54", features = [
  "rt-multi-thread", "macros", "sync", "time", "net", "io-util", "fs", "signal",
] }
tokio-util = { version = "0.7", features = ["rt"] }
futures = "0.3"
bytes = "1"
reqwest = { version = "0.13.5", default-features = false, features = [
  "rustls", "http2", "json", "stream", "gzip", "brotli", "charset",
] }
scraper = "0.29"
url = "2.5"
percent-encoding = "2.3"
mime = "0.3"
mime_guess = "2.0"
html-escape = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
```

### Media

```toml
image = { version = "0.25.10", default-features = false, features = [
  "jpeg", "png", "gif", "webp", "tiff", "rayon",
] }
resvg = { version = "0.48.1", default-features = false, features = [
  "text", "system-fonts", "raster-images",
] }
symphonia = { version = "0.6.1", default-features = false, features = [
  "all-codecs", "all-formats", "all-meta", "opt-simd",
] }
rodio = { version = "0.23", default-features = false, features = ["playback"] }
rubato = "1"
zvidlib = { version = "0.5.0", default-features = false, features = [
  "vp9-decoder", "vp8-decoder", "opus-decoder", "vorbis-decoder",
] }
```

- Do not enable rodio’s `symphonia-all`; it pins Symphonia 0.5. Feed rodio a
  custom `Source` from Symphonia 0.6 instead.
- zvidlib’s container support is a base dependency; decoder-only features keep
  hardware/encoder code out.
- No `ffmpeg*` crate. CI must grep `Cargo.lock` and fail on matches.

### Storage

```toml
rusqlite = { version = "0.40.2", features = ["bundled-full", "backup", "functions"] }
zstd = "0.13"
blake3 = "1.8"
fs2 = "0.4"
lru = "0.16"
```

Use `PRAGMA user_version` migrations; no separate migration crate is required.

### Auth, config, logging, CLI

```toml
oauth2 = { version = "5.0", default-features = false, features = ["rustls-tls", "timing-resistant-secret-traits"] }
keyring = "3"
rpassword = "7"
zeroize = { version = "1", features = ["derive"] }
secrecy = "0.10"
webbrowser = "1"
base64 = "0.22"
sha2 = "0.10"
rand = "0.9"
clap = { version = "4.5", features = ["derive"] }
clap_complete = "4.5"
color-eyre = "0.6"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
directories = "6"
arboard = "3"
```

`oauth2` may be replaced with a small hand-written PKCE client if its API
fights the loopback flow; the protocol is more important than the crate. Never
store a client secret.

### Dev/test

```toml
insta = { version = "1", features = ["yaml"] }
wiremock = "0.6"
tempfile = "3"
tokio-test = "0.4"
pretty_assertions = "1"
proptest = "1"
criterion = { version = "0.7", features = ["html_reports"] }
assert_cmd = "2"
predicates = "3"
```

## Test strategy

### Unit

- `Loadable<T>` transitions and request-id stale-result rejection.
- Theme derivation, contrast, color parsing, blend/readable-fg.
- Link classification and scheme rejection.
- Title/URL normalization, redirect handling.
- HTML sanitization (ANSI, bidi controls, scripts, event handlers).
- Cache key construction and eviction ordering.
- Edit conflict state machine.
- Token redaction.

### Fixtures

`tests/fixtures/` contains real HTML captured from Wikimedia (with attribution
and license notes):

- `rust_programming_language.html` (~500 KB)
- an article with a wide table and infobox
- an article with references and math
- a gallery with images/SVG
- an article with audio and video
- disambiguation, red link, protected page, redirect
- malformed/truncated HTML
- localized/non-Latin title (CJK, RTL)
- a page with 200+ sections

Fixtures are trimmed when practical, but at least one full 500 KB fixture is
kept for performance.

### Integration

- `wiremock` server for REST/Action API; no live network in CI.
- Simulate latency, 304, 429 with Retry-After, 500, disconnect, malformed JSON,
  redirect loop, edit conflict, badtoken, captcha.
- Fake clock for TTL/stale/backoff.
- Terminal test harness: render into `TestBackend` at 80×24, 120×40, 200×60,
  and 40×15; snapshot every screen and loading state.
- Input responsiveness test: start a 500 KB parse, inject keys, assert they are
  handled before the parse completes.

### Performance

- Criterion benchmarks for HTML extraction, IR layout, syntax highlighting,
  and image resize.
- Budgets: startup warm < 300 ms; article first content < 1 s warm / < 3 s
  network; idle CPU near zero; no allocation storms while scrolling; video
  memory bounded.
- Use `dhat` or a simple allocator counter for long-running leak tests.

### CI

`.github/workflows/ci.yml`:

1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --workspace`
4. `cargo build --workspace --release`
5. `cargo deny check` (licenses, advisories, bans)
6. no-ffmpeg check:
   ```bash
   ! grep -E '^name = "ffmpeg|ffmpeg-sidecar|ffmpeg-the-third' Cargo.lock
   ! grep -RIn --include='build.rs' -E 'ffmpeg|libav' crates
   ```
7. Linux x86_64 + aarch64 jobs (aarch64 via cross or native runner).
8. Optional nightly `cargo test --features unstable` for fuzz targets.

Do not run live Wikimedia requests in CI. A separate, manually triggered smoke
test may use a real API with a clear User-Agent and rate limits.

## Manual acceptance checklist

- Terminal: Ghostty (Kitty), Kitty, WezTerm (sixel/iTerm2), tmux, a
  halfblock-only terminal, and a 40×15 terminal.
- Keyboard only: search, open, scroll, TOC, back/forward, bookmark, peek,
  media, login, edit, quit.
- Mouse: wheel, click, hover peek, tab click.
- Offline: disable network, reopen cached article, search local, pin bookmark.
- Media: image, SVG, GIF, audio, video, seek, pause, external fallback.
- Auth: OAuth success/cancel/timeout; bot password + 2FA; logout.
- Edit: preview, diff, save, conflict, draft restore.
- Themes: all 16, light/dark, no-color, monochrome.
- Effects: auto/always/never; reduced motion; no flicker on cache hits.
- Crash/restart: draft recovery, terminal restored, no leaked temp files.
