---
status: accepted
---

# Cross-track coverage is group-scoped pairing, not a frame-wide union

> **Amends [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md).** Its
> severity rule — *"compute the union of visual coverage; a gap uncovered in that
> union is `review`"* — is **withdrawn**, not merely superseded: [ADR-0011](./0011-tool-surface-reads-checks-renders.md)'s
> final section found it fires on **zero** of the fixture's eleven real visual gaps,
> across every one of the 2¹³ possible track subsets. This ADR is the resolution
> ADR-0006 deferred to [#23](https://github.com/MBehtemam/Montaget/issues/23).

`validate` computes cross-track coverage by **pairing**, not by a project-wide union.
For every `group` whose members include both an audio element and a visual element,
`validate` reports where either side's time-union is not covered by the other's —
**symmetric**: audio active with no visual in the group is reported exactly like
visual active with no audio in the group. **No new field is introduced.** `group`
stays exactly what [`CONTEXT.md`](../../CONTEXT.md) already says it is — an optional,
render-inert free-text label — and gains no rendering or timing semantics.

Gaps that fall **outside** any audio/visual group pairing — a hole between two
consecutive elements in a single visual track, with no audio member to be measured
against — remain exactly what [ADR-0005](./0005-absolute-integer-milliseconds.md)
already made them: **legal, reported, and not an error.** This ADR does not invent a
second, elevated severity for them.

## Why the union rule had to go, not just be tuned

Ticket #23 carries three pieces of hard evidence that per-track checks (gap/overlap
within one track) cannot see:

1. **Narration outran the last visual by 0.4–0.5 s** in three independent edits —
   audio present, nothing on screen. Invisible to every check the agents ran.
2. **An 800 ms hole in `photo` and `caption` together**, mid-video — visual absent,
   audio unaffected. Self-reported "medium-high confidence," survived a hand-written
   overlap checker.
3. **A lower-third's card and text drifted 64 px apart** during a shared "slide in"
   motion — a `group`-mediated defect, not a coverage one (see Scope, below).

ADR-0006 proposed the union rule to catch (1) and (2). Tested against the committed
fixture rather than the defective, uncommitted file its own jury had reviewed, the
rule is dead on arrival: eight header-chrome elements (`group:"header"`) span the
full 65.2 s runtime, and `photo`/`caption` are independently contiguous end-to-end.
Any coverage basis that includes any of them reports full coverage forever, even with
every real photo, card and caption deleted from the file. There is no subset of the
13 visual tracks for which the rule fires on exactly one of the fixture's real gaps.

**Area-awareness — "how much of the frame is opaque" — was considered and rejected
on the same fixture, not in the abstract:**

- The frame is **never** close to fully covered even by design: instantaneous opaque
  coverage peaks at 79.64%.
- Two chrome elements (`chip-panel`, `handle-panel`) are filled the exact byte value
  of the project `background`. An area rule blind to colour counts invisible pixels
  as coverage.
- Real occlusion exists (`chip-panel` at layer 30 sits over `photo` at layer 10), so
  a correct area answer needs painter's-order resolution and per-element opacity.
- 22 of the fixture's 40 visual elements are `text`, whose drawn extent is not yet a
  settled quantity, and `photo` uses a time-varying `fit: cover` scale ramp.

Area-awareness is opacity, colour, z-order, a settled text model and an unprincipled
threshold — a much larger check than the one it was meant to replace, and this
project has already ruled a check that size out once, on ADR-0006's own "much larger
check than it appears" language.

## Why the fix is not a declared field

The one signal that happens to separate the always-on header chrome from real timed
content — spanning the full project duration — is **circumstantial, not designed**,
and codifying it fails in both directions:

