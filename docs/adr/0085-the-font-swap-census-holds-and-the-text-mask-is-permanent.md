---
status: accepted
---

# The font-swap census holds, and the reference frames' text mask is permanent

**Ticket:** [#186](https://github.com/MBehtemam/Montaget/issues/186). Evidence:
`crates/montaget-core/tests/font_swap_census.rs`, which re-derives every number below
through the `measure` verb and fails the suite the moment one stops reproducing; the
census table is committed at
[`docs/research/font-swap-census/`](../research/font-swap-census/README.md).

[#143](https://github.com/MBehtemam/Montaget/issues/143) re-vendored the fixture's typeface
from **SF Pro Rounded** to **Open Runde**, because SF Pro Rounded is not redistributable
([ADR-0057](0057-font-vendoring-licence-gate-and-path-keyed-attestation.md)). That was
correct and the fixture was unrenderable without it. #143's own step 5 named the
consequence and did not resolve it:

> Note in the PR that this changes the fixture's visual font, and record it as a font-swap
> the way ADR-0007's mandatory census requires — the fixture's hand-tuned sizes/line-breaks
> were measured against SF Pro Rounded's metrics and are not guaranteed to hold for the new
> font's.

ADR-0057 records that *"no font claims formal metric compatibility"*, so 22 of the
fixture's 60 elements carried layouts nothing had checked.

## The two questions are independent, and #186 conflated them

This is the ADR's substantive finding, and it is a correction to the ticket rather than a
confirmation of it. #186 offered two outcomes, the first of which reads:

> **They still hold** (the deltas are inside every threshold). Record it, cite the script,
> and the falsification test can treat text regions as gating.

**The first clause is true and the second does not follow from it.** Fitting a declared box
is a *layout* claim. The reference-frame comparison makes a *pixel* claim. A face could fit
every declared box perfectly and still draw every glyph somewhere else — which is precisely
what happened here. The two are answered separately below.

#186 had already struck two of its own three original justifications, both traced to one
mistake it named: *"assuming 'text metrics' means 'font metrics.'"* This is a third claim
from the same family, and it is struck on the same grounds.

## 1. The layout census holds, with margin

Measured through `measure` — the verb whose output *is* the text engine's output (spec
#168) — over all 22 elements, under both faces:

- **Nothing overflows its declared `width`.** The tightest element is `quiz-question` at
  **91.8%** of its declared width under Open Runde. A face **8.9% wider than Open Runde**
  would be the first to overflow anything in this fixture.
- **No line partition moved**, and none could: [ADR-0008](0008-line-breaks-belong-to-the-agent.md)
  forbids auto-wrap, so `line_count` is the author's own `\n` count and a wider face
  overflows rather than re-wrapping.
- **No block height moved**, and none could:
  [ADR-0028](0028-text-block-arithmetic-is-exact-tenths.md)'s
  `ceil(size × line_height × line_count)` carries no font term.

The last two are **asserted rather than assumed**. They are the premises the census rests
on, and a premise that is only reasoned about is the kind of thing ADR-0075 exists because
of.

**This discharges ADR-0007's census obligation for #143's swap**, and it satisfies
[ADR-0014](0014-stroke-is-paint-the-text-box-is-required.md)'s condition on the only
font-dependent axis. ADR-0014 parks the overflow finding's width term in the `UNCHECKED`
category *"unless `measure` has been run"*; `measure` has now been run over every element
of this fixture, so for this fixture it is checked. **ADR-0014's rule is unchanged** — this
meets its condition rather than altering it, and a different project's widths remain
`UNCHECKED` until measured in the same way.

## 2. The text mask is permanent, not provisional

Open Runde's advances run **2.0%–9.4% wider** than SF Pro Rounded's across the 22 elements
— the narrow end being the countdown's large tabular digits, the body text sitting at
7.6%–9.4%. Every line in this fixture is centred or centre-left, so a line 8% narrower does
not merely end sooner: every glyph on it lands somewhere else.

`crates/montaget-core/tests/reference_frames.rs` already measures the consequence on every
run and reports it beside the gate rather than inside it:

| frame | text region SSIM | drawn region beside it | text share |
| --- | ---: | ---: | ---: |
| `frame-intro.png` at 400 ms | **0.6743** | 0.9874 (gate 0.975) | 14.2% |
| `frame-05-at-11s.png` at 14 000 ms | **0.6597** | 0.9646 (gate 0.953) | 18.8% |

A region at 0.66 cannot join a gate set above 0.95. No threshold admits it and stays
sensitive to anything else — and tuning one loose to make it fit is the *"completed, looked
plausible, was wrong"* failure [ADR-0010](0010-skia-safe-rasterizer-text-beside-it.md)
records this project having already had twice.

`CONTEXT.md`'s **Reference frame** entry states the governing rule: a divergence is
*"masked only while its cause is unfixed."* The rule is right and is not touched here. What
is decided is its application:

> **The cause of this divergence is a licence, and a licence does not get fixed.** SF Pro
> Rounded cannot be vendored (ADR-0057), so the published video's typeface is permanently
> unavailable to any render this repository can perform. The text mask is therefore
> **settled, not pending** — it is not waiting on a ticket, and no future work discharges
> it short of re-rendering the reference video itself, which would destroy the one artifact
> in this repository capable of falsifying the format.

That last clause is the point. The reference MP4's value comes from having been produced by
a pipeline that knows nothing about Montaget. Re-typesetting it in Open Runde to make the
gate green would be replacing the falsifier with a golden — the exact inversion
`CONTEXT.md`'s Reference frame entry warns against in its own `_Avoid_` list.

## Consequences

- **ADR-0007's font-swap census for #143 is produced**, as a committed, re-executable test
  and a committed table. The obligation is discharged.
- **The fixture's declared sizes are not corrected, because they are not wrong.** All 22
  elements fit their declared boxes under both faces.
- **ADR-0014's `UNCHECKED` width term is measured for this fixture** and passes. Its rule is
  unchanged for every other project.
- **The reference frames' text regions stay masked out of the gate, permanently**, and stay
  measured beside it. `reference_frames.rs`'s module documentation should stop describing
  #186 as *"still open"* and cite this ADR instead.
- The text-region SSIM numbers (0.6743, 0.6597) become the recorded quantification #186
  asked for. They are printed on every run and are free to move; they gate nothing.
- **A margin worth knowing:** the fixture's tightest element has 8.9% of headroom against
  Open Runde. A future face swap is safe by this census only if the new face is within that
  — which is a fact about *this* fixture, not a general tolerance.

## Evidence

- `crates/montaget-core/tests/font_swap_census.rs` — the census, asserted per element.
  Open Runde asserts unconditionally; the SF Pro Rounded delta asserts where the face is
  installed at `/Library/Fonts/SF-Pro-Rounded-Bold.otf` and **skips with a printed note**
  where it is not, which is what CI does. Vendoring the face to avoid the skip would be the
  licence violation ADR-0057 exists to prevent, and estimating the numbers instead is what
  #186 explicitly forbids: *"if it is not obtainable, say so rather than estimating."*
- It also asserts that measuring the fixture's elements in a synthesised project reproduces
  measuring the committed fixture directly — without which the census would be a statement
  about a document nobody ships.
- `crates/montaget-core/tests/reference_frames.rs` — already the source of the two text
  SSIM figures, unchanged by this ADR.
- **A recorded near-miss, because it is the reason the tolerance is what it is.** This
  file's first draft carried several advances transcribed by hand from a two-decimal table
  with their remaining digits invented. A per-pixel epsilon would have accepted every one
  of them. The committed test compares against the engine's own `f64`s at `1e-9`, which is
  round-trip slack rather than measurement tolerance, and that is what caught it.
