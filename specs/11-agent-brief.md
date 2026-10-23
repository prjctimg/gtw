# Coding-agent brief

This file is also present at the repository root as `AGENTS.md`.

## Read first

1. `specs/README.md`
2. `specs/01-architecture.md`
3. `specs/06-loading-states.md`
4. `specs/09-roadmap.md`
5. `specs/10-dependencies-and-testing.md`

## Non-negotiables

- Rust edition 2024, ratatui 0.30, crossterm 0.29.
- No `ffmpeg` binary/crate/runtime dependency. CI check must enforce this.
- Audio: Symphonia + rodio. Video: zvidlib pure-Rust pipeline, fallback
  `rusty_vp9` + `matroska-demuxer` only after the M0.5 spike.
- Wordmark `gtw` with `tui-big-text` 0.8.
- Tachyonfx 0.25 for startup/page/image/feed transitions.
- All 16 themes from `themes.toml`; GPL-3.0.
- Every async surface uses `Loadable<T>` + `ActivityCenter`; no ad-hoc
  `loading: bool`.
- Keep the UI thread non-blocking; parse/layout/decode off-thread.
- Every failure is a rendered state with retry/cancel/fallback where possible.
- No secrets in logs or config; use the OS keyring.

## Work order

1. Bootstrap the workspace and copy specs.
2. M0 shell + themes + wordmark + effects + loading demo.
3. M0.5 media spike; do not start M3 UI before it passes.
4. M1 read-only TUI; make it genuinely usable before adding editing.
5. M2 images/peeks; M3 media; M4 auth/bookmarks/watchlist; M5 editing;
   M6 feeds/polish.

## Quality gates

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo deny check`
- no-ffmpeg grep
- loading-state snapshots for every screen
- 500 KB article input-responsiveness test
- no live network in tests

## Commit style

Conventional commits, one behavior per commit. Example:

```text
feat(core): add Loadable state machine
feat(render): render infobox and wide tables
fix(cache): keep pinned media during eviction
test(app): snapshot article loading states
```

Do not commit `target/`, cache data, tokens, `.env`, or editor swap files.
