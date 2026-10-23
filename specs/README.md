# gtw — Wikipedia TUI client

`gtw` is a terminal client for Wikipedia with cached reading, rich article
rendering, inline media, link previews, bookmarks, daily feeds, authentication,
and editing.

**Status:** specification complete; implementation begins at milestone M0.
This bundle is the source of truth for the coding agent working in the
`prjctimg/gtw` checkout.

## Non-negotiable decisions

- Rust edition **2024**; Linux-first, portable core.
- TUI: **ratatui 0.30** + **crossterm 0.29**.
- Architecture: Elm-style `Msg` / `Cmd` update loop with Tokio.
- No `ffmpeg` binary, crate, or runtime dependency.
- Audio decoding: **Symphonia 0.6**; output: **rodio**.
- Video: pure-Rust **zvidlib 0.5** pipeline, with
  `rusty_vp9` + `matroska-demuxer` as the fallback spike.
- Wordmark: **`gtw`**, rendered with **`tui-big-text` 0.8**.
- Effects: **tachyonfx 0.25**, used for startup, page load, image reveal, and
  short UI transitions.
- Loading is a first-class state on every asynchronous surface. No ad-hoc
  `loading: bool` fields.
- Themes: all 16 built-ins copied/adapted from `prjctimg/gtm`.
- License: **GPL-3.0** (required because theme/logo logic is ported from
  `gtm`, also GPL-3.0).

## Documentation index

| File | Contents |
|---|---|
| `00-original-prompt.md` | Original feature request, preserved verbatim |
| `01-architecture.md` | Workspace, crates, update loop, concurrency, boundaries |
| `02-api.md` | Wikimedia REST/Action API map, caching headers, edit protocol |
| `03-media.md` | No-ffmpeg media pipeline, formats, caching, playback |
| `04-ui-ux.md` | Screens, layout, keymap, mouse, peeks, editor, accessibility |
| `05-themes-and-effects.md` | Wordmark, tachyonfx effects, 16 themes, config |
| `06-loading-states.md` | `Loadable<T>`, `ActivityCenter`, complete state matrix |
| `07-storage-and-cache.md` | SQLite schema, blobs, FTS5, eviction, offline |
| `08-security-and-auth.md` | OAuth PKCE, bot passwords, tokens, edit safety |
| `09-roadmap.md` | Milestones, acceptance gates, risks |
| `10-dependencies-and-testing.md` | Crate versions/features, test plan, CI |
| `11-agent-brief.md` | Instructions to copy to the repo root as `AGENTS.md` |
| `themes.toml` | All 16 built-in theme palettes + user-theme schema |

## Repository layout

```text
gtw/
├── Cargo.toml                 # workspace, edition 2024, GPL-3.0
├── README.md
├── LICENSE                    # GPL-3.0
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── .cargo/config.toml
├── .github/workflows/ci.yml
├── crates/
│   ├── gtw-core/              # domain types, states, Msg/Cmd, Loadable
│   ├── gtw-api/               # REST + Action API, OAuth, HTTP policy
│   ├── gtw-cache/             # SQLite, zstd blobs, FTS5, media cache
│   ├── gtw-render/            # HTML -> IR -> terminal layout, syntax, themes
│   ├── gtw-media/             # image/SVG, Symphonia audio, zvidlib video
│   ├── gtw-app/               # runtime, UI, input, effects, screen flows
│   └── gtw/                   # binary: CLI, config, terminal lifecycle
└── specs/                     # this bundle, copied into the repo
```

The tree above is the target layout. This repository currently contains the
specification bundle, license, and agent brief only; the Cargo workspace is
created in the codespace as part of M0.

## Build target

- Rust **1.90+** (edition 2024); development machine currently has 1.98.1.
- Linux aarch64 is the primary test machine; the core must remain portable.
- Ghostty is the primary terminal; Kitty/sixel/iTerm2/ halfblock fallbacks must
  work.

## Remote

`origin` is `https://github.com/prjctimg/gtw` (public).
