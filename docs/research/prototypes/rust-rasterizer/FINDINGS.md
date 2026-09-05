# skia-safe vs tiny-skia under a Rust host, decoding real video

Prototype for [#34](https://github.com/MBehtemam/Montaget/issues/34), part of the map
[#2](https://github.com/MBehtemam/Montaget/issues/2). **Throwaway.** Nothing here is a
proposed design; `scene.json` is #6's intermediate, not a candidate project format.

## Verdict

**The axis discriminates, and it names `skia-safe`.**

`skia-safe` is **2.0× faster per frame** than `tiny-skia` on this scene (6.7 ms vs 13.5 ms
resident at 1080×1920), and the gap decides the only budget that is actually a gate: the
10 s preview. **With a real video clip on the timeline, `tiny-skia` misses the < 5 s
preview budget at both mid-timeline positions (7.14 s @ t=40, 6.02 s @ t=55) while
`skia-safe` clears every one (max 4.31 s).** Without video `tiny-skia` sits *on* the line
(4.92–5.03 s) — so it was already marginal, and decode is what pushes it over.

**What the decode numbers did to the picture: less than #6 feared, and in the opposite
direction to the worry.** Decode is 0.63–0.68 s of a 10 s preview (~16 %), it is
*identical in both arms* (so it does not discriminate), and **#6's one real pathology does
not reappear.** Time-to-first-frame after an input seek is flat in position —
0.10–0.18 s at t=0, 10, 20, 30, 40, 50, 60. #6 guessed this correctly: the pathology
"is architectural for an FFmpeg backend specifically… because a synthesised filtergraph
has no decoder to seek." Give it a decoder and it goes away.

Decode's real effect is that it **raises the floor under every preview by ~1.3–2.1 s**,
which is exactly enough to convert `tiny-skia` from marginal to failing.

## Machine and versions

Apple **M1 Pro**, 10 cores, 16 GB, macOS 26.5.2. `rustc` **1.95.0**, ffmpeg **8.0.1**.
`skia-safe` **0.153.2**, `tiny-skia` **0.12.0**, `parley` **0.11.1**, `skrifa` **0.46.2**,
`image` **0.25.10**. Same machine as #6 and the FFmpeg baseline. Wall clock, single runs.

## Method

Both arms consume **one shared ops list** — `src/ops.rs` is a line-for-line port of #6's
`ops.js`, over #6's `scene.json` unchanged. They differ only in `src/skia_arm.rs` and
`src/tiny_arm.rs`. Ground rules from the ticket, all held:

- **Text comes from `parley` in both arms** (ADR-0009). `parley` shapes and positions,
  `skrifa` scales the outlines, the rasterizer fills the paths. `src/text.rs` is shared
  verbatim; the glyph-outline cache is shared, and each arm caches its own native path.
- **FFmpeg is a subprocess in both arms** (ADR-0009 — the licence, not the speed).
  `src/media.rs` is shared: one `-ss`-seeked process per clip, RGBA over a pipe in,
  RGBA over a pipe out to `libx264`.
- **No automatic wrapping** (ADR-0007): each `\N` line is one `parley` layout with no wrap
  width. Block placement is #6's `size × 1.2` leading and ASS `\an` anchor, kept so the
  frames stay comparable.

### The video clip

#6 rendered **zero decoded video frames**. This run puts the fixture's own published MP4 —
1080×1920 h264, 50 fps, 65.26 s, the only real video file in the repo — on the timeline as
a clip from **t=30 to t=60, entering the source at 5 s**, composited into the 1080×1300
card. The previews at t=40 and t=55 therefore both open the decoder **mid-clip**, which is
the seek the ticket asked for. (Its own baked-in badge and subtitles show through in the
frames; using the output as an input is incestuous but the codec, the bitrate and the
decode work are real.)

### Correctness before timing

The #6 ground rule — two renders that completed, looked plausible and were wrong. Every
number below was recorded only after this table:

| | mean Δ/255 | max Δ | px differing > 8 |
| --- | --- | --- | --- |
| skia-safe vs tiny-skia @ t=0 | 0.053 | 57 | 0.22 % |
| skia-safe vs tiny-skia @ t=40 | 0.067 | 61 | 0.28 % |
| skia-safe vs tiny-skia @ t=40, **with video** | 0.060 | 61 | 0.28 % |
| skia-safe vs **#6's skia-canvas** @ t=0 | 0.204 | 159 | 0.36 % |
| tiny-skia vs **#6's skia-canvas** @ t=0 | 0.205 | 152 | 0.37 % |

The two arms agree to antialiasing noise, and they diverge from #6's independently
produced Node frame **by the same amount** — so neither arm is quietly wrong in a way the
other hides. Frames are in `frames/`.

## Results

### 1. Full render — 65.216 s of output at 1080×1920/30 (budget < 2 min)

| Backend | Wall | raster | decode | pipe+encode | vs realtime | Budget |
| --- | --- | --- | --- | --- | --- | --- |
| **skia-safe** | **20.72 s** | 13.85 s | — | 5.29 s | 3.1× | met, ~5.8× over |
| **tiny-skia** | **31.05 s** | 26.48 s | — | 4.43 s | 2.1× | met, ~3.9× over |
| **skia-safe**, with the video clip | **23.25 s** | 17.32 s | 1.79 s | 4.62 s | 2.8× | met, ~5.2× over |
| **tiny-skia**, with the video clip | **37.69 s** | 33.60 s | 1.68 s | 3.95 s | 1.7× | met, ~3.2× over |

Both clear it comfortably; **the full-render budget does not discriminate**, which is what
#6 concluded about every backend it measured. Note that neither Rust arm beats #6's Node
`skia-canvas` (13.07 s) — see *"Slower than Node"* below.

### 2. 10 s preview (budget < 5 s) — the gate that eliminated the browser pipelines

| Backend | @ 0 s | @ 40 s | @ 55 s | Budget |
| --- | --- | --- | --- | --- |
| **skia-safe** | 3.07 s | 2.95 s | 2.95 s | **met** |
| **tiny-skia** | 4.96 s | 5.03 s | 4.92 s | **on the line** |
| **skia-safe**, with video | 2.97 s | **4.31 s** | 3.59 s | **met** |
| **tiny-skia**, with video | 4.80 s | **7.14 s** | **6.02 s** | **fails** |

**This is the whole decision.** It is the same gate, on the same fixture, that #6 used to
remove Remotion and DIY Chrome — and `tiny-skia` fails it by 1.2–1.4× once the timeline
contains video, which the map says timelines do.

### 3. Single frame, cold process — the agent self-verification loop

| Backend | @ 0 s | @ 40 s | @ 55 s |
| --- | --- | --- | --- |
| **skia-safe** | 0.12 s | 0.12 s | 0.11 s |
| **tiny-skia** | 0.13 s | 0.13 s | 0.13 s |
| **skia-safe**, with video | 0.12 s | 0.25 s | 0.25 s |
| **tiny-skia**, with video | 0.13 s | 0.27 s | 0.26 s |

**The resident-server question dissolves.** ADR-0009 noted nobody has ever measured
whether the server is resident; on these numbers it does not need to be. A cold process —
launch, load the scene, discover fonts, decode the still, seek the clip, rasterize, encode
PNG — costs **0.11–0.27 s**, against #6's `skia-canvas` **0.50 s** cold and FFmpeg
**0.93 s**. This is the axis where the Rust host pays off, and **it does not discriminate
between the rasterizers** (0.12 vs 0.13 s).

Encoding is 10–12 ms of that, all of it PNG. That number belongs to the open "what image
format `frame` returns" question, not here.

### 4. Resident per-frame cost, and where the frame actually goes

| | skia-safe | tiny-skia | ratio |
| --- | --- | --- | --- |
| full frame @ t=40 | **7.27 ms** | **13.75 ms** | 1.9× |
| …without text | 6.80 ms | 12.46 ms | 1.8× |
| …without the still | 0.72 ms | 1.23 ms | 1.7× |
| …neither (background + rects only) | 0.32 ms | 0.13 ms | **0.4×** |
| full frame @ t=40, with video | 11.16 ms | 20.81 ms | 1.9× |

Decomposed: **bilinear resampling of the Ken Burns still is ~93 % of the frame in both
arms**, and it is where the entire gap lives (6.5 ms vs 12.3 ms). Glyph-path filling is
0.4 ms vs 1.1 ms — real, but 5 % of the frame. And on flat fills **`tiny-skia` is 2.5×
*faster*.**

So the honest statement is narrow: *`tiny-skia`'s image resampler is about half the speed
of Skia's, and this scene is an image resampler benchmark.* Every workload Montaget cares
about is — stills, video frames, and scaled clips are the substance of a video editor.

### 5. The 4K pass

The budget was written for 1080×1920/30 and has not been re-derived since ADR-0003
generalised the scope. At 2160×3840:

| | skia-safe | tiny-skia |
| --- | --- | --- |
| resident frame @ t=0 | 28.76 ms (34.8 fps) | 50.52 ms (19.8 fps) |
| 10 s preview @ t=40, with video | **19.04 s** | **29.99 s** |

**Both blow the < 5 s preview budget by 4–6× at 4K**, decode included (3.2–3.8 s of it).
The ratio between the arms holds at 1.8×, so this does not change the choice — but it does
say the preview budget is a 1080p statement, and #7 should not carry it to 4K unexamined.
This is a fact about the *budget*, not about either rasterizer.

### 6. Build story

| | clean-machine `cargo build --release` | notes |
| --- | --- | --- |
| `tiny-skia` alone | **4.7 s** | pure Rust, 17 MB of target |
| `skia-safe` alone | **15.8 s** | prebuilt hit; 210 MB of target |
| this harness (both, + parley + image) | **57.4 s** | 11.4 MB binary, 9.6 MB stripped |

Each measured with an **empty `CARGO_HOME` and an empty target directory** — no warm
registry, no cached download.

**`skia-safe` hits its prebuilt, and the key is the one ADR-0009 predicted.** The real
feature set — `skia-safe` defaults, `aarch64-apple-darwin` — resolves to exactly:

```
https://github.com/rust-skia/skia-binaries/releases/download/0.153.2/
  skia-binaries-b0260d93e48425b4b39f-aarch64-apple-darwin-jpegd-jpege-pdf.tar.gz
```

`jpegd-jpege-pdf` — the default key. ADR-0009 was right that textlayout-free prebuilts
exist and right that matching is on the exact sorted feature set; **on the feature set this
design actually needs, it matches.** No local Skia compile, no `ninja`, no Python.
15.8 s is not a build-time argument against `skia-safe`, and the ~200 MB of build artifacts
is the honest cost instead.

### 7. Release cadence, corrected

The ticket carries "one release in ~2 years" as an objection to `tiny-skia`. Precisely:
**`0.11.4` shipped 2024-02-04 and `0.12.0` shipped 2026-02-02** — a genuine two-year gap,
which **ended seven months ago**. `skia-safe` shipped `0.153.2` on 2026-09-03. The gap was
real; describing `tiny-skia` as unmaintained *today* is not supported.

## What this does not settle

- **The other objections to `tiny-skia` stand untested here**: no GPU, and no image filters
  or blur. This ticket measured speed and decode. Blur is an effect-model question
  ([#22](https://github.com/MBehtemam/Montaget/issues/22)), and on these numbers a CPU
  rasterizer at 7 ms/frame does not need a GPU at 1080p — but it does at 4K, where
  `skia-safe` is 29 ms/frame on CPU alone.
- **Neither arm was used idiomatically for text.** Both fill `skrifa` outlines per glyph,
  because ADR-0009's ground rule says so and because it keeps the arms comparable. Skia's
  own glyph atlas would make `skia-safe` faster still and has no `tiny-skia` equivalent —
  so this measurement *understates* the gap. It does not overstate it.
- **Slower than Node.** #6's `skia-canvas` renders this scene in 13.07 s where `skia-safe`
  here takes 20.72 s. The likely cause is the text path above plus `skia-canvas`'s
  multithreaded page rendering; this harness is single-threaded throughout. The comparison
  that is sound is **arm against arm on one shared ops list**; the cross-language absolute
  is not, and #7 should not read "Rust is slower than Node" out of this file.
- **The font is a system font.** `SF Pro Rounded` is resolved through `fontique`, identically
  in both arms. ADR-0007 requires fonts to be declared files and no font is checked into
  this repo — that gap is still open and untouched by this run.

## Reproducing

```
./run.sh          # every number above, ~10 min
cargo build --release
./target/release/rast-bench --backend=skia|tiny \
   [--scene=scene-video.json] [--from=S --to=S | --still | --bench=N] \
   [--scale=2] [--notext] [--noimage] [--noaudio] --out=…
```

`scene.json` is #6's, verbatim. `scene-video.json` is that file plus one `clips` entry.
