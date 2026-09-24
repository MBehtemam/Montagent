# FINDINGS -- preview degradation under a heavy composite stack

For [#159](https://github.com/MBehtemam/Montagent/issues/159). Every number below comes
from `./run.sh`; re-run it to reproduce (~10 min on an M1 Pro -- the 8K/native tier alone
is 130s). Methodology, scale-fraction table and backend choice all copy
[#87](https://github.com/MBehtemam/Montagent/issues/87)'s harness exactly, so the two are
directly comparable: same `[10, 20)` 10s window, same `<5s` scrub-preview budget, same
`skia-safe` load-bearing backend (ADR-0009/ADR-0010), same 4K/8K canvases.

## The scene

#87 measured **one video clip, one Ken Burns still, one overlay block** -- explicitly out
of scope of "multiple simultaneous clips or effects." This harness's `scene-4k.json` /
`scene-8k.json` are the direct plural: **2 spans, 2 clips, 20 events**, all live in the
same window -- a full-card Ken Burns still + a PiP-inset Ken Burns still, a full-card video
clip + a PiP-inset video clip, and two overlapping translation-card overlay blocks (the
same block #87 measured once). Correctness verified before timing, per the #6/#34/#87
ground rule: `skia-safe` vs `tiny-skia` mean pixel diff 0.016 (4K) / 0.008 (8K), both well
under the `max=8` threshold used throughout this project's prior perf work.

## Headline: the ladder still clears budget at 720p, with roughly half the margin

| tier | 4K wall | 4K vs budget | 8K wall | 8K vs budget |
| --- | --- | --- | --- | --- |
| native | 21.26 s | 4.3x over | 130.08 s | 26x over |
| 1080p | 5.77 s | **missed** | 6.85 s | **missed** |
| **720p** | **3.40 s** | met, 1.60 s / 32% margin | **4.35 s** | met, **0.65 s / 13% margin** |
| 540p | 2.18 s | met, 2.82 s / 56% margin | 3.35 s | met, 1.65 s / 33% margin |

(`wall` is the harness's own measured wall-clock per its printed line, not the shell
`real` time, which carries ~0.1-0.2s of process-startup overhead on top.)

Compare against #87's single-clip numbers on the identical backend/window/budget:

| tier | #87 (1 clip/1 still) 4K | this (2/2/20) 4K | #87 8K | this 8K |
| --- | --- | --- | --- | --- |
| 720p | 2.68 s | 3.40 s | 3.78 s | 4.35 s |
| 540p | 1.63 s | 2.18 s | 3.07 s | 3.35 s |
| **720p margin** | 2.32 s (46%) | **1.60 s (32%)** | 1.22 s (24%) | **0.65 s (13%)** |

**720p still clears budget at both sizes under this heavy stack** -- the ladder's floor
tier (the one ADR-0065 actually adopted) survives. But the margin has roughly halved at
both sizes for a stack that is itself only ~2x heavier by every count (spans, clips,
events) than #87's. At 8K the margin is down to 13% -- inside the range #87 itself flagged
as too thin to certify without more samples (its `tiny-skia` 8K/720p 1.8% case). This one
is `skia-safe`, the load-bearing backend, at 13%: still on the right side of the line, but
closer to it than a single sample should be trusted to certify.

## The composite-stack trigger fires independently of, and before, the 8K-source trigger

**At 4K alone -- no 8K source involved -- 1080p now misses budget (5.77s) where #87's
single-clip 4K/1080p met it comfortably (4.28s, met).** This is the sharpest finding here:
ADR-0021 named two *independent* triggers (an 8K source, and a heavy composite stack), and
this is direct evidence they are independent -- a stack heavy enough can blow a resolution
tier's budget at a source size that was never in question on its own. The ladder ADR-0065
actually shipped (720p/540p, not 1080p) survives this particular stack, but that's a
property of *which* tier was chosen as the floor, not evidence the composite-stack trigger
is toothless -- a stack heavy enough could reach 720p the same way this one reached 1080p.

## Does the ladder rescue *more* of a heavy-stack miss, or does raster cost erase its headroom?

Neither framing the ticket posed turns out right on its own. **The 720p->540p step buys
more absolute time under the heavy stack than under #87's single-clip scene, at both
sizes** -- 1.22s vs 1.05s at 4K, 1.00s vs 0.71s at 8K. That's the mechanism ADR-0021
predicted operating exactly as predicted: raster cost scales with target resolution, and a
heavier stack has more of it to shed at every step down the ladder, so the same relative
step recovers more wall-clock. **The ladder's absolute rescue capacity does not shrink
under a heavier stack -- it grows.** What shrinks is the *margin at the floor tier*,
because the heavier stack also costs more at every tier including 720p itself, and the
ladder was never asked to buy back more than "clear the line" -- it wasn't designed with
headroom to spare in reserve.

So: raster cost does not dominate enough to erase the ladder's headroom (720p still clears
comfortably in absolute terms, even gaining rescue capacity from the ladder) -- but it does
erode the *safety margin* at the currently-adopted floor, measurably, in one direct
comparison.

## Where the cost goes: resident per-frame ablations (`--notext`/`--noimage`)

`--noimage` in this harness (inherited from #34/#87, not modified here) skips only `Span`/
badge `Image` ops -- it does **not** skip `Video` ops, since neither #34 nor #87 ever
needed to isolate clip cost from a fixed base. That's a real limitation of this run, noted
rather than worked around: the numbers below isolate **stills+badge** cost from a base
that still includes both video clips' decode+blit.

| tier | full | no stills | base (no text, no stills) | stills+badge share of full |
| --- | --- | --- | --- | --- |
| 4K native | 55.30 ms | 29.35 ms | 28.43 ms | ~49% |
| 4K 720p | 8.76 ms (avg) | 5.64 ms (avg) | 5.35 ms | ~39% |
| 4K 540p | 4.65 ms (avg) | 3.16 ms (avg) | 2.89 ms | ~40% |
| 8K native | 305.0 ms (avg) | 208.6 ms (avg) | 206.2 ms | ~32% |
| 8K 720p | 10.5 ms (avg) | 8.68 ms (avg) | 8.65 ms | ~18% |
| 8K 540p | 7.90 ms (avg) | 7.16 ms (avg) | 6.78 ms | ~15% |

Text cost is negligible everywhere (native 4K: 55.30 -> 54.97 ms with `--notext`, i.e.
text is <1% of the frame -- consistent with every prior measurement in this project).
**At native resolution, stills+badge resampling is still a large slice of the frame
(~32-49%)** -- matching #34's original finding that bilinear resampling of a still
dominates. **At preview resolutions (720p/540p), the un-isolated base (bg + rects + both
video clips' decode+blit) is the majority of the frame, 60-85%** -- meaning under this
heavy stack, at the resolutions that actually matter for the `<5s` budget, video-clip cost
(not still-resampling, and not text) is what the ladder is actually buying back. This
wasn't measurable from #87's scene, which had only one clip to begin with.

## Answering the ticket's three questions directly

1. **Does the ladder's rescue capacity hold?** Yes, and it grows in absolute terms (see
   above) -- the heavier raster load raises what a step down the ladder recovers, exactly
   as ADR-0021 predicted for the raster side of the budget. What erodes is the *margin* at
   the floor tier the ladder was actually built around (720p), not the ladder's mechanism.
2. **Does 720p itself still clear budget under a heavy stack, or does the composite-stack
   trigger fire even before an 8K source does?** Both, in a sense: 720p clears budget for
   *this* stack at both 4K and 8K, but the composite-stack trigger demonstrably fires
   independently of source size -- this scene's 4K/1080p missed budget on its own, at a
   source resolution where #87 found no problem at all. A stack heavier than this one
   (more than 2 simultaneous clips/effects) would plausibly push 4K/720p or 8K/720p the
   same way 4K/1080p was pushed here; that has not been measured.
3. **Does this call for a stack-size-aware degradation rule, or confirm the hard-fail
   floor is doing its job?** Neither cleanly. The current ladder (720p/540p, hard-fail
   below) is not broken by this stack -- it holds, with reduced margin. But the margin
   shrinking by roughly half for a stack only ~2x heavier by every count is evidence
   against treating 720p as a resolution-only concern: **stack size is a second axis the
   budget is sensitive to, measured here for the first time**, and extrapolating linearly
   (not validated -- only two points on this axis exist: #87's stack and this one)
   suggests a stack a further 2-3x heavier could plausibly miss 720p's budget at 8K. That
   is evidence for eventually pricing stack size explicitly rather than assuming any stack
   clears whatever a single-clip measurement cleared -- it is not evidence the current
   ladder is wrong for the stacks actually measured so far.

## What's committed vs regenerated

Media (`media/*.png`, `media/*.mp4`) is **not** committed -- `a-8k-video.mp4` alone is
~73 MB, matching #87's reasoning for the same call. `generate-media.sh` regenerates all of
it deterministically (synthetic `testsrc2`, hue-shifted per source so the two simultaneous
sources are visually distinguishable in the committed correctness frames). `scene-4k.json`
and `scene-8k.json` **are** committed (small, and the actual input to every number above);
`gen_scenes.py` is the generator that produced them, kept for provenance though not re-run
by `run.sh`.

## Caveats

- **`--noimage`'s scope is inherited, not extended.** As noted above, it does not isolate
  video-clip cost from the base -- a `--novideo` flag would be needed to fully decompose
  the heavy-stack frame the way #34 decomposed the single-still frame. Not added here; the
  "base is mostly video clips at preview resolutions" reading is an inference from what
  *is* isolated (text, stills), not a direct measurement of clip cost alone.
- **Two points on the stack-size axis, not a curve.** This measures #87's stack (1/1/~9)
  and this one (2/2/20) -- extrapolation to "how many simultaneous clips/effects until
  720p misses budget" is not measured, only motivated as a next question.
- **Single run, one machine, no repeats** -- same limitation #87 recorded, carried forward
  unchanged; the 8K/720p 13% margin should not be read as more reliable than #87's own
  flagged 8K/720p tiny-skia 1.8% case was.
- **Synthetic media (`testsrc2`), same as #87** -- not real footage; encode/decode
  characteristics of the actual fixture's H.264 sources could differ.
- **GPU rasterization remains untouched**, as ADR-0021 and #87 both already note; nothing
  here bears on it.
