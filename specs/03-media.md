# Media pipeline (no ffmpeg)

## Hard rules

- No `ffmpeg` binary, no `ffmpeg-sidecar`, no `ffmpeg-the-third`, no linking
  libavcodec.
- No shelling out to `mpv`, `ffplay`, `chafa`, `viu`, or `yt-dlp`.
- External open is a user-visible fallback only, never the default media path.
- Audio decoding: Symphonia 0.6.
- Video: pure-Rust VP9/VP8 + WebM demux via `zvidlib` 0.5.
- A CI check must fail if `ffmpeg` appears in `Cargo.lock` or in `build.rs`
  invocations.

Symphonia is an audio library. It does not decode video and does not provide a
native Opus decoder. Do not try to route video through it.

## Supported media

| Kind | Formats | Path |
|---|---|---|
| Image | JPEG, PNG, APNG, GIF, WebP, TIFF, SVG | `image` + `resvg` → decoded RGBA |
| Audio | MP3, Ogg Vorbis, FLAC, WAV, Opus | Symphonia → PCM → rodio (Opus via zvidlib/pure-Rust decoder) |
| Video | WebM VP8/VP9 + Vorbis/Opus | zvidlib demux/decode → RGBA + PCM |
| Animated | GIF, APNG, animated WebP | frame scheduler → protocol image or halfblock |
| MIDI/Score | MIDI, LilyPond-generated images | score image first; optional synth later |
| Document/3D | PDF, STL, glTF, OBJ | card + metadata + open externally |
| Captions | WebVTT/TimedText | optional caption overlay; hidden if unsupported |

### Wikimedia video selection

Preferred source order:

1. `video/webm` with codecs `vp9` + `opus`, largest width ≤ viewport width
   (or ≤ 2× viewport for zoom), capped at 1080p.
2. `video/webm` with `vp8` + `vorbis`.
3. Original WebM if it is VP8/VP9.
4. If only Ogg Theora or MPEG-4 Part 2 is available: show the poster and an
   “Open in browser” action. Do not pretend to play it inline.

Most current Wikimedia videos have VP9 transcodes. This is what makes the
no-ffmpeg requirement practical.

## Image path

```text
media-list / <img srcset>
    -> choose thumb width (320/640/1024/original)
    -> disk cache (blob by SHA-256 of URL + variant)
    -> decode (spawn_blocking)
    -> optional SVG rasterization at target pixel size
    -> resize to fit terminal cells * font pixel size
    -> protocol encode (Kitty/sixel/iTerm2) or halfblocks
    -> StatefulProtocol -> ratatui-image widget
```

- Prefer Wikimedia thumbnail URLs over originals to avoid 20–100 MB JPEGs.
- Extract `srcset` and pick the smallest candidate that covers the target;
  never download the original until the user opens the lightbox or zooms.
- SVG is rasterized with `resvg` at target pixel dimensions. Keep a text alt
  fallback if fonts are missing.
- Animated images use a shared 10 fps clock; only the visible image advances.
  Pause animation when the pane is not focused or the image is scrolled off.
- Decoded frames are bounded by an LRU (default 64 MB); encoded protocol
  buffers are not shared across terminal sessions.
- Failure placeholder: alt text, dimensions, and a retry action. Never collapse
  the layout to zero height.

## Audio path

```text
URL / <audio><source>
    -> cache or stream
    -> Symphonia probe + decode (or zvidlib Opus for video tracks)
    -> resample to device rate/channels (rubato)
    -> rodio Sink
```

- Controlled by one mini-player: play/pause, seek ±5 s, volume, mute, speed
  (0.5×–2×), loop, close.
- `<audio>` tags in the article HTML are authoritative for audio sources.
- Media-list audio entries may omit `sources`; do not fail the article if the
  audio manifest is incomplete.
- If the output device is unavailable, show `Audio unavailable` with a retry
  action; the rest of the app keeps working.
- Opus: use the pure-Rust decoder shipped by zvidlib for WebM audio; for
  standalone `.opus`, use the same decoder or convert to PCM on demand. Do not
  add a C libopus dependency unless the pure-Rust path fails the spike.

