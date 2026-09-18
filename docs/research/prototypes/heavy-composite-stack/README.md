# heavy-composite-stack — measuring `preview` under multiple simultaneous clips/effects

Prototype for [#159](https://github.com/MBehtemam/Montaget/issues/159), graduated from
[#87](https://github.com/MBehtemam/Montaget/issues/87), part of the map
[#2](https://github.com/MBehtemam/Montaget/issues/2). **Throwaway. The answer is in
[FINDINGS.md](FINDINGS.md).**

Reuses [#34](https://github.com/MBehtemam/Montaget/issues/34)/[#87](https://github.com/MBehtemam/Montaget/issues/87)'s
`rast-bench` binary (`../rust-rasterizer/`) with one small, backward-compatible extension:
a `Span` (Ken Burns still) may now carry an optional destination rect (`dx`/`dy`/`dw`/`dh`),
and every span whose time window covers the current frame is drawn (previously only the
first match, via a `break` that's now removed). This is what lets a scene express **two**
simultaneous Ken Burns stills — a full-card background and a PiP inset — where before only
one was expressible. Scenes with no `dx`/`dy`/`dw`/`dh` on their spans (every prior scene
in this repo, including `../proxy-preview-savings/scene-*.json`) render byte-identically
to before: the defaults reproduce the old hardcoded full-card placement exactly.

`Clip` (video) already supported more than one simultaneous entry with no change — #87's
scene just never used more than one.

| file | |
| --- | --- |
| `scene-4k.json`, `scene-8k.json` | 2160x3840 / 4320x7680 canvases, each with **2 spans, 2 clips, 20 events** live in the same `[5, 25)` window (#87's scenes: 1 span, 1 clip, ~9 events) |
| `gen_scenes.py` | one-off generator that produced the two scene files above (not run by `run.sh` for the committed scenes, but idempotent — reruns produce the same files) |
| `generate-media.sh` | generates the synthetic 4K/8K `a-*` (main) and smaller `b-*` (PiP inset) stills/video into `media/` (not committed — see FINDINGS.md) |
| `run.sh` | every number in FINDINGS.md |
| `frames/` | correctness-check frames (skia-safe vs tiny-skia, committed) |

Needs `ffmpeg`/`ffprobe` on `PATH` and a built `../rust-rasterizer/target/release/rast-bench`
(`run.sh` builds it if missing — this also rebuilds the shared harness with the span-dest-rect
extension above).

```
./run.sh
```
