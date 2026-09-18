---
status: accepted
amended-by: 0054 (audio-backing and minimum-duration checks)
---

# `validate` gains two caption checks: a reading-pace floor and repeat-duration disagreement

> **Amended by [ADR-0054](0054-caption-audio-backing-and-minimum-duration-checks.md).**
> Settles the audio-backing and minimum-duration deferrals.

[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)'s check list has no
entry for either defect this ADR names — both were found by eye while resolving
[#11](https://github.com/MBehtemam/Montaget/issues/11), on the committed fixture, and
both are computable from `runs`, `start` and `end` with no I/O.

## The evidence

| caption | chars | ms shown | cps |
| --- | --- | --- | --- |
| intro-title | 40 | 3018 | 13.2 |
| hook-05 | 31 | 2298 | 13.5 |
| quiz-question | 46 | 2260 | 20.4 |
| word-quiz | 17 | 2900 | 5.9 |
| **hook-loop** | **31** | **1200** | **25.8** |

`hook-05` and `hook-loop` carry the identical string — *"What is this called\nin
English?"* — and the loop-out gives it 1200 ms against 2298 ms, a 1.92x ratio, at
exactly the seam a looping short is supposed to make invisible.

## Check 1: `R-CAPTION-PACE`

**Metric.** Characters-per-second, computed over an element's **grapheme clusters**
(not UTF-16 code units — combining marks and emoji must not double-count), **spaces
included, `\n` excluded**, divided by the element's on-screen wall-clock duration
(`end - start`, never any source-media span).

**Threshold.** `cps > 20` → `review`, citing Netflix's and the BBC's published
timed-text style guides, which cap general-content reading speed in the same band
(Netflix: 20 cps for adult content, 17 for children's).

**This is a different species of constant than ADR-0014's overflow check**, and the
ADR says so rather than letting the precedent be read as a blank cheque: overflow is
derived from rendering geometry — the text does not fit the box, a fact about pixels.
20 cps is an assumption about human reading capability — a fact about readers, not
about the document. Both are externally documented rather than invented, which is
what makes either defensible at all, but only the first is purely computable from the
render; the second borrows its authority from a citation and should be read that way.

**Known limitation, stated rather than solved:** 20 cps is calibrated for
Latin/Cyrillic-family scripts. Published CJK subtitle guidance runs roughly 8–10 cps
— under half the Latin figure, each character carrying more information — so a fixed
20 cps threshold applied to CJK text does not misfire; it goes **silent** on lines
that are genuinely too fast. The error this ships with is under-flagging, never
over-flagging, which is the acceptable direction for a `review`-level check. Per-script
calibration is deferred to the map's fog.

**On the fixture:** two of five captions fire — `hook-loop` at 25.8 cps, and
`quiz-question` at 20.4, marginal but over the line. Both are legitimate `review`
findings; the finding text carries the computed cps, the character count and the
duration so a reader can judge the margin without re-deriving it.

## Check 2: `R-CAPTION-REPEAT-DURATION`

**Not "shorter on repeat."** The ticket's own framing — a repeated line getting less
time on its second showing — bakes in a claim about what a repeat is *for*: that a
callback deserves at least as much time as the original. That is exactly the kind of
authorial-intent claim [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)
forbids a finding from making, and the opposite convention (a recognized line clipped
short *because* the viewer already read it once) is equally ordinary. The finding is
symmetric, in the same shape as [ADR-0033](./0033-same-source-cut-continuity-is-a-review-check.md)'s
same-source check: state that the durations disagree, name every value, claim nothing
about which one is right.

**Trigger.** Elements whose concatenated `runs` text is byte-identical (after NFC
normalization) are grouped project-wide — no dependency on `track` or `group`, the
same reasoning ADR-0033 applied to `source`: a caption is recognized by its text, not
by where it sits. Within a group, if the longest and shortest on-screen duration
differ by more than **one frame at the project's `fps`**, the group is flagged.

**One finding per distinct-text group, not pairwise.** A group of five occurrences of
one repeated line produces one finding listing all five (id, start, duration), not
ten pairwise comparisons — the same "scope the output, never the analysis" discipline
ADR-0006 already applies elsewhere.

**Tolerance is required, not optional.** "Needs no threshold at all" undersells the
mechanism: two elements independently snapped to frame boundaries can differ by a few
milliseconds with no author decision behind it, and a bare inequality would flag that
as noise. One frame at `fps` is the tolerance — coherent because duration is a single
unit here, unlike ADR-0033's multi-property case.

**On the fixture:** one finding, text *"What is this called / in English?"*, two
occurrences — `hook-05` 2298 ms, `hook-loop` 1200 ms, ratio 1.92.

## Severity

**`review` for both.** Neither is guaranteed wrong: a fast caption can be a
deliberate flash-card beat, a shortened repeat can be a deliberate callback. Neither
is trivial: both are real, visible defects the fixture shipped with, and the
resolving exercise found them expensive to catch by eye. `review` — legal, renders,
you must look at a frame to know if it was meant — is exactly ADR-0006's case for
this shape of finding.

## What this ADR does not cover

**Audio backing is out of scope.** `hook-loop` has no narration under it while
`hook-05` has 1848 ms; that is real evidence but a different fact, and it is not
derivable from the document alone — it requires analyzing the media on disk (and,
for anything stronger than raw presence, distinguishing narration from ambience).
Bundling it into `R-CAPTION-PACE` would make one finding carry two independent
claims. Recorded as a map fog entry: a future audio-presence census tied to a
caption element, `note`-level, its own ticket.

**An absolute minimum display duration is not this ADR.** CPS alone cannot catch a
two-character caption ("Go!") shown for 200ms — it reads as fast but legible by the
rate metric alone. A duration floor (industry guidance runs ~5–6 frames minimum,
independent of content length) is a different check with a different
justification — a floor, not a rate — and is recorded as its own fog entry rather
than folded in here.

## Consequences

- **`validate` gains `R-CAPTION-PACE`** (cps > 20 on grapheme-cluster count,
  `review`) and **`R-CAPTION-REPEAT-DURATION`** (byte-identical text project-wide,
  duration disagreement beyond one frame, `review`), both computed from `runs`,
  `start`, `end` and `fps` with no I/O.
- **No schema change and no amendment to the text-model ADRs** (0007, 0008, 0014) —
  both checks read data the format already carries.
- **Two fog entries added to the map**: per-script pace calibration (CJK and other
  complex scripts), and an audio-presence census / minimum-duration floor as
  separate future checks.
