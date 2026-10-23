# Roadmap and acceptance

## Bootstrap (completed)

- `specs/` contains this bundle, including `themes.toml` and the original
  prompt.
- `LICENSE` (GPL-3.0), root `README.md`, `AGENTS.md`, and `.gitignore` are in
  place.
- The repository is `https://github.com/prjctimg/gtw` and `main` is pushed.
- The Cargo workspace is **not** created yet; that is the first M0 task in the
  codespace.

Do not commit tokens, `.env` files, cache data, or target artifacts.

## M0 — Skeleton, themes, wordmark, loading primitives

Deliverables:

- Workspace with all seven crates and dependency boundaries.
- `gtw-core`: IDs, `Msg`/`Cmd`, `Loadable<T>`, `ActivityCenter`, `AppState`
  shell, navigation enums.
- `gtw-app`: terminal lifecycle, event loop, input map, screen router, footer.
- `gtw-render`: `Theme` type, theme loader, semantic derivation, contrast
  helpers, minimal text renderer.
- `themes.toml` bundled with all 16 themes; theme picker with preview.
- `tui-big-text` wordmark `gtw` with gtm-style fit logic.
- `tachyonfx` startup/page transitions and a demo of skeleton shimmer.
- Config loading, logging, panic/signal recovery, `--offline`,
  `--no-animations`, `--theme`.
- Mock async jobs that exercise every `Loadable` state.
- CI: fmt, clippy `-D warnings`, test, build on Linux x86_64 and aarch64 if
  available.

Acceptance:

- `cargo run` opens a usable shell with logo, sidebar, footer, theme picker.
- Theme switching updates every widget; all 16 themes load.
- Simulated slow jobs show skeleton/progress/error/retry/cancel; input remains
  responsive.
- `cargo test` and `cargo clippy --all-targets -- -D warnings` pass.
- No network required for the demo.

## M0.5 — No-ffmpeg media spike (blocking gate)

Deliverables:

- A `gtw-media` example that downloads a short VP9/Opus WebM, demuxes it,
  decodes 120 frames, decodes Opus to PCM, plays via rodio, and seeks to 50%.
- Timing/memory notes in `specs/media-spike.md`.
- Confirm no ffmpeg/libavcodec in `Cargo.lock` or linkage.

Acceptance:

- Decode works on aarch64 Linux.
- UI-thread responsiveness is unaffected.
- Seek resumes at the requested timestamp (±1 frame).
- If zvidlib fails, document the fallback spike with
  `rusty_vp9` + `matroska-demuxer` + pure-Rust Opus.
- If both fail, stop and escalate: inline video cannot meet the no-ffmpeg
  constraint.

## M1 — Read-only TUI

Deliverables:

- REST client with ETag revalidation, retries, cancellation, User-Agent.
- Search (REST + local FTS), article fetch, HTML extraction, IR, basic rich
  rendering, TOC, links, references, code highlighting.
- Tabs, back/forward, history, bookmarks (local), scroll restoration.
- SQLite + zstd + FTS5 cache; offline reopen; stale-while-revalidate.
- Full loading states for startup/search/article/peeks(placeholder)/cache.
- CLI/config/logging/terminal protocol detection.

Acceptance:

- Search → open → scroll → follow link → TOC → back/forward works.
- Reopen the same article offline; cached content renders identically.
- 500 KB article (`Rust_(programming_language)`) never blocks input; first
  content appears < 1 s on a warm cache and < 3 s on network.
- `cargo test`, clippy, and the loading-state snapshot suite pass.
- No blank pane during loading; errors offer retry/open-cache.

## M2 — Rich rendering, images, link peeks

Deliverables:

- Tables, infoboxes, quotes, callouts, galleries, references, math alt text,
  collapsible sections, navbox handling.
- `ratatui-image` protocols + halfblocks; SVG via resvg; animated GIF/WebP/
  APNG; thumbnail selection; lightbox/gallery.
- Image loading pipeline with priorities, progress, cancellation, cache.
- Link peeks: hover/keyboard, summary + thumbnail, disambiguation/red links,
  LRU, cancellation.
- Tachyonfx image reveal (with protocol limitation handled).
- Media-list integration; `<audio>`/`<video>` detection.

Acceptance:

