# Ballot — Opus

## Recommendation

**Option 1: make the seam x-aware, before `R-LINE-INK-COLLISION` ships.** The defect is not
a tail case that shows up on contrived sparse stacks — **MEASURED**, the whole-line seam
fires on ordinary Thai prose in Sarabun at the format's own **default `line_height` of 1.2**,
claiming a +18.27 px collision in a render with **zero overlapping pixels**, and keeps firing
through 1.3, 1.4 and 1.5. Worse, the instrument barely reads the document: on Noto Sans Thai
the whole-line seam returns **byte-identical numbers for #130's ordinary prose and for the
artificial full-stack string** at every tenth (+15.08 / +9.58 / +4.08 / −1.42 at 1.0–1.3),
while the x-aware seam separates them by 5.5 px. A number that is the same for two different
documents is not a measurement of either; it is a function of (face, `line_height`) — which
is to say it is **candidate 2's face floor**, the thing ADR-0087 rejected on the measurements,
recomputed at runtime and reported as though it were a fact about the file. That is why
"conservative but never misses" does not save it: ADR-0061's *internal* provenance was
granted because "the deciding number is zero and both sides come from the document and the
fonts" — but one side of the comparison is, empirically, document-independent. The fix is
small and the inputs are already in the loop: `ink.rs` already reads `metrics.bounds()` (which
carries `x_min`/`x_max`) and already iterates `positioned_glyphs()` (which carries `x`), and
`place.rs::offset` — RTL included, unit-tested — already computes the per-line shift. What
must be rewritten is ADR-0087's separability paragraph, which is **half wrong**: the *extents*
are genuinely vertical-only and should ship as they are; the *seam* is not, because "do these
two things touch" is not a vertical question. If the court will not pay for option 1 now, the
fallback is option 2 **with the seam withdrawn from `measure` as well as from `validate`** —
shipping `ink_seams` while holding the check would publish the same false number with none of
the finding's hedging prose. Option 3 is not available: there is no honest rewording, because
CONTEXT.md's own ratified vocabulary already says *"a seam … stated as an overlap: positive
means they collide"*, and for this instrument that sentence is false.

**Disclosure.** Nothing was withheld from me and I read the implementation, ADR-0087, ADR-0011,
ADR-0006, ADR-0014, ADR-0057/0061, `query::geometry::ink_box`, and the first court's Fable
ballot in full. I did **not** open `docs/research/juries/thai-ink-seam-x-aware/sonnet.md`; one
line of it surfaced incidentally in a `grep -rn "R-LINE-INK-COLLISION"` over the tree, showing
a fragment of that juror's verdict. It reached my screen before my own measurements were run
and I record it rather than pretend it did not. Every number below is from my own harness, in
my scratchpad, against my own copy of the two pinned faces; I modified no file in the repo
other than this ballot.

**Method (MEASURED, re-derivable).** A standalone crate copied from
`docs/research/prototypes/thai-vertical-metrics/probe` into my scratchpad, same pins
(`parley =0.11.1` + `complex-scripts`, `skrifa =0.46.2`, `tiny-skia 0.11`), fonts read by
absolute path from the committed `fonts/` directory. One parley layout per `\n` line, baseline
= `slot_centre + (ascent − descent)/2` (ADR-0029), parley's per-glyph `y` offsets **kept** (as
`place.rs:162` does, and unlike the author's probe), each line shifted by a faithful copy of
`place.rs::offset`. Four instruments on the same glyph set: whole-line bbox seam (the shipped
one), column-wise seam over glyph contour bounds on a 0.25 px grid, per-glyph-pair seam, and
rasterised coverage (each line filled into its own pixmap, counting pixels where both have any
coverage and where both are ≥50%). I reproduce the brief's numbers exactly (+9.58 / +0.61,
+4.08 / −4.89, Sarabun 1.3 +15.52 / −1.31), so the disagreement below is not arithmetic.

---

## 1. Which option, and why?

**Option 1.** Three findings decide it.

**(a) The over-report is not confined to sparse stacks. The brief's own fourth row is wrong.**
The brief states that on "either face, ordinary Thai prose" the shipped seam is *identical to
x-aware*. **MEASURED, size 55, #130's two prose lines:**