- **As a declared field** (`role: content|chrome`), it is the `kind`-on-track field
  this project already rejected 8/8 in [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md#severity-is-computed-from-the-consequence-at-an-instant):
  a second fact that can drift from the truth, wrong exactly when it matters — an
  author repurposing a chrome element as content leaves the label stale and
  `validate` confidently blind.
- **As a heuristic** ("spans the full project duration ⇒ chrome"), it misclassifies
  a legitimate full-length asset — a single background video running the whole
  project — as invisible to every coverage check, with no way for an author to see
  that it happened.

`group` survives as the mechanism precisely because it makes no claim about what an
element **is**. Membership can be absent; it cannot be wrong the way a declared
`role` can. A gap between two elements that do not share a group produces no
coverage finding at all — not a false "covered," just nothing asserted — which keeps
`validate`'s discipline from [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md#findings-state-facts-never-repairs):
state what the document proves, never what would have to be inferred about intent.

## What this gives up, stated rather than glossed

There is no longer any check that answers **"was the picture on at every instant of
the project."** A file that plays chrome continuously while leaving black between
every ungrouped photo produces per-track gap facts — `note` severity, per ADR-0005 —
and nothing stronger. The union rule never actually delivered that stronger
guarantee either; it only appeared to, on a file that was never committed. An
always-green check is worse than an absent one, and this ADR chooses absent.

`validate`'s report must say what was checked, not imply completeness:
group-pairing coverage findings apply only to elements that share a `group`, and the
report does not claim to have evaluated ungrouped visual content against anything.

## Scope: two findings graduated out, not answered here

Two related defects surfaced downstream of this ticket, from
[#21](https://github.com/MBehtemam/Montaget/issues/21)'s work on the transform
model. Both were considered for inclusion in this ADR and **excluded**:

- **Group-shared keyframe-time disagreement** (the lower-third drift, evidence 3
  above): two elements sharing a `group` whose transform keyframes disagree in
  *timing* drift apart during a shared motion. This is a semantic promotion of
  `group` — from a label `validate` merely groups by, to one it reasons about
  animation curves through — with its own false-positive envelope (a card that
  *deliberately* settles before its text is a legal edit this check must not flag).
  That promotion is a decision on its own terms, not a corollary of coverage.
  Graduated to a new ticket.
- **The frame-change census**: after a `frame`/aspect-ratio edit, which elements now
  fall outside the visible canvas. A spatial-bounds question triggered by a
  different edit class (`frame`, not `shift` or ordinary timing edits), sharing no
  data path with tracks, groups or audio. Graduated to a new ticket.

Neither is answered by this ADR. Both are cross-element consistency predicates in
the same family as this one — per-track validation blind to them — but neither
shares this ADR's specific mechanism or its false-positive shape.

## What remains open, and is not this ADR's to close

- **The segment boundary is not a field.** Several elements in the fixture end at
  the identical timestamp by coincidence, not by any declared relationship, and
  nothing here gives an author a way to see which times multiple tracks agree on.
  Unchanged fog.
- **Whether an ungrouped visual gap should ever carry elevated severity** on some
  basis other than a declared field or full area analysis is not settled by this
  ADR. Absent a new idea, ungrouped gaps stay `note`.

## Consequences

- **Amends [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)**: its
  severity section's union-of-visual-coverage computation is replaced by this ADR's
  group-scoped pairing rule for the coverage class specifically. The `kind`-on-track
  rejection and the "severity from consequence, not from the check or the track"
  framing both stand unchanged.
- **No schema change.** `group` remains optional and free-text.
- **`validate` reports what it checked.** A coverage finding is scoped to grouped
  audio/visual pairs; the report does not imply ungrouped content was evaluated.
- Two new tickets graduate: group-shared keyframe-time consistency, and the
  frame-change census — both out of this ADR's scope.

## The court

Put to an independent panel of three jurors (Claude Opus, Sonnet and Haiku, blind to
each other's ballots) on four questions: the coverage mechanism itself, whether the
check must be symmetric, whether the keyframe-time check belongs in this ticket, and
whether the frame-change census does.

- **Mechanism: unanimous 3/3** for group-scoped pairing plus leaving per-track gap
  reporting as-is (no new field, no heuristic).
- **Symmetry: unanimous 3/3** yes — the 800 ms hole carried no audio anomaly, so a
  one-directional check is falsified by evidence already in hand, not merely
  incomplete.
- **Keyframe-time check in scope: split 2–1** against inclusion. The dissent argued
  a shared question ("what does `group` license"); the majority held that the two
  checks diverge in false-positive shape and deserve independently argued
  resolutions. This ADR follows the majority.
- **Frame-change census in scope: unanimous 3/3** against inclusion.
