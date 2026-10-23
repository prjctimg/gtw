# UI / UX specification

## Design principles

- The article is the primary surface; chrome recedes.
- Loading, stale, offline, empty, and error states are visible and actionable.
- Keyboard-first, mouse-capable. No essential action is mouse-only.
- No color-only meaning: stale/error/selection also use glyphs and text.
- Motion is short and purposeful; reduced-motion is a first-class setting.
- The UI never freezes; a slow operation always shows a cancellable state.

## Window layout

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ gtw   [Article: Rust] [Search: memory] [+]            offline · cached 3m  │
├──────────────┬───────────────────────────────────────────────┬─────────────┤
│ Home         │                                               │ Contents    │
│ Search       │   Article title / metadata                    │ 1 History   │
│ Bookmarks    │                                               │ 2 Design    │
│ History      │   Body, links, images, tables, code...        │ 3 ...       │
│ Watchlist    │                                               │             │
│ Settings     │                                               │             │
│              │                                               │             │
├──────────────┴───────────────────────────────────────────────┴─────────────┤
│ ⠋ Parsing article…  [########------] 63%   r retry   ? help   q quit       │
└────────────────────────────────────────────────────────────────────────────┘
```

- Sidebar width: 22–28 cells; collapsible (`Ctrl-B`), hidden below 80 columns.
- Right TOC: 24–32 cells; collapsible (`t`); hidden below 110 columns unless
  opened as an overlay.
- Tabs appear in the header; overflow scrolls.
- Footer is one or two rows: activity summary, progress, key hints, connection,
  cache age.
- A floating “activity center” (`:jobs`) lists all background jobs with cancel.

## Screens

### Home / feed

- Wordmark `gtw` at the top on first run; compact wordmark after.
- Date selector with yesterday/today/tomorrow; cached dates marked.
- Sections: featured article, picture of the day, did you know, on this day,
  in the news, most read.
- Each section loads independently; one failed section does not blank the page.
- Section cards show thumbnail, title, extract, and a skeleton while loading.
- `Enter` opens; `Tab` moves sections; `r` refreshes the date.

### Search

- Input at the top; 250 ms debounce; previous request cancelled.
- Results: title, description, excerpt, thumbnail, “cached” badge.
- Sections: best matches, local cached results (FTS5), recent searches.
- Offline: show local FTS results with an offline banner.
- `Ctrl-N/P` or arrows select; `Enter` opens; `Esc` clears/back.

### Article

- Header: title, description, languages, protection/lock, revision, last
  updated, source (network/cache), bookmark star.
- Body: rich blocks; links are underlined on focus/hover; reference markers
  open a reference popup; code blocks have a language label and copy action.
- Infobox: rendered as a compact table on wide screens, stacked cards on narrow
  screens; collapsible.
- Tables: horizontal scroll when wider than the pane; sticky header row.
- Images: inline thumbnails with caption; `Enter` opens the viewer.
- TOC: floating or side pane; current section highlighted; `Enter` jumps.
- Find-in-page: `/` when the article pane is focused; builds a match index in
  the background for large pages.
- Link peek appears after 180 ms hover on a wiki link (see below).
- Footer: article-specific actions (`e` edit, `b` bookmark, `m` media, `o`
  browser, `y` copy link).

### Bookmarks

- List/folder view; pinned (offline) and normal; sort by added/title/updated.
- `a` add current article; `d` remove; `p` pin/offline-download; `Enter` open.
- Pin shows per-article download progress for images and metadata.
- Import/export JSON/OPML (progress state, cancellable).
- Empty state explains how to add a bookmark.

### History

- Grouped by day; search/filter; delete entry/clear range; pagination.
- Cached/offline entries are marked; opening restores scroll.

### Watchlist

- Requires login; shows recent changes with diff size, user, comment, timestamp.
- Pagination and filters; refresh; open diff in a read-only viewer.
- Anonymous state shows a login card, not an error.

### Settings

- Tabs: General, Appearance, Media, Network, Cache, Auth, Keys.
- Theme picker with live preview and light/dark filters.
- Cache stats with prune/clear; show disk usage per category.
- Keymap editor: capture a key, detect conflicts, reset to defaults.

### Login

- OAuth: “Open browser” → waiting card with cancel/timeout → success/error.
- Bot password: username, bot name, password, optional 2FA.
- Explains scopes and what is stored. Never echoes tokens.
- Session expired state offers re-login without losing the article.

### Editor

- Split view: wikitext on the left, preview on the right; toggle single-pane.
- Section editing from the article’s section menu; whole-page editing too.
- Toolbar/command palette: bold, italic, link, heading, template, reference.
- Preview debounced and cancellable; diff modal before save.
- Save dialog: summary (required), minor, watchlist, preview again.
- Conflict: three-way diff with “keep mine / take theirs / merge”.
- Draft autosave every 2 s idle; restore prompt on reopen.
- Read-only/protected pages show the reason and disable editing.

### Media viewer

See `03-media.md`. It is a modal overlay with its own loading/buffering/error
states and does not block article navigation when closed.

### Help / command palette

- `?` opens a searchable keymap.
- `:` opens a command palette with fuzzy matching and recent commands.
- Commands include navigation, theme, cache, auth, editor, media, debug.

## Link peeks

- Trigger: mouse hover for 180 ms or `K` on a focused link.
- Popup: up to 44×14 cells, positioned to stay on screen; arrow points to link.
- Contents: title, description, extract (3–5 lines), thumbnail, “cached” badge,
  `Enter` open, `Esc` close, `o` browser.
- Loading: compact skeleton with a spinner; no layout jump.
- Cache: in-memory LRU (32 entries, 15 min) + on-disk summary cache.
- Cancel on mouse leave, scroll, tab change, or new hover.
- Disambiguation pages show “Disambiguation” and a few options.
- Red links show “Page does not exist” with a create action (if logged in).

## Keymap (defaults; configurable)

| Context | Key | Action |
|---|---|---|
| Global | `q` | quit (confirm if editing/draft) |
| Global | `?` | help |
| Global | `:` | command palette |
| Global | `/` | search (or find in article) |
| Global | `Ctrl-B` | toggle sidebar |
| Global | `Ctrl-J` / `:jobs` | activity center |
| Global | `Ctrl-C` / `Esc` | cancel current job / close overlay |
| Global | `F12` | debug overlay |
| Navigation | `g h` | home |
| Navigation | `g s` | search |
| Navigation | `g b` | bookmarks |
| Navigation | `g y` | history |
| Navigation | `g w` | watchlist |
| Tabs | `Ctrl-T` / `+` | new tab |
| Tabs | `Ctrl-W` | close tab |
| Tabs | `Tab` / `Shift-Tab` | next/previous tab |
| Tabs | `Alt-Left` / `Alt-Right` | back/forward |
| Article | `j`/`k`, `↓`/`↑` | scroll line |
| Article | `Space`/`b` | page down/up |
| Article | `gg` / `G` | top/bottom |
| Article | `t` | TOC |
| Article | `Enter` | follow focused link |
| Article | `K` | peek focused link |
| Article | `b` | bookmark |
| Article | `e` | edit |
| Article | `m` | media gallery |
| Article | `y` | copy canonical URL |
| Article | `o` | open in browser |
| Media | `Space` | play/pause |
| Media | `←`/`→` | seek −5s/+5s |
| Media | `↑`/`↓` | volume |
| Media | `f` | fullscreen |
| Media | `o` | external |
| Editor | `Ctrl-S` | save |
| Editor | `Ctrl-P` | preview |
| Editor | `Ctrl-D` | diff |
| Editor | `Esc` | close with draft prompt |
| Peek | `Enter` | open peek target |
| Peek | `o` | external |
| Peek | `Esc` | close |

`j/k` scroll; `Ctrl-D/U` half-page; `Ctrl-F/B` page. Horizontal scroll in
tables uses `Shift-←/→`. Mouse wheel scrolls; click follows links/buttons;
hover triggers peeks; middle-click opens in a new tab.

## Focus and selection

- One focus ring per pane; `Tab` moves panes, `Shift-Tab` reverses.
- Focus is always visible with a border/underline, not only color.
- Link focus uses a single-line underline and a subtle background; keyboard
  focus does not trigger peeks until `K` or 180 ms hover.
- Overlays trap focus; `Esc` closes and restores the previous focus.

## Theming and effects

See `05-themes-and-effects.md`. The UI consumes only semantic colors from
`Theme`; no widget hard-codes a palette color.

## Accessibility

- `--no-color`, `--no-unicode`, `--no-animations`, and a high-contrast theme.
- ASCII glyph fallbacks for spinners, list markers, and borders.
- `NO_COLOR` and `CLICOLOR=0` respected.
- Text alternatives for every image; alt text is visible in the viewer and
  on decode failure.
- No timed auto-dismiss for errors; toasts can be disabled.

## Error and empty states

Every screen defines:

- initial loading skeleton,
- refreshing/stale banner,
- empty state with the next action,
- recoverable error with retry/cancel,
- unrecoverable error with open-cache/open-browser/login,
- offline variant,
- permission/auth-required variant.

A blank pane is never an acceptable terminal state.