## Video path

```text
selected VP9/Opus WebM URL
    -> resumable HTTP download to media cache (range chunks)
    -> WebM demux (zvidlib-container)
    -> VP9 decode (zvidlib-vp9-decoder)         -> RGBA frames
    -> Opus/Vorbis decode (zvidlib)             -> PCM
    -> frame resize/letterbox
    -> bounded frame queue (2 frames)
    -> ratatui-image protocol (or halfblocks)
    -> rodio sink for audio
```

- Decode in `spawn_blocking`; never on the UI thread.
- Render at 12–15 fps max; drop late frames rather than queueing.
- Audio clock is the master clock. Video frames are presented when their PTS is
  within one frame interval; keep a 300–500 ms decode lead.
- Bounded queues: one decoded frame in flight, one ready frame; applying
  backpressure to the decoder when the UI is slow.
- Seek: use zvidlib's frame-accurate seek if available; otherwise restart at
  the nearest keyframe and discard until the target PTS. Show “buffering” until
  the first target frame is ready.
- Pause stops presentation but may keep a small decode lead; stop decoding
  after the queue is full.
- End of stream returns to poster with replay.
- Audio-only fallback is allowed if video decode fails: keep the audio player
  and show the poster with an error badge.
- Subtitles: parse WebVTT if present; render at most two lines above the video.
  If parsing fails, show “captions unavailable” without failing playback.

## Cache layout

```text
$XDG_CACHE_HOME/gtw/
├── cache.db
├── blobs/ab/<sha256>.zst          # HTML, JSON, wikitext
├── images/ab/<sha256>.<ext>       # decoded/encoded image source
├── media/ab/<sha256>.<ext>        # audio/video downloads
└── tmp/
```

- Media keys include URL, selected variant, and expected content type.
- Video/audio downloads are written to `tmp/` then atomically renamed; partial
  downloads keep a sidecar `.part.json` with ETag, total bytes, and chunks.
- Size policy: images 512 MB, audio 1 GB, video 4 GB (configurable). Pinned
  bookmarks and explicitly pinned media never evict.
- Eviction is LRU by last access, never while a player has the file open.
- Do not evict the currently displayed article, its images, or the active
  player.

## Terminal protocols

- `ratatui-image` `Picker` queries the terminal with a timeout.
- Preference: Kitty → sixel → iTerm2 → halfblocks.
- Kitty/sixel images are protocol layers, not cells. Tachyonfx cannot fade the
  pixels themselves; animate the placeholder/border/caption and swap the image
  atomically. Halfblock images are cells and can be crossfaded.
- On terminal resize or focus regain, re-detect the protocol and re-encode
  visible images; cancel stale encodes.
- In tmux/screen, detect passthrough support; otherwise fall back to
  halfblocks with a status note.

## Media viewer

- Full-screen or large centered modal.
- Header: title, MIME, dimensions/duration, source/license link.
- Image: fit/actual/zoom, pan, next/previous in gallery, caption, alt text.
- Audio: waveform (optional), elapsed/total, seek bar, volume, speed.
- Video: frame, buffering spinner, progress, play/pause, seek, mute, captions.
- Footer: `Space` play/pause, `←/→` seek, `↑/↓` volume, `f` fullscreen,
  `o` external, `Esc` close.
- Loading, buffering, paused, ended, unsupported, and error are distinct visual
  states.

## M0.5 media spike (blocking gate)

Before building the player UI:

1. Download a 10–30 s VP9/Opus WebM from Wikimedia.
2. Demux and decode the first 120 frames on aarch64 with zvidlib.
3. Decode the Opus track to PCM and play through rodio.
4. Seek to 50% and resume.
5. Verify no `ffmpeg` process, no libavcodec linkage, and no UI-thread block.
6. Record decode throughput and memory in `specs/media-spike.md`.

If zvidlib fails, try `rusty_vp9` + `matroska-demuxer` + a pure-Rust Opus
decoder. If both fail, the fallback is poster + external open, and the user
must be told that inline video cannot meet the no-ffmpeg constraint.
