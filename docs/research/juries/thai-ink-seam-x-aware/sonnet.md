# Ballot — Sonnet

## Recommendation

**Option 1: make the seam x-aware.** The measured table shows the shipped check firing on
two of three colliding-in-the-arithmetic cases that are *clean renders* (`line_height` 1.2
and 1.3, both negative x-aware overlap) — a `review`-class check that is wrong two times out
of three whenever it fires is not a conservative approximation, it is a check an author will
learn to ignore, which is worse than not shipping it. The fix is also cheap relative to its
payoff: `place.rs::offset` already computes the one missing input (an align- and
RTL-resolved per-line horizontal offset), `measure` already has per-line advance widths, and
`crate::text::ink::line_ink` already walks every positioned glyph per line — it currently
discards `glyph.x` after using it for nothing, so adding a column dimension to the same loop
is additive, not a rewrite of the walking code. ADR-0087's separability argument is not
wrong, but it is broader than what remains true: it holds for what `measure` publishes
(`ink_top`/`ink_bottom` per line need no `x`, no `origin`, no `align` — READ, confirmed
below), and it does **not** hold for what `validate`'s seam *compares*, because a seam
compares two lines against each other and that comparison needs their relative horizontal
offset even though it needs nothing about the block's absolute position in the frame. Ship
`measure`'s ink extents as-is (ADR-0087's vertical work stands); rewrite only the seam half
of `checks/ink.rs` to consume per-column ink and the two lines' resolved offsets before
retiring `R-LINE-INK-COLLISION` in its current whole-line form. Option 3 is the fallback if
the x-aware rewrite turns out costlier than it looks from `place.rs` alone (e.g. mixed-run
bidi inside one line), but nothing found while reading suggests that; option 2 throws away a
`review`-class, gate-nothing signal for a defect that is a reporting-fidelity bug, not a
reason to distrust the underlying vertical measurement.

## Q1 — Which option, and what survives of ADR-0087's separability argument

