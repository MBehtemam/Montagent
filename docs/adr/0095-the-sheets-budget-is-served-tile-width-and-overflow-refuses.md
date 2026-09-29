---
status: accepted
amends: 0067 (narrows its legibility floor to a standalone proxy frame, so the floor does not govern a contact-sheet tile, and adds a second legibility floor in a different currency), 0021 (states that its enforced `frame` budget does not bind the range mode, and adds a third budget that is observational)
---

# The sheet's budget is served tile width, and overflow refuses rather than thins

> **Amended by [ADR-0103](0103-the-sheet-is-never-cropped-and-the-crop-stays-a-single-frame-instrument.md).**
> Section 2's two cropped-band figures are corrected: the band's 18-tile served width is
> **429 px, not 448**, and it admits **98 tiles at the 180 px target, not 112**. Both of the
> original numbers reproduce exactly — on a band carrying **no label strip**, which
> [ADR-0098](0098-the-tile-label-is-a-floored-fitted-line-naming-the-change-at-its-own-boundary.md)
> has since made impossible by making a label mandatory and flooring its type. **The argument is
> unaffected** — 98 against 18 is still 5.4×, so count is still aspect-dependent and still
> derived from width. ADR-0103 also settles the per-tile crop this section's deferral pointed at
> ([#406](https://github.com/MBehtemam/Montagent/issues/406)): the sheet is **never** cropped,
> so the constant this ADR supplies for a cropped sheet's count now has no caller.

[#399](https://github.com/MBehtemam/Montagent/issues/399), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). This ADR fixes **what bounds a
contact sheet, what happens when a range does not fit, and what wall clock the range mode
is judged against.**

It stands on two closed tickets' evidence:
[#397](https://github.com/MBehtemam/Montagent/issues/397)'s cost model
(`research/visual-token-cost`) and [#396](https://github.com/MBehtemam/Montagent/issues/396)'s
legibility measurements
([`docs/research/contact-sheet-legibility/`](../research/contact-sheet-legibility/FINDINGS.md)).
[ADR-0094](0094-the-sheets-instants-are-visual-states-sampled-at-the-first-painted-frame.md)
decided which instants the sheet shows and explicitly deferred this question here.

Every number below is re-derived by
[`docs/research/contact-sheet-budget/check_tile_budget.py`](../research/contact-sheet-budget/check_tile_budget.py),
which exits non-zero the moment any of them stops holding — **28 geometry and cost claims on
their own, 33 with a built binary and the fixture supplied.** All pass as committed.

## Decision

**The budget is denominated in served tile width. The target is 180 px, the floor is
140 px, and a range that does not fit at the floor is refused.**

1. **Served tile width in pixels is the budget's currency.** Not visual tokens, not tile
   count, not wall-clock seconds. Tile count is **derived** from the width constant and the
   near-square grid, so it is an output of the policy rather than an input to it.
2. **Two constants, mirroring the proxy ladder's two floors.** A **target of 180 px** every
   caller gets by default, and a **floor of 140 px** — #396's measured cliff — reached by
   exactly one degrade step.
3. **One degrade step, then a refusal.** A range whose visual states all serve at ≥180 px is
   drawn at the target. One that does not degrades **once** to the floor, and the degradation
   is disclosed. One that does not fit at 140 px is **refused**, and the refusal **names
   sub-ranges that would fit**.
4. **Overflow may never thin, split, or reshape the sheet.** Not dropping states (ADR-0094
   forbids it outright), not splitting across several images, not falling back to a strip.
5. **No enforced wall-clock budget.** The range mode does not inherit ADR-0021's `<500 ms`
   `frame` budget, which it cannot meet. It gets a third number that is **observational**:
   measured reference examples, recorded and never enforced.
6. **The disclosure carries the served tile width and which rung produced it**, on every
   answer including an undegraded one, per ADR-0094's unconditional-disclosure rule.

## Why

### 1. A token budget never fires, so it cannot be the denominator

The map's Notes say *"budget in input tokens, not tiles or pixels"*, on #397's finding that
the caption is a second uncosted budget at 20.1×. That instinct was right about captions and
wrong about the sheet, and the sweep is unambiguous: across **every** tile count from 4 to
48, the near-square sheet spends **1518–1568 tokens** — never below 96.8% of the standard
tier's 1568-token cap.

The sheet saturates the tier at every count because the tier caps **served** size, so the
composite is downscaled to the same ~1.15 MP whatever it was authored at. A cap that is
already spent at every point on the curve does not bound anything. **A denominator has to
vary with the thing it bounds**, and tokens do not; served tile width falls monotonically
from 392 px at n=4 to 114 px at n=48, and is the quantity #396's legibility verdicts were
actually measured in.

The most useful way to state the cost is therefore not a budget at all but an equivalence:
on the standard tier a 1080×1920 frame serves as 819×1456 = **1560 tokens**, and the 18-tile
sheet is **1560 tokens**. **Eighteen visual states cost exactly what one `frame --full`
costs**, and 2.23× the half-scale default. That is the number to put in the tool's
description, and it is an argument for the feature rather than a constraint on it.

*(ADR-0011's `2691` for a full frame holds only on the high-res tier, which is
[#405](https://github.com/MBehtemam/Montagent/issues/405), not this ADR's to fix.)*

### 2. Width rather than count, because count is aspect-dependent

Tile count looks like the natural knob and is the one #399's own title reaches for. It fails
on a second instrument the same feature is likely to grow: #396 measured that 18 tiles
cropped to the caption band serve at **429 px** against a whole frame's **184 px**, because
a 3:1 tile grids far better than 9:16. Held at 180 px, the band admits **98** tiles where
whole frames admit **18**.

*(Both figures are corrected by ADR-0103, from 448 px and 112 tiles. The originals were
computed on a band with no label strip; ADR-0098 has since made a label mandatory. The ratio
this section rests on is 5.4× either way.)*

So a single tile-count constant would mean two different legibility outcomes depending on
tile shape, while a single width constant means one outcome across both. This is the same
argument ADR-0046 used to choose a long-edge **cap** over a scale factor for the proxy
target: express the constant in the units the thing actually fails in.

It also settles a question that would otherwise recur. Because count is derived, the caller
never sets it, and there is no flag for "give me 40 tiles" to have to refuse.

### 3. The refusal is the feature, not the fallback

ADR-0094 contributed exactly one constraint here — *"silently shrinking the boundary set is
the one thing overflow must never do"* — which rules out dropping states. Two more options
fall to measured facts rather than taste:

**Splitting across several sheets loses the only defect class the sheet uniquely catches.**
#396 found the wrong-photo defect **invisible in any single frame**: it exists only as a
relation between tiles, `skeleton` captions over the `spider` picture. ADR-0094 already
names the cost — *"a range split across two sheets loses exactly that"* — and #397 adds a
correctness ceiling: **more than 20 image blocks in one request silently clamps every image
in that request, earlier turns included.** A split is capped at 20 and corrupts context that
has already scrolled past.

**Reshaping into a strip is strictly dominated.** The 18×1 strip spends **392** of 1568
tokens and serves **87 px** tiles against the grid's **184 px** — narrower pictures while
leaving three quarters of the budget unspent. There is no budget at which it is right, so it
is not an overflow response any more than it is a default.

That leaves thinning or refusing, and this is where Juror 1's proposal — recorded in
ADR-0094 as *"the strongest idea in the panel"* and deferred to this ticket — is adopted.
**A 40-tile sheet is structurally unable to show fine detail and looks complete.** It is not
a degraded answer; it is a confident one that is wrong, which is precisely the failure this
map exists to prevent. The trial agent that missed the pixel-only defect did so because *its
silence read as coverage*; a sheet thinned past the floor rebuilds that failure inside the
tool built to fix it.

Naming sub-ranges that would fit is what makes the refusal actionable rather than merely
honest. The caller asked a question the instrument cannot answer at the fidelity that would
make the answer mean anything, and the remedy — ask twice over two halves — is one the tool
can compute exactly.

### 4. ADR-0067's legibility floor does not govern a tile, and saying so is mandatory

This is the one place this ADR amends an accepted decision rather than adding to one, and
#399 flagged it in advance: ADR-0067 records 360p as a legibility floor and calls it
*"a guard on the ladder's future, and on any caller-specified proxy resolution"*. A tile
scale is a caller-influenced resolution, so the guard reaches for this case **by name**.

**If it governed a tile the feature would be dead.** That floor is 640 px long edge; an
18-tile sheet's tile is 184×328, long edge 328 — barely half of it. There is no tile count at
which a contact sheet clears 360p, so either the floor does not apply or the map has no
destination.

It does not apply, and the reason is in how each number was measured. ADR-0050 measured 360p
on the real fixture **downscaled and scaled back to native for equal-size viewing — the way
a preview player shows a proxy in a fixed viewport**, judging a frame standing alone as the
whole of what is being read. #396 measured the **composite**: a tile in a grid, beside its
neighbours, under a fitted label. Those differ in kind, not just degree, and the difference
is load-bearing — the invisible-sentence defect survived to 92 px tiles precisely *because*
the neighbouring cards have text and the label says this one should, a cue no standalone
frame carries. ADR-0067 already anticipated this shape of correction: *"legibility tracks
the smallest on-screen text, not resolution alone"*, and *"the two measurements are not
directly comparable, and are not being compared."*

So ADR-0067's floor is **narrowed in writing to a standalone proxy frame**, and the sheet
gets its own floor in its own currency. The amendment is not optional bookkeeping.
**ADR-0067 exists because ADR-0050 and ADR-0065 both landed on `main` using the word "floor"
for different things, neither amending the other** — recorded as
[#178](https://github.com/MBehtemam/Montagent/issues/178). Shipping a third floor silently
would be that failure a third time, in the file whose entire subject is that failure. The
glossary now carries three limits under distinct names.

The honest residue: **140 px is a weaker number than 360p.** ADR-0050's came from three
independent jurors; #396's is one primed session's reading, and #396 says so in terms —
*"an upper bound on legibility, not a floor"*. That is exactly why the **target** is 180 px
and not 140: the 31% margin is doing the work the missing jurors would have done. The floor
is where the tool refuses, not where it is comfortable.

### 5. Wall clock is observational, because the width floor already bounds it

Measured on the committed fixture (1080×1920/25, 18 instants, release build, reference
hardware), rasterization is **~80 ms per full-resolution frame**, so 18 frames take
**~1.4 s** (1.37–1.47 s across four runs; ~2.1 s if each is also encoded and written to
disk). Cost is linear in tile count.

Three consequences, and they all point the same way:

- **The range mode cannot inherit ADR-0021's `<500 ms`.** One sheet is 2.8× it. Inheriting
  it would put the verb in permanent violation of its own budget, and that has to be stated
  here rather than discovered from a failing test.
- **No new deadline needs to exist, because the width floor is the tighter constraint.** The
  floor caps the count at 30, which projects to **~2.4 s** — inside `preview`'s 5 s with
  nothing enforcing it. The densest sheet #396 tested (72 tiles) would run ~5.9 s and miss,
  but the legibility floor forbids that sheet on other grounds first. **Two independent
  constraints agree, and the one we are already adopting binds first.**
- **A time miss would have no honest remedy anyway.** Dropping tiles is forbidden by
  ADR-0094, and rasterizing smaller barely helps: half-scale rasterization saves **6–12%
  across runs**, not the ~75% the pixel-area ratio suggests, because the dominant cost is
  source decode and composite, which does not shrink when the output does. That is the **same** finding
  ADR-0065 used to reject a third proxy rung — *"the cost that remains is decode, which is
  driven by the source size and does not shrink when the target does"*. An enforced budget
  whose only available response is a refusal the caller cannot act on is worse than a
  recorded number, which is ADR-0021's own reasoning for full-resolution `preview` and
  ADR-0072's for `render`.

**So tiles are rasterized at true project pixels and composited down.** No proxy ladder
enters `frame`. This is worth stating as a decision rather than an omission, because it is
the obvious optimisation and it buys ~10% at the cost of the one guarantee ADR-0021 built
`frame` around — *"one tool it can unconditionally trust for pixel-accurate checks... without
first asking whether what it's looking at is a lie."*

## What this ADR does not decide

- **Flag spelling and the verb table** — [#401](https://github.com/MBehtemam/Montagent/issues/401).
  This ADR fixes that a refusal exists and what it must name, not what the refusal's code is
  called or how the range arguments are spelled. **#401 now carries more weight than its
  title suggests** — see the risk below.
- **Whether the refusal and the `skipped` entries carry finding codes of their own**, and
  their class under ADR-0006/ADR-0043. The map's open fog, sharpened by this ADR: a refusal
  is not a finding about the document, and ADR-0094 already ruled that a blind spot of the
  *rule* earns no code while a fact about the *document* does. Where an over-long range
  falls in that split is not obvious and is not decided here.
- **What the label carries** — [#400](https://github.com/MBehtemam/Montagent/issues/400).
  #396 established the label is not the binding constraint (readable at 92 px tiles), so it
  does not bound this budget; it must still be *fitted* to tile width, since a fixed-size
  label overflows and overprints its neighbours.
- **Per-tile crop** — [#406](https://github.com/MBehtemam/Montagent/issues/406), blocked by
  [#402](https://github.com/MBehtemam/Montagent/issues/402). This ADR supplies the constant a
  cropped sheet's count would be derived from (180 px against its own tile aspect), not
  whether the mode ships.

## Trade-offs and risks

- **The refusal is the common case for anything longer than the fixture, not the edge
  case.** The fixture's ~18 visual states land on the 180 px target exactly, which is luck
  rather than design: 65 seconds of this density fits, and roughly 110 seconds does not fit
  even at the floor. Callers will hit the refusal routinely. This is accepted — the
  alternative is a sheet that lies — but it makes the refusal's ergonomics load-bearing, and
  that is #401's, which is why the pointer above is emphatic. A refusal that does not make
  narrowing obvious will read as the feature being broken.
- **140 px rests on one primed observer.** Stated in §4 and mitigated by the target's margin
  rather than designed away. A three-juror pass of ADR-0050's shape would strengthen it and
  has not been run.
- **The width constants are certified for this corpus, not universally** — the limit
  ADR-0067 records about 540p applies here unchanged. Legibility tracks the smallest
  on-screen text, so a project with proportionally smaller captions than the fixture reaches
  the risky range at a wider tile. The disclosure of served tile width is what lets a caller
  notice.
- **Two constants in a currency `proxy.rs` does not speak.** That module is arithmetic on
  long edges of whole frames; tile width is neither. Putting them there would overload the
  word *floor* a fourth time, and putting them elsewhere splits the legibility numbers across
  two modules. The second is preferred, and the glossary carries the distinction.
- **Timing is hardware-bound and stated as such.** `check_tile_budget.py` re-measures rather
  than asserting a constant, and the numbers above are bounds on one machine. The *ordering*
  claims it checks — under 5 s at 30 tiles, over 500 ms at 18, half-scale saving far less
  than pixel area — are what the decision rests on, and they are robust to a fast or slow
  machine in a way the absolute figures are not.

## Consequences

- `frame`'s range mode takes no tile-count argument. The count is derived from the width
  constant, the tile aspect, and the near-square grid.
- Two new constants live with the sheet's geometry, not in `montagent-render::proxy`:
  a 180 px target and a 140 px floor, both **served** tile widths.
- Every range answer discloses the served tile width and which rung produced it, degraded or
  not, alongside ADR-0094's `rule` / `skipped` / `coverage` / `blind_to` fields.
- A range that does not fit at the floor is refused, naming sub-ranges that would fit. The
  refusal's code and class are #401's and the map's open fog, not settled here.
- ADR-0021's `<500 ms` `frame` budget is stated as not binding the range mode; a third,
  observational number joins the two it defines.
- ADR-0067's 360p legibility floor is narrowed to a standalone proxy frame and confirmed not
  to govern a sheet tile. It remains unreachable by the `preview` ladder, exactly as before.
- `CONTEXT.md` gains the sheet's two width limits beside the existing two, under names that
  do not reuse *floor* unqualified.
