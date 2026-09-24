---
status: accepted
amends: 0007 (states the baseline rule the slot definition never gave), 0011
  (`measure`'s per-line output gains `baseline_y`)
---

# The line baseline is half-leading, read from every run's metrics

[ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) gives a line's **slot** —
*the largest `size` among the runs on that line* × `line_height` — and a worked
example that places the block (size 55, `line_height` 1.1, centred on 1537 →
1506.75–1567.25). It never says where the glyph baseline sits **inside** that slot.
[#59](https://github.com/MBehtemam/Montagent/issues/59) measured why that gap is
live, not academic: against the fixture's actual font at size 55, ascent + descent
**exceeds** the 60.5 px slot (negative leading), so the three candidate conventions
— CSS half-leading, ascent-anchored-to-top, descent-anchored-to-bottom — disagree by
roughly 5 px on every caption in the corpus, an order of magnitude past the
half-pixel questions this area otherwise raises. Two independently-built renderers
had already picked different, undocumented conventions and shipped two videos.

## Decision

**Baseline placement is half-leading, centred on the slot:**

```
baseline_y = slot_centre_y + (ascent − descent) / 2
```

where `slot_centre_y` is the slot's already-defined centre and `ascent`/`descent`
are read from the font at render time — facts the renderer already has to open the
font to draw a single glyph, never authored fields. This is CSS's `line-height`
half-leading and canvas `textBaseline:"middle"`, matched to prior art an agent
already knows rather than invented here.

**On a line with runs at different sizes, `ascent` and `descent` are each the
maximum across every run on that line — not the metrics of the one run that sets
the slot height.** The slot-height rule and the baseline rule answer different
questions: slot height asks how much vertical space the line reserves, and ADR-0007
already answers that with one privileged run; baseline placement asks where the
shared baseline must sit so that **no** run's glyphs are clipped, and every run
draws on that same baseline regardless of which run is tallest. Taking the largest
run's metrics alone is unsound the moment a smaller-size run carries an unusually
tall ascender (a mixed-script fallback, an icon glyph, a different family) — it can
out-ascend the "authoring" run and clip against the slot with every field in the
document individually legal. Half-leading around the per-run maxima keeps the
symmetry the slot's own centre-based definition assumes: the overflow of any run,
in either direction, splits evenly around the slot's centre rather than landing
entirely on whichever edge the chosen convention favours.

**No schema change.** `ascent` and `descent` are not in the document and are not
becoming fields — the same reasoning that has already kept `speed`'s divisor and
`fit`'s box dimensions out of the schema applies here: a field is only legitimate
when its value is derivable from what the document itself declares, and font
vertical metrics are not.

**`measure` reports the resolved value.** Per [ADR-0011](./0011-tool-surface-reads-checks-renders.md),
`measure`'s per-line output gains `baseline_y` — the absolute y-coordinate the
formula above resolves to for that line — so an agent verifying a caption's
placement reads a number instead of re-deriving the arithmetic by hand or rendering
a frame to inspect pixels. This is the same shape of gap ADR-0008 closed for break
opportunities: a quantity the renderer must compute anyway, now made checkable
without executing code in the agent's head.

## Evidence

Three-model independent court (Claude Opus 5, Claude Haiku 4.5, Claude Fable 5.1),
each blind to the others' ballots, briefed on the measured ~5 px divergence and the
worked slot example, put to four sub-questions.

**Unanimous 3/3** on two questions: that this needs **no schema field** (ascent and
descent are renderer-internal facts, structurally identical to the `speed`/`fit`
inputs this project has already excluded from the document), and that **`measure`
should report the resolved `baseline_y`** (the settled precedent — extend `measure`
whenever a new derived layout quantity is pinned down — applies directly, and the
alternative leaves an agent with no way to get an exact number short of pixel
inspection).

**Split 2–1 on the convention** — Opus and Fable for half-leading, arguing from the
slot's own centre-based definition: a symmetric box needs a symmetric placement
rule, or the overflow silently favours one edge in a font- and content-dependent
way. Haiku argued for ascent-anchored-to-top on simplicity and single-reference-
point grounds, without addressing what happens to the descent side once the slot is
already too short.

**Split 2–1 on multi-run metrics, in the other direction** — Haiku and Fable for
"largest run only," preserving ADR-0007's one-privileged-run story; Opus dissenting
with a concrete failure case (a smaller run's taller ascent clips against the slot)
that neither majority juror's reasoning addressed.

**Resolved by the human**: half-leading (majority) for the convention, and
**max-across-all-runs (minority) for multi-run metrics** — the same symmetry
argument that decided the first question requires the second: a rule that can
silently clip an unprivileged run's glyphs is not "no run overflows the slot
asymmetrically," it is exactly the asymmetry the first decision was chosen to
avoid. The "one author run" simplicity argument is a documentation preference, not
a correctness property, and this project has repeatedly found that the elegant
single-author story is the one a concrete case falsifies (ADR-0012's straddling
keyframe, ADR-0014's paint-versus-pixels line). Full ballots: `docs/research/juries/line-baseline/`.

## Consequences

- Every conforming renderer computes the identical baseline for a given slot and
  font, closing the divergence that motivated this ticket.
- The existing (undocumented) prototype at `docs/research/prototypes/rust-rasterizer/src/text.rs`
  already implements half-leading for the single-run case — this ADR gives it
  documentary authority and does not change its output there.
- **Untested by the fixture.** `fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json`
  contains zero multi-run text elements with differing run sizes, so the
  max-across-all-runs rule has no worked example to check against and changes zero
  committed bytes. This is the same shape of gap ADR-0022's effect-vocabulary entry
  and ADR-0007's own text-primitive discussion already flag: a real decision made
  with no fixture evidence to fall back on.
- `measure`'s response schema gains `lines[].baseline_y` — additive, not breaking.

## Not settled here

- **Subpixel precision of `baseline_y` is not constrained.** Unlike `line_height`
  (ADR-0028) or fitted extents (ADR-0013), baseline placement feeds no
  boundary-sensitive operation (`ceil`/`floor`/rounding a container) — it is a
  continuous coordinate consumed by antialiased rasterization, the same exemption
  ADR-0028 already carves out for `scale`/`rotation`/`opacity`/`ease` control
  points. If a future defect ties baseline drift to float non-determinism across
  renderers, that is a new, measured question, not a reopening of this one.
- **Whether `frame` or `query` should also expose `baseline_y`**, beyond `measure`.
  Scoped out: `measure` is the tool ADR-0007/ADR-0008 already mandate in the
  authoring loop, and nothing here shows a need to duplicate the value elsewhere.