**Option 1.** ADR-0087 wrote (READ, `docs/adr/0087-...md`, "`measure` gains the ink extents
it already computes"): *"an ink box is an absolute rect, so it needs the block's horizontal
placement — `x`, `origin`'s horizontal component, and how `align`'s `start`/`end` resolve
against a line's base direction under bidi... None of that blocks the vertical half."* That
claim is **still true for `measure`** — `crate::text::ink::line_ink` (READ,
`crates/montagent-text/src/ink.rs:88-134`) computes `top`/`bottom` purely from each glyph's
`y` and its font-space bounds; it never reads `glyph.x`, `origin`, or `align`, and needn't to
report a correct per-line vertical extent. The claim is **not true for the seam
comparison** the way it's currently built: `checks/ink.rs::candidate` (READ) compares whole-
line `ink_bottom`/`ink_top` with no x-term at all, which silently assumes every column of
line *i* is stacked over every column of line *i+1* — true only when a line is dense edge to
edge, false whenever stacked marks are sparse (READ, `crates/montagent-text/src/ink.rs:1-19`
names exactly this: *"a upper vowel and a tone mark above the base and a lower vowel below
it"* — position-dependent, not uniform across the line). So the argument survives narrowed:
**absolute horizontal placement (`x`, `origin`) is still never needed for a seam**, because
it's the same constant added to both lines of any pair within one block and cancels in a
same-block comparison. What does not survive is the claim that *no* horizontal information
is needed — `align` (resolved against each line's own base direction) is needed to place
each line's glyphs in a shared local coordinate system before comparing columns, per Q2.
ADR-0087 would need one paragraph rewritten, not retracted: from "the horizontal half is
unbuilt because it needs `x`/`origin`/`align`" to "the *absolute ink box* is unbuilt for that
reason; the *seam*, which only ever compares two lines already sharing one block's `x` and
`origin`, needs `align` alone, and got it wrong by omitting even that."

## Q2 — Is `align` actually required?

**Checked, not taken.** `place.rs::offset` (READ, lines 195-202):

```rust
fn offset(align: Align, rtl: bool, block_width: f64, advance: f64) -> f64 {
    let free = (block_width - advance).max(0.0);
    match (align, rtl) {
        (Align::Start, false) | (Align::End, true) => 0.0,
        (Align::Start, true) | (Align::End, false) => free,
        (Align::Center, _) => free / 2.0,
    }
}
```

I evaluated this exact function by hand on two lines of unequal advance (200 vs. 150, block
width 300) under each `(align, rtl)` branch (INFERRED from the read source, computed
directly, not run against the built pipeline):

| align | rtl | offset(adv=200) | offset(adv=150) | delta |
|---|---|---|---|---|
| Start | false | 0.0 | 0.0 | **0** |
| Start | true | 100.0 | 150.0 | 50 |
| End | false | 100.0 | 150.0 | 50 |
| Center | false | 50.0 | 75.0 | 25 |

So the author's observation is correct as far as it goes but understates the condition. At
`(Align::Start, rtl=false)` — the branch the measured table's cases were run at — offset is
**identically zero regardless of advance**, so it's not that unequal advances happen to
produce equal offsets here; that branch never varies with advance at all, equal or not.
The three other branches (`Start`+RTL, `End`+non-RTL, `Center`) produce a *nonzero* delta
**exactly when** the two lines' advances differ, and a zero delta when they're equal —
matching the author's "cancels when advances are equal" claim precisely, but only in those
branches. **Conclusion: a correct x-aware seam needs `align` (and each line's resolved
`rtl`) unconditionally, to know *which* branch it's in — but the offset it computes from
that branch is a no-op exactly when either (a) the branch is `Start`/non-RTL (or the
mirror-symmetric `End`/RTL) regardless of advance, or (b) the two lines' advances are
equal.** A document that is uniformly `Align::Start`, uniformly LTR, and never mixes base
direction within one text element (the common case, and the one the measured table used)
can skip the offset term and still be correct — but the check has no way to know it's in
that case without first reading `align` and resolving `rtl`, so "only when advances differ"
is a description of when the *number* changes, not a spec for when the *code* can omit
reading `align`. Mixed bidi across adjacent lines of one element (RTL differs by line, per
`place.rs`'s own doc comment: *"the direction is parley's, read off the layout... a `dir`
override on a run reaches this the same way it reaches shaping"*) is a case I did not find a
fixture for and did not measure — flagged as untested rather than asserted safe.

## Q3 — Does the over-report matter, at `review`-class, gating nothing?

ADR-0006's noise budget (READ, `docs/adr/0006-...md`, "The noise budget is a safety
property") is stated as a **volume** control: *"One thing is worse than never running it:
running it and getting 47 lines... report volume is not a presentation preference."* Read
literally, this check doesn't violate it — ADR-0087 already caps it at one finding per
element, and `review`-class findings collapse to counted lines in the clean-output case.
So **the literal noise-budget rule, which is about count, is not violated (READ + INFERRED
from applying the rule as written).**

But I think the brief's question is really asking about the budget's *purpose*, not its
letter, and there the over-report does cost something the letter doesn't capture: ADR-0006
frames `review` as *"legal, renders, and you must look at a frame to know if it was
meant"* — an invitation to spend attention. A finding that is wrong on 2 of the 3 measured
firing cases (MEASURED, brief's own table: `line_height` 1.2 and 1.3 both x-aware-clean but
shipped-seam-positive) spends that attention on nothing, repeatedly, for the exact class of
document (sparse stacked marks) the check exists to protect. A single-count `review`
finding is cheap to *emit* under the noise budget's counting rule but not cheap to *trust*
after the first few false alarms — and an advisory that an author has learned to discount is
functionally the unrun check ADR-0006's own epigraph warns is safer to be uneasy about than
to falsely pass. (INFERRED: this is my extrapolation of the noise-budget *principle* to a
magnitude-accuracy failure the ADR text itself only states in terms of count; I'm not
claiming ADR-0006 says this in words.) A check gating nothing is entitled to be
*imprecise at the margin* — a seam at −0.3px isn't worth resolving to the pixel — but "gates
nothing" is not the same license as "may be wrong by an order of magnitude in the direction
that erodes trust in itself," and the measured 1.25×–15× overstatement (brief's table) is
the second, not the first.

## Q4 — Is the bounding box the right instrument?

**No principled stopping point exists between "whole-line box" and "count the pixels";** the
choice is a cost/precision tradeoff, not a correctness threshold, and I think that's fine
for what this check is. Column-wise glyph-bound ink profiles (option 1's proposal) already
fix the defect this brief measures — false collisions between non-overlapping stacked marks
— because the defect is about columns having ink at all in each line, not about the fidelity
of each column's edge. Rasterized coverage (the first court's Fable juror, READ,
`docs/research/juries/thai-vertical-metrics/fable.md`) finds "floors another tenth lower"
because glyph *bounding boxes* are convex rectangles around a possibly non-convex, antialiased
contour — a curved stroke's box always contains empty corners the actual ink doesn't fill —
so rasterization will always report less overlap than bounds-based measurement for curved
glyphs. That's a real, measured gap, but it's a gap of the same *kind* the shipped whole-line
box already has relative to a per-column box: each refinement (whole-line → per-column →
per-pixel) trades a further order of magnitude of engine cost (shaping-only → shaping +
metrics table → full rasterization) for a further, shrinking correction to the reported
number, with no point at which the next refinement stops being worth its cost in principle —
only in practice, against a specific defect's size. Per-column bounding boxes are the right
stopping point **for this defect** because they close the gap the brief measures (whole-line
mixing unrelated columns) without paying rasterization's cost, and the residual gap
rasterization would still close (box-vs-contour tightness on curved glyphs) hasn't been shown
to produce false review findings the way whole-line pooling did — no case in the measured
table or the first court's ballots shows a per-column-clean case that rasterization would
still flag, or vice versa. ADR-0014's stroke sitting outside the contour (READ,
`docs/adr/0014-...md`: *"stroke falls outside the glyph contour... the stroke grows into the
box"*) is a **separate, unmeasured gap in every version of this instrument** — a stroked
Thai line's true painted extent is wider than any of bounds, columns, or unstroked
rasterization would report, and nothing in this brief's evidence bounds that gap's size. I
did not find or run a measurement of it; flagged as a genuine open residual, not folded into
either of my other answers.

## Blocked on

Nothing blocked the reading or the reasoning. I did not build or run the Rust workspace
(the two-Rust-crate offset/ink code was read, not executed under the actual font/shaping
pipeline), so the Q2 table above is a hand-evaluation of `offset()`'s literal match arms —
faithful to the source as read — rather than a pipeline-measured number like the brief's own
table; I've tagged it accordingly rather than presenting it as MEASURED.