- A fixture set of complex articles renders without panic or overflow.
- Images appear in Kitty/sixel/iTerm2/halfblocks; failure shows alt text.
- Hover peek appears/cancels correctly and never blocks scrolling.
- Image downloads/decodes are cancellable and cached.
- Snapshot tests for all peek/image states.

## M3 — Audio and video

Deliverables:

- Symphonia + rodio audio player; mini-player; audio sources from HTML.
- zvidlib video player: transcode selection, download/buffer, demux, VP9,
  Opus/Vorbis, A/V sync, seek, pause, end, captions optional.
- Media cache with resume, pin, LRU, disk budgets.
- Loading/buffering/error states; poster/external fallback.
- No ffmpeg anywhere.

Acceptance:

- Play an Ogg/MP3/FLAC/WAV audio file from an article.
- Play a VP9/Opus WebM inline with synchronized audio, pause/seek/end.
- Unsupported Theora/MPEG-4 shows poster + external action, not a hang.
- Memory stays bounded during a 10-minute video; UI remains responsive.
- Cache survives restart and resumes a partial download.

## M4 — Authentication, bookmarks, watchlist

Deliverables:

- OAuth2 PKCE loopback + refresh + logout; bot-password fallback + 2FA.
- Keyring storage with session-only fallback.
- Bookmarks UI with folders/pin/offline download/import/export.
- Watchlist fetch/pagination/filter/diff viewer.
- Auth-required and session-expired states everywhere.

Acceptance:

- Login via OAuth, fetch profile, edit permission check.
- Bot password login and 2FA path tested against a mock.
- Pin a bookmark and reopen offline with images.
- Watchlist loads and refreshes; logout clears secrets.
- Secret-redaction tests pass.

## M5 — Editing

Deliverables:

- Wikitext editor with split preview, toolbar/palette, syntax highlighting.
- Preview via Action API, local diff, save with CSRF/basetimestamp, conflict
  three-way merge, draft autosave/restore.
- Protection/abuse filter/captcha/rate-limit error rendering.
- One-write-at-a-time policy.

Acceptance:

- Edit a user sandbox page end-to-end.
- Simulated edit conflict never overwrites; merge/keep-mine works.
- Draft survives a crash/restart and is deleted after save.
- Preview and save are cancellable/safe; no duplicate writes.
- CSRF `badtoken` retries once and succeeds.

## M6 — Home feed, polish, packaging

Deliverables:

- Featured feed with independent sections, date navigation, caching.
- Command palette, keymap editor, settings, accessibility modes.
- Performance profiling, memory caps, startup time, large-page tests.
- Packaging: install script, completions, man page, release workflow.
- Documentation and screenshots/GIF.

Acceptance:

- All screens have loading/empty/error/offline states.
- Feed partially fails gracefully; cached dates work offline.
- `--no-color`, `--no-unicode`, `--no-animations` work.
- Startup < 300 ms warm; idle CPU near zero; no leaks over 1 hour.
- CI green on Linux; portable core compiles on macOS/Windows where feasible.

## Risk register

| Risk | Impact | Mitigation |
|---|---|---|
| zvidlib is young / API changes | video blocked | M0.5 spike; fallback rusty_vp9 + matroska-demuxer; external fallback as last resort |
| No native Opus in Symphonia | video/audio gap | pure-Rust Opus in zvidlib; do not add ffmpeg |
| Complex Wikipedia HTML | rendering gaps | fixture corpus; prioritize article body over navboxes; graceful unknown-block rendering |
| Tachyonfx cannot fade protocol images | visual mismatch | animate placeholder/frame; halfblock crossfade only |
| OAuth loopback rejected | login blocked | manual code paste; bot password fallback |
| Edit conflicts/data loss | trust | basetimestamp, three-way diff, drafts, no auto-overwrite |
| GitHub API rate limits | fetch failures | ETag, backoff, concurrency caps, cache |
| GPL compliance | legal | GPL-3.0 license, attribution, source availability |
| Codespace lacks image protocol | media not visible | halfblocks default; document terminal requirements |
| Large media cache | disk pressure | budgets, LRU, pin limits, disk-full handling |

## Definition of done for every milestone

- Code formatted, clippy clean, tests pass.
- Loading/error/empty/offline states implemented and snapshot-tested.
- No UI-thread blocking; cancellation works.
- No secrets in logs; no ffmpeg.
- Specs updated for any deviation.
- Commit message describes behavior, not just files.
