# Security and authentication

## Threat model

Protect against:

- token/password leakage through logs, crash reports, config files, shell
  history, or process listings;
- malicious HTML/links/media in articles;
- CSRF and edit conflicts;
- session theft on a shared machine;
- accidental publication of private drafts or watchlists;
- OAuth redirect interception on loopback.

Not in scope: a compromised OS account, malicious terminal emulator, or
Wikipedia server compromise.

## OAuth 2.0 Authorization Code + PKCE

### Flow

```text
1. Generate 64-byte verifier, S256 challenge, 32-byte state.
2. Start loopback listener on 127.0.0.1:<ephemeral>.
3. Open browser:
   https://meta.wikimedia.org/w/rest.php/oauth2/authorize
     ?response_type=code
     &client_id=<consumer>
     &redirect_uri=http://127.0.0.1:<port>/callback
     &scope=editpage%20watchlist
     &state=<state>
     &code_challenge=<challenge>
     &code_challenge_method=S256
4. Wait for GET /callback?code=...&state=...
   - reject wrong state, duplicate callback, or missing code;
   - 120 s timeout; cancel button closes the listener.
5. POST /w/rest.php/oauth2/access_token
     grant_type=authorization_code
     code=<code>
     redirect_uri=<same>
     client_id=<consumer>
     code_verifier=<verifier>
6. GET /w/rest.php/oauth2/resource/profile with Bearer token.
7. Store tokens in the OS keyring; keep only the username in the session.
```

- Do not log the authorization code, verifier, state, or tokens.
- The loopback listener binds only to `127.0.0.1`, accepts one request, and
  closes immediately after the callback.
- The callback page is a static HTML “You can return to gtw” page; it never
  echoes the code.
- If the provider rejects plain-HTTP loopback redirects, show a manual
  authorization-code paste dialog. The code is exchanged with the same PKCE
  verifier.
- Consumer registration is user-supplied; `client_id` belongs in config, and
  the client is public (no secret). Never ship a shared secret.
- Scopes are minimal. Request `watchlist` only when the user enables the
  watchlist; re-authorize if the scope is missing.

### Refresh and logout

- Refresh 60 s before expiry with the refresh token.
- A failed refresh sets `AuthSession::Expired`; preserve the article and ask
  for re-login.
- Logout revokes locally, deletes keyring entries, clears CSRF and user info,
  and drops watchlist data. Offer “revoke on Wikimedia” as a browser action.
- Device/session list is not available via the API; do not invent one.

## Bot password fallback

- Username format: `User@BotName`.
- Login tokens via `action=query&meta=tokens&type=login`; then `action=login`.
- Handle `NeedToken`, `WrongPass`, `NotExists`, `Throttled`, `Blocked`, `2FA`.
- 2FA prompt: `OATHToken`; never store the OTP.
- Bot passwords are stored in the OS keyring under a distinct key; never in
  config or the cache DB.
- Warn that bot passwords grant the bot’s rights and should be scoped to the
  minimum grants (edit, watchlist).

## Secret storage

- Primary: OS keyring (`keyring` crate; Secret Service on Linux).
- If no keyring is available:
  - do not write plaintext;
  - offer session-only credentials (prompt on each start) or
    `GTW_TOKEN`/`GTW_BOT_PASSWORD` environment variables for CI/testing;
  - display a clear “credentials will not be saved” warning.
- Zeroize token/password buffers on drop where practical (`zeroize`).
- Never include secrets in `Debug` output; wrap in `SecretString`.
- Never pass secrets on the command line (visible in process listings).
- File permissions: config/data/cache dirs `0700`, files `0600` where they
  contain user data; drafts are user data.

## Article and media safety

- Treat all HTML, alt text, captions, and metadata as untrusted.
- Allowlist link schemes: `http`, `https`, `mailto`, `wiki`, `media`.
  Reject `javascript:`, `data:`, `vbscript:`, `file:`, and unknown schemes.
- Do not execute scripts, iframes, or event handlers. `scraper` extracts text
  and attributes only; never render raw HTML.
- Sanitize control characters and bidi overrides from titles, snippets, and
  captions to prevent terminal spoofing.
- Escape ANSI escape sequences from external text; only gtw emits styles.
- Media URLs must be `https://upload.wikimedia.org` or the configured wiki’s
  upload host; reject redirects to arbitrary hosts unless explicitly allowed.
- SVG rendering must not fetch external resources; disable network/file access
  in `resvg` and strip scripts/foreignObject.
- Do not auto-open external links; require a keypress.
- Downloads are content-length limited (configurable; images 100 MB, audio
  500 MB, video 4 GB) and streamed to disk.

## Edit safety

- Always send `basetimestamp`; never overwrite a newer revision.
- On conflict, fetch latest, show a three-way diff, and require an explicit
  choice. Never auto-merge silently.
- Preview before save is recommended but not mandatory; the save dialog shows
  the diff summary and byte delta.
- Warn on blanking a page, large deletions (> 500 bytes or > 20%), or removing
  references/categories.
- Show the exact abuse filter/captcha/protection message; do not retry writes
  automatically.
- One write at a time; disable save while in flight.
- Drafts are local and never uploaded unless the user saves.
- Do not add “bot” flags or tags the user did not request.

## Network security

- HTTPS only; TLS verification always on. No `danger_accept_invalid_certs`.
- Respect system proxy settings via `reqwest` defaults.
- Certificate errors are fatal for that request; do not fall back to HTTP.
- OAuth endpoints are pinned to `meta.wikimedia.org`; do not follow redirects
  to other hosts during token exchange.
- Validate `state`; use constant-time comparison for codes/tokens where
  applicable.
- No telemetry, analytics, or remote logging.

## Logging and diagnostics

Allowed:

- request ID, method, host, path without query secrets, status, bytes, timing;
- cache key hash (not full URL with tokens);
- decoder/protocol names and dimensions;
- error categories.

Forbidden:

- Authorization headers, cookies, tokens, CSRF, passwords, OTPs;
- full wikitext or article bodies;
- clipboard contents;
- watchlist contents at info level.

Crash reports must be reviewed for secrets before display; default to a local
log path, not a remote upload.

## Tests

- OAuth state mismatch, timeout, cancellation, duplicate callback, token
  exchange failure, refresh failure, keyring unavailable.
- Bot password 2FA, wrong password, throttle.
- CSRF badtoken retry once.
- Edit conflict: base revision changes between load and save.
- Link scheme rejection and ANSI injection fixtures.
- SVG with external image/script references is sanitized.
- Secret redaction in logs (grep test).
