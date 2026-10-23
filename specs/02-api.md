# Wikimedia API contract

## Base endpoints

```text
REST:   https://{lang}.wikipedia.org/api/rest_v1/
Action: https://{lang}.wikipedia.org/w/api.php
Search: https://{lang}.wikipedia.org/w/rest.php/v1/
OAuth:  https://meta.wikimedia.org/w/rest.php/oauth2/
```

`lang` is a BCP-47-ish Wikipedia language code (`en`, `de`, `ja`, `zh`).
All titles are normalized with spaces → `_` and percent-encoded exactly once.
The canonical title returned by the API is stored and displayed.

## Read endpoints

| Purpose | Request | Notes |
|---|---|---|
| Article HTML | `GET /api/rest_v1/page/html/{title}` | Response body is the full HTML document; extract `.mw-parser-output`. |
| Summary | `GET /api/rest_v1/page/summary/{title}` | Title, description, extract, thumbnail, originalimage, coordinates, redirects. |
| Media list | `GET /api/rest_v1/page/media-list/{title}` | Items have `type`, `title`, `sources`, `showInGallery`. |
| Featured feed | `GET /api/rest_v1/feed/featured/{yyyy}/{mm}/{dd}` | TFA, most read, image, news, on this day, DYK. |
| Search | `GET /w/rest.php/v1/search/page?q={q}&limit={n}&offset={o}` | Page results with id, key, title, excerpt, description, thumbnail. |
| Related | `GET /api/rest_v1/page/related/{title}` | Optional home/article suggestions. |
| Random | `GET /w/api.php?action=query&list=random&rnnamespace=0&rnlimit=1` | Then open the returned title. |
| Site info | `GET /w/api.php?action=query&meta=siteinfo&siprop=general|namespaces|namespacealiases` | Cache per wiki for 24h. |

Article HTML is authoritative for the reading view. The summary endpoint is
used for peeks, search cards, bookmarks, and fast first paint. The media list
is used for gallery order and video transcode selection; `<audio>` tags in the
HTML are parsed directly because media-list audio entries do not always carry
`sources`.

### Cache headers

- Send `If-None-Match` with the stored `ETag` and `If-Modified-Since` with
  `Last-Modified`.
- `304` refreshes `fetched_at` and reuses the blob.
- Store `ETag`, `Last-Modified`, revision id when available, and final URL.
- REST responses may redirect; follow up to 5 redirects and remember the final
  canonical title.

### Error mapping

| Condition | User-visible state | Retry |
|---|---|---|
| DNS/TLS/timeouts | offline / network error | yes, manual + backoff |
| 404 | page not found; offer search | no |
| 403 | permission/protected | no |
| 429 + `Retry-After` | rate limited with countdown | automatic once |
| 5xx | server error, serve stale cache | yes |
| invalid JSON/HTML | parse error, keep stale | manual |
| redirect loop | error | no |
| maxlag | transient, backoff | yes |

GET retries: at most 2, exponential backoff with jitter, only for
408/429/5xx/network. Never retry an edit automatically.

## Action API reads

```text
action=query
  &prop=info|revisions|pageprops
  &titles={title}
  &rvprop=ids|timestamp|content|contentmodel|sha1
  &rvslots=main
  &inprop=protection|url
  &format=json&formatversion=2
```

- Load wikitext and `revid` together before editing.
- Parse sections/TOC from REST HTML; use `action=parse&prop=sections` only when
  the HTML does not expose a usable TOC.
- Use `action=query&prop=extracts&exintro=1&explaintext=1` only as a fallback
  for summaries.

## Authentication

### OAuth 2.0 Authorization Code + PKCE (primary)

