# Storage and cache

## Locations

```text
$XDG_CONFIG_HOME/gtw/
├── config.toml
├── themes/
└── keys.toml                 # keymap overrides only; no secrets

$XDG_DATA_HOME/gtw/
├── bookmarks.json            # optional export/import
└── drafts/                   # crash-safe wikitext drafts

$XDG_CACHE_HOME/gtw/
├── cache.db                  # SQLite metadata
├── cache.db-wal
├── blobs/ab/<sha256>.zst     # HTML, JSON, wikitext
├── images/ab/<sha256>.<ext>  # image bytes
├── media/ab/<sha256>.<ext>   # audio/video
└── tmp/
```

Override with `--cache-dir`, `--config-dir`, `--data-dir`. Create directories
with `0700` on Unix. Never follow symlinks when evicting.

## SQLite

- `rusqlite` with bundled SQLite and FTS5.
- WAL mode, `synchronous=NORMAL`, `foreign_keys=ON`, `busy_timeout=5000`.
- `PRAGMA user_version` for migrations; each migration is transactional.
- One writer task; readers use a small pool or serialized access. Never hold a
  connection across an `.await`.
- DB writes are batched for history/scroll updates (debounce 500 ms).

### Schema (v1)

```sql
CREATE TABLE meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE articles (
  id INTEGER PRIMARY KEY,
  wiki TEXT NOT NULL,
  title TEXT NOT NULL,
  display_title TEXT NOT NULL,
  page_id INTEGER,
  revision_id INTEGER,
  lang TEXT NOT NULL,
  summary TEXT,
  description TEXT,
  html_blob TEXT,              -- sha256 of zstd blob
  plain_text_blob TEXT,        -- sha256 of zstd blob, for FTS rebuild
  size_bytes INTEGER NOT NULL,
  etag TEXT,
  last_modified TEXT,
  fetched_at INTEGER NOT NULL,
  last_viewed_at INTEGER,
  stale INTEGER NOT NULL DEFAULT 0,
  redirect_from TEXT,
  protection_json TEXT,
  UNIQUE(wiki, title)
);
CREATE INDEX articles_lru ON articles(last_viewed_at);

CREATE VIRTUAL TABLE article_fts USING fts5(
  title, plain_text,
  content='', tokenize='unicode61 remove_diacritics 2'
);

CREATE TABLE links (
  src_article_id INTEGER NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
  target_title TEXT NOT NULL,
  anchor TEXT,
  kind TEXT NOT NULL,          -- wiki, red, external, media, citation
  position INTEGER NOT NULL
);
CREATE INDEX links_src ON links(src_article_id);

CREATE TABLE bookmarks (
  id INTEGER PRIMARY KEY,
  wiki TEXT NOT NULL,
  title TEXT NOT NULL,
  page_id INTEGER,
  folder TEXT,
  notes TEXT,
  pinned INTEGER NOT NULL DEFAULT 0,
  added_at INTEGER NOT NULL,
  last_opened_at INTEGER,
  UNIQUE(wiki, title)
);

CREATE TABLE history (
  id INTEGER PRIMARY KEY,
  wiki TEXT NOT NULL,
  title TEXT NOT NULL,
  anchor TEXT,
  scroll INTEGER NOT NULL DEFAULT 0,
  viewed_at INTEGER NOT NULL
);
CREATE INDEX history_time ON history(viewed_at DESC);

CREATE TABLE search_history (
  id INTEGER PRIMARY KEY,
  query TEXT NOT NULL,
  searched_at INTEGER NOT NULL
);

CREATE TABLE feeds (
  id INTEGER PRIMARY KEY,
  wiki TEXT NOT NULL,
  feed_date TEXT NOT NULL,     -- YYYY-MM-DD
  payload_blob TEXT NOT NULL,  -- compressed JSON
  fetched_at INTEGER NOT NULL,
  etag TEXT,
  UNIQUE(wiki, feed_date)
);

CREATE TABLE media (
  id INTEGER PRIMARY KEY,
  wiki TEXT NOT NULL,
  title TEXT NOT NULL,         -- File:...
  kind TEXT NOT NULL,          -- image, audio, video, document, other
  mime TEXT,
  width INTEGER,
  height INTEGER,
  duration_ms INTEGER,
  source_url TEXT,
  thumb_url TEXT,
  cache_key TEXT,              -- sha256 URL+variant
  cache_path TEXT,
  bytes INTEGER,
  license_json TEXT,
  pinned INTEGER NOT NULL DEFAULT 0,
  fetched_at INTEGER NOT NULL,
  last_access_at INTEGER NOT NULL,
  UNIQUE(wiki, title, cache_key)
);
CREATE INDEX media_lru ON media(last_access_at);

CREATE TABLE media_links (
  article_id INTEGER NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
  media_id INTEGER NOT NULL REFERENCES media(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  PRIMARY KEY(article_id, media_id, position)
);

CREATE TABLE drafts (
  id INTEGER PRIMARY KEY,
  wiki TEXT NOT NULL,
  title TEXT NOT NULL,
  section INTEGER,
  base_revision INTEGER,
  wikitext TEXT NOT NULL,
  summary TEXT,
  updated_at INTEGER NOT NULL,
  UNIQUE(wiki, title, section)
);

CREATE TABLE watchlist (
  id INTEGER PRIMARY KEY,
  wiki TEXT NOT NULL,
  title TEXT NOT NULL,
  page_id INTEGER,
  revid INTEGER,
  old_revid INTEGER,
  user TEXT,
  comment TEXT,
  timestamp TEXT,
  size_delta INTEGER,
  fetched_at INTEGER NOT NULL,
  UNIQUE(wiki, title, revid)
);

CREATE TABLE recent_errors (
  id INTEGER PRIMARY KEY,
  surface TEXT NOT NULL,
  message TEXT NOT NULL,
  detail TEXT,
  occurred_at INTEGER NOT NULL
);
```

