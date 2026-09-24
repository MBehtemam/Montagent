# Ballot — Fable 5.1 (second court)

## Recommendation

**Option 1: make the seam x-aware, now, in this ticket — per-glyph bounding boxes, compared
only where two glyphs share an x-column, offset by `place.rs::offset` — and rewrite ADR-0087's
separability sentence from "vertical needs nothing horizontal" to "the seam needs nothing about
where the block *sits*."** The shipped instrument is not conservative in the harmless sense; it
is wrong on the common case. **MEASURED:** on #130's own two-line Thai prose, in the canonical
vendorable face, at the format's default `line_height` 1.2, the shipped seam reports **+4.07 px**
of collision where the x-aware seam reports **−10.50 px** of clearance and the rasterised lines
share **zero pixels**. Sarabun prose at 1.2: shipped **+18.26**, x-aware **0.00**, zero pixels. The
brief's row "ordinary Thai prose: identical to x-aware" does not reproduce on the ticket's own
prose. The shipped floor is one to four tenths above the real one on every string I ran, and it
sits above the default on both faces — so `R-LINE-INK-COLLISION` will fire on essentially every
Thai element in both faces at every `line_height` an author would plausibly write, whether or
not anything touches. That is not a residual to name; it is ADR-0006's *"0 errors, 47 notes
reads as a pass"* on a `review`, which is worse. Holding the check (option 2) does not isolate
the defect, because `measure`'s `ink_seams[].overlap` is the same whole-line number and an
author reading it gets the same over-report; rewording (option 3) changes what the finding says
and nothing about when it fires. The x-aware design is not open: `align` is required exactly
when adjacent lines differ in advance (**MEASURED**, Part C below: the same document flips
between −16 px clear and +9.35 px colliding as `align` moves from `start` to `end`), the offset
it needs is the one `place.rs::offset` already computes and the renderer already draws with, and
the seam needs neither `x`, `origin`, nor the container `width` — only the *difference* of two
lines' offsets, which is a function of `align`, each line's direction and each line's advance.
The right level of the instrument is the glyph bounding box, not the line box and not the
pixel: the line-to-glyph step is worth 9–20 px on every case; the glyph-to-pixel step is worth
at most one pixel, is the only step that introduces a free parameter (the coverage threshold),
and is the only one that cannot absorb ADR-0014's stroke.

## Disclosures

- I read everything the brief allowed: the implementation (`ink.rs`, `checks/ink.rs`,
  `tests/ink.rs`, `engine.rs`, `place.rs`, `verbs/measure.rs`, `frame.rs`, `query/geometry.rs`,
  `registry.rs`), ADR-0087, ADRs 0006/0011/0014/0024, the research write-up, the author's probe
  and `results.txt`, and all three first-court ballots.
