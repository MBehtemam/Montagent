# Migrating the sample project onto the accepted ADRs

The one real project file this repo has was hand-written to answer
[#9](https://github.com/MBehtemam/Montaget/issues/9) and then sat on an unmerged prototype
branch while ADR-0007 and ADR-0012 landed. This migration brings it up to accepted ADRs
0001–0012 and commits it to `main` as
[`fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json`](../../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json),
beside the media it references — so every `source` path resolves.

`migrate.py` performs the transformation and `verify.py` checks it. Both are committed so
the mapping is checkable rather than asserted, which is the complaint the map's fog entry
*"Where a decision's evidence lives, when the ADR outlives the file it measured"* records
against ADR-0006.

## What changed

| | ADR | change |
| --- | --- | --- |
| 12 rects, 8 images | 0012 | `box:[x,y,w,h]` → `x`, `y`, `origin:"top-left"`, `width`, `height` |
| 8 images | 0012 | `align` → `gravity`; gained the aperture `clip`, which the old `box` was doing silently |
| 7 Ken Burns lists | 0012 | positional `[[t,v],…]` → `{"t","v","ease"}` records, `v` always `[sx,sy]`, no `ease` on the first record |
| 22 text elements | 0007 | `"text"` → `runs`; `font`/`weight` → the `brand` key of a declared `fonts` table |
| 2 text elements | 0012 | `align:"left"` → `align:"start"` + `origin:"center-left"` — it was transcribing ASS `\an4`, *left-middle*, so `y` was a centre and nothing in the file said so |
| 22 text elements | 0012 | gained literal `width`/`height`, the retired `box:"<id>"` |

## What did not change, and was checked

Every `start`, `end`, `source`, `source_start`, `source_end`, `speed`, `group`, `type`,
`layer` and track is byte-identical. 60 elements, 14 tracks, 0 per-track overlaps, one
element per line, stable key order. `speed` (#25), `mask` (#22), `fit`/`gravity` and `fill`
(#13) are still governed by open tickets and carry over untouched — they remain guesses.

**Frame alignment is untouched and still wrong**: 109 of 120 time values are not multiples
of 40 ms at `fps: 25`. That is already on the map as fog and is not this migration's to fix.

## Findings

### D1 — the photos' `width`/`height` is not computable, and the ADR contradicts itself

Exact cover for a 1536×2720 source in a 1080×1300 box is **1080 × 1912.5**. ADR-0012's only
stated rounding rule — *"ties away from zero"* — yields **1913**. The `photo-06` element
**published in ADR-0012 itself** says **1912**. The ADR is aware the number is fractional
and explicitly declines to settle it: *"The renderer must publish a sampling rule, because
the rectangle is not integral: exact cover here is 1912.5 px."*

All **7** photo elements depend on this. The migration wrote 1912, following the ADR's
published element over its published rule. **Needs an owner** — the rounding rule is a
schema-level fact an author needs before writing an element, not a renderer detail.

### D2 — the required text box is unavailable for 15 of 22 text elements

ADR-0012 retires `box:"<id>"` in favour of literal `width`/`height`, on the ground that
*"fifteen of the fixture's twenty-two text elements have no rect element behind them at
all"* and would otherwise name something that does not exist. That is true, and the
migration confirms the count exactly: **7 measurable, 15 not**.

But it leaves those 15 with no source for a number. The only value derivable from the
document is ADR-0007's own block formula — lines × max `size` × `line_height` — and a box
derived that way **makes ADR-0006's overflow check tautological**: the check compares the
block height against a box computed from the block height, so it can never fail on exactly
the 15 elements it was introduced to protect. The 7 measured boxes give a check that can
really fail; the 15 give one that cannot.

The migration wrote `width: 984` (the design's 48 px margin, evidenced by every panel and
card rect in the ASS) and the block-formula height, and marks it here rather than hiding it.
**Belongs to [#13](https://github.com/MBehtemam/Montaget/issues/13).**

### D3 — the fixture's Ken Burns is `linear`, and ADR-0012's worked example assumes `ease-in-out`

Measured against `reference/kenburns/06.mp4` (375 frames, 15.000 s, 25 fps — matching the
15 000 ms keyframe span) by SSIM of the first frame rescaled to each candidate's predicted
scale. The move is a **centre**-pivot zoom (centre beats top at every sample; the joint fit
returns `dy = 0`). The method was validated against a synthesised known-linear 1.0 → 1.08
zoom, encoded the same way, and recovered ground truth to **±0.002**.

| t (s) | u | linear scale | SSIM | ease-in-out scale | SSIM | winner |
| --- | --- | --- | --- | --- | --- | --- |
| 1.50 | 0.100 | 1.0080 | **0.9127** | 1.0016 | 0.8641 | linear |
| 2.25 | 0.150 | 1.0120 | **0.8824** | 1.0036 | 0.8290 | linear |
| 3.00 | 0.200 | 1.0160 | **0.8638** | 1.0065 | 0.8195 | linear |
| 3.75 | 0.250 | 1.0200 | **0.8637** | 1.0103 | 0.8122 | linear |
| 4.50 | 0.300 | 1.0240 | **0.8399** | 1.0150 | 0.8037 | linear |
| 5.25 | 0.350 | 1.0280 | **0.8161** | 1.0204 | 0.7943 | linear |
| 5.63 | 0.375 | 1.0300 | **0.8095** | 1.0234 | 0.7839 | linear |

**7 of 7 for linear.** Only the first half discriminates: the two curves coincide at the
midpoint by construction and converge again at the end.

This is load-bearing for ADR-0012, not a detail of one field. The ADR's canonical `photo-06`
element carries `"ease":"ease-in-out"`, and its argument that **"the raw form is forced, not
chosen"** is computed by splitting `ease-in-out` at `photo-06`'s own insert point —
concluding that neither half is a named ease, that snapping to the nearest name costs
10.3–10.9 px, and therefore that a shifted file grows raw bezier arrays. ADR-0012 also
states that **`linear` and `step` are the only eases closed under subdivision**.

So on the actual fixture, `shift` through `photo-06` preserves the name and **that argument
does not fire at all**. The decision may well still be right — it was argued from the
general case, and a real file will eventually carry a named non-linear ease — but its
worked example is measuring a motion this fixture does not have. The migrated file records
the **measured** value, `linear`, because the ease is a fact about the reference video
rather than something ADR-0012 decided. **Needs a ticket.**

Caveat, stated rather than buried: the pure-zoom model's fit quality falls with `t`
(SSIM 0.91 → 0.72), so something in the move is unmodelled after the midpoint and the
**amplitude** at the tail is not independently confirmed. The `1.08` endpoint is inherited
from the prototype, and the first-half fit is consistent with it.

### D4 — `clip` on `handle-logo` is a no-op

ADR-0012 says the aperture is needed by *"7 of 7 photo elements plus `handle-logo`"*. The
logo is **800 × 800** into a 68 × 68 box; cover gives exactly **68 × 68**, so there is zero
spill and nothing to clip. The migration wrote the `clip` to match the ADR, but it clips
nothing. The ADR's count of elements needing an aperture is **7, not 8**.

### D5 — the `fonts` table cannot resolve, and the file is unrenderable as committed

ADR-0007 requires fonts to be files the project declares by path. **No font file exists
anywhere in this repo**, and the fixture's font — SF Pro Rounded — **is not
redistributable**, so the correct file cannot be vendored here. The migrated file declares
`"brand": [{"file": "fonts/SFProRounded-Bold.ttf"}]` and that path does not exist.

This is deliberate. The alternative — inventing a substitute, or omitting the table — would
either fake a decision nobody made or leave the file stale against an accepted ADR. Written
this way, the map's fog entry *"Font vendoring, licensing and the `fonts` discovery tool"*
stops being a paragraph and becomes a concrete failure any future `validate` will report on
its first run.

### D6 — `origin: "center-left"` is a spelling no ADR publishes

`CONTEXT.md` gives the nine keywords by ellipsis — `top-left` … `center` … `bottom-right` —
and no document spells the six middle names. Two text elements (`chip-text`, `handle-text`)
need the left-middle point to express ASS `\an4`. `center-left` was chosen for consistency
with `center` being the middle keyword; `middle-left` is equally defensible. Cheap to
settle, and a schema needs it settled.
