# `benchmark` — the project `render`'s speed target is stated for

[ADR-0142](../../docs/adr/0142-render-has-one-speed-target-the-benchmark-project-in-three-minutes.md)
gives `render` one number: **the benchmark project renders in at most 3 minutes, median wall
clock, on the dev's M1 Pro** (`montagent_render::budget::RENDER_TARGET`). This directory holds
the script that builds that project. Everything it builds is generated from the committed
fixtures into a directory you name, and nothing generated is committed.

```sh
python3 fixtures/benchmark/make_benchmark.py <out-dir>              # all three projects
python3 fixtures/benchmark/make_benchmark.py <out-dir> --only-1080  # skip the 4K sources
```

The directory also holds the two **paint-heavy yardsticks** of the paint-speed map
([#641](https://github.com/MBehtemam/Montagent/issues/641)). Neither carries a number yet:

- **[`spy-trailer/`](spy-trailer/README.md), the paint target.** It is a reduced copy of the
  36 s spy trailer that rendered at 0.13× realtime
  ([#509](https://github.com/MBehtemam/Montagent/issues/509)). It has no `video`: 104 texts,
  43 images, 17 rects and 10 ellipses on 114 tracks. It is committed media, with its frame
  hashes beside it.
- **[`--paint`](#the-paint-bench), the profiling bench.** It is a generated project with one
  knob per suspect the trailer raised.

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

## The paint bench

`--paint` writes one project in place of the benchmark projects. It is 36 s of 1920x1080 at
30 fps with no `video` and no audio, so painting is the only work in the render. It needs
no ffmpeg to build, and it takes its media from `spy-trailer/` (the `lair` photo, the grain
plate and Oswald). With every knob at its default, it is a still photo over the whole
timeline: the floor each knob is measured against.

| knob | default | what it adds |
|---|---|---|
| `--blur-texts N` | 0 | N texts on screen for the whole timeline, drifting left, each with the trailer's `blur` (radius 14) |
| `--glow-texts N` | 0 | the same, each with the trailer's zero-offset `shadow` glow (radius 16) |
| `--plate off\|still\|moving` | `off` | the trailer's 2400x1400 alpha grain over the whole frame: absent, still, or jumping to a seeded offset every other frame, the way the trailer moves it |
| `--extra-tracks N` | 0 | N more tracks, each holding one 24 px rect on screen for 1 s and staggered across the timeline, so the track count grows while the painting barely does |

```sh
python3 fixtures/benchmark/make_benchmark.py <out-dir> --paint                    # the floor
python3 fixtures/benchmark/make_benchmark.py <out-dir> --paint --plate moving
python3 fixtures/benchmark/make_benchmark.py <out-dir> --paint --blur-texts 14 --glow-texts 38 --plate moving --extra-tracks 108
```

The last line is about the trailer's own counts: 14 blurred and 38 glowing texts, a moving
plate, and 108 single-element tracks. Its texts are on screen for all 36 s, though, while
the trailer's come and go. The file name records the knobs, e.g.
`paint-b14-g38-moving-t108.montagent.json`, so one directory can hold a whole sweep.

The knobs only apply with `--paint`; given alone they are an error. Without `--paint`, the
script writes the same bytes it wrote before `--paint` existed. This was checked by hashing
all three projects, so ADR-0142's benchmark project has not moved.

## Known limits

- **The footage is an animated short**, not camera footage, so at CRF 20 the 1080p sources are
  about 1.75 Mb/s (6 Mb/s at 2160p) — well under a phone's 10–20 Mb/s. Decode cost grows with
  bitrate, so a real project's decode may cost more per frame than this one's.
- **The 2160p sources are upscaled** from the 1080x1920 original. They decode like 4K and look
  like 1080p.
