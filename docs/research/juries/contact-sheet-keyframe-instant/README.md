# Jury: which painted frame is a keyframe tile sampled at?

The second panel on [#418](https://github.com/MBehtemam/Montagent/issues/418). It was convened
because the first, [`../contact-sheet-flag-names/`](../contact-sheet-flag-names/README.md), found
that the disclosed keyframe counts could not mean anything until this was fixed. It is a new
question, not a re-run to break a tie. Resolved as
[ADR-0106](../../../adr/0106-the-sheets-opt-ins-are-keyframes-and-infill-ceiling-and-a-keyframe-tile-is-sampled-where-its-change-first-paints.md)
decisions 8 and 10–13.

Each juror received `BRIEF.md` **verbatim and identical**, with no stance and no sight of each
other's ballots or the judge's recommendations. Ballots are recorded here unedited.

| juror | model | ballot |
| --- | --- | --- |
| 1 | Opus 5.5 | [opus-5-5.md](opus-5-5.md) |
| 2 | Sonnet 5.5 | [sonnet-5-5.md](sonnet-5-5.md) |
| 3 | Fable 5.1 | [fable-5-1.md](fable-5-1.md) |

| | Q1 sample frame | Q2 coincidence | Q3 no frame left in the run | Q4 change points only |
| --- | --- | --- | --- | --- |
| Juror 1 | (a) first painted ≥ `t` | (a); population grid-aware | **(c)**, else (a) | stays |
| Juror 2 | (a) | (a); counts grid-aware | **(a)** | stays |
| Juror 3 | (a) | (a); raw-ms population, grid-aware counts | **(c)**, else (a) | stays |

## The split

On Q3, Juror 2's case for "untiled with a reason" rests on *"the video also never shows it"*. That
is false whenever the element is still visible in the next run, because the frame right after `t`
is on screen and **is** that run's tile. Jurors 1 and 3 both observed that (c) adds no tile and no
rule: it is Q1's rule applied across the boundary. The human took that read.

Q2 hid a difference in *where* the jurors drew the line:

- Juror 1 would drop from the population a change point that lands on a run tile's frame.
- Juror 3 keeps membership a raw-millisecond test anyone can read off the document, and makes
  only the `tiled`/`untiled` split grid-aware.

The judge took Juror 3's version. On the fixture both print `0 tiled, 0 untiled`.

## What the panel added

**Juror 3: the label prints the painted millisecond, the provenance line prints
`element.property@t`**, so `frame --at <painted ms>` reproduces the tile exactly. Adopted.

**Juror 3: a peak tile, if one is ever wanted, is a fourth opt-in class** (`extremum`), never a
reinterpretation of `keyframe`. Recorded in ADR-0106 decision 13.

**Juror 1: the `between-keyframes` sentence should say where keyframe tiles are sampled.**
Adopted as an amendment to ADR-0105.

## Where the panel is thinner than it looks

Q1 was unanimous, and the brief leaned on ADR-0094 §2's run rule as the obvious analogue.
Unanimity there is weak evidence of anything beyond that precedent being clear. Neither panel saw
a keyframe population at a frame rate other than 25 fps.