- Authorization: `GET https://meta.wikimedia.org/w/rest.php/oauth2/authorize`
- Token: `POST https://meta.wikimedia.org/w/rest.php/oauth2/access_token`
- Profile: `GET https://meta.wikimedia.org/w/rest.php/oauth2/resource/profile`
- PKCE: `S256`, random 64-byte verifier, 32-byte state.
- Redirect: `http://127.0.0.1:{ephemeral}/callback` with a loopback listener.
  The registered OAuth consumer must allow the loopback callback. If the
  provider rejects plain-HTTP loopback redirects, fall back to a manual
  authorization-code paste dialog.
- Scopes: request the minimum needed: `editpage` and, only when the watchlist
  feature is used, `watchlist`. Do not request `read` unless required.
- Store access and refresh tokens in the OS keyring. Never write them to the
  config, cache DB, logs, or crash reports.
- Refresh 60 seconds before expiry; a failed refresh moves the session to
  `Expired` and prompts for re-login.

### Bot password (fallback)

```text
action=query&meta=tokens&type=login
action=login&lgname=User@BotName&lgpassword=...&lgtoken=...
```

- Handle `NeedToken`, `WrongPass`, `NotExists`, `Throttled`, and `2FA`.
- For 2FA, prompt for `OATHToken` and retry once.
- Store the password only in the keyring, never on disk in plaintext.
- Display a warning that bot passwords grant the bot's rights and should be
  scoped narrowly.

### CSRF

```text
GET /w/api.php?action=query&meta=tokens&type=csrf&format=json&formatversion=2
```

Refresh the token when an edit returns `badtoken`, then retry once.

## Editing protocol

1. Fetch current wikitext, `revid`, timestamp, content model, and protection.
2. Get CSRF token if not cached.
3. Local edit buffer; autosave a draft every 2 seconds of idle time.
4. Preview (debounced 400 ms, cancellable):
   `action=parse&title={title}&text={wikitext}&prop=text&preview=true`.
5. Diff: local diff preferred; `action=compare&fromrev={base}&torev={?}` cannot
   compare unsaved text, so use a local diff for preview and the server compare
   only for revision-vs-revision.
6. Save:
   `action=edit&title={title}&text={wikitext}&summary={summary}`
   `&basetimestamp={base}&starttimestamp={start}`
   `&token={csrf}&format=json&formatversion=2`
   plus `minor=1`, `watchlist=watch|unwatch|nochange`, and section if editing
   a section.
7. On success: store the new `revid`, refresh the article, clear the draft.
8. On `editconflict`: fetch the latest revision, show a three-way merge/diff,
   and never silently overwrite.
9. On `protectedpage`, `permissiondenied`, `blocked`, `spamblacklist`,
   `abusefilter-disallowed`, `captcha`, or `ratelimited`: render the exact
   server message with a recovery action. Do not retry automatically.
10. Only one edit in flight. Disable the save button while saving.

## Request policy

- `User-Agent`: `gtw/{version} (https://github.com/prjctimg/gtw; contact)`.
- `Accept-Encoding: gzip, br`; `Accept: application/json` or `text/html`.
- `maxlag=5` on Action API calls; on `maxlag` errors, wait and retry once.
- Minimum 100 ms between writes; one write at a time.
- GET concurrency limit 4; background 2.
- Respect `Retry-After`; cap automatic wait at 30 s and show a countdown.
- Never scrape Special pages, never bypass authentication, never use a
  headless browser for article content.

## Data freshness

| Resource | TTL / policy |
|---|---|
| Article HTML | ETag revalidation; show cached immediately, revalidate in background |
| Summary/peek | 24 h; in-memory LRU 15 min |
| Media list | 7 d |
| Feed | per date, 1 h; older dates immutable |
| Search | 5 min; debounce 250 ms |
| Siteinfo | 24 h |
| Bookmarks/history | local only |
| Watchlist | 60 s while open |
| User profile | session lifetime |

## URL and title rules

- Decode HTML entities before storing titles.
- Strip URL fragments into `anchor`; anchors map to section ids.
- Normalize `%20` and `+` once; never double-encode.
- Preserve interwiki and language links as external unless the target is the
  current wiki.
- Reject `javascript:`, `data:`, and unsupported schemes in article links.
