---
status: accepted
amends: 0006 (adds `R-LINE-INK-COLLISION` to the check list), 0011 (builds the vertical half of the per-line ink box it names and leaves the horizontal half where it was), 0024 (`measure` gains the ink extents beside the slot numbers, still with no verdict)
---

# Thai's `line_height` collision is a font-selection problem; the repair is a `validate` finding and ink in `measure`, not a metric change or a schema floor

[ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) makes a line's height *"the
largest `size` among the runs on that line × `line_height`"* — a function of two numbers
the document declares, and **never of the font's ink**. [#130](https://github.com/MBehtemam/Montagent/issues/130)
measured what that costs on a script whose marks stack and stopped there, by design: it was
scoped to find out whether the assumption breaks, not to fix it. It breaks.
[#325](https://github.com/MBehtemam/Montagent/issues/325) named three candidate repairs and
evaluated none. This ADR measures all three and picks one.

The scope is the **vertical** axis only. Literal sizes are not reopened — ADR-0007's
decision is what makes the file readable without evaluation — and auto-wrap is not reopened
either, because [ADR-0008](./0008-line-breaks-belong-to-the-agent.md) settled that line
breaks belong to the agent, and the Thai break-opportunity finding lives with it.

## The evidence

`docs/research/thai-vertical-metrics.md`, with its probe committed at
`docs/research/prototypes/thai-vertical-metrics/`. `./run.sh` fetches both faces by SHA-256,
builds, runs, and **re-asserts all thirteen headline numbers, exiting non-zero the moment
one stops reproducing** — the re-executable check `docs/agents/domain.md` requires of a
numeric claim. Every figure below is measured, not computed, unless it says otherwise.

The faces are the ones [ADR-0057](./0057-font-vendoring-licence-gate-and-path-keyed-attestation.md)'s
gate actually admits, and the gate was run rather than read: `montagent fonts vendor`
returns `licence OFL-1.1 (recognised)`, bucket 2, for both.

| | Noto Sans Thai 2.002 | Sarabun 1.000 |
| --- | --- | --- |
| `hhea` ascender / descender / lineGap | 1061 / −450 / 0 | 1068 / −232 / 0 |
| `OS/2` sTypoAscender / sTypoDescender / sTypoLineGap | **1061 / −450 / 0** | **1068 / −232 / 0** |
| `OS/2` usWinAscent / usWinDescent | 1061 / 450 | 1286 / 567 |
| `fsSelection` bit 7, `USE_TYPO_METRICS` | set | set |
| `sTypo` baseline-to-baseline | 1.511 em | 1.300 em |
| **first `line_height` tenth with no ink collision** | **1.3** | **1.6** |
| ink-to-ink overlap at `line_height` 1.1, size 55 | **+9.58 px** | **+23.77 px** |
| the same, Latin in the same face | −8.69 px (clear) | −6.38 px (clear) |

`1.1` — the fixture's own value, and a correct one for Latin — is **below the collision
floor on both vendorable faces**. The collision is visible in rendered frames
(`frames/noto-thai-fullstack-lh11.png` against `-lh13.png`), not only in the arithmetic.

Two cross-checks against the shipped binary rather than against the argument: `montagent
measure` reported this face's ascent/descent to three decimals of the probe's own figures,
and `montagent validate` on a colliding project reported **0 errors and 0 reviews** about
it. That last number is the defect this ADR exists to fix.

## The three candidates

### 1. Read `OS/2` typo metrics instead of `hhea` — rejected, because it is already done and moves nothing

This candidate assumed the stack reads `hhea`. It does not.
`skrifa-0.46.2`'s `Metrics::new` (`src/metrics.rs:139–191`) checks `fsSelection` bit 7 and
prefers `sTypoAscender`/`sTypoDescender`/`sTypoLineGap` when it is set; `parley-0.11.1`
(`layout/data.rs:411–415`) calls that function and adds no logic of its own. Both faces set
bit 7, so the typo metrics are already what Montagent lays out with. **And on both faces
the two tables are field-for-field identical**, so the question is moot twice over: the
change would move zero pixels.

Read generously — derive the slot from the `sTypo` baseline-to-baseline sum rather than from
`size × line_height` at all — it clears Noto Sans Thai (1.511 em > the 1.3 floor) and
**still collides by +12.77 px on Sarabun**, whose sum is 1.300 em against a 1.6 floor. The
only metric set that clears both is `usWin*`, and the OpenType spec
([`OS/2`](https://learn.microsoft.com/en-us/typography/opentype/spec/os2), v1.9.1) calls
those *"strongly discouraged"* for line spacing while describing their extra height as
there *"to accommodate tall glyphs or mark positioning"* — which is to say: they are large
enough precisely because they are not line spacing.

### 2. A script-aware `line_height` floor — rejected, because the measurements say the sentence is false

The floor is **1.3 on Noto Sans Thai and 1.6 on Sarabun**. It is a property of the *face*,
not of the *script*, and two faces bound nothing — a third could need more.

A schema field carrying that number would also be the thing
[ADR-0029](./0029-line-baseline-half-leading.md) already refused for `ascent`/`descent`, on
the rule that *a field is only legitimate when its value is derivable from what the document
itself declares*. A floor is worse than that field, because it is a guess **about fonts
that do not exist yet**.

### 3. A font-selection problem — adopted, and sharpened

The variance lives in the pair (face, script). `Noto Sans Thai` needs 1.3, `Sarabun` needs
1.6, Latin in either needs 1.0 — and that pair is exactly what the author fixes when they
choose a font and write a `line_height`. Which is what ADR-0007 already says: the literal
is the author's.

**But candidate 3 as #325 states it is incomplete, and shipping it unchanged would leave
the defect silent.** The author today gets no signal at all. So:

## Decision

**The format declines to close the authoring gap, and the tools stop being silent about
it.**

### Not a `render`-time computation change

ADR-0007's slot rule stands. Nothing in the metrics path needs to change: skrifa's
arbitration is already correct and already spec-conformant.

### Not a schema change

No `line_height` floor field and no per-script table. There is no number here the document
could carry that is derivable from the document. **The format will not learn to express
"this script needs a larger `line_height`," because the measurements show that sentence is
false** — it is *this font* that needs one, and the format already has a field for that. It
is called `line_height`.

### `validate` gains `R-LINE-INK-COLLISION`, `review`-class

For every multi-line text element, the real ink extents of adjacent lines are compared, and
a seam where line *i*'s ink passes line *i+1*'s ink top is reported. The renderer already
opens the font, already shapes the runs and already has every glyph's bounds, so this needs
no new machinery and no new input.

It is a **fact about the file and the fonts on disk**, which is exactly what
[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) says `validate` reports.
It is not a rule.

**`review`, never `error`.** A deliberately tight `line_height` is a real typographic
choice, and ADR-0006 reserves `error` for *guaranteed wrong*.

**It states no repair.** Two are legitimate — raise `line_height`, or set the text in a
face whose marks fit — and the document does not determine which. That is
[ADR-0043](./0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md)'s
question answered by not arising: `repair` is an axis of `error` alone.

**One finding per element**, naming the worst seam and counting the rest. ADR-0006's noise
budget: a ten-line Thai block collides at all nine seams by nearly the same amount, and the
second finding tells a reader nothing the first did not.

**Threshold provenance: internal.** The deciding number is zero, and both sides of the
comparison come from the document and the fonts it declares. Nothing is borrowed from
outside the format — which is the second reason candidate 2 failed, since a script floor
would have been an external constant under
[ADR-0061](./0061-validate-judgment-boundary-threshold-provenance-and-a-fenced-exception.md).

### `measure` gains the ink extents it already computes

Each line's real `ink_top`/`ink_bottom`, the block's own, and the seam between adjacent
inked lines. This is the **vertical half** of the per-line ink box
[ADR-0011](./0011-tool-surface-reads-checks-renders.md) names and
[ADR-0024](./0024-measure-writes-the-fit-repair-not-the-verdict.md)'s verb did not build.

The horizontal half stays unbuilt, for the reason already recorded: a full ink box is an
absolute rect, so it needs the block's horizontal placement — `x`, `origin`'s horizontal
component, and how `align`'s `start`/`end` resolve against a line's base direction under
bidi — and no ADR settles the last of those. **The vertical half was never blocked on any
of that**, which is why it is separable and why it is built here.

Still no verdict (ADR-0024): `measure` reports the seam and never judges it. What makes
this worth adding is that it turns the authoring loop ADR-0007 already makes mandatory into
one that can answer this question without rendering a frame and inspecting pixels — the
same argument ADR-0029 used to add `baseline_y` and ADR-0008 used to add break
opportunities.

### `fonts vendor` is left alone

It is a licence gate, not a typography adviser, and ADR-0057 already refuses to have it
re-adjudicate anything at `validate` time.

## Consequences

- A project setting a stacking script at a Latin `line_height` now says so on every
  `validate` run, at `review`, and gates nothing.
- `validate` now shapes every text element. That is the most expensive thing it does
  without a subprocess, and it is the price of reading ink rather than declared numbers.
- `measure`'s answer grows three keys. The canonical JSON is additive; no existing key
  changes meaning.
- A face whose marks fit its own default `line_height` produces nothing, which is every Latin
  project in this repo — including the committed fixture, asserted.

## What is still not measured

Recorded as gaps, not as findings:

- **Only two faces.** 1.3 and 1.6 bound nothing.
- **One size (55) and one weight (Regular).** The ratios are size-invariant by construction
  — everything scales with the em — but this was not checked at a second size.
- **Thai only.** Khmer, Lao, Myanmar, Devanagari and Vietnamese stack differently and are
  untested. #130 flagged the same gap and it is still open. The check itself is not
  Thai-specific and needs no list of scripts, so it will fire on them if they collide; what
  is untested is whether they do.
- **The block-top escape.** The first line's own ink escapes `block_top` by +8.45 px at
  `line_height` 1.1 on Noto Sans Thai and is still escaping at 1.4 (+0.20). ADR-0029's
  half-leading centres on the font's ascent and descent, which on these faces undershoot
  real Thai ink even for a single line. Whether that is a second defect or this one seen
  from the other end is **not settled** by these measurements. `measure`'s block-level
  `ink_top`/`ink_bottom` are deliberately not clamped to the block, so the number is
  visible to whoever takes it up; no check fires on it here.
