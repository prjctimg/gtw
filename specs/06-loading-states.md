# Loading states

## Rule

Every asynchronous resource is a `Loadable<T>`. There are no ad-hoc
`is_loading`, `is_fetching`, or `loaded` booleans in screen state.

A resource can be loading and still show a previous value. A blank pane is
never used as a loading state.

## Core types

```rust
pub struct RequestId(u64);

pub enum Loadable<T> {
    Idle,
    Queued {
        queued_at: Instant,
        priority: Priority,
    },
    Loading {
        stage: LoadStage,
        progress: Option<Progress>,
        started_at: Instant,
        previous: Option<Arc<T>>,
        cancel: CancelHandle,
    },
    Refreshing {
        stage: LoadStage,
        progress: Option<Progress>,
        previous: Arc<T>,
        started_at: Instant,
        cancel: CancelHandle,
    },
    Ready {
        value: Arc<T>,
        updated_at: SystemTime,
        source: Source,
        stale: bool,
    },
    Failed {
        error: LoadError,
        previous: Option<Arc<T>>,
        retryable: bool,
        failed_at: SystemTime,
    },
    Unsupported {
        reason: String,
        fallback: Fallback,
    },
}

pub enum Source { Network, Cache, Prefetch, Local }

pub struct Progress {
    pub current: u64,
    pub total: Option<u64>,
    pub unit: ProgressUnit, // Bytes, Items, Frames, Percent
    pub label: String,
}

pub enum LoadStage {
    Connecting,
    Downloading,
    Parsing,
    Extracting,
    LayingOut,
    Decoding,
    Buffering,
    Indexing,
    Saving,
    Committing,
}
```

`Loadable::value()` returns `Option<&T>` for the last-known-good value.
`Loadable::begin_refresh()` moves a ready value into `Refreshing` without
clearing the screen.

## Activity center

Every operation is also an `Activity`:

```rust
pub struct Activity {
    pub id: RequestId,
    pub kind: ActivityKind,
    pub label: String,
    pub surface: SurfaceId,
    pub priority: Priority,
    pub progress: Option<Progress>,
    pub cancellable: bool,
    pub started_at: Instant,
    pub cancel: CancelHandle,
}
```

- The footer shows the highest-priority active activity: spinner, label,
  progress, and `Esc cancel`.
- `:jobs` / `Ctrl-J` opens the full list; `Enter` focuses the owning surface,
  `c` cancels, `r` retries.
- Background prefetch/refresh is shown as a subtle dot, not a full skeleton.
- Activities are removed on success, failure, or cancellation; failures remain
  in the owning `Loadable` and in a short “recent errors” list.

## State transitions

```text
Idle -> Queued -> Loading -> Ready
                 Loading -> Failed -> Loading (retry)
                 Loading -> Cancelled (back to previous/Idle)
Ready -> Refreshing -> Ready
                    -> Failed { previous }
Ready -> Stale (TTL exceeded; revalidate in background)
```

Cancellation during `Loading` with no previous value returns to `Idle` or
`Failed { cancelled }` depending on whether the user initiated it. Cancellation
during `Refreshing` keeps the previous ready value.

## Complete loading-state matrix

