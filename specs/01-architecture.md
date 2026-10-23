# Architecture

## Goals

- Never block input while fetching, parsing, laying out, decoding, or
  rendering a 500 KB article.
- Keep every asynchronous resource cancellable and observable.
- Separate I/O, domain state, rendering, and terminal concerns so the core can
  be tested without a terminal.
- Make cached reads indistinguishable from network reads except for an
  explicit `Source` / stale badge.

## Workspace

| Crate | Responsibility | Must not |
|---|---|---|
| `gtw-core` | IDs, domain types, `Msg`/`Cmd`, `Loadable<T>`, `Activity`, `AppState` transitions, selection/navigation logic | perform I/O, know ratatui |
| `gtw-api` | REST/Action API clients, URL building, auth requests, retries, rate limiting | own UI state |
| `gtw-cache` | SQLite metadata, zstd blobs, FTS5, media files, migrations, eviction | depend on app UI |
| `gtw-render` | HTML extraction, IR, table/list/link layout, syntax highlighting, themes, hit-testing | fetch or mutate app state |
| `gtw-media` | image/SVG decode, terminal protocols, Symphonia audio, zvidlib video | own navigation |
| `gtw-app` | runtime, screens, widgets, input map, effects, orchestration | hide loading/error states |
| `gtw` | CLI, config discovery, logging, terminal init/restore, panic hook | contain domain logic |

Dependency direction: `gtw` → `gtw-app` → {`gtw-core`, `gtw-api`,
`gtw-cache`, `gtw-render`, `gtw-media`}. `gtw-core` has no dependency on the
other workspace crates.

## Update loop

```text
                      ┌──────────────────────────────┐
   crossterm events ──▶│ input -> Msg                 │
   API/cache/media ───▶│ results -> Msg               │
   timers ────────────▶│ tick/effects -> Msg          │
                      └──────────────┬───────────────┘
                                     ▼
                         ┌───────────────────────┐
                         │ AppState::update(msg) │
                         │  -> Vec<Cmd>          │
                         └──────────┬────────────┘
                                    ▼
                         ┌───────────────────────┐
                         │ Cmd executor (Tokio)  │
                         │ jobs + cancellation   │
                         └──────────┬────────────┘
                                    ▼
                          Msg channel (bounded)
                                    │
                         ┌──────────▼───────────┐
                         │ view(&state, frame)  │
                         │ + EffectManager      │
                         └──────────────────────┘
```

- `Msg` is the only way to mutate state.
- `Cmd` describes work and carries a `RequestId`.
- Every result message carries the same `RequestId`; late results are dropped
  if the id no longer matches the resource generation.
- The executor owns a `CancellationToken` per job. Navigation cancels
  non-essential work; explicit cancellation is exposed for downloads, video,
  previews, and cache maintenance.
- Input, resize, and tick handling never await. Rendering is synchronous over
  already-computed state.

## Concurrency classes

| Class | Default limit | Examples |
|---|---:|---|
| Interactive API | 4 | article, summary, peek, save |
| Background API | 2 | feed refresh, watchlist, prefetch |
| Image decode | 2 (blocking pool) | thumbnails, galleries |
| Image protocol | 1 | Kitty/sixel transmit |
| Video decode | 1 | active player only |
| Audio decode | 1 | active player only |
| DB writes | 1 writer task | cache/history/bookmarks |
| Writes to Wikipedia | 1 | edits, auth token exchange |

- Deduplicate identical in-flight GETs by cache key.
- Priority: visible image/peek > current article > next section > prefetch.
- A bounded result channel provides backpressure; no unbounded job queues.
- CPU-heavy work (`parse`, `layout`, `decode`) uses
  `tokio::task::spawn_blocking` or a dedicated rayon pool.

## State model

