# Jury: does the range mode crop every tile?

Three jurors, three different models, put to
[#406](https://github.com/MBehtemam/Montagent/issues/406) on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Resolved as
[ADR-0103](../../../adr/0103-the-sheet-is-never-cropped-and-the-crop-stays-a-single-frame-instrument.md).

Each juror received `BRIEF.md` **verbatim and identical** — no assigned stance, no persona,
no sight of each other's ballots, dispatched in parallel. Ballots are recorded here unedited.

| juror | model | ballot |
| --- | --- | --- |
| 1 | Opus 5 (1M context) | [opus-5.md](opus-5.md) |
| 2 | Fable 5.1 | [fable-5-1.md](fable-5-1.md) |
| 3 | Sonnet 5 | [sonnet-5.md](sonnet-5.md) |

The brief was put as **one multiple-choice question with four options** (ship caller-typed,
ship document-derived, refuse, or reject the framing), with a rider that a vote to ship must
also specify the disclosure. That is the
[instant-selection panel](../contact-sheet-instant-selection/README.md)'s shape rather than the
[tool-surface panel](../contact-sheet-tool-surface/README.md)'s seven-sub-question shape,
chosen because the ticket turns on a single disposition.

## The verdict was unanimous, which is itself weak evidence

All three chose **(C) refuse**, so there is no split to read. Three same-family models agreeing
is not independent confirmation, and the panel's own value here is in the arguments rather than
the count — two of which the judge did not hold going in.

## What the panel added that the ticket did not have

**The blindness is variable per call and authored by the reader.** All three reached this
independently and it is the panel's central contribution. The ticket framed a cropped sheet's
blindness as a disclosure problem; the jury reframed it as a *taxonomy* problem. Juror 1:
*"blindness of the rule is constant and learnable, so a fixed enumeration eventually gets
internalised. A caller-typed rect makes the blind spot variable per call and authored by the
reader."* Juror 3 sharpened it into the panel's single strongest line — the structurally blind
sampler that founded map #395 was in one respect **less dangerous**, because its blindness was
uniform and therefore statable (*"this method cannot see color"*), where a caller-typed rect
*"excludes whatever the caller didn't think to include, which is exactly the shape of blind spot
no fixed `blind_to` line can characterize, because it changes every invocation."* Juror 2 closed
it from the third side: the cropping agent and the reading agent are **the same agent**, so the
disclosure *"tells it something it already decided and will discount."*

**Option B is unsound, not merely unbuilt.** The judge had offered a document-derived union
region as the one shape that could earn its way back, to be parked as fog pending measurement.
Two jurors rejected it on the merits. Juror 1: the guarantee *"cannot exclude a changed
element"* is **narrower than the guarantee the sheet needs** — `photo-07`'s wrongness lives in a
photo whose rect need not be in the union at that instant, and a colour-collapse defect is a
relation between an element and *what is behind it*, so a union of changed rects can still
annihilate both planted defects. Juror 2 added that on a busy span the union *"collapses toward
the whole frame anyway"*, buying the most specification for the least demonstrated gain while
inheriting `ink_box`'s refusal surface (rotation, non-unit scale, RTL).

**The refusal message is the teaching surface.** All three volunteered it: the refusal must name
the two-step loop rather than merely block the invocation. Juror 3 went further and wanted the
*sheet itself* to point at `frame --crop --at`, which ADR-0103 records as a question for
[#412](https://github.com/MBehtemam/Montagent/issues/412) rather than deciding.

## Where the panel is thinner than it looks

All three leaned on the brief's fact 3 — ADR-0101's read-past disclosure — as *the* reason
disclosure cannot rescue a crop. That is **one incident, one agent, one flag**, and ADR-0103
records it as honest residue rather than a settled principle. No juror followed it through to
its wider consequence: if a correct disclosure loses to a doc-string promise, that is a live
problem for ADR-0094's whole unconditional-disclosure strategy, not only for a crop. ADR-0103
routes that to the map's fog rather than answering it.