| Surface | States that must be rendered | UI |
|---|---|---|
| **Process startup** | config reading, config invalid, cache opening, migration, cache locked/read-only, protocol detection, auth restore, siteinfo, first feed | Splash wordmark + one-line stage; error panel with file/line; “continue read-only” / “retry” |
| **Home/feed** | initial, per-section loading, per-section empty, partial failure, stale cache, date switch, refresh, offline | Independent card skeletons; section error row with retry; cached date badge; offline banner |
| **Search** | idle, debouncing, in-flight, results, no results, pagination, local-only, error, cancelled | Skeleton rows after 150 ms; previous results dimmed while refreshing; “Searching…” + cancel |
| **Article resolve/open** | title resolve, redirect, 404, protected, network error, offline cache hit, parse error | Title bar skeleton; cached article remains; error panel with retry/search/open-browser |
| **Article fetch** | connecting, headers, downloading (bytes), 304, error | Thin progress bar; stale content + “Updating…” badge |
| **Article parse/extract** | parsing, extracting body, building links, building TOC, building FTS text, failed | Body skeleton with real title/header; “Parsing…” stage; cancel not offered after fetch unless > 2 s |
| **Article layout** | initial layout, re-layout on resize, re-layout on theme/width change, failure | Previous layout kept; subtle “Rendering…” if > 120 ms |
| **Images** | queued, downloading, decoding, resizing, protocol encoding, ready, animated, failed, unsupported | Shimmering placeholder with alt text; per-image progress for large originals; retry; caption visible |
| **Image gallery** | grid loading, per-tile loading, lightbox loading, full-resolution loading, error | Tile skeletons; lightbox spinner + progress; “Open original” action |
| **Video** | metadata, transcode selection, download, buffering, demux, decoder init, first frame, playing, paused, seeking/buffering, ended, audio-only fallback, unsupported | Poster + spinner; buffering label; progress bar; frame area; error panel; external action |
| **Audio** | metadata, download, decode init, buffering, playing, paused, seek, ended, device unavailable | Mini-player states; spinner; time; retry; device error |
| **Link peek** | debounce, fetching summary, fetching thumbnail, cached, no extract, disambiguation, red link, error, cancelled | Compact skeleton popup; no layout jump; inline error + retry |
| **Bookmarks** | DB load, empty, add/remove optimistic, pin download, unpin, import/export, sync, conflict, offline | Optimistic row with pending dot; pin progress; rollback on failure |
| **History** | first page, pagination, empty, filter, clear, delete, error | Skeleton rows only for first page; spinner for “load more” |
| **Watchlist** | anonymous, loading, pagination, empty, filter, refresh, error, session expired | Login card; rows; refresh; re-login |
| **Auth: OAuth** | starting listener, browser launch failed, waiting for callback, callback received, token exchange, profile fetch, success, timeout, cancelled, error | Modal with stage, cancel, copy URL, “open browser again” |
| **Auth: bot password** | validating, 2FA required, wrong password, throttled, blocked, success, error | Inline modal; password never echoed; 2FA field appears on demand |
| **Auth: session** | anonymous, valid, expiring, refreshing, expired, refresh failed | Header avatar/indicator; silent refresh; expiry banner |
| **Editing: load** | loading wikitext, protected, missing revision, error | Editor skeleton; read-only reason |
| **Editing: preview** | debounce, parsing preview, rendering preview, error | Preview pane dims + spinner; previous preview remains |
| **Editing: diff** | computing local diff, loading server diff, error | Diff modal skeleton; cancel |
| **Editing: save** | acquiring token, saving, saved, conflict, captcha, abuse filter, blocked, rate limited, network error | Save dialog disables save; progress; conflict merge view; exact server message |
| **Editing: draft** | autosaving, saved, failed, restore prompt | “Draft saved 12:04” / warning; restore modal |
| **Cache maintenance** | stats, prune, clear, vacuum, FTS rebuild, migration, disk full, read-only, error | Settings progress bar + cancel; never block UI |
| **Media cache** | pinned download progress, paused, failed, evicted | Per-item progress; pause/resume; retry |
| **Navigation** | tab opening, back/forward cached/uncached, random, language switch, TOC jump | Cached instant; uncached tab uses article skeleton; TOC jump is synchronous |
| **Resize** | re-layout pending, image re-encode pending, protocol re-detect | Subtle footer activity; no blank frames |
| **Theme/config** | loading theme, applying, invalid theme, config reload | Live preview; error panel with path |
| **Connectivity** | online, offline, degraded, timeout, 429 countdown, probe | Global status chip + per-resource stale/offline badge |
| **Background** | prefetch, token refresh, feed refresh, thumbnail queue, eviction | Activity center; footer dot; never a modal |
| **Quit** | cancelling jobs, flushing drafts, checkpointing DB, done | “Saving drafts…” before exit; force quit with confirmation |

## UI primitives

```text
LoadingPane::skeleton(rows, header)
LoadingPane::spinner(label)
LoadingPane::progress(progress)
LoadingPane::stale_badge(updated_at, source)
LoadingPane::error(error, actions)
LoadingPane::empty(message, action)
LoadingPane::offline(cached_at)
LoadingPane::unsupported(reason, fallback)
```

- Skeletons preserve the final layout dimensions to avoid scroll jumps.
- Spinners use braille frames when Unicode is available, ASCII otherwise.
- Progress is determinate only when `total` is known; otherwise use a stage
  label plus indeterminate bar.
- Stale badge: `cached 3m ago`, `offline · cached`, `updated just now`.
- Error actions: `r retry`, `c cancel`, `o open cached`, `b browser`,
  `l login`. Only show actions that apply.
- Empty states are distinct from loading: “No bookmarks yet — press `b` on an
  article.”
- Optimistic UI is allowed only for bookmark add/remove and settings; failures
  roll back and show an inline error.

## Timeouts, retries, cancellation

| Operation | Timeout | Retry |
|---|---:|---|
| REST GET | 20 s connect/read; 60 s total for HTML | 2, backoff+jitter |
| Summary/peek | 8 s | 1 |
| Search | 10 s | 1 |
| Image download | 30 s | 1 |
| Audio/video download | no total limit; idle timeout 30 s | resume |
| OAuth token exchange | 15 s | 1 |
| Save edit | 30 s | never automatic |
| Cache migration | no total limit | restartable |
| Parse/layout | no hard timeout, but show stage and allow navigation | cancel on navigation |

- `Retry-After` overrides backoff; cap automatic wait at 30 s.
- User-initiated cancellation is not an error; show “Cancelled” only if the
  surface would otherwise be blank.
- Navigation cancels: search debounce, peek, prefetch, stale image decodes,
  previous article parse if no tab references it, video/audio when leaving
  unless playing in the background (config).

## Test requirements

- Unit-test every `Loadable` transition.
- Snapshot-test every screen in idle/loading/ready/empty/stale/offline/error.
- Mock API server with per-endpoint latency, 304, 429, 500, malformed HTML,
  disconnect, and cancellation.
- Fake clock for TTL/stale and retry timing.
- A 500 KB HTML fixture must parse/layout off-thread; input events must be
  processed during the operation.
- CI check: every `Cmd` that performs I/O has start/success/failure/cancel/
  timeout handling in `update`. A table of `CmdKind` variants and their result
  messages is maintained in `gtw-core`.
- No screen may render a blank body while a `Loadable` is `Loading` or
  `Refreshing` with a previous value.