| face / string | `lh` | shipped (whole-line) | column-wise | per-glyph-pair | overlapping px (any / ≥50%) |
| --- | --- | --- | --- | --- | --- |
| Noto, prose | 1.1 | **+9.58 fires** | −5.00 | −5.00 | 0 / 0 |
| Noto, prose | 1.2 *(default)* | **+4.08 fires** | −10.50 | −10.50 | 0 / 0 |
| Sarabun, prose | 1.2 *(default)* | **+18.27 fires** | 0.00 | 0.00 | 0 / 0 |
| Sarabun, prose | 1.3 | **+12.77 fires** | −5.50 | −5.50 | 0 / 0 |
| Sarabun, prose | 1.5 | **+1.77 fires** | −16.50 | −16.50 | 0 / 0 |
| Sarabun, prose | 1.6 | −3.73 | −22.00 | −22.00 | 0 / 0 |
| Noto, fullstack | 1.1 | +9.58 | **+0.61** | +0.61 | 3 / 0 |
| Sarabun, fullstack | 1.2 | +21.02 | **+4.19** | +4.19 | 103 / 56 |
| Sarabun, fullstack | 1.3 | **+15.52 fires** | −1.31 | −1.31 | 0 / 0 |

The first tenth that clears, by instrument (**MEASURED**, sweep 1.0–2.0):

| case | whole-line | column-wise | rasterised (any px) |
| --- | --- | --- | --- |
| Noto, prose | 1.3 | **1.1** | **1.1** |
| Noto, fullstack | 1.3 | **1.2** | **1.2** |
| Sarabun, prose | 1.6 | **1.2** | **1.2** |
| Sarabun, fullstack | 1.6 | **1.3** | **1.3** |

So the false-firing band is **1 to 4 tenths wide on every case I measured**, and on Sarabun it
contains the value the schema hands an author who writes nothing. The check's firing set is,
for practical purposes, *"this element is set in Thai in one of these two faces below the
face's own bbox floor"* — which is the sentence ADR-0087 proved false and refused to put in the
schema.

**(b) The instrument is nearly document-blind.** **MEASURED:** on Noto Sans Thai the whole-line
seam is identical to the hundredth for the prose pair and the full-stack pair at every tenth
(+15.08, +9.58, +4.08, −1.42, −6.92 …), because a whole-line box takes the extreme mark
*anywhere* on the line and the extreme descender *anywhere* on the other, and both strings
contain one of each. The column-wise seam distinguishes them (+0.50 vs +6.11 at 1.0). ADR-0006
requires that a finding **state a fact**; a number that cannot tell two different documents
apart is stating a fact about the face, not about the file in front of it, and ADR-0061's
internal-provenance grant was argued on the opposite premise.

**(c) The fix is cheap and the inputs are already there.** `ink.rs` already calls
`metrics.bounds()`, which returns `x_min`/`x_max` alongside the `y_max`/`y_min` it uses, and
already walks `run.positioned_glyphs()`, which carries `glyph.x`; today both x values are
discarded on the floor of the loop. `place.rs::offset(align, rtl, block_width, advance)` exists,
is RTL-resolved against `layout.is_rtl()` rather than guessed, and has unit tests. `Spec`
already carries `vertical_origin`, a *placement* keyword, so adding `align` beside it is
symmetric with what the engine already accepts rather than a new category of input.

**What ADR-0087's separability argument becomes: half salvageable, half simply wrong.**

- **Salvageable, unchanged:** per-line `ink_top` / `ink_bottom` and the block's own. These need
  the baseline and the outlines and nothing else. ADR-0011's *ink box* — an absolute rect —
  still needs `x` and `origin`'s horizontal component, and still is not built. That part of the
  ADR stands as written.
- **Simply wrong:** the claim that this was separable *"precisely because it needed nothing
  horizontal — no `x`, no `origin`, no bidi-resolved `align`."* For the seam, `x` and `origin`
  are indeed not needed — a seam is a **block-local, relative** question, and a rigid
  translation of the whole block moves both lines equally. But `align` is needed (see §2), and
  the ADR's third disjunct is therefore false. The error is categorical, not incidental: the
  ADR sorted the seam into the vertical half because the *quantity* is a y-distance, when what
  makes it a seam is a **2-D predicate** — whether two sets of ink intersect — and intersection
  has no vertical-only reading.