```text
AppState
├── session          AuthSession, user info, csrf token
├── connectivity     Online | Offline | Degraded { reason }
├── tabs: Vec<Tab>   one per open article/search
├── active_tab
├── home             Loadable<Feed>
├── search           SearchState
├── bookmarks        Loadable<BookmarkList>
├── history          Loadable<HistoryPage>
├── watchlist        Loadable<WatchlistPage>
├── editor           Option<EditorState>
├── media            MediaState (images, audio, video, viewer)
├── peeks            PeekCache (LRU)
├── activity         ActivityCenter
├── theme            ThemeState
├── config           Config
└── toasts           Vec<Toast> (non-critical feedback only)
```

`AppState::update` is pure with respect to external systems: it may enqueue
commands, mutate in-memory state, and mark resources loading/ready/failed. It
never performs HTTP, disk, or decoder work.

## Tab and navigation model

- A tab stores its own article `Loadable<ArticleView>`, scroll offset, link
  focus, TOC state, and back/forward stack.
- History entries store `{wiki, title, anchor, scroll}`; reopening a cached
  article restores scroll after layout with the same width.
- Back/forward displays the cached view immediately if present and starts a
  stale-while-revalidate fetch.
- Link activation: internal wiki link opens/reuses a tab according to config;
  external links go to the system browser; media links open the viewer.
- Redirects update the canonical title while preserving the original in the
  back stack.

## Rendering pipeline

1. **Fetch** HTML/summary/media manifest (or read cache).
2. **Extract** the article body from `.mw-parser-output` with `scraper`.
3. **Normalize** into an IR of blocks/inlines; collect links, headings,
   references, media placeholders, and plain text for FTS.
4. **Layout** at the current content width and theme: styled lines, tables,
   list markers, code blocks, image rectangles, link hitboxes.
5. **Render** only visible sections into the ratatui buffer.
6. **Overlay** protocol images, video frames, peeks, and effects.

Layout is cached per `(article revision, width, theme, font metrics)`.
Scrolling reuses the layout. Resize debounces re-layout by ~80 ms and cancels
stale image encodes.

## Error model

- Library errors: `thiserror` enums (`ApiError`, `CacheError`, `MediaError`,
  `RenderError`, `AuthError`).
- The UI never displays raw errors only; it maps them to a `Loadable::Failed`
  with:
  - a human message,
  - retryability,
  - optional last-known-good value,
  - recovery actions (`retry`, `open cache`, `open browser`, `login`,
    `cancel`).
- Panics in worker tasks are converted to failed messages with the job label;
  the app keeps running.
- The binary installs a panic hook that restores the terminal before printing a
  report.

## Configuration

Files, lowest to highest priority:

1. built-in defaults,
2. `$XDG_CONFIG_HOME/gtw/config.toml`,
3. project-local `.gtw.toml` (optional; explicit `--config` only),
4. CLI flags.

Key sections: `[general]`, `[network]`, `[cache]`, `[render]`,
`[media]`, `[effects]`, `[theme]`, `[auth]`, `[editor]`, `[keys]`.
Invalid values produce a startup error panel with the file and line, not a
silent fallback. Unknown keys warn but do not fail.

## Logging

- `tracing` + `tracing-subscriber` with `RUST_LOG` and `--log-file`.
- Default log path: `$XDG_STATE_HOME/gtw/gtw.log`.
- Never log tokens, passwords, Authorization headers, CSRF tokens, or full
  wikitext bodies. Log request IDs, status, bytes, and timings.
- A debug overlay (`F12`) shows recent jobs, cache hits, layout timings, and
  active effects.

## Terminal lifecycle

- Detect protocol with `ratatui-image::Picker` with a short timeout; fall back
  to halfblocks.
- Enter alternate screen, raw mode, bracketed paste, mouse capture, keyboard
  enhancement flags where supported.
- Restore terminal on normal quit, error, panic, SIGINT/SIGTERM, and suspend
  (`Ctrl-Z`). Re-enter and repaint after resume.
- On resize, invalidate layout and image protocol placements before drawing.
