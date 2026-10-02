# `benchmark` — the project `render`'s speed target is stated for

[ADR-0142](../../docs/adr/0142-render-has-one-speed-target-the-benchmark-project-in-three-minutes.md)
gives `render` one number: **the benchmark project renders in at most 3 minutes, median wall
clock, on the dev's M1 Pro** (`montagent_render::budget::RENDER_TARGET`). This directory holds
the script that builds that project. It holds no media: everything is generated from the
committed fixtures into a directory you name, and nothing generated is committed.

```sh
python3 fixtures/benchmark/make_benchmark.py <out-dir>              # all three projects
python3 fixtures/benchmark/make_benchmark.py <out-dir> --only-1080  # skip the 4K sources
```

It is called the **benchmark project**, never the "reference project":
`RENDER_REFERENCES` already means reference *measurements*.

## What it builds

| file | role |
|---|---|
| `benchmark-1080p30.montagent.json` | **the benchmark project**, judged against `RENDER_TARGET` |
| `benchmark-2160p30.montagent.json` | the same timeline at 3840x2160, from 2160x3840 sources; observed, no number |
| `benchmark-1080p30-1video.montagent.json` | the centre lane only; observed, the source of the per-element rate |
| `media/lane-{a,b,c}-{1080,2160}.mp4` | three sources, re-encoded from `../en-halloween-decorating/reference/en-halloween-decorating.mp4` |
| `manifest.json` | the encode settings and input hash each source was built with |

The benchmark project is 6 minutes of 1920x1080 at 30 fps:

- **three `video` elements visible at once for 330 of the 360 s** — three portrait lanes
  side by side, each a run of back-to-back cuts (6 + 7 + 7 = 20 `video` elements), cut
  in at different source offsets and staggered so no two lanes cut on the same frame;
- **text**: an opening title and a new lower-third line every 10 s (37 `text` elements);
- **rects**: a lower-third strip, a title card, and a progress bar whose `x` moves on every
  frame;
- **an audio mix**: the three videos' own sound at `volume` 0.15, a narration line every 20 s
  (18 `audio` elements cycling the fixture's eleven) and a 6 s bed looped under all of it.

## The pinned re-encode

So the benchmark does not drift with the ffmpeg version, the sources are re-encoded with
settings fixed in the script: H.264 (libx264) High profile, level 4.1 (5.1 at 2160p),
`yuv420p`, preset `medium`, CRF 20, 3 B-frames, and a **closed, fixed 10 s GOP**
(`-g 300 -keyint_min 300 -sc_threshold 0`, `open-gop=0`): a keyframe exactly every 300
frames. The long GOP is the point. A seek lands up to 299 frames from the keyframe it has to
decode from, which is what camera footage costs and what the committed fixture's short GOP
hid. x264's bytes can still differ between builds; the structure cannot.

A re-run reuses sources whose `manifest.json` entry matches the script's settings and input,
and re-encodes otherwise (`--force` always re-encodes). Generating the media is never part of
a timed run.

## Measuring it

`crates/montagent/tests/render_target.rs` is the measurement, `#[ignore]`d and run on purpose:

```sh
cargo test --release -p montagent --test render_target -- --ignored --nocapture
```

Its module doc lists the environment variables (a pre-built directory, the variant, the run
count) and says exactly what its frame-hash check does and does not prove.

## Known limits

- **The footage is an animated short**, not camera footage, so at CRF 20 the 1080p sources are
  about 1.75 Mb/s (6 Mb/s at 2160p) — well under a phone's 10–20 Mb/s. Decode cost grows with
  bitrate, so a real project's decode may cost more per frame than this one's.
- **The 2160p sources are upscaled** from the 1080x1920 original. They decode like 4K and look
  like 1080p.
