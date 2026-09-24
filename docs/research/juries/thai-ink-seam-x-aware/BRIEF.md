# Brief: the ink seam over-reports. What should be done about it, and what does that cost ADR-0087?

Second court, on one question the first court opened. You are one of three jurors, each a
different model, blind to each other. **State a recommendation and defend it.**

Unlike the first court, **nothing is hidden from you.** Read the implementation, ADR-0087,
the research write-up, the first court's three ballots in
[`../thai-vertical-metrics/`](../thai-vertical-metrics/) — all of it. What you are not being
given is the author's preference among the options below. Do not try to infer it.

## What shipped

`validate` gained `R-LINE-INK-COLLISION` (`review`-class, gates nothing) and `measure` gained
per-line ink extents plus a seam between adjacent lines. ADR-0087 argues the **vertical**
half of ADR-0011's per-line ink box was separable from the horizontal half *precisely because
it needed nothing horizontal* — no `x`, no `origin`, no bidi-resolved `align`.

## The defect, measured

The seam compares **whole-line** ink extents. So line 1's descender at x=10 counts as
colliding with line 2's tone mark at x=200, which never touch. Measured by the author via
per-x-column ink profiles off `place()`, and independently by the first court's Fable juror
via rasterised pixel coverage:

| case (size 55) | shipped seam | x-aware | verdict |
| --- | --- | --- | --- |
| Noto Sans Thai, full stack, `line_height` 1.1 | **+9.58** | **+0.61** | fires, ~15× overstated |
| Noto Sans Thai, full stack, `line_height` 1.2 | +4.08 | **−4.89** | **fires on a clean render** |
| Sarabun, full stack, `line_height` 1.3 | +15.52 | **−1.31** | **fires on a clean render** |
| either face, ordinary Thai prose | identical to x-aware | — | correct |

It over-reports and never misses. The over-report appears on strings whose stacked clusters
are sparse across the line, and vanishes on dense prose.

ADR-0011 contains the relevant warning: *"nominal `size × line_height` overstates real
rendered ink by 1.25×–1.48×, so a text-overflow check run on nominal metrics is a
false-positive generator, three out of five on a legal restyle of the fixture."* The shipped
check moved one level down — from nominal to bounding box — and the question is whether it
moved far enough.

## The options on the table

1. **Make the seam x-aware.** Per-column ink profiles; compare only columns where both lines
   have ink. Needs each line's horizontal offset relative to its neighbour, which is `align`
   — `place.rs::offset` already computes it, RTL included. ADR-0087's "vertical only"
   argument would have to be rewritten.
2. **Hold the check.** Ship `measure`'s ink extents and ADR-0087's two rejections; take the
   check to its own ticket with the x-aware design settled first.
3. **Ship as-is with a named residual.** Keep the bounding-box seam, reword the finding so it
   claims only what it measures, record the x-aware version as unbuilt.
4. **Something none of these is.** Say so.

## Answer these

1. **Which option, and why?** One recommendation, defended. If you pick 1, say what ADR-0087's
   separability argument becomes — is it salvageable, or simply wrong?

2. **Is `align` actually required?** The author observes that when adjacent lines have equal
   advance width their offsets are equal and cancel, and that the over-report was measured at
   `Align::Start` with the offsets cancelling — so it is intrinsic to within-line glyph
   distribution, not to alignment. Does a correct x-aware seam need `align`, or only when
   adjacent lines differ in advance width? **Check this yourself** rather than taking it.

3. **Does over-reporting actually matter here?** ADR-0011's false-positive warning was about
   an *overflow* check. This finding is `review`-class and gates nothing. Argue whether
   ADR-0006's noise budget is violated by a conservative over-report that never misses — and
   whether a check that says "look at a frame" is entitled to be conservative.

4. **Is the bounding box even the right instrument?** The first court's Fable juror used
   rasterised coverage and got floors another tenth lower than column-wise bounds. Is there a
   principled stopping point between "whole-line box" and "count the pixels", or is any
   choice here arbitrary? Note that ADR-0014's stroke falls *outside* the contour and is in
   no version of this measurement.

## Form

One Markdown file. Lead with your recommendation in one paragraph, then the four answers. Tag
claims **MEASURED** / **READ** / **INFERRED**; never present an inference as a measurement.
The fonts are fetched by `docs/research/prototypes/thai-vertical-metrics/run.sh`.

You are being asked to decide, not to survey. "It depends" is not a recommendation.