- **My scratchpad directory already contained the first court's Fable harness** (`probe/src/bin/
  pairs.rs`, `pixels.rs`, a `fable-thai-probe` binary — same session id). I did not open or run
  them. My probe is `probe/src/main.rs`, written from scratch in this session; its full output is
  saved beside it as `results-fable-court2.txt`. The fonts I measured are copies whose sha256
  match `run.sh`'s pins (`61cf814e…`, `226d4f36…`).
- I modified nothing under the repository. `git status` shows three modified test files
  (`tests/disk.rs`, `finding_fixtures.rs`, `frame.rs`) that were modified before I started; I
  did not touch them.

## Method (MEASURED, all numbers below)

Independent Rust crate on the workspace pins (`parley =0.11.1` with `complex-scripts`,
`skrifa =0.46.2`, `tiny-skia 0.11`). Each `\n` line is its own parley layout, as `engine.rs`
does; ascent/descent are the max over runs (ADR-0029); baseline = `slot_centre + (ascent −
descent)/2`; per-line `dx` is `place.rs::offset` copied verbatim, with `layout.is_rtl()`. Four
seams per adjacent pair, all in block coordinates, y-down, positive = collision:

| name | what it is |
| --- | --- |
| **whole** | `max(bottom of A's glyph bounds) − min(top of B's glyph bounds)` — **exactly what `ink.rs` ships** (`GlyphMetrics::bounds`, same skip of empty boxes) |
| **gbox** | x-aware over glyph bounds: max over glyph pairs (a∈A, b∈B) whose x-ranges intersect of `a.bottom − b.top`; `none` if no pair shares a column |
| **pxcol** | x-aware over rasterised coverage, 1 px columns: A's lowest inked row minus B's highest inked row, max over columns both ink |
| **px_any / px_solid** | pixels where both lines have coverage > 0 / ≥ 50 % |

Size 55 throughout. Strings: the author's full-stack `ปู่ปี้ปุ๋ยปิ๊งปู๊ปื้น` ×2, #130's two-line and
three-line prose verbatim, and two constructed sparse pairs for the `align` question.

## Q1 — Which option, and why

**Option 1.** The evidence that decides it is not the brief's table (which I reproduce to the
hundredth) but the prose rows the brief says are "correct":

| case, `Align::Start` | lh | shipped (whole) | x-aware (gbox) | pxcol | px any / solid |
| --- | --- | --- | --- | --- | --- |
| Noto, full stack | 1.1 | **+9.57** | +0.61 | +1 | 3 / 0 |
| Noto, full stack | 1.2 | **+4.07** | −4.89 | −4 | 0 / 0 |
| Noto, full stack | 1.3 | −1.43 | −10.39 | −10 | 0 / 0 |
| Sarabun, full stack | 1.2 | +21.01 | **+4.18** | +5 | **103 / 56** |
| Sarabun, full stack | 1.3 | **+15.51** | −1.32 | −1 | 0 / 0 |
| Sarabun, full stack | 1.5 | **+4.51** | −12.32 | −12 | 0 / 0 |
| Sarabun, full stack | 1.6 | −0.99 | −17.82 | −17 | 0 / 0 |
| **Noto, #130 prose, 2 lines** | 1.0 | +15.07 | +0.50 | +2 | 10 / 0 |
| **Noto, #130 prose, 2 lines** | **1.1** | **+9.57** | **−5.00** | −5 | **0 / 0** |
| **Noto, #130 prose, 2 lines** | **1.2 (default)** | **+4.07** | **−10.50** | −10 | **0 / 0** |
| Noto, #130 prose, 3 lines, seam 0/1 | 1.1 | +9.57 | −2.64 | −5 | 0 / 0 |
| Noto, #130 prose, 3 lines, seam 1/2 | 1.1 | −4.45 | −4.45 | −4 | 0 / 0 |
| **Sarabun, #130 prose** | **1.2 (default)** | **+18.26** | **0.00** | 0 | **0 / 0** |
| **Sarabun, #130 prose** | 1.3 | **+12.76** | −5.50 | −5 | 0 / 0 |
| Sarabun, #130 prose | 1.5 | **+1.76** | −16.50 | −15 | 0 / 0 |
| Sarabun, #130 prose | 1.6 | −3.74 | −22.00 | −22 | 0 / 0 |

First tenth that clears, shipped vs x-aware: Noto stack **1.3 vs 1.2**; Noto prose **1.3 vs
1.1**; Sarabun stack **1.6 vs 1.3**; Sarabun prose **1.6 vs 1.2** (1.2 is a touch at exactly
0.00, not an overlap — fragile, but the pixels agree it is clean). The shipped floor is above
the real floor by one to four tenths in every case, and above the format default in every case.
The only true collision in the table — Sarabun full stack at 1.2, 56 solid pixels — the x-aware
seam also catches (+4.18). "Never misses" holds for both instruments; only one of them is also
right when it fires.

**INFERRED from the above.** The brief's row "either face, ordinary Thai prose: identical to
x-aware" holds for one of the three prose seams I ran (3-line, seam 1/2) and fails for the other
two by 12–18 px. The over-report is not a property of "sparse stacked strings"; it is a property
of any line pair whose lowest mark and highest mark are not in the same column — which is
ordinary Thai. The one seam where they coincide is the exception.

**Why not option 2 (hold the check).** The brief's option 2 ships "`measure`'s ink extents".
The per-line `ink_top`/`ink_bottom` are correct per-line facts and should ship — they are what
the first court's single-line block-escape finding needs. But `ink_seams[].overlap` is the same
whole-line subtraction the check makes (`ink.rs::seams`, `above_bottom − below_top`), so an author
who reads `measure` to pick a `line_height` gets the same wrong floor with no `validate` in the
loop. Holding the check while shipping that number does not quarantine the defect; it moves it
from the verb that judges to the verb that advises. And there is nothing left to settle before
building: the x-aware form is fixed by Q2 and Q4 below, the offset rule is code that already
ships with tests (`place.rs:195–200, 252–267`), and the default for an absent `align` is
already recorded by the renderer (`frame.rs:1433`: *"An absent `align` is `start`. No ADR states
a default."*).

**Why not option 3 (ship with a named residual).** Rewording changes the sentence, not the set
of elements it fires on. ADR-0006's own definition of `review` is *"you must look at a frame to
know if it was meant"* — the human judges intent, the check establishes existence. A finding
that fires at +4.07 on a render with zero overlapping pixels asks the human to look at a frame
to know whether the thing *happened*, which is the check's job, not the reader's.

**What ADR-0087's separability argument becomes.** It is half right and should be rewritten,
not struck. READ: ADR-0087 says the vertical half *"was never blocked on"* `x`, `origin`'s
horizontal component, or bidi-resolved `align`. **MEASURED (Q2):** it is not blocked on `x`,
`origin`, or the container `width` — none of those enters the seam, because the seam depends
only on the *difference* between two lines' offsets, and every container term cancels in that
difference. It **is** blocked on `align` × per-line direction × per-line advance, which is a
block-local quantity, not a placement one. The honest sentence is: *the seam is separable from
the block's placement, not from its alignment; and alignment is already resolved, in code, at
`place.rs::offset`, the way the renderer draws it.* The absolute horizontal ink box — the thing
ADR-0011 named — stays unbuilt for the reason ADR-0087 gives; the seam was never that thing.
**READ, and worth a line in the rewrite:** `query --at` already computes an `ink_box` that
resolves `align` horizontally (`query/geometry.rs:436–459`) and refuses only when a run declares
`dir:"rtl"` — so "how `align` resolves under bidi is unsettled" is, in this codebase, a scoped
refusal on one input, not a blocker on the axis.

**The shape of the change (INFERRED, from the code read).** `ink.rs::line_ink` already visits
every glyph's `bounds` and its `glyph.x`; it discards the x and keeps two numbers. Keep the
boxes instead (x-range from `glyph.x + x_min..x_max`, y-range as now), add the line's `dx` from
`offset(align, layout.is_rtl(), advance_width_max, advance)`, and define the seam as the max
over pairs with intersecting x-ranges, `null` when no pair shares a column. `Spec` (or
`measured()`) gains `align`; `Asked` carries it; `try_measure_element` reads it with
`frame.rs::align_of`'s default. Dilate each box by its run's `stroke_width` (ADR-0014) — free
with boxes, impossible with the shipped extents. Cost is O(|A|·|B|) glyph pairs per seam,
sortable to O(|A|+|B|); a caption line is tens of glyphs.

## Q2 — Is `align` actually required?

**Yes, whenever adjacent lines differ in advance; provably not when they are equal; and never
`x`, `origin` or `width`.** I checked rather than took it.

**READ**, `place.rs:195–200`: `dx = free × k` where `free = max(0, block_width − advance)` and
`k ∈ {0, ½, 1}` chosen by `(align, rtl)`. **INFERRED, arithmetic:** the seam between lines A and
B depends on `dx_B − dx_A`. If `advance_A = advance_B` and both lines have the same direction,
`free_A = free_B` and `k_A = k_B`, so the difference is zero for every `align` — the author's
cancellation, confirmed. If the advances differ, the difference is `k·(advance_A − advance_B)`
(or with different `k` per line under mixed direction), which is nonzero for `center` and `end`
and zero for `start` (LTR). Note that `block_width` cancels too — so whether the container is
the block's own advance (`place.rs`) or the declared `width` (`query/geometry.rs:448–452`, a
different convention), the seam is the same. That is why `x`, `origin` and `width` are not
inputs.

**MEASURED**, a short line with a below-vowel-plus-tone cluster (`ปุ๋`, advance 33 px) over a
long line (708 px) whose only tall stack is at one end:

| Noto, lh 1.1, size 55 | whole | gbox `start` | gbox `center` | gbox `end` |
| --- | --- | --- | --- | --- |
| long line's stack at its **end** | +9.35 | **−16.00** | −16.00 | **+9.35** (36 px / 28 solid) |
| long line's stack at its **start** | +9.35 | **+9.35** (35 / 27) | −16.00 | **−16.00** |
| Sarabun, lh 1.3, stack at end | +14.52 | −21.12 | −21.12 | **+14.52** (66 / 48) |

Same document, same fonts, same `line_height`; the only field that changed is `align`, and the
answer moves by 25–35 px between a clean render and a real collision. The whole-line seam cannot
tell any of the six apart. Equal advances, as a control: the full-stack pair (both 295.6 px)
gives gbox **+0.61** under all three alignments — the cancellation exactly as claimed.

**MEASURED**, the realistic case: #130's prose under `center`/`end` (advances 747/853 px in
Noto, 764/880 in Sarabun) clears by −13.75…−17.49 px, further than under `start` (−5.00 /
−5.50), because the shift moves the two lines' marks out of each other's columns. So on prose
the dependence is real but benign; on sparse lines it is the whole verdict.

**Direction (READ, not measurable here).** Both faces lack an RTL script, and parley reports
`rtl=false` for every line I laid out, so the mixed-direction branch of `offset` — where `k`
differs between lines even at `start` — is confirmed by reading the function and its tests
(`place.rs:252–260`), not by measurement. The x-aware seam should take `layout.is_rtl()` from the
same layout the renderer draws with, as `place` does; refusing on an `rtl` run the way `query`'s
`ink_box` does would report nothing about a line pair the renderer will paint. Recorded as the
same residual `query` already carries, not as a new one.

## Q3 — Does over-reporting matter here?

**Yes. ADR-0006's noise budget is violated, and a "look at a frame" check is entitled to be
conservative only when the conservative margin is thin relative to the true region. Here it is
one to four tenths thick and covers the default.**

**READ**, ADR-0006 §"The noise budget is a safety property": *"running it and getting 47 lines
about frame alignment. `0 errors, 47 notes` reads as a pass. A noisy validator manufactures
false confidence faster than an unrun one does."* That passage is about a **non-gating** class.
The argument "it gates nothing, so it costs nothing" is the argument the budget was written
against. READ, same ADR: stable codes exist so a reader can *"suppress or skim a whole class
without re-reading prose"* — the designed response to a class that always fires is to skim the
class, which discards the one true positive in my table (Sarabun full stack at 1.2, 56 solid
pixels) along with the false ones.

**MEASURED, what the shipped check does on a realistic project.** Every multi-line Thai element
in Noto Sans Thai below 1.3, and in Sarabun below 1.6, gets one `review` per element, on every
`validate` run, permanently. At the format default (1.2), on #130's prose, in both faces, the
render has zero overlapping pixels and the finding fires anyway (+4.07, +18.26). ADR-0087's own
Consequences say *"A face whose marks fit its own default `line_height` produces nothing"*; Noto's
marks fit at 1.2 (x-aware −10.50, no pixel shared) and it produces a finding. The committed
fixture test (`tests/ink.rs::the_committed_fixture_has_no_ink_collision`) passes only because the
fixture is Latin; the same test on a Thai fixture at the default would fail.

**Against ADR-0011's precedent, READ:** the nominal check produced *"three false positives out
of five"* on a legal restyle and was called *"a false-positive generator"* and *"alarm fatigue"*.
On my table the shipped seam's positive rate on clean renders is higher than that: of the
sixteen `Align::Start` rows above, ten fire; two of those ten are real (Noto stack 1.1 at 3
faint pixels, Sarabun stack 1.2). The brief says the check *"moved one level down"*; on the
false-positive axis it moved from 60 % to 80 %.

**Is a `review` entitled to be conservative? INFERRED.** Conservative means: the set it fires on
contains the true set. That is necessary. It is *sufficient* only if the difference between the
two sets is small enough that a reader's look is usually rewarded. ADR-0061's provenance rule
says a `review` may compare a document-derived fact to a threshold; it does not license a fact
that is a loose upper bound on the fact the finding names. The finding's field is `overlap`; the
number in it is not an overlap, it is a difference of two extremes that may be 700 px apart
horizontally. A conservative check that is right on the *sign* most of the time is defensible;
one that is wrong on the sign at the format default in both vendorable faces is not.

## Q4 — Is the bounding box the right instrument?

**The glyph bounding box is; the line bounding box is not; the pixel is not needed. The stopping
point is principled: it is the finest level that is still a pure function of the document and
the font tables, with no free parameter.**

**MEASURED**, the size of each step, from the table in Q1:

| step | worth |
| --- | --- |
| line box → glyph box (x-aware) | 9.0 to 18.3 px on every case that differs; sign flips on 8 of 16 rows |
| glyph box → pixel column | ≤ 1 px on every row (the difference is the 1 px column binning) |
| glyph box → pixel intersection | Noto stack 1.1: gbox +0.61 says collide; 3 pixels at coverage > 0, **0** at ≥ 50 % |

**INFERRED, why the glyph box is the stopping point:**

1. **It is the object the font defines.** A `glyf` face stores each glyph's bounds; skrifa hands
   them over without decoding a contour (`ink.rs` module doc says so, correctly). The number is
   deterministic, hinting-free, antialiasing-free, and identical at every size in em terms.
   ADR-0061's provenance test — document plus media on disk — is met with nothing added.
2. **It stays conservative.** A box contains its contour, so a seam over boxes never
   under-reports a contour seam. "Never misses" survives the move; it does not survive a pixel
   threshold (the 3-vs-0 row).
3. **The residual is bounded by mark geometry and is under a pixel here.** What a glyph box
   over-reports is the slack between a mark's box and its contour where it meets another mark's
   box; tone marks and below-vowels are small and nearly box-shaped. pxcol and gbox agree within
   binning on all 16 rows.
4. **Pixel counting has a free parameter and the box does not.** "any coverage" vs "≥ 50 %"
   gave different verdicts on the same render (Noto stack 1.1). The first court's Fable floor
   "another tenth lower" is that threshold choice, not a fact about the ink. A finding whose
   sign depends on an AA threshold has borrowed a constant from the rasteriser — the external
   provenance ADR-0061 fences.
5. **ADR-0014's stroke.** READ: text stroke falls *outside* the contour; `measure` already returns
   the stroked extent so authors do not add `2 × stroke_width` by hand. A glyph box absorbs the
   stroke exactly (dilate by `stroke_width` on each side). Rasterised coverage of the *fill*
   under-reports a stroked seam by up to `2 × stroke_width` — so pixel counting of unstroked
   glyphs is not the "true" answer either; it is a different approximation, and the one that
   errs in the direction that misses. Neither the shipped seam nor any pixel method in either
   court includes it; the box method can, in one line.

So the choice is not arbitrary. There are exactly three levels that are functions of the
document and the tables — line box, glyph box, contour — and the contour buys under a pixel
over the glyph box at the cost of walking every outline. Everything past the contour is a
rasteriser's opinion. The glyph box is where the instrument should stop, and it is one field
away from what `ink.rs` already reads.

## What blocked me

Nothing blocked a measurement I needed. Two things are stated as unmeasured rather than
estimated: the mixed-direction branch of `offset` (no RTL script in either face; confirmed by
reading `place.rs` and its tests), and the stroked seam (the fixture sets no stroke, and I
declined to invent one; the arithmetic is ADR-0014's, not mine). The shared scratchpad is
disclosed above; it changed no number here.