- **And the bidi blocker was already stale when it was written.** `crates/montagent-core/src/verbs/query/geometry.rs::ink_box`
  ships today, resolves `align` into absolute horizontal placement for every text element, and
  handles the one genuinely unsettled case by a **named narrow refusal** on `dir:"rtl"` —
  "no ADR settles that resolution … so the ink box refuses rather than guess." So the repo
  already contains both the pattern and the precedent. An x-aware seam can do exactly the same
  (fall back to the whole-line seam, or emit nothing, on an RTL run), and it need not even do
  that if it is willing to trust `place.rs`, which is the code that actually puts the pixels on
  the screen: if `validate`'s horizontal resolution disagreed with `place.rs`'s, the bug would
  be in `render`, not in the check.
- One correction the rewrite should carry with it: `query::ink_box` and `measure` now hold two
  different horizontal resolutions — `place.rs` aligns within the **block width** (the widest
  line's advance, per ADR-0007's *"how lines align to each other"*), `geometry.rs` aligns within
  the **declared `width`**. For a *relative* seam only the first matters, but two resolutions in
  one tree is the drift `place.rs`'s own module doc warns about.

**Why not option 2.** Holding is defensible engineering and I would accept it over shipping —
but only with the seam removed from `measure` as well. `measure` publishes `ink_seams` with the
identical whole-line arithmetic, and CONTEXT.md's ratified vocabulary already tells readers that
a positive seam means the lines collide. Shipping that number with the check held is strictly
worse than shipping the check: the finding at least carries "look at a frame", and the raw
number carries nothing. Option 2 is correct only as: ship `ink_top`/`ink_bottom` and the block's
own, hold `ink_seams` and the check, keep ADR-0087's two rejections (which my reading and the
first court's independently confirm), and take the seam to its own ticket.

**Why not option 3.** There is no wording that makes this honest and still useful. "Line 0's
whole-line ink box extends 18.27 px below line 1's whole-line ink box, which may or may not mean
they touch" is a true sentence that no reader can act on and that ADR-0006's noise budget
forbids emitting at volume. And to reword it that way is to concede the measurement is not the
one the code's own doc comment promises — `ink.rs` says it *"answers one question and no other:
where is the ink"*, and where the ink is includes which column it is in.

## 2. Is `align` actually required?

**Yes — and the author's cancellation observation is true but does not generalise.** I checked
it rather than taking it.

**READ, `place.rs::offset`:** `dx = f(align, rtl, block_width, advance)`. Two lines with equal
advance get equal `dx` for every `align` and every base direction, so the difference that a
seam depends on is zero. That is exactly right, and it explains why the author's own
measurement — two copies of the same string, `Align::Start` — could not have seen the effect.
Both lines were the same width, so the experiment could not distinguish the hypothesis from its
negation.

**MEASURED, unequal advances.** Line 1 = `"ป           ปุ"` (a below-vowel at the far right, a
blank middle), line 2 = `"ปี้"` (short, tone-marked); advances 223.9 / 33.3 px on Noto and
219.1 / 36.0 px on Sarabun, size 55. "Truth" is rasterised coverage of the two lines.

| face | `align` | `lh` | whole-line | column **with** `dx` | column **without** `dx` | truth (px any / ≥50%) |
| --- | --- | --- | --- | --- | --- | --- |
| Noto | Start | 1.0 | +14.86 | +1.05 | +1.05 | 9 / 5 |
| Noto | **Center** | 1.0 | +14.86 | **no shared columns** | **+1.05 — false positive** | **0 / 0** |
| Sarabun | **Center** | 1.2 | +20.91 | **no shared columns** | **+3.30 — false positive** | **0 / 0** |
| Noto | **End** | 1.1 | +9.36 | **+9.36** | **−4.45 — MISS** | **12 / 8** |
| Sarabun | **End** | 1.3 | +15.41 | **+15.41** | **−2.20 — MISS** | **38 / 27** |
| Sarabun | **End** | 1.4 | +9.91 | **+9.91** | **−7.70 — MISS** | **27 / 20** |

