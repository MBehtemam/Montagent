# Measured: Remotion, DIY headless Chrome and skia-canvas against the performance budget

Prototype for [#6](https://github.com/MBehtemam/Montaget/issues/6), part of the map [#2](https://github.com/MBehtemam/Montaget/issues/2).

This completes #6. The [FFmpeg baseline](../ffmpeg-baseline/README.md) measured the
incumbent; this run measures the three candidates that were scoped out of it, on the same
machine, in one session, against the same fixture.

**This is a throwaway.** Nothing here is a proposed design. `scene.json` in particular is
an *intermediate* built to make the four backends comparable — it is not a candidate
project format and should not be read as one.

## Method

All four backends render the **same content**: the real fixture at 1080×1920/30fps,
65.216 s = 1956 frames — 4 stills Ken-Burns'd into the top 1300 px card, 73 subtitle and
shape events, the badge overlay, and 12 narration placements mixed onto one clock.

To keep the comparison honest, `build_scene.py` derives a single **`scene.json`** from the
fixture (the same `merged.ass` events and span/audio tables the baseline rendered), and
`ops.js` turns *(scene, frame)* into a flat list of draw ops. **All three backends consume
the same ops.** They differ only in how they execute them, so a difference in the numbers
is the renderer and not the scene.

Correctness was verified before anything was timed — every backend was checked against the
baseline's own frame at t=0, and against a mid-timeline frame at t=40 s.

| | | |
| --- | --- | --- |
| ![skia](skia-frame-0s.png) | ![remotion](remotion-frame-0s.png) | ![chrome](chrome-frame-40s.png) |
| skia-canvas, t=0 | Remotion, t=0 | DIY Chrome, t=40 s |

## Machine

Apple **M1 Pro**, 10 cores, 16 GB, macOS 26.5.2. Node **v22.23.2**, ffmpeg **8.0.1**,
`skia-canvas` **3.0.8**, `remotion` **4.0.520** (Chrome Headless Shell 149.0.7790.0),
`puppeteer` **24.x**. Same machine as the baseline. Wall-clock, not published benchmarks.

## Results

FFmpeg column is the baseline, re-stated for comparison.

### Full render — 65.216 s of output

| Backend | Wall clock | vs realtime | Budget (< 2 min) |
| --- | --- | --- | --- |
| **FFmpeg** (incumbent) | **13.13 s** | 4.97× | met, ~9× over |
| **skia-canvas** | **13.07 s** | 4.99× | met, ~9× over |
| **DIY Chrome** (jpeg, 4 browsers) | **30.58 s** | 2.13× | met, ~4× over |
| **Remotion** (defaults) | **46.85 s** | 1.39× | met, ~2.5× over |

### 10 s preview — the budget half that matters

| Backend | @ 0 s | @ 40 s | @ 55 s | Budget (< 5 s) |
| --- | --- | --- | --- | --- |
| **FFmpeg**, naive `-ss`/`-to` | 2.51 s | 9.96 s | 12.82 s | **fails** past ~t=20 |
| **FFmpeg**, graph restructured | 2.19 s | 2.29 s | 2.23 s | met |
| **skia-canvas** | **2.06 s** | **2.18 s** | **1.99 s** | **met** |
| **Remotion**, cold CLI | 7.28 s | 7.04 s | 8.83 s | **fails** |
| **Remotion**, resident | 7.47 s | 7.38 s | 7.69 s | **fails** |
| **DIY Chrome**, best config | — | **8.25 s** | — | **fails** |

**Only FFmpeg and skia-canvas clear the preview budget. Both browser pipelines miss it by
~1.5–1.7×**, and no amount of tuning closed the gap — 8.25 s was the best DIY figure found
across formats, worker counts and process topologies.

### Single frame — the agent self-verification loop

| Backend | Cold (fresh process) | Resident (already up) |
| --- | --- | --- |
| **FFmpeg** | 0.93 s (png) · 0.48 s (jpg) | n/a — no resident mode |
| **skia-canvas** | **0.50 s** | **2.4–18 ms** raster + 163 ms png encode |
| **Remotion** | 1.74–1.88 s | **0.37–0.39 s** |
| **DIY Chrome** | 2.00–2.26 s | ~0.32 s (png) · 46 ms (jpeg) |

Every backend is flat in position — a frame at 55 s costs what a frame at 0 s costs.

## The four findings that matter for [#7](https://github.com/MBehtemam/Montaget/issues/7)

### 1. The survey's startup worry was wrong, and backwards

The baseline predicted browser startup would be "precisely where the browser pipelines are
structurally weakest." **It is not.** Remotion's bundle is 0.81 s, its browser opens in
**0.13 s**, and a resident single frame lands in **0.38 s — faster than FFmpeg's 0.93 s.**
Chrome boot is not the problem.

The problem is **per-frame capture**. Browser pipelines pay a serialization cost at the
process boundary that FFmpeg and skia don't pay at all, and it dominates everything:

| DIY Chrome, per frame | Cost |
| --- | --- |
| DOM update + layout + paint | **4.5 ms** |
| `Page.captureScreenshot`, png | **312.8 ms** |
| `Page.captureScreenshot`, jpeg | 43.1 ms |

Chrome *rasterizes* this scene at ~220 fps. Getting the pixels out is 70× the cost of
drawing them. **The browser is not slow; the pipe is.**

### 2. A browser backend buys its speed with lossy intermediate frames

Because capture dominates, both browser pipelines are only viable on JPEG. Same 10 s
window, format the only change:

| | png | jpeg |
| --- | --- | --- |
| Remotion (`--image-format`) | 21.32 s | **8.55 s** |
| DIY Chrome | 32.50 s | 31.69 s → **8.25 s** once processes are isolated |

Remotion defaults to JPEG for video for this reason. **That is an architectural
consequence, not a setting**: a browser-backed master render goes through a lossy
intermediate on every frame. FFmpeg and skia hand pixels to the encoder with no
intermediate at all. #7 should treat this as a quality property of the backend choice, not
a flag.

### 3. Partial render is flat everywhere *except* the incumbent

The baseline's warning — "any candidate whose range support is 'pass a flag' should be
checked for this exact failure" — was worth making, and **every candidate passes it.**
Remotion's `--frames=a-b`, skia's frame loop and the DIY loop are all flat in position,
because each evaluates frame *n* from data rather than accumulating from t=0.

**FFmpeg is the only backend with the pathology**, and only because a synthesised
filtergraph has no decoder to seek. So partial render is an architectural requirement *for
an FFmpeg backend specifically* — it is free in all three others. This weakens what looked
like a general design constraint into a backend-specific workaround.

### 4. Doing it yourself costs three non-obvious discoveries to land where Remotion already is

DIY Chrome at its best (**8.25 s**) is Remotion (**7.4–8.5 s**). Getting there required
finding, in order:

- **PNG is unusable** — 313 ms/frame (finding 1).
- **Tabs don't parallelize; processes do.** Two tabs in one browser were *slower than one*
  (13.5 s → 31.6 s) and 4 and 8 tabs no better — a step change, not gradual contention:
  Chrome throttles background tabs. Separate browser processes fixed it (4 → 8.25 s).
  Beyond 4, contention returns (8 → 11.34 s).
- **Frames must not be rebuilt per frame.** Recreating `<img>` elements each frame races
  the screenshot: with several captures in flight the capture wins and **frames come out
  blank**. The first full DIY render measured 98.6 s and was silently wrong — correct
  frames, once a persistent image pool was used, cost 30.58 s.

That last one is worth dwelling on. It is the same failure class the baseline recorded for
`zoompan`: **a render that completes, looks plausible, and is wrong.** It appeared here in
a completely different technology, which suggests it is a property of frame-accurate video
pipelines rather than of FFmpeg.

## Where this leaves the choice

**On speed alone, skia-canvas is the only candidate that matches the incumbent** — 13.07 s
vs 13.13 s on the full render, and it clears the preview budget *by construction* where
FFmpeg needs the tool to restructure its graph to get there. It also has the cheapest
single frame of any backend, cold (0.50 s) or resident (single-digit ms before encoding).

But the baseline's conclusion still holds and is now better supported: **the budget is
slack, not a constraint.** Three of the four backends clear the full-render budget by 2.5×
or more. The preview budget is the only real gate, and it eliminates the browser pipelines
on this workload — Remotion and DIY Chrome are both ~1.6× over, with no tuning left.

So #7's decision is not "which is fastest." It is a trade between:

- **skia-canvas** — fastest, lossless, flat previews; but the survey scored it **C3 = N**:
  no keyframe or animation model at all. Montaget would own easing, interpolation and the
  timeline. This measurement says nothing about that cost, which is the real one.
- **Remotion / DIY Chrome** — best-in-survey typography and animation (C2/C3 = Y), and now
  known to fail the preview budget, to require a lossy intermediate, and (for DIY) to carry
  three non-obvious engineering traps that Remotion has already paid for.

The performance column of the survey's scorecard can now be filled in. **What it settles is
narrower than expected: it removes the browser pipelines from contention on preview
latency, and it removes speed as a differentiator between FFmpeg and skia.** Everything
still open in #7 — expressiveness, licence, agent-legibility — is untouched by these
numbers.

## Reproducing

```
python3 ../ffmpeg-baseline/build_ass.py    # merged.ass, the shared event source
python3 build_scene.py                     # scene.json

cd skia       && npm i && node render.mjs --from=0 --to=65.216 --out=out.mp4
              &&           node bench.mjs                     # resident per-frame cost
cd diy-chrome && npm i && node render.mjs --from=40 --to=50 --format=jpeg --workers=4 --isolate --out=out.mp4
              &&           node probe.mjs                     # dom vs capture split
cd remotion   && npm i && npx remotion render src/index.js short out.mp4
              &&           node resident.mjs --windows        # bundle/browser/still/window timings
```
