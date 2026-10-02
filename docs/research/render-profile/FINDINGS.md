# Where a long render's time goes

Profile for [#623](https://github.com/MBehtemam/Montagent/issues/623), part of the map [#622](https://github.com/MBehtemam/Montagent/issues/622) (render a 6-minute video in minutes, not an hour). Measured 2026-10-02.

## Pinned

- **Commit:** `c851cd70` (v0.1.2), plus the profiling patch on this branch. It adds counters in `montagent-render/src/prof.rs`, timers in `decode::frame_at` and in `render.rs`'s frame loop, and prints only when `MONTAGENT_PROFILE` is set. It is never meant for `main`.
- **Machine:** Apple M1 Pro (8 performance + 2 efficiency cores), 16 GB, macOS 27.0, Homebrew ffmpeg 9.0.2. Release build. Other sessions were running: load average 5–9 during the renders, after another agent's benchmarks had stopped.
- **Projects:** the slow 6-minute project wasn't available, so this follows the ticket's fallback, a committed fixture extended past a minute. Both projects are 64 s of 1920x1080 at 30 fps:
  - `screen/session.mp4` (1080p25 H.264, no audio) fills the frame from 0–55 s;
  - `presenter/take-1.mp4` (1080p25 H.264 + AAC) fills the frame from 55–64 s, with audio mixed;
  - a rect and a text layer sit on top for the whole span.
  - `long-gop` uses the committed fixtures as they are. The screen recording's keyframes are up to 10 s apart (8 keyframes in 55 s), and the presenter clip has one keyframe in 9.8 s.
  - `short-gop` uses copies re-encoded with a keyframe every second (`-g 25`), the spacing typical of camera footage.
- **Reproduce:** `docs/research/render-profile/profile.sh` from the repo root.

## The table

1,920 frames per render; one video element visible on every frame.

| stage | long GOP, ms/frame | 1 s GOP, ms/frame | share of wall |
|---|---|---|---|
| `frame_at` (decode, incl. spawns) | **112.7** | **93.2** | 95% |
| ↳ inside the spawned `ffmpeg` | 112.2 | 92.5 | |
| paint, excluding decode (Skia blit, rect, text) | 2.3 | 2.3 | 2% |
| canvas readback to RGB | 1.2 | 1.2 | 1% |
| encoder backpressure (`push`) | 1.8 | 1.7 | 1.5% |
| seal (encoder drain + mux), whole render | 108 ms | 107 ms | ~0 |
| **wall** | **118.2** (226.9 s) | **98.5** (189.1 s) | |
| CPU user + sys, all processes | 588 s (2.6 cores) | 332 s (1.8 cores) | |

| counter | long GOP | 1 s GOP |
|---|---|---|
| `frame_at` calls | 1,920 | 1,920 |
| `ffmpeg` spawns | 1,920 | 1,920 |
| fallback spawns | **0** | **0** |
| rgba frames piped back, only the last of each kept | 9,890 (5.2 per call, ~43 MB) | 9,890 |
| source frames decoded (`gop-cost.py`) | 205,230 (107 per call, worst 255) | 33,620 (17.5 per call, worst 30) |
| source frames a single stream would decode | ~1,600 | ~1,600 |

Max RSS: 1.14 GB.

## Spawn overhead against window decode

Each row is the mean of 30 spawns of `frame_at`'s own command, on `session.mp4` at 1080p rgba:

| what the spawn does | ms |
|---|---|
| bare `ffmpeg` start, lavfi 16x16, 1 frame | 35 |
| open + seek to a keyframe + decode, scale and pipe 1 frame | 55 |
| `frame_at` 0.2 s past a keyframe | 63 |
| `frame_at` 9.9 s past a keyframe | 144 |

- **About 55 ms of every call is fixed:** process start, open, seek and the first frame. That is half of a long-GOP call and most of a short-GOP one.
- **Decoding forward from the keyframe** costs the rest: up to ~80 ms at a 10 s GOP.
- **Shortening the GOP sixfold cut decoded frames by 84% but wall time by only 17%**, while CPU dropped 44%. The fixed per-spawn cost, not the decode, is what a short GOP can't remove.

## Encode and audio, measured alone

- **Encoder** (`encode.rs`'s exact arguments, libx264 medium / CRF 20, rgb24 on a pipe): 1,920 frames in 6.9 s, **~280 fps**. In the render it runs concurrently and stays ahead of the painter: 1.8 ms/frame of backpressure.
- **Audio mix:** a 6-minute 48 kHz stereo bus with two inputs (aresample, atrim, adelay, amix, apad) encoded to AAC 160k in **4.7 s**. It runs inside the encoder process, alongside the video.

## What it says

1. **Decode is the render.** At 95% of wall it is the only stage worth attacking first. Everything else together is ~5.3 ms/frame. Without the decode, one thread would paint at ~190 fps, ahead of the encoder's ~280 fps. A 6-minute 1080p30 render would then take about a minute.
2. **Per-frame spawning wastes in two ways, and only streaming fixes both.** Every output frame pays (a) ~55 ms of fixed spawn/open/seek cost, and (b) a re-decode from the keyframe: 107 source frames per output frame here, 128× what a single stream decodes. It also pipes ~5 whole rgba frames to keep one. A shorter GOP only shrinks (b).
3. **Cost scales with visible video elements.** At this rate a 6-minute render with one visible video takes ~18–21 min, and each extra video on screen adds ~95–115 ms/frame, ~17–20 min. The reported hour fits about three visible videos, or one at 4K.
4. **The fallback spawn never fired.** It is not part of the cost.
5. **The machine is mostly idle during a render:** 1.8–2.6 cores of 10. Parallelism has room, but parallelising today's path would multiply the waste across cores, as the map's notes say.