### FTS5

- `article_fts` is an external-content table; populate from extracted plain
  text on article write.
- Store `plain_text_blob` so FTS can be rebuilt after a schema change.
- Search query: FTS5 `MATCH` with `bm25(article_fts, 5.0, 1.0)`, then a title
  prefix boost. Escape FTS operators from user input; do not pass raw queries.
- Offline search result indicates `cached` and the last-updated time.
- Rebuild is a background job with progress and cancel; the DB remains usable.

## Blobs

- Hash: SHA-256 (or BLAKE3; pick one and keep it stable) over uncompressed
  bytes.
- Text/JSON: zstd level 3, extension `.zst`; metadata stores hash and original
  size.
- Images/audio/video: raw bytes in `images/`/`media/`, named by hash + guessed
  extension.
- Atomic write: `tmp/<uuid>` → `fsync` → `rename`.
- Reference counting is logical (article/media rows point at hashes). Run a
  mark-and-sweep only during explicit prune, not on every write.
- A blob is never overwritten if the hash already exists.

## Eviction

Default budgets (configurable):

| Category | Budget |
|---|---:|
| Article text/JSON | 512 MB |
| Images | 512 MB |
| Audio | 1 GB |
| Video | 4 GB |
| Total | 6 GB |

- Evict LRU by `last_access_at`/`last_viewed_at`, oldest first.
- Never evict:
  - pinned bookmarks and their media,
  - article currently displayed in any tab,
  - active or paused media player files,
  - drafts (separate store, never evicted).
- Run eviction after a write only when the category exceeds 95% of budget, in
  a background task; show activity if it takes > 200 ms.
- Disk-full: mark cache writes failed, keep the in-memory result, prompt to
  prune. Never crash.

## Offline mode

- `--offline` or detected network loss:
  - read-only article/bookmark/history/search flows from cache;
  - feeds use cached dates;
  - peeks use cached summaries only;
  - editing is disabled with an explanation (drafts still save locally);
  - auth refresh is deferred.
- Stale content is served with a badge; do not silently present it as fresh.
- A connectivity probe is lightweight and jittered (30–60 s) to avoid battery
  and API noise.

## Bookmarks

- Add: optimistic insert; if the article is not cached, offer “pin for offline”
  which downloads article, summary, TOC, images, and optional media.
- Pin progress: article 0–40%, images 40–90%, media 90–100%; cancellable.
- Remove: unpin media only after no other bookmark references it.
- Export/import: versioned JSON with wiki, title, folder, notes, pinned. Import
  reports added/updated/skipped/error counts.
- Sync with Wikipedia watchlist is optional; bookmarks are local-first.

## Drafts

- Stored in `drafts` and mirrored to `$XDG_DATA_HOME/gtw/drafts/<hash>.json`
  for crash recovery.
- Autosave 2 s after the last keystroke; write is atomic.
- On editor open, if a draft is newer than the base revision, show a restore
  prompt with timestamps and diff.
- On successful save, delete the draft. On conflict, keep it.

## Maintenance UI

Settings → Cache shows:

- DB size, blob sizes per category, item counts, hit rate;
- “Prune to budget”, “Clear images”, “Clear media”, “Vacuum”, “Rebuild search”;
- each operation has progress, cancel, and a result summary;
- cache location and a “copy path” action.

## Backup / portability

- Export bookmarks and config as JSON/TOML; never export auth tokens.
- The cache DB is disposable; no user data lives only in `cache.db` except
  history (exportable) and drafts (mirrored to data dir).
