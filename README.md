# gtw

A Wikipedia client for the terminal, built in Rust with
[Ratatui](https://ratatui.rs): cached reading, rich article rendering, inline
images/audio/video, link previews, bookmarks, daily feeds, authentication, and
editing.

> **Status:** specification phase. The repository currently contains the design
> and implementation spec in [`specs/`](specs/README.md). The Cargo workspace
> is created in the codespace as part of milestone M0.

## What it will do

- Read English and other Wikipedia editions from the REST + Action APIs.
- Render articles with headings, infoboxes, tables, quotes, references, code
  highlighting, and inline media.
- Preview wiki links on hover, like an LSP hover in Neovim.
- Cache articles and media for fast, offline reading.
- Browse bookmarks, history, watchlists, and the daily featured feed.
- Log in with OAuth 2.0 PKCE (bot-password fallback) and edit articles with
  preview, diff, conflict handling, and drafts.
- Stay responsive while fetching, parsing, laying out, and decoding.

## Non-negotiables

- Rust edition **2024**, Ratatui 0.30, crossterm 0.29.
- Elm-style `Msg` / `Cmd` update loop with Tokio.
- **No ffmpeg**: Symphonia for audio, zvidlib (pure Rust) for video.
- **`tui-big-text`** wordmark reading `gtw`.
- **tachyonfx** for startup, page-load, image-reveal, and feed transitions.
- Every async surface has explicit loading, stale, empty, offline, and error
  states via `Loadable<T>`.
- All 16 built-in themes adapted from
  [`prjctimg/gtm`](https://github.com/prjctimg/gtm).
- GPL-3.0.

## Specifications

Start here: **[`specs/README.md`](specs/README.md)**

| Document | Contents |
|---|---|
| [`specs/01-architecture.md`](specs/01-architecture.md) | Workspace, crates, update loop, concurrency |
| [`specs/02-api.md`](specs/02-api.md) | Wikimedia endpoints, caching, edit protocol |
| [`specs/03-media.md`](specs/03-media.md) | No-ffmpeg media pipeline and players |
| [`specs/04-ui-ux.md`](specs/04-ui-ux.md) | Screens, layout, keymap, editor |
| [`specs/05-themes-and-effects.md`](specs/05-themes-and-effects.md) | Wordmark, tachyonfx, themes |
| [`specs/06-loading-states.md`](specs/06-loading-states.md) | Complete loading-state matrix |
| [`specs/07-storage-and-cache.md`](specs/07-storage-and-cache.md) | SQLite, blobs, FTS5, offline |
| [`specs/08-security-and-auth.md`](specs/08-security-and-auth.md) | OAuth, secrets, edit safety |
| [`specs/09-roadmap.md`](specs/09-roadmap.md) | M0–M6 and acceptance gates |
| [`specs/10-dependencies-and-testing.md`](specs/10-dependencies-and-testing.md) | Crates, tests, CI |
| [`specs/themes.toml`](specs/themes.toml) | 16 theme palettes + user theme schema |

## Development

The coding agent should read [`AGENTS.md`](AGENTS.md) and
[`specs/09-roadmap.md`](specs/09-roadmap.md), then start with M0 in the
codespace:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

No live Wikimedia requests in CI; use the fixture and mock-server plan.

## License

GPL-3.0-or-later. See [`LICENSE`](LICENSE).

The theme palettes and wordmark sizing logic are adapted from
`prjctimg/gtm`, also GPL-3.0.