So a `dx`-blind x-aware seam is wrong in **both** directions once adjacent lines differ in
advance width: it invents collisions under `center` (the short line is shifted away from the
descender it is compared against) and it **misses real, visible ones under `end`** — 27 solid
overlapping pixels reported as −7.70 px of clearance. Missing is the property the whole-line
seam is defended for having; a half-built x-aware seam would give that property up.

The answer to the question as posed is therefore: **the over-report the author measured is
indeed intrinsic to within-line glyph distribution and not to alignment — and that is a
statement about that experiment, not about the check.** A correct x-aware seam needs `align`
whenever adjacent lines differ in advance width, which is the ordinary case for every
multi-line block that is not two copies of one string. The dependency cannot be conditioned
away either, because "do these two lines have equal advance" is itself only knowable after
`measure` has run, at which point `offset` costs one subtraction.

One caveat I could not remove: `dx` also depends on `block_width`, which is the **widest line
in the block**, so an x-aware seam between lines 3 and 4 depends on line 7's advance. That is
not a new coupling — it is how `place.rs` already draws — but it means the seam is a property of
the block, not of the pair, and a rewritten ADR should say so.

## 3. Does over-reporting actually matter here?

**Yes. ADR-0006's noise budget is violated, and "it gates nothing" is not a defence — the noise
budget was written about the classes that gate nothing.**

**READ, ADR-0006:** *"One thing is worse than never running it: running it and getting 47 lines
about frame alignment. `0 errors, 47 notes` reads as a pass. A noisy validator manufactures
false confidence faster than an unrun one does."* The exemplar is **notes**, the weakest class;
the ADR calls report volume *"a safety property"*, explicitly *"not a presentation preference"*,
and ADR-0035 and ADR-0039 have already retracted commissioned checks on this ground. The claim
"`review`-class, gates nothing, therefore a conservative over-report is free" inverts the ADR:
gating is what `error` does, and the noise budget exists precisely because the non-gating
classes are where confidence is manufactured. `review` is also the class that carries
`R-VISUAL-GAP` — an 800 ms hole with nothing on screen. Spending that class on "you are setting
Thai" is the collapse ADR-0006 refused when it declined to merge `review` into `note`.

**Is a check that says "look at a frame" entitled to be conservative?** Only on two conditions,
and this check fails both.

1. **Conservatism is entitled when the better instrument is unavailable or expensive.** Here it
   is neither: the column-wise answer uses data already inside the same loop and matches the
   rasterised answer to the tenth on every case I measured (§4). A check is not entitled to
   spend a reader's attention to save itself thirty lines.
2. **Conservatism is entitled when the reader's check is cheap relative to the miss.** "Look at
   a frame" is the most expensive verification in this product — `render` or `preview`, then
   human inspection of a script the author may not read. Charging that to an author whose render
   is clean, **at the schema's own default `line_height`**, and on *every* `validate` run of that
   project forever, is the 47-notes failure with a different code on it.

There is also a precision argument the ADR itself supplies. ADR-0011's warning — nominal
metrics overstate real ink by 1.25×–1.48×, three false positives out of five — was cited in the
brief as being about an *overflow* check, as if the class mattered. It did not: that finding was
about the **instrument**, and the ratio is the same here. Moving from nominal to bounding box
closed part of the gap; **MEASURED**, the bounding box still overstates the true seam by 5.5 px
(Noto) to 18.3 px (Sarabun prose at the default) — which, in `line_height` tenths, is 1 to 4
tenths of pure false positive. It did not move far enough.

**And one direction in which the check is not conservative at all.** **READ, ADR-0014:** on text
the stroke *"falls outside the glyph contour"*, the ASS `\bord` model, growing painted geometry
by `2 × stroke_width` on both axes. **READ, `ink.rs`:** the extents come from
`GlyphMetrics::bounds`, the contour box, with no stroke term — and `Spec` carries `stroke_width`
already. **INFERRED (arithmetic, not run):** two adjacent stroked lines therefore paint
`2 × stroke_width` more ink into the seam than the check measures, so a `stroke_width` of 6 turns
Noto's −10.39 px column clearance at `lh` 1.3 into a real collision that no instrument in any
version of this measurement sees. So *"it over-reports and never misses"* is false as stated: it
never misses **for unstroked text**. If the seam is rebuilt, it should absorb the stroke —
the term is derivable from the document, which is the test ADR-0029 set for what may be
computed.

