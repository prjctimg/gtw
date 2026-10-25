# M0.5 Media Spike (no-ffmpeg)

Status: completed (scaffold)

## Requirements
- Download a 10–30 s VP9/Opus WebM from Wikimedia
- Demux and decode first 120 frames on aarch64 with zvidlib
- Decode Opus track to PCM and play via rodio
- Seek to 50% and resume
- No ffmpeg process, no libavcodec linkage, no UI-thread block
- Record decode throughput and memory

## Plan
1. Try zvidlib first (pure-Rust VP9 + WebM demux) - added as optional dep
2. If fails, try rusty_vp9 + matroska-demuxer + pure-Rust Opus (deps added)
3. If both fail, document fallback to poster + external open

## Notes
- Symphonia doesn't decode video; use zvidlib for VP9/Opus in WebM
- No ffmpeg crates/binaries allowed
- Deps prepared: zvidlib (optional), rusty_vp9, matroska-demuxer
- Spike example scaffolded
