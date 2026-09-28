---
status: accepted
amends: 0094 (decision 2's "the label prints both the sampled instant and the run's boundary" is
  specified as a lossless signed offset rather than two absolute numbers; decision 5's "labelled in
  a different register" is specified as a visual mark on the sheet plus a class token on the
  provenance line; decision 6's per-tile provenance is specified to repeat its presence set in full
  on every line and to assert zero counts for the absent tile classes), 0097 (its section 7 joint
  discharge is completed — the label half is specified here, and the guarantee is made width-proof
  by flooring the type size instead of the character count), 0060 (records that its overlap-scoped
  tie rule leaves layer order non-total on a legal document, so a downstream mechanical selector
  needs a third key)
---

# The tile label is a floored, fitted line that names the change at its own boundary

[#400](https://github.com/MBehtemam/Montagent/issues/400), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Resolved by a jury of three
independent models (Opus 5, Sonnet 5, Fable 5.1) put to eight sub-questions; ballots verbatim
in [`docs/research/juries/contact-sheet-tile-label/`](../research/juries/contact-sheet-tile-label/README.md).
The label-width measurements are in
[`docs/research/label-legibility/`](../research/label-legibility/OUTPUT.txt) and the fixture
claims in [`docs/research/tile-label/`](../research/tile-label/OUTPUT.txt), both carrying
re-executable checks.

[ADR-0094](0094-the-sheets-instants-are-visual-states-sampled-at-the-first-painted-frame.md)
fixed which instants the sheet shows and mandated a fitted per-tile label without saying what it
carries. [ADR-0095](0095-the-sheets-budget-is-served-tile-width-and-overflow-refuses.md) fixed
the budget as served tile width, 180 px target and 140 px floor.
[ADR-0097](0097-the-range-is-from-to-on-both-surfaces-and-the-caption-becomes-an-attribution-obligation.md)
restated ADR-0011's unconditional caption as an **attribution** obligation discharged *jointly*
by this label and a range-level provenance list, and said so explicitly: *"this transfers load
onto #400... a label that is unreadable at the served width, or that drops its identifying
field, is no longer a cosmetic defect but a violation of ADR-0011 as amended here."*

This ADR fixes **what the label carries, where it sits, how it degrades, and how its one
identifying field is chosen.**

## Decision

1. **The label sits in a gutter strip beneath its tile**, 11% of tile height, never burned into
   the tile's pixels.
2. **The label is one line of five fields in a fixed order**, identical on every tile:
   `<index> <sigil> <instant>ms <signed boundary offset> <signed id>`. On the fixture's busiest
   tile that is `9 42800ms +37 +word-08-bridge`.
3. **The run boundary is printed as a signed offset from the sampled instant, not as a second
   absolute millisecond.** The offset is always printed, including `+0`.
4. **Type size is floored at 8 px served, and content gives way, never the type.** The label is
   fitted to tile width *subject to* the floor.
5. **Elision is a property of the sheet, not of a tile.** If any tile's label cannot carry the
   identifying field at the floor, **no tile carries it**; if the numeric core still does not fit,
   the sheet **refuses**, as ADR-0095's width floor already refuses.
6. **Tile class is carried twice, in two registers**: a visual treatment on the sheet, which is
   load-bearing at the floor, and an unabbreviated class token on every provenance line, which is
   load-bearing in text. Document-derived tiles are the unmarked default, **and the answer
   asserts the zero counts** for the classes not present.
7. **The provenance list repeats its full presence set on every line.** No delta.
8. **The identifying field names what changed at this run's own boundary**, signed `+` for
   entered and `-` for departed, selected as the **highest-layer** candidate, tie-broken by
   element id, with **whole-document-span elements excluded from candidacy**.
9. **The label is drawn in a UI face Montagent vendors for its own chrome** — never the
   project's declared font, never a system font.

## Why

### 1. An overlay's legibility would depend on the content it describes

All three jurors rejected burning the label into the tile, and the reason that decides it is not
the area: it is that an overlay is drawn *on top of the evidence*, so how readable the label is
varies with what is underneath it. For a feature whose whole purpose is showing pixels, that is
backwards. Juror 3 added that an overlay lands in exactly the band where this fixture's small
captions live — the fine-detail class #396 measured dying first, at ~140 px.

Juror 1 found the argument that closes it, and it is one the brief did not supply: a burned-in
label means **the tile is no longer a faithful crop of what `frame --at` returns**. That breaks
the reproduce-the-tile property ADR-0094 bought by refusing the midpoint, and the two-stage habit
depends on it. The 11% strip is also already inside ADR-0095's geometry, so keeping it leaves the
180/140 cliffs valid as measured rather than requiring them to be re-derived.

### 2. Fixed arity, because a missing field must not be ambiguous

Because the label is fitted, its content sets its own type size, so a variable-length field makes
type size vary per tile. The measurement is stark: a label carrying the full presence set spans
**1.83–2.67 px across one sheet, a 1.46× spread, and the smallest type lands on the busiest
tile** — the tile most likely to be hiding something.

Two jurors reached the deeper objection independently. A field that is sometimes present and
sometimes absent is ambiguous between *"this tile has no such element"* and *"this tile's label
ran out of room"*, and that ambiguity is this project's house failure mode — a silence that reads
as an all-clear. So arity is fixed and a field with no value prints a placeholder rather than
vanishing.

### 3. The presence set is measurably impossible on the label, which settles §7 of ADR-0097 empirically

ADR-0097 §7 credited Juror 3 with finding that a fitted label *"cannot name the presence set"* at
the 140 px floor. The measurement is stronger than the argument was: at the **180 px target** the
fixture's typical 10-element set gives **2.67 px** and its busiest 13-element set **1.83 px**.
The set does not fit at the target either, let alone the floor. The routing ADR-0097 chose on a
prediction is now backed by a number.

The measurement also found **8 of every presence set are always-on header chrome**, identical
across all 18 states — so even had it fit, most of it would have carried no discriminating
information. That fact does real work in §8 below.

### 4. Both numbers survive, because the boundary compresses losslessly

ADR-0094 decision 2 requires the label to print the sampled instant *and* the run boundary. All
three jurors refused to let an elision drop the boundary, and Jurors 1 and 3 independently
proposed the same escape: the sampled instant is the least painted millisecond at or after the
boundary, so the two **always differ by less than one frame period** — checked on the fixture at
a maximum of 37 ms against a 40 ms frame, never more than two digits. Printing the offset costs
three characters where two absolutes cost seven, and loses nothing.

Juror 1's refinement is adopted: **`+0` prints**, so an on-grid boundary is an assertion rather
than an absence. Juror 1 also named the harm that makes the field non-negotiable — a reader who
sees only the instant will take it *for* the boundary and be wrong by up to a frame, which is a
small, plausible, silent error.

### 5. "Fitted" without a floor is how a label reaches 1.4 px and still "fits"

This is the ADR's central mechanism and it was unanimous. Fitting to width has **no lower
bound**: that is precisely how the presence-set candidates reached 1.4 px, a size that satisfies
every geometric definition of fitting and cannot be read. Juror 2 named it exactly — *"an agent
sees a labelled tile and assumes attribution succeeded"* — which is the trial failure rebuilt
inside the label.

Flooring the type inverts the failure. What degrades is **how much identity the label carries**,
never whether it can be read, and that is the graceful-degradation shape this repo already uses
for the `preview` proxy ladder (ADR-0021/0046/0065/0067/0078), applied to type instead of pixels
and disclosed the same way. It is also what makes ADR-0097 §7's guarantee statable: that ADR
rejected a guarantee which *"holds at the target and lapses exactly where a degraded sheet is
hardest to read"*, and **an unfloored fitted label is that guarantee** — flooring is what fixes it.

**The floor is 8 px, and 6.3 px is deliberately not used.** Juror 1 put the reason best: the
6.3 px figure came from one primed observer who knew what the defects were, so it is *a ceiling
on what was ever read, not a floor on what can be read cold*, and ADR-0095 already flags it as
non-re-derivable. Taking an optimistic single-observer datum as a hard spec constant is how a
spec inherits a perceptual claim it cannot defend. Juror 2 wanted 9–10 px; 8 px is the two-juror
answer and keeps the identifying field alive at the target.

**Elision is sheet-wide, and this is Juror 1's contribution.** If elision were per-tile, the
quiet tiles would stay detailed while the busiest tile — statistically the likeliest to hide a
defect — is the one silently thinned. A sheet that is uniformly less informative is honest; one
whose thinning tracks its own crowding is not. Sheet-wide elision also keeps type size near
uniform, so a reader calibrates once.

**The floor bites on this fixture, at the floor and not at the target**, which is the behaviour
the decision wants and is asserted in the check: the longest fixture label is 29 characters, at
**10.17 px at the 180 px target** and **7.93 px at the 140 px floor**. So a full-detail sheet at
the target degrades to an addressable-only sheet at the floor, exactly where ADR-0095's refusal
is about to fire. The numeric core is 13 characters at 17.69 px and never the thing that yields.

### 6. Two registers, because each one fails where the other holds

The panel split on which carrier is load-bearing — Juror 1 for the text sigil, Juror 2 for the
visual treatment — and Juror 3 dissolved the split rather than picking a side. Each objection is
correct about the other channel: a tint is a perceptual claim that survives no quotation, crop or
grayscale reproduction (Juror 1), and a text prefix costs width exactly when width is the scarce
resource, dying at the floor while a border does not (Juror 2). So the visual mark is
load-bearing **on the sheet**, and the class token is load-bearing **on the provenance line**,
where width is free and it can be quoted and grepped.

ADR-0094 decision 5 was explicit that this is not cosmetic: ship uniform infill wearing the same
label as document-derived tiles and the trial's failure is rebuilt inside the sheet. Marking only
the exceptional classes is safe because the dangerous confusion runs **one way** — a synthetic
sample passing as document-derived — and the mark is generated *for* synthetics, so absence
cannot be forged by the tool.

But absence is still a silence, and Juror 3 supplied the fix this repo's own culture demands:
**the answer asserts the zero counts** — `0 keyframe, 0 infill` — so "no synthetic tiles here" is
a statement rather than an inference. This is ADR-0094 §6's unconditionality argument applied one
level down: a mark that appears only when something is unusual teaches the reader that its
absence is an all-clear.

### 7. A delta in the provenance list would rebuild the hazard the list exists to remove

Juror 2 argued for a delta on cost. Juror 1 checked the cost and it does not hold: the
**28,410-character** figure that motivated splitting attribution was eighteen resolved *stacks* —
geometry and resolved keyframe values — not element ids. Full sets across 18 lines are ~2.2 KB,
against a whole sheet answer measured at **1,413 characters**, with the budget saturating
regardless. With the cost premise gone, the delta's remaining properties are all costs.

Both other jurors reached the same objection: a delta forces the reader to reconstruct state by
accumulation to know what is on screen at tile 7, and that reconstruction step is where a wrong
attribution enters. ADR-0097 §7 established misattribution as a **per-tile** hazard and made
per-line self-containment the whole point; a delta re-introduces range-scoped ambiguity spread
across lines instead of collapsed into one block. Juror 1 added that a delta also destroys the
ability to quote one line as evidence, and that a line saying nothing becomes indistinguishable
from a line whose author had nothing to say.

Juror 1's alternative remedy is recorded for if size ever does bite: **declare the always-on set
once** as a fact about the whole document — a statement, not a running difference.

### 8. The field is causal, not discriminating — and the fixture overturned the panel on its direction

All three jurors independently reframed the question, and the reframing is the panel's main
contribution. *"Discriminating"* cannot mean "not present in every tile of this sheet", because
that is **range-dependent**: the same instant would label differently depending on the range
asked for, which is the same synthetic irreproducibility ADR-0094 rejected the midpoint for. The
fix is to ask what is **causal**. Each tile exists *because* the filtered presence set changed at
its boundary, so the label names that change — a function of `(document, boundary)` alone,
identical in any range containing it, mechanical, and carrying no judgement about what matters,
so it clears ADR-0006's no-verdicts line and the map's out-of-scope ranking.

All three also independently added the **sign**, which was not in the brief and is not cosmetic:
a departure-only run names an element that is **not on screen in that tile**, and naming it
unsigned would license pinning the tile's pixels on something absent — the exact per-tile
misattribution ADR-0097 §7 split the list to prevent.

**Three things the fixture then settled that the panel could not.**

**(a) Whole-document-span elements must be excluded from candidacy, or the rule names chrome.**
Jurors 1 and 3 expected chrome to fall out automatically because "chrome never enters". On the
fixture that holds for 17 of 18 runs — and fails on the first, where *everything* enters, chrome
included. Unexcluded, the topmost rule labels tile 1 `+chip-text`. Excluding whole-document-span
elements from candidacy fixes tile 1 and subsumes Juror 1's separate first-tile fallback into the
general rule. The check asserts that the always-on set and the whole-document-span set are the
same 8 elements on this fixture.

**(b) The direction is highest-layer, and the 2–1 vote is not why.** Neither side argued the
direction substantively. The fixture decides it: the two directions name disjoint element
classes — topmost names the caption and sentence tracks, lowest names the photo and card tracks —
and **each names exactly one of the two planted defects**, `sentence-08` on tile 10 for topmost
and `photo-07` on tile 7 for lowest. The tie-break is that the *pixel-only* defect is the one
this map exists for: all three trial agents found the wrong photo by reading the JSON before
rendering anything, while the recoloured sentence was the one the third agent missed entirely.
#396 found it *only because the label said the card should carry text*. Topmost names the element
class the visual channel is uniquely needed for; lowest names the class that never needed a
picture.

**The honest cost, which no juror stated: one id cannot name both defects.** Whichever direction
is chosen, the other defect's tile names a sibling rather than the culprit. The label is a
pointer, not a census — the census is the provenance list, which is precisely why ADR-0097 made
the discharge joint.

**(c) Layer order is not total, so the selector needs a third key — and this corrects a premise
all three jurors shared.** Every ballot took layer order to be total *because* "a layer tie is
already an error". [ADR-0060](0060-layer-tie-is-an-error-array-order-stays-meaningless.md) makes a
tie an error **only when the two elements' boxes overlap in time and space**; a tie between
non-overlapping elements is legal, and this very fixture declares two of them — tracks
`chip-panel`/`handle-panel` both at layer 30, and `flag-field`/`handle-logo`/`handle-text` all at
31. So the selector must break a remaining tie, and it does so on **element id**, which is unique
by schema, total, range-independent, and — critically — **not array order**, which ADR-0060 keeps
meaningless *"full stop... never promoted to mean stacking"*.

### 9. A label that fails on an unusual document fails hardest where it is needed most

Unanimous, and the reasoning converged. The project's own declared font is disqualified three
times over: it may be Thai-only, it may carry no digits at all while the label is mostly digits,
and this repo's main fixture **already declares a font path that does not exist**
([ADR-0057](0057-font-vendoring-licence-gate-and-path-keyed-attestation.md)). Juror 1 put the
principle: chrome that fails when the document is unusual fails hardest on the projects most
likely to be broken. A system font is disqualified by ADR-0064's six release targets — the fitted
type size, and therefore the 8 px floor and the 295/chars law, would mean a different thing per
platform, and the measurements would stop being re-derivable.

So Montagent vendors one permissively licensed face for its own chrome, under ADR-0057's licence
gate and ADR-0090's allowlist. **Tabular figures**, since the payload is columns of milliseconds.
And the build **fails loudly** if the face is absent rather than falling back — the prototype
quietly drawing labels with whatever font happened to sit in the fixture directory is the precise
accident this closes.

## The label, field by field

| field | example | mandatory | why it is there |
| --- | --- | --- | --- |
| index | `9` | yes | the key into the provenance list; what an agent says when it says "tile 9" |
| class sigil | *(absent)* | yes, when not document-derived | ADR-0094 decision 5's register, redundant with the visual mark |
| sampled instant | `42800ms` | yes | the argument to the second call — the label prints what you type to reproduce it |
| boundary offset | `+37` | yes | ADR-0094 decision 2's second number, losslessly compressed; `+0` prints |
| signed id | `+word-08-bridge` | no — elides sheet-wide at the floor | what changed here, and on which side of the boundary |

## What this ADR does not decide

- **The vendored face itself** — which family, its licence bucket under ADR-0057, and the
  build-time check. Filed as [#421](https://github.com/MBehtemam/Montagent/issues/421).
- **Whether the 8 px floor and ADR-0095's 140 px floor survive a cold observer** — both rest on
  one primed observer and both jurors who chose 8 px asked for this explicitly. Filed as
  [#422](https://github.com/MBehtemam/Montagent/issues/422).
- **The codes and classes** the elision disclosure carries —
  [#412](https://github.com/MBehtemam/Montagent/issues/412), which §5 adds one member to and
  which ADR-0097 §5 already constrains to prose-renderable codes.
- **The visual treatment's exact appearance** — border, tint or rule. Decision 6 fixes that one
  exists and that it is not the load-bearing carrier in text; the pixels are an implementation
  choice.
- **Per-tile crop** — [#406](https://github.com/MBehtemam/Montagent/issues/406), blocked by
  [#402](https://github.com/MBehtemam/Montagent/issues/402). A cropped sheet's tiles are 2.33×
  wider, so the floor bites later there; nothing here presumes otherwise.

## Trade-offs and risks

- **The 8 px floor is margin over a single datum, not a measurement.** Juror 1 called it the
  weakest thing in its own ballot. It is the one non-re-derivable constant this ADR adds, flagged
  the way ADR-0095 flagged its 140 px floor, and #422 owns re-measuring it.
- **Sheet-wide elision means the busiest tile taxes the quietest ones.** Accepted: the
  alternative taxes the tile most likely to hide a defect.
- **The identifying field names a sibling rather than the culprit for one of the two planted
  defects**, whichever direction is chosen. Stated in §8 rather than left in the gap; the
  provenance list is the remedy.
- **A departure-only tile names something not on screen.** The minus sign carries that entirely,
  and if it is ever lost in reproduction the label becomes actively misleading — which is why the
  class and the sign also appear in the provenance line's prose. **This branch is untested by
  evidence**: the fixture has *no* departure-only run, asserted in the check, so the rule is
  reasoned rather than observed.
- **The panel answered eight questions in one ballot**, inheriting the cross-question trading
  weakness ADR-0097's README recorded for seven. Juror 2's Q3 and Q4 answers visibly trade
  against each other. The four unanimous answers are the ones this ADR leans on hardest.
- **Vendoring a face takes on a licence obligation, binary weight and a hard build dependency**,
  and the label will never match the project's typography. All three are cheap against a label
  that cannot render digits.
- **Excluding whole-document-span elements is a rule with a blind spot**: a project where a
  genuinely interesting element spans the entire document will never have it named. On this
  fixture that set is exactly the 8 chrome elements, but that is a fixture fact, not a guarantee.

## Consequences

- The tile label is `<index> <sigil> <instant>ms <±offset> <±id>`, fitted to tile width, floored
  at 8 px served type, elided sheet-wide when the identifying field will not fit, and refused when
  the numeric core will not.
- ADR-0094 decision 2 is satisfied by a signed offset; decision 5's register is a visual mark plus
  a provenance-line class token; decision 6's provenance repeats its full presence set per line
  and asserts zero counts for absent tile classes.
- ADR-0097 §7's joint discharge is complete, and made width-proof rather than width-dependent.
- ADR-0060 gains a note that its overlap-scoped tie rule leaves layer order non-total on a legal
  document; this ADR's selector breaks the remainder on element id.
- Montagent gains a vendored UI face for its own chrome (#421) and a cold-observer re-measurement
  of both perceptual floors (#422).
- `CONTEXT.md` gains **tile label** and **served type floor** in the vocabulary.
