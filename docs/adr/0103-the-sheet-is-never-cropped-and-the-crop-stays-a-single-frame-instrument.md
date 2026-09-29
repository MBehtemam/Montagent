---
status: accepted
amends: 0097 (section 7's `--crop`-with-a-range refusal stops being *not-yet-legal pending #406*
  and becomes permanent: the spelling it fixed is kept, the placeholder it left is closed, and
  the refusal's message is given a required content), 0095 (section 2's two cropped-band figures
  are corrected — a 448 px tile at 18 tiles becomes **429 px** and 112 tiles admitted at the
  180 px target becomes **98** — because both were computed on a band carrying no label strip,
  which the tile-label ADR has since made impossible; the argument they support is unaffected)
---

> **Amended by [ADR-0105](0105-the-sheets-refusals-are-invocation-errors-its-blind-spots-are-not-findings-and-an-unpainted-state-is-quantization.md).** The refusal's "stable code" is **`E-INVOCATION`**; the
> two-step loop is its `reason` text. Juror 3's request, left open below, is settled: the sheet's
> own `below-tile-width` blind-spot sentence names `frame --crop --at <instant>`.

# The sheet is never cropped, and `--crop` stays a single-frame instrument

[#406](https://github.com/MBehtemam/Montagent/issues/406), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Resolved by a jury of three
independent models (Opus 5, Fable 5.1, Sonnet 5) put to one four-option question; ballots
verbatim in [`docs/research/juries/contact-sheet-crop/`](../research/juries/contact-sheet-crop/README.md).

The numbers this ADR corrects are re-derived by
[`docs/research/contact-sheet-crop/check_band_geometry.py`](../research/contact-sheet-crop/check_band_geometry.py)
— **19 claims in five groups**, all passing as committed, using #396's own grid code verbatim
rather than a reimplementation of it.

## Decision

**`--crop` composed with `--from`/`--to` is refused, permanently. The contact sheet is always
whole frames.** The refusal is an invocation error, not a not-yet-legal placeholder, and its
message **must name the two-step loop**: `frame --from/--to` to locate the instant, then
`frame --crop --at <instant>` to look closely.

Per-tile crop is **out of scope for map #395**, which means it does not return by graduating
from that map's fog. It returns only if someone redraws the destination, as a fresh effort with
its own evidence.

## Why

### 1. It deletes the one capability the sheet uniquely has

[#396](https://github.com/MBehtemam/Montagent/issues/396) measured the cropped band as the
highest-value mode in the feature — **429 px tiles against a whole frame's 184 px at the same
~1564 tokens**, with the fine-detail `word-08` defect still trivially legible at 36 tiles where
a whole-frame sheet lost it by 30. That gain is real and this decision spends it.

It spends it because the same sheet lost the planted `photo-07` defect **entirely**, and #396's
own write-up says why that is not a trade of one defect for another:

> the wrong photo *"is invisible in any single frame and only exists as a relation between
> tiles. The sheet does not merely make it cheaper to find; it is the only view in which it is a
> defect at all."*

A crop does not resolve that class more coarsely. It removes the only view in which the class
exists, in a sheet that — #396 again — *"looks clean and complete and is blind."* The map's
destination asks for an artifact *"honest enough that 'I checked the whole thing' is an artifact
rather than an argument."* A banded sheet cannot make that claim about anything but its band,
and nothing in the answer stops a reader treating it as the sheet.

### 2. The blindness is variable per call and authored by the reader

This is the jury's central contribution and it is the reason the disclosure route fails rather
than merely underperforms. All three jurors reached it independently.

[ADR-0094](0094-the-sheets-instants-are-visual-states-sampled-at-the-first-painted-frame.md) §6's
`blind_to` works because the rule's blindness is **constant**: a fixed enumeration, emitted on
every answer, is eventually internalised by the reader precisely because it never changes. A
caller-typed rectangle is the opposite shape. Juror 3 put it best, and the comparison is with
this map's own founding failure:

> *"A caller-typed crop is worse than the bounding-box sampler in one respect: the bounding-box
> agent's blindness was at least uniform and disclosable as a rule (\"this method cannot see
> color\"). A caller-typed crop's blindness is call-specific and content-dependent — it excludes
> whatever the caller didn't think to include, which is exactly the shape of blind spot no fixed
> `blind_to` line can characterize, because it changes every invocation."*

Juror 2 closed it from the third side: **the cropping agent and the reading agent are the same
agent**, so the disclosure *"tells it something it already decided and will discount."* And
Juror 3: *"a caller who typed the crop already believes they know where the defect would be,
which is precisely the belief the whole effort exists to distrust."*

ADR-0094 drew a two-way taxonomy — **`blind_to` describes the rule, `skipped` describes the
document** — and a caller-typed region is a property of neither; it is a property of this call's
arguments. That misfit is not an inconvenience to be patched with a third category. It is the
taxonomy correctly reporting that the flag is foreign to the instrument.

### 3. The value is already available, one call later

[ADR-0101](0101-a-crop-is-served-at-true-scale.md) made a single-instant `--crop` true scale. So
the loop this refusal preserves — **whole-frame sheet to locate the instant, `frame --crop --at`
to read the fine detail** — works today, needs no specification, and puts the crop where its
blindness is harmless: on one frame the caller has already chosen, after looking at all of them,
with no coverage claim attached.

What the refusal genuinely costs is *scanning* fine detail across many instants **without
knowing where to look**, and this is the honest hole in the decision. The mitigation is partial
and worth stating precisely: the one class measured to need it is `word-08`, a **text** defect,
and text has channels that do not go through pixels at all — `measure`'s extents, `query --at`'s
`ink_box`, and a plausible factual `validate` check (the fixture's own instance is filed as
[#404](https://github.com/MBehtemam/Montagent/issues/404)). Meanwhile the class with **no**
non-pixel channel, the pixel-only `sentence-08`, survives a whole-frame sheet down to 92 px —
the smallest width #396 ever tested. So the crop's measured win lands on the defect class that
has three other instruments, and the class that justifies the visual channel does not need it.

### 4. The evidence for the mode is thinner than its headline

The band was the hardcoded constant `(0, 1300, 1080, 360)`, written by hand three times across
#396's scripts, never computed from any element's geometry, in no normalized form. **No
alternative region was ever measured** — no photo band, no centre crop, no derived region. And
the tally on the cropped sheets is one defect confirmed legible, one confirmed annihilated, and
**`sentence-08` never re-read on a cropped sheet at all**, though the band's own name implies it
sits inside. A permanent hole in the instrument's central claim is not bought with one
hand-eyeballed rectangle on one fixture and one untested case.

### 5. A document-derived region is unsound as posed, not merely unbuilt

The obvious repair — derive the band from the document, e.g. the union of the rects of every
element that changed, so it cannot exclude a changed element by construction — was put to the
panel as its own option and **rejected on the merits by two jurors**. It is recorded here
because it is the shape anyone will reach for next, and the objection is not about maturity:

- **The guarantee is narrower than the guarantee the sheet needs** (Juror 1). `photo-07`'s
  wrongness lives in a photo whose rect need not be in the union at that instant, and a
  colour-collapse defect like `sentence-08` is a relation between an element and *what is behind
  it*. A union of changed rects can therefore still annihilate **both** planted defects — so
  "cannot exclude a changed element" does not buy what it appears to buy.
- **On a busy span the union collapses toward the whole frame anyway** (Juror 2), so it buys the
  most specification for the least demonstrated gain.
- It would inherit a **refusal surface**: `ink_box` declines on non-zero rotation, on non-unit
  `scale`, and on any run declaring `dir:"rtl"`, so the highest-value mode would refuse
  unpredictably on ordinary documents.

This does not say a derived region can never work. It says the version everyone reaches for
first does not, and that anything in this direction must be **built and measured against all
three planted defects** before it is a decision. That is a fresh effort, not fog on this map.

## What the refusal message must carry

The refusal is the teaching surface — all three jurors volunteered this unprompted — so it is
specified here rather than left to the implementer:

- it names **both** halves of the two-step loop, with the verb and flags spelled;
- it says the sheet is whole frames **by design**, not pending work, so no reader files it as a
  gap or waits for a flag;
- it carries a stable code, whose class is [#412](https://github.com/MBehtemam/Montagent/issues/412)'s
  to assign — and per ADR-0097 §5 that code must render as prose, since the no-`--json` path is
  the one an agent reaching for pixels takes.

It is an **invocation error**, in the verb, on the `measure` pattern ADR-0097 §3 used for `--at`
with a range. Note the asymmetry this leaves, deliberately: with a range, `--full` is an
invocation error (ADR-0097 §4) and `--crop` is now one too, while with a region `--full` is
redundant-but-legal (ADR-0101). The three-way combination `--crop --from --to --full` is
therefore unreachable, which is why ADR-0101's redundancy rule and this refusal never have to be
reconciled.

Juror 3 asked for more — that the **sheet itself** point at `frame --crop --at` for a closer
look. That is a good idea and it is **not decided here**: it is content in ADR-0094 §6's
disclosure prose, which #412 owns.

## ADR-0095 section 2's cropped-band figures, corrected

ADR-0095 §2 argues that **count is aspect-dependent** — the reason tile count is derived from a
width constant and never set by the caller — and cites the band at *"448 px"* for 18 tiles and
*"112 tiles where whole frames admit 18"* at the 180 px target. #396's FINDINGS.md says **429
px** for what is apparently the same sheet, and neither ADR-0095 figure appears in that ADR's own
check.

Both of ADR-0095's numbers **reproduce exactly** — on a band composed with **no label strip**:

| 3:1 caption band, 1080×360 | 18 tiles | max tiles at 180 px |
| --- | --- | --- |
| no label strip | **448.0 px** | **112** |
| with #396's 11% label strip | **429.3 px** | **98** |

#396's committed sheets all carry the label strip, and
[ADR-0098](0098-the-tile-label-is-a-floored-fitted-line-naming-the-change-at-its-own-boundary.md)
has since made a label **mandatory** and floored its type at 8 px served. So the labelless
variant is not a sheet this project can ship, and **429 px and 98 tiles are the admissible
pair**. Corrected in ADR-0095 §2 in place, under this ADR's banner.

**The argument is unaffected, and that is why this is a correction rather than a reversal**: 98
against 18 is still **5.4×**, so a single tile-count constant would still mean two different
legibility outcomes depending on tile shape, and count is still rightly derived. The reason to
fix it anyway is the one ADR-0098 gave for the label's own floor — a number routinely wrong by a
seventh is the one a reader learns to skip, and this one sits in the clause that explains why the
caller cannot ask for a tile count.

*(This ADR corrects figures describing a mode it declines to ship. That is deliberate: ADR-0095
§2 does not cite them to justify the crop, it cites them to justify **deriving count from
width**, which stands regardless.)*

## Costs, recorded honestly

- **The feature ships without its highest-measured-value mode.** 2.33× linear scale and ~5×
  tile density for band review are real and forgone. Juror 2's framing is the accurate one:
  *"my vote is that it cannot yet be shown honest, not that it is worthless."*
- **Fine detail across many instants has no single-call instrument**, and the two-step loop
  requires already suspecting a location. Juror 3 named this squarely — the workaround *"requires
  the agent to already suspect a location, which is not guaranteed."* The text channels in §3
  cover the measured instance, not the class.
- **The disclosure-was-read-past finding is one incident.** §2's argument leans on ADR-0101's
  observation that a correct caption lost to a doc-string promise, and all three jurors leaned on
  it harder than this ADR does. It is **one agent, one flag, one occasion**, and it is recorded
  as residue rather than as a principle. It also has a consequence no juror followed through: if
  a correct disclosure can lose to a doc string, that is a live problem for ADR-0094's whole
  unconditional-disclosure bet, not only for a crop. Recorded on map #395 as fog, not answered
  here.
- **The panel was unanimous and same-family.** Three models from one family agreeing is weak
  evidence; the ballots' value is in the two arguments the judge did not hold going in (§2's
  variable-blindness reframing and §5's demolition of the derived region), not in the count.
- **One mode is refused on an untested case.** `sentence-08` was never re-read cropped. Had it
  been, and had it survived, the case for the band would be one confirmed loss against two
  confirmed wins rather than one against one. Nobody measured it, and this decision does not
  wait for it — because §1's objection is structural and §2's is about the taxonomy, and neither
  turns on the third defect.
