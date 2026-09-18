# Jury: the v1 implementation ticket breakdown

For [#168](https://github.com/MBehtemam/Montaget/issues/168), the v1 implementation spec, at the
`/to-tickets` step of the map [#2](https://github.com/MBehtemam/Montaget/issues/2)'s hand-off.

Three independent jurors (Opus, Sonnet, Fable), each given the same brief ([`BRIEF.md`](BRIEF.md)
— the author's proposed 22-ticket breakdown plus the `/to-tickets` vertical-slice rules), blind to
each other, instructed to default to "refuted" and to verify against the repository rather than
against the brief. Full ballots: [`opus.md`](opus.md), [`sonnet.md`](sonnet.md),
[`fable.md`](fable.md).

Unlike this project's other juries, the subject was not a format decision. It was a **work
breakdown** — and the court's finding is that the breakdown's defects were symptoms of two
upstream defects it had no authority to fix.

## Verdicts

| | Opus | Sonnet | Fable |
| --- | --- | --- | --- |
| Q1 — vertical or horizontal? | accept w/ modification | accept w/ modification | accept w/ modification |
| Q2 — blocking edges | **refute** | accept w/ modification | **refute** |
| Q3 — granularity | **refute** | accept w/ modification | **refute** |
| Q4 — coverage | **refute** | accept w/ modification | accept w/ modification |
| (a) CI early with an uncalled `skia-safe` | accept w/ mod | accept | accept |
| (b) falsification at frames, not whole video | accept w/ mod | accept | accept w/ mod |
| (c) refuse-class folded into the finding type | accept w/ mod | accept | accept w/ mod |

**All three of the author's stated judgment calls survived.** Nothing else about the draft's
structure did.

## Unanimous findings

**The per-check tickets are genuinely vertical.** The court was invited to attack the four-way
split of the check engine (structural time, structural document, caption, geometry) as four
horizontal cuts wearing a vertical costume. All three rejected that attack, on the same reasoning:
once the report spine exists, a check ticket with #168's mandated firing/non-firing **fixture
pair** is narrow-but-complete and demoable through `validate` over both adapters. The
vertical-slice rule forbids stopping at one layer, not shipping one rule family.

**Story 29 (`R-EASE-INERT`) is misfiled.** All three moved it out of the caption-and-text ticket.
It is a held-keyframe check (ADR-0052, *"literal exact equality of author-written `v`"*) with no
text in it, and it needs the keyframe resolver, not the text engine.

**The `probe` ticket does not depend on the document model.** `probe` answers questions about a
media file by path (ADR-0023, stories 76–78) and reads no project. The edge was spurious, and
removing it is what lets `probe` land early enough to unblock the four tickets that turn out to
need it.

**Three tickets cannot fit one context window:** the document model, the text engine, and the
rasterizer. The rasterizer was worst — Sonnet decomposed it into six.

## Where the court split

**The caption ticket's dependency on the text engine — 2–1 against the author.** The author had
flagged this edge as his least confident. Sonnet found it real for one of five stories
(`R-BOX-SLACK`) and spurious for the rest. Fable refuted it entirely: ADR-0058 computes
`R-BOX-SLACK` *"with no I/O"*, and ADR-0028's block height is
`(size × line_height×10 × lines + 9) // 10` — integer arithmetic on the document, with no font
query in it. Opus alone preserved the edge, on the ground that the arithmetic *lives in* `measure`.
**Fable's reading governs**: where the code lives is a packaging choice, and the dependency is
about data.

**Whether the whole-video comparison is a ticket at all.** Opus moved it into `render` as an
acceptance criterion. Fable refuted that directly and the refutation holds: frames cannot falsify
duration, frame count, audio placement, or the `speed: 0.645` on four narration elements. It
survives as the only test of the audio path.

**Whether the wide-refactor exception applies to the document model — 2–1 against.** Sonnet argued
for expand–contract, sequencing the closed-schema half after `probe` and `measure` have each
touched real files. Opus and Fable both refused, on the same ground: expand–contract exists for a
mechanical change with thousands of live call sites, and on day one there are zero; it would only
let a wrong type survive longer beside a right one. Both named the real risk instead — the type
will be specified against too few consumers — and both proposed mitigations at the type's own
ticket rather than a sequencing rule. Fable adds the sharper reason to freeze early: ADR-0041
makes Rust struct field order *the canonical key order*, so reordering a field is a format change
visible in `fmt` and `LAYOUT` on every file.

## The three headline findings

Each juror named a different "most important thing the author got wrong". They are compatible, and
they point the same way.

**Opus — the breakdown was sized against the story list, but #168 says the ADR series is the
specification.** Every ticket carried story numbers and the coverage walk came out clean, which is
what hid the problem: the format's largest bodies of work have **no story at all** — the effect
model (ADR-0040), the four colour filters (ADR-0049), transitions (ADR-0059), keyframable `volume`
(ADR-0055), `loop` (ADR-0062), `rect`/`ellipse` and stroke-as-paint (ADR-0014), `highlight` windows
(ADR-0048), required unique `id`s (ADR-0019), and `fonts vendor`'s licence gate (ADR-0057). All of
it lands unnamed inside the two tickets the court independently found unsizeable.

**Fable — the blocking graph was drawn from the verb table, which is not the dependency
structure.** Each ticket was placed by the verb it *ships* rather than the data it *consumes*.
Sharpest instance: FFmpeg resolution sits in the packaging ticket, blocked by `render`, while
`probe` cannot spawn `ffprobe` without it — a four-ticket dependency inversion. The same error put
`preview` behind `frame` when `preview` is a span of video in every preview ADR and needs the
encoder.

**Sonnet — story 29, and what it implies about the other 21 tickets.** A numeric coverage audit
passes clean because story 29 *is* claimed; it is just claimed by the wrong ticket, which would
send an implementer looking for font metrics and finding a keyframe check. The generalisation is
the finding: if grep-level care assembled the story ranges without reading what each story says,
every other ticket deserves the same second pass.

## Two defects the court escalated past ticketing

**1. ADR-0040 contradicts itself against the committed fixture** (Fable, verified by the author).
`handle-logo` carries a bare `"mask": "circle"`. ADR-0040's schema clause puts masks inside
`effects: [{name, ...params}]`; its Consequences say the bare key *"becomes a valid declaration …
no migration needed"*. Under ADR-0017's closed schema those cannot both hold. ADR-0040's own body
contains the unresolved fork verbatim — *"converts that stray field into either a valid declaration
or a validation error"* — and the Consequences picked a branch without reconciling it with the
schema clause. This is an ADR question, not a ticket question, and it is settled separately in
[`../mask-spelling/`](../mask-spelling/).

**2. The falsification test compares two typefaces** (Opus and Fable independently, verified by the
author). `fixtures/en-halloween-decorating/README.md` records the reference MP4 as set in *"SF Pro
Rounded, bold throughout"*. [#143](https://github.com/MBehtemam/Montaget/issues/143) re-vendored
`OpenRunde-Bold.otf` because SF Pro Rounded was never in the repo and is not redistributable, and
its own step 5 flagged the consequence without resolving it: the fixture's hand-tuned sizes were
measured against the old metrics. Twenty-two of sixty elements are text. No SSIM threshold both
passes that and stays sensitive enough to catch a real defect — and tuning it loose is precisely
the *"completed, looked plausible, was wrong"* failure ADR-0010 says this project has already had
twice. Two consequences the court drew: the comparison is **diagnostic, not golden**, and must be
region-masked and named as such in the ticket, or the first implementer will "fix" the renderer to
match a dead font; and a ticket nobody has written is needed to re-derive the fixture's 22 text
sizes under Open Runde, because until it exists `R-BOX-SLACK` may fire on the fixture — which by
#168's own rule would mean the check is wrong.

**And a gap no juror was asked about and two found anyway:** nothing owns **audio mixing**. It is
not a story in #168 and not in any ticket's scope, yet the fixture is 20 audio elements and 22.4 s
of narration, four of them at `speed: 0.645`.

## Method note

The brief was written by the author of the thing being judged, and carried at least two errors a
diligent reader could catch blind: it said #168 has 83 user stories when it has 84 (Opus — stories
1–83 plus `62a`, which a coverage walk over 83 rows drops), and it described the contested
caption-to-text-engine edge in terms of the author's own uncertainty rather than the ADRs, which is
what let two jurors find the misfiled story underneath it. Every juror verified the fixture's
counts independently; all confirmed 14 tracks, 60 elements, 155 lines.

The exercise also bounded its own authority correctly. Asked four questions about a work
breakdown, the court returned two findings that no ticket set could absorb — one requiring an ADR
amendment, one requiring new measurement — and said so rather than routing them into a ticket
body. That is the outcome the *"do not decide what the ADRs left open"* rule in #168 is for.
