# Jury: the ink seam over-reports. What should be done about it?

Second court on [#325](https://github.com/MBehtemam/Montagent/issues/325), convened after
the [first court](../thai-vertical-metrics/README.md) found that the shipped
`R-LINE-INK-COLLISION` fires on documents the renderer draws cleanly. Three jurors (Opus,
Sonnet, Fable) on [`BRIEF.md`](BRIEF.md).

Unlike the first court this one was **not blind** — the question is about code the jurors had
to read. What was withheld was the author's preference among the options.

## Verdict: unanimous for Option 1 — make the seam x-aware

| | Sonnet | Opus | Fable |
| --- | --- | --- | --- |
| Option 1, x-aware, before shipping | ✅ | ✅ | ✅ |
| Is `align` required? | yes | yes | yes |
| Does the over-report violate ADR-0006's noise budget? | in purpose, not letter | yes | yes |
| Right instrument | per-column glyph bounds | column-wise contour bounds | per-glyph bounds |
| Rasterised pixels | rejected | rejected | rejected |

No juror chose to hold the check (option 2) or to ship it with a residual (option 3). The
design below is the court's, and it is what
[ADR-0087](../../../adr/0087-thai-line-height-collision-is-a-font-selection-problem.md) now
records.

## Why option 3 was unavailable

Opus made the argument that closes it: `CONTEXT.md`'s ratified vocabulary already said
*"positive means they collide"*, which is **false** for a whole-line instrument. Option 3
changes the sentence, not the firing set — and the sentence was the part that was already
wrong.

Fable closed option 2 the same way: `measure`'s `ink_seams[].overlap` is the identical
whole-line subtraction, so withdrawing only the `validate` check would have left the wrong
number in the tool an author is told to use.

## The three findings that decided it

### 1. The over-report reaches ordinary prose at the format's default

The brief's table — and the author's own reassurance — said prose was correct. It is not.
On #130's own two-line Thai prose at `line_height` **1.2**, the format's default:

| | whole-line seam claims | x-aware | rasterised |
| --- | --- | --- | --- |
| Noto Sans Thai | +4.07 px | −10.50 px | 0 pixels shared |
| Sarabun | +18.26 px | 0.00 px | 0 pixels shared |

Fable's tally: **10 of 16 rows fire, 2 are real.** ADR-0011 calls 3-of-5 a false-positive
generator by name.

### 2. The instrument was nearly document-blind

Opus's argument, and the sharpest thing either court produced. A whole-line maximum is
reached by *any* cluster with a deep below-mark and *any* cluster with a tall above-mark, so
on Noto Sans Thai it returned **identical numbers** for #130's ordinary prose and for an
artificial string of pure worst-case stacks, at every tenth — while the column-wise seam
separates them by 5.5 px.

A number that cannot distinguish two documents is not a fact about the file. It is the
**face's own floor recomputed at runtime** — which is candidate 2, the thing ADR-0087
rejects, smuggled back in and reported as a document fact. It also undermines the
`ThresholdProvenance::Internal` grant the check was registered under.

### 3. `align` is required, and the author's evidence that it was not was a degenerate case

The author observed that adjacent lines of equal advance width have equal offsets, and
measured the over-report at `Align::Start` with the offsets cancelling. Sonnet found the
flaw by reading `place.rs::offset`'s four branches:

```rust
(Align::Start, false) | (Align::End, true) => 0.0,     // always zero, any advance
```

At `Start` LTR the offset is identically zero — so the experiment could not have detected a
dependency it was blind to by construction. Opus measured the consequence with unequal
advances (223.9 vs 33.3 px): a dx-blind seam invents **+1.05 / +3.30 px** collisions under
`center` where the truth is zero pixels, and under `end` reports **−4.45 px of clearance**
where the truth is 12 overlapping pixels. That last direction is the one that matters: it
gives up the "never misses" property.

Confirmed in the shipped engine — same text, only `align` moving: `start` +0.61, `end`
−13.47.

## Why contour bounds and not pixels

All three rejected rasterised coverage, and Opus gave the principled form: **take the
tightest instrument that is still a strict upper bound on painted overlap and needs no
non-derivable choice.** Pixels fail the second test — they import an anti-aliasing coverage
threshold, a resolution and a rasterizer dependency into `validate`, and that threshold is an
external constant of exactly the kind ADR-0061 fences. It is the same objection that sinks
candidate 2.

Measured, column-wise contour bounds agree with rasterised coverage on the clearing tenth in
every case tested (1.1 / 1.2 / 1.2 / 1.3); the whole-line box was 1–4 tenths off. Fable
quantified the two steps: line-box → glyph-box is worth 9–18 px and flips the sign on half
the rows; glyph-box → pixels is worth ≤1 px and buys a free parameter.

Per-glyph and per-column gave identical numbers, so the implementation is pairwise — a pair
loop needs no bucket width, which would be one more arbitrary constant.

## What the court added that the brief did not ask for

**The stroke.** Raised by both courts: ADR-0014 puts a text stroke *outside* the glyph
contour, so two outlined lines have `2 × stroke_width` less clearance than their contours
show, and no version of the measurement accounted for it. Fable's point that a box "absorbs
it by trivial dilation" is what shipped — each glyph box is grown by its line's
`stroke_width`. Without it, *"over-reports but never misses"* held only for text nobody
outlined.

**The stale blocker.** Opus found that ADR-0087's bidi objection was already out of date:
`query::geometry::ink_box` ships today with `align` resolved into absolute horizontal
placement plus a named narrow refusal on `dir:"rtl"` — the pattern the seam now copies.

## Disclosures

- Opus's `grep` for the finding code surfaced one line of the co-juror file `sonnet.md`
  before its measurements ran; it recorded the leak and did not open the file.
- Sonnet did not build the workspace, so its `offset` branch table is a reading of the match
  arms rather than a measured figure, and its ballot tags it that way.
- The RTL branch is unmeasured by every juror — neither vendorable Thai face exercises it.
- Fable notes the brief's own row "ordinary Thai prose: identical to x-aware" does not
  reproduce; it held for 1 of the 3 prose seams it ran. That row was the author's, and it was
  wrong.

## The ballots

- [`opus.md`](opus.md) — document-blindness, the unequal-advance measurement, the stale blocker
- [`sonnet.md`](sonnet.md) — the `offset` branch analysis that broke the author's `align` claim
- [`fable.md`](fable.md) — the prose-at-default measurement, and the two-step instrument costing