## 4. Is the bounding box even the right instrument?

**There is a principled stopping point, it is not arbitrary, and it is the column-wise
intersection of glyph contour bounds — one step tighter than what shipped and one step looser
than counting pixels.** The principle: **take the tightest instrument that is still a strict
upper bound on the painted overlap and still requires no choice that is not derivable from the
document and the fonts.** Rasterising fails the second clause; the whole-line box fails the
first in usefulness rather than in soundness.

**MEASURED.** Column-wise bounds and rasterised coverage agree on the clearing tenth in **all
four** cases (table in §1: 1.1 / 1.2 / 1.2 / 1.3 by both), while the whole-line box is 1–4
tenths away from both. Where they differ at all, bounds are conservative by a fraction of a
pixel rather than by tenths: Noto full-stack at `lh` 1.1 reads +0.61 px column-wise against 3
faintly-covered pixels and 0 solid ones. Per-glyph-pair and per-column gave **identical numbers
on every case I ran**, so that sub-choice is empirically free; per-column is the better
implementation because it is linear in glyphs rather than quadratic.

**Why the pixel count is the wrong stopping point, and why that is a principle and not a
preference.** Fable's first-court ballot is right that pixels are the ground truth of what a
viewer sees, and I reproduce its direction. But a rasterised check would import three things
this format has spent eighty-seven ADRs keeping out:

- **A coverage threshold.** "Any coverage" and "≥50% coverage" are different checks (Noto
  full-stack at 1.1: 3 px vs 0 px), and neither number is derivable from the document. Under
  ADR-0061 that is an **external constant**, which is the second reason ADR-0087 gave for
  rejecting candidate 2. A check cannot reject a script floor for borrowing a constant and then
  borrow an anti-aliasing threshold.
- **A resolution and a rasterizer.** The answer would depend on the pixel grid, the fill rule
  and the AA implementation; ADR-0006's whole architecture is that `validate` reports facts
  about the file, and "how many pixels touched at this scale in this rasterizer" is a fact about
  a render.
- **A dependency direction.** `validate` already pays for shaping every text element — ADR-0087
  names that cost. Making it also fill paths is a different order of expense in the verb that
  runs on every edit.

Contour bounds have none of those: they are exact rational geometry off the font binary, they
are an upper bound on the filled region by construction (the fill lies inside its own bounding
box), and the only free parameter — my 0.25 px column grid — can be removed entirely by sweeping
the glyph-box x-intervals as events rather than sampling, which makes the answer exact.

So the ordering is not a slippery slope with no landing: **nominal box → whole-line ink box →
column-wise ink bounds → pixels**, where the first three are all sound upper bounds and only the
third is tight enough to be informative, and the fourth stops being a fact about the document.
ADR-0011 measured the first-to-second step as worth 1.25×–1.48×; I measure the second-to-third
step as worth 1–4 `line_height` tenths; and the third-to-fourth step as worth **zero tenths on
every case in evidence**. That is the stopping point, and it is where the evidence says to stop.

Finally, on the stroke: **READ**, ADR-0014 puts it outside the contour, and no instrument named
in this brief carries it. That is not a reason to prefer one instrument over another — it biases
all of them identically — but it is the one place where the shipped check's "never misses"
guarantee is untrue, and it should be closed in the same change rather than recorded as a
second residual.

---

## What blocked me

Nothing blocked a measurement. Three limits on what I claim:

- **Two faces, one size, one script.** ADR-0087's own gap list still applies to my numbers;
  everything here is size-invariant by construction (the ratios scale with the em) but I checked
  only 55.
- **The stroke arithmetic in §3 and §4 is READ + INFERRED, not MEASURED.** I did not build a
  stroked render to confirm that `2 × stroke_width` enters the seam; I read ADR-0014's model and
  `ink.rs`'s omission.
- **The `sonnet.md` fragment** described under Disclosure. My numbers and my recommendation are
  my own, but I cannot claim the ballot was written without ever having seen a co-juror's
  verdict phrase.
