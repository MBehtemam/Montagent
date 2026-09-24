---
status: accepted
---

# `validate` reports facts, and `render` is what enforces them

> **Amended by [ADR-0076](./0076-the-four-structural-time-finding-codes-are-ratified.md)**,
> which ratifies the four structural-time codes — `E-TRACK-OVERLAP`, `N-TRACK-GAP`,
> `E-SPEED-MISMATCH`, `E-OVERRUN-UNNEEDED` — that ADR-0004 and ADR-0020 name the
> condition for but not the code, class or repair class of; and ratifies `N-TRACK-GAP` as
> `note`-class, the `review` belonging instead to `R-VISUAL-GAP` (ADR-0018), which alone
> sees the whole frame.

> **Amended by [ADR-0078](0078-preview-is-the-ninth-mcp-verb-and-its-unstated-readings-are-ratified.md)**,
> which extends the enforcement half of this ADR to a verb that did not exist when it was
> written: **`preview` runs the identical check engine and refuses on any `error`, exactly
> as `render` does.** A document the checks refuse is one no painter can be handed, and a
> preview that rendered what `render` refuses would be the one artefact in the product
> showing an illegal project. Nothing about `validate`'s side moves — it still reports and
> never enforces.

> **Amended by [ADR-0079](0079-fmt-check-stays-exit-0-l-layout-is-ratified.md)**, which
> settles that `validate`'s exit code stays untouched by `LAYOUT` findings, the same
> `error`-only rule this ADR and ADR-0011 already state, on a file that is legal but not in
> canonical convention — the identical question `fmt --check` raises.

> **Amended by [ADR-0069](./0069-probe-sidecar-is-a-per-user-json-cache-keyed-on-what-montagent-observed.md)**,
> which designs the *"the probe cache is a gitignored sidecar"* consequence
> this ADR stated and left undesigned, and **places it outside every
> repository** rather than beside the project: one JSON file per user under
> the platform cache directory, keyed on a canonicalised `(path, size, mtime)`,
> holding the whole probe. A missing, corrupt or unwritable sidecar is a cache
> miss and never a finding. The gitignore this ADR asked for is thereby
> structural rather than a rule each repository must remember.

> **Amended by [ADR-0061](./0061-validate-judgment-boundary-threshold-provenance-and-a-fenced-exception.md)**,
> which names the test this ADR's "may not state anything that requires
> knowing what the video is for" sentence left implicit — **threshold
> provenance, not severity, decides whether a check states a fact** — and adds
> a fenced exception: a check may compare a document-derived fact against a
> cited external numeric threshold only at `review`/`note` severity, never
> `error`, stating the raw measurement and citing the source. `R-CAPTION-PACE`
> (ADR-0034) is the first and, to date, only member of that exception; every
> other existing check was already compliant.

> **Amended by [ADR-0052](./0052-review-check-for-inert-ease-on-held-keyframes.md)**,
> which adds `R-EASE-INERT` (`review`): consecutive keyframe records on one
> element/property whose `v` holds identical (whole-value, exact, no tolerance)
> while still carrying an `ease` that therefore describes no motion.

> **Amended by [ADR-0034](./0034-caption-pace-and-repeat-duration-checks.md)**, which
> adds two checks this ADR's list has no entry for: `R-CAPTION-PACE` (a reading-speed
> floor, `review`) and `R-CAPTION-REPEAT-DURATION` (identical text at disagreeing
> on-screen durations, `review`). Both are computed from `runs`, `start`, `end` and
> `fps` with no I/O; neither changes the text model.

> **Amended by [ADR-0033](./0033-same-source-cut-continuity-is-a-review-check.md)**,
> which fully specifies the "animation discontinuity at a same-source cut" check
> named below and corrects its fixture timestamp: the second same-source cut is at
> **64016**, not 64816 as stated further down this document.

> **Amended by [ADR-0062](./0062-loop-declares-a-boolean-wrap-r-source-cut-pop-extends-mechanically.md)**,
> which adds a project-level boolean `loop` field and extends
> `R-SOURCE-CUT-POP` (ADR-0033) to a track's wrap-around pair when declared —
> no new finding code.

> **Amended by [ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md)** on the
> overflow check this ADR parked. It **must not be worded "text overflows its box"** —
> 15 of the fixture's 22 text elements have no container behind them — and its extent
> gains a `2 × stroke_width` term on both axes. Its width term counts as `UNCHECKED`
> until `measure` has been run, because that axis is the one that goes stale silently.
> This ADR's opt-in rule is what kept the text box required.

> **Amended by [ADR-0011](./0011-tool-surface-reads-checks-renders.md)** (and, on the
> text-overflow check, by [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) and
> [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md), which adds an
> aperture term to it and three checks: transform keyframe *times* that disagree across one
> `group`, a frame-change census, and the resolved segment table for any eased property).
> The design below stands. **Three of its stated facts do not**, and a reader should
> not rely on them: the union-of-visual-coverage severity rule **fires zero times** on
> the committed fixture; *"one visual gap out of eleven"* is **unreproducible** (the
> denominator is right, and the file its judges saw is not in this repository); and the
> exemplar finding *"no element on any visual track … the frame is background plus the
> header"* **contradicts the rule it illustrates**. What counts as content coverage is
> [#23](https://github.com/MBehtemam/Montagent/issues/23). See ADR-0011's final section.

> **The claim that every numeric claim was verified by script is itself
> unreproducible** — surfaced by
> [#154](https://github.com/MBehtemam/Montagent/issues/154). The project file
> the scripted verification ran against, from #9's second (defective) editing
> exercise, was never committed:
> `git log --all --diff-filter=A -- '*.montagent.json'` returns exactly one
> project file, and it is not that one. No later reader can re-run the
> verification this ADR describes below, so the sentence should be read as an
> unverifiable historical claim rather than a standing guarantee. See
> `docs/agents/domain.md`'s evidence-commit checklist, tightened by #154 to
> catch this case going forward.

> **A fourth stated fact does not hold either** — surfaced by
> [#60](https://github.com/MBehtemam/Montagent/issues/60). The *"`sentence-quiz`
> overhangs its card's top edge by ~6 px"* claim below is **false against the
> committed fixture**: `sentence-quiz` is one run with no `\n`, so under
> [ADR-0008](./0008-line-breaks-belong-to-the-agent.md)'s no-auto-wrap rule it is one
> line, and that line sits 53.75 px **inside** its card, not over it. Reproducing
> ~6 px needs roughly three lines — automatic wrapping, which the format forbids —
> so the claim most likely predates ADR-0008 or was measured against the file this
> amendment block already describes as not in this repository. The overflow check
> itself is unaffected: it was already resolved on independent grounds by ADR-0007's
> width term and ADR-0011/ADR-0012's aperture and stroke terms. Re-executable check:
> `docs/research/juries/contain-slack/contain_slack_scan.py` on branch
> `domain/contain-slack`.
>
> **Also amended by these, not summarised above** —
> `docs/adr/README.md` carries the full *Amended by* view:
>
> - [ADR-0013](0013-fitted-extents-floor-and-the-nine-origin-keywords.md) — gains the 
>   aperture-coverage error, the fit-deviation note, and the UNCHECKED category
> - [ADR-0019](0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md) — two new 
>   validate checks
> - [ADR-0035](0035-keyframe-grid-alignment-is-a-review-check-not-a-schema-rule.md) — 
>   adds one check to the list
> - [ADR-0036](0036-shift-preambles-coincident-instants-validate-and-compare-stay-out.md) 
>   — declines a coincidence census, and states why
> - [ADR-0039](0039-group-keyframe-time-check-retracted-from-validate-reassigned-to-compare.md) 
>   — retracts a commissioned check that violates the noise-budget principle
> - [ADR-0041](0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md) 
>   — `validate` gains a fourth report category, `LAYOUT`, alongside 
>   `error`/`review`/`note`/`UNCHECKED`
> - [ADR-0051](0051-word-alignment-is-external-validate-and-compare-catch-drift.md)
> - [ADR-0058](0058-text-box-slack-is-a-note-with-sibling-census.md) — adds `R-BOX-SLACK` 
>   to the check list
> - [ADR-0060](0060-layer-tie-is-an-error-array-order-stays-meaningless.md) — the 
>   geometry-aware same-layer check finally gets a home and a severity
> - [ADR-0043](0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md)
>   — report format: the `repair` field, uniform per check, non-bypassable
> - [ADR-0044](0044-off-canvas-is-a-standing-review-check-not-a-frame-change-census.md) —
>   adds `R-OFF-CANVAS` to the check list

`montagent validate` answers exactly one question: **is this project file internally
legal, and does it agree with the media on disk?** It never answers *"does this file
say what you meant it to say."* That boundary is printed in the report itself.

Its output is **prose by default, generated from a canonical JSON finding set**.
`--json` prints the JSON *instead of* the text, never alongside it.

It **always runs every check on the whole project.** There is no fast mode and no
way to narrow what is checked. Output may be filtered; analysis may not.

And because an opt-in check is worth little, **`render` runs the identical checks
unconditionally and refuses on `error`**, so a project cannot become a video
without passing.

## Why this ADR exists, and what it is answering

[ADR-0004](./0004-tracks-as-constrained-lanes.md) rejected a proposed `sequence`
label — checked only by `validate` — as **actively risky**, not merely redundant:

> an opt-in check they might not tag, on a command they might not run, would
> replace real vigilance with false confidence

Two of eight agents ranked that option *below doing nothing*. `validate` is an
opt-in check on a command you might not run. The first obligation of this ADR is
to say what saves it, and the honest answer is: **nothing, while it is a discrete
thing you invoke.** Fifteen agents were asked and not one constructed a defence.

> *"Nothing stops the argument. I would rather say that than construct a
> distinction."*

There is one real difference and it is a difference of degree: `sequence` needed a
conjunction — tag the track **and** run the command — and an untagged track was
indistinguishable from a decision not to check, so silence meant nothing.
`validate` needs one conjunct and checks uniformly, so a clean run is meaningful.
Halving a conjunction is an improvement, not a defence.

The defence has to be structural. ADR-0005 states the intent — *"`validate` is part
of the edit loop, not a final check"* — with no mechanism attached. Two mechanisms
are adopted here:

- **`render` runs the checks and refuses on `error`.** The checks are then not
  optional for the only operation that ships anything.
- **The MCP write tools return the findings as their result.** `shift` hands back
  the new state's findings, not an `ok`. *"Then I don't run `validate`; `validate`
  runs me."*

**What is not defensible, and is recorded rather than papered over:** an agent
editing the project with ordinary file tools — which is this project's own stated
authoring model — bypasses both mechanisms. For that path `validate` is exactly as
risky as `sequence`, and the renderer is the only true chokepoint.

## The noise budget is a safety property

The strongest finding in the exercise, and the one that shapes every rule below:

> *"One thing is worse than never running it: running it and getting 47 lines about
> frame alignment. `0 errors, 47 notes` reads as a pass. A noisy validator
> manufactures false confidence faster than an unrun one does, because the unrun one
> at least leaves you uneasy."*

This is ADR-0004's failure mode arriving by a different road, and it means report
volume is **not** a presentation preference. A literal reading of ADR-0005's check
list emits roughly 150 findings on the fixture, of which three matter.

Consequently: **errors and near-errors print in full; informational classes collapse
to one counted line** carrying their code, expandable on request. And the clean case
— by far the most common output in an edit loop — is **one line**.

## Severity is computed from the consequence at an instant

Not from the check, and not from the track. Both halves of that dichotomy are wrong,
and the fixture shows why: `photo` having an 800 ms hole would be harmless if
`caption` covered it. What made 30603–31403 a near-error is that **nothing on any
visual track** covered it — a fact about the *union*, not about a track.

So: compute the union of visual coverage; a gap whose interval is uncovered in that
union is `review`, and every other visual gap is a note. On the fixture this reports
**one visual gap out of eleven**.

This also disposes of "a track declares no kind" **without adding a field.** Every
*element* declares `type`, and `type` already partitions into rendered and audible;
a track's kind is the kind of its elements — derived, and therefore impossible to
forget, mistype or leave stale. A `kind` field on a track would be a second fact
that drifts from the first, which is the `sequence` label wearing a different hat,
and ADR-0005's stale-half objection besides.

The dependency is deliberate and is named: this makes severity **rest on the
cross-track coverage question**, which is
[#23](https://github.com/MBehtemam/Montagent/issues/23) and is not settled here.

**Three levels**, named for what the reader does, not for how bad it is:

| level | meaning |
| --- | --- |
| `error` | the render is refused or is guaranteed wrong |
| `review` | legal, renders, and you must look at a frame to know if it was meant |
| `note` | a fact you may want and will not act on today |

Two levels is not enough — collapsing `review` into `note` puts an 800 ms hole in
the same bucket as 47 alignment lines. Four is too many: a distinction the reader
cannot reliably make is one they start ignoring.

**The level is never the finding.** `review: gap` is useless. *"No element on any
visual track for 800 ms; the frame is background plus the header"* is the finding,
and the level is a sort key.

## Findings state facts, never repairs

A finding **may state anything derivable from the project, the media on disk, and
the published rendering semantics.** It **may not state anything that requires
knowing what the video is for.**

Facts, all of which belong:

- *"the file is 2568 ms"* — measured.
- *"3368 / 0.645 = 5222; the element has 3981"* — arithmetic.
- *"the other four elements of this track are at y = 1597"* — a census of the document.
- *"nothing is on any visual track for 800 ms"* — the document evaluated at an instant.

Verdicts, none of which do:

- *"the sentence should be at 1597"* — requires knowing a restyle was intended.
- *"close the 800 ms gap"* — the gap might be a deliberate beat; ADR-0005 is
  emphatic that gaps are content.
- anything auto-fixed.

**The sibling census is the move that makes a finding actionable without deciding
anything**, and it is routinely mistaken for a verdict. *Four of five are 1597, one
is 1537* is inert data about the file: it carries the fix without proposing it — the
tool supplies the distribution, the agent supplies the intent. This generalises
ADR-0005's *"12.0s is 1.5s into line-03.mp3; nearest boundaries are 10.1 and 13.5"*
from the time axis to every axis.

**Enumerating the nearest legal alternatives is still fact**, and is wanted: *"`source_end`
2568 would make the element 2568 ms; at speed 0.645 that is 3981 ms of timeline,
which is what the element already has."* Four facts in a row, very nearly a patch,
and still not a decision — the media might instead need re-recording, and the tool
cannot know.

This rule was **settled by demonstration, not argument.** Two agents independently
derived the same false conclusion from the same evidence — *"the restyle was not
done"* — when the restyle had in fact been applied correctly. One published it as
nine `error` lines above the two real defects, and its ideal validator would have
told it to revert a correct edit. The other reached the identical inference, named
its own method — *"that is not verification, it is archaeology"* — and kept it out
of the report on precisely this rule. Three other agents found the **one** element
the restyle did miss, by census.

## The report's own boundary is printed in the report

The single most valuable output of the exercise came from an agent briefed to kill
the feature:

> *"The validator I designed would print zero errors on a file that fails two of its
> session's three tasks."*

A file can be perfectly legal and still have half a requested change missing, with
no trace in the document. So the report ends with its own scope, unconditionally:

```
NOT CHECKED
  This file was not compared against any prior version or instruction. validate
  verifies that the file is internally legal; it cannot tell you whether it says
  what you meant it to say.
```

Without it, a clean run is read as *"the file is right"* — and *"run this and the
file is fine"* is the `sequence` label again, wearing a `validate` label instead.

The organising principle, reached independently from the other side:

> *"Every wrong number in this file is mutually consistent. It is a careful file
> built on one false premise — that the audio got 800 ms longer — which the disk
> contradicts and the document cannot. Every check that compares this document to
> itself, it passes. Only the checks that compare it to something outside it — the
> media, the previous version, a rendered frame — find anything."*

## Wire format

**JSON is canonical; text is generated from it; text prints by default; `--json`
prints JSON *instead*, never both in one invocation.** (4 of 5, including the agent
briefed to dissent from the majority.)

ADR-0005's stale-half objection was raised against this and **does not transfer.**
That objection is about a *persisted, hand-editable* file where two fields desync
across edits. Validator output is derived fresh from one in-memory finding set on
every run and is never edited by anyone, so the two projections cannot diverge. What
*is* real is the token cost of printing both, and that is solved by never doing it.

The reason JSON is canonical rather than a projection is what it forces on the
finding's **design**: ADR-0005's exemplar sentence is valuable for the facts in it —
the offset into the source, the two nearest boundaries — not for its grammar, and
every one of those is a named field. Requiring each code to declare its fields and
render through a template is what stops the tool editorialising without giving up
prose. The inverse arrangement makes the JSON the half that rots.

Prose is the denser encoding of an *explanation*; JSON is the denser encoding of a
*list you want to filter*. This report is explanation — which is why text is the
default even though JSON is canonical.

Two things the prose must borrow from the JSON:

- **Stable finding codes** (`E-SOURCE-OVERRUN`, `N-QUANTIZATION`), 4–6 tokens each.
  They buy three things: they are the contract that keeps the two renderings in step
  (one code, one field set, one template); they let a reader suppress or skim a whole
  class without re-reading prose to decide each line is the same noise as the last;
  and they give a finding an **identity to diff on**, which any future
  compare-two-versions capability depends on entirely — prose matching breaks on any
  rewording.
- **Every number inline** — especially the numbers that are *not in the file*: the
  probed duration, the delta, the neighbour's boundary. A finding that says *"see
  `vo-sentence-06-a`"* forces a re-read of the whole project to act on it, and **that
  re-read, not the validator's output, is the real context cost.**

## Cost: always everything, and the cache is the mechanism

**No fast mode. No `--no-probe`. No scoping of what is checked.** Unanimous, 5 of 5.

Probing every media file in the fixture costs **~0.04 s** of work. There is no
latency to trade away, and the moment a fast path exists it becomes the mode used in
the edit loop — so the single highest-value check in the tool surface is the one that
gets skipped. That is this ADR's own opening argument with extra steps.

**Scoping the probe to "the sources this write touched" fails on the defect class it
is meant to survive.** In the fixture the truth changed **on disk, not in the
project**, so a touched-sources scope would never re-probe it. Worse, the scope is
unsound in principle: in that file **every media source is referenced by 2–4
elements** — one audio file by three, forty seconds apart on the timeline, and one
image by four. "I already checked that file" is wrong the moment you look elsewhere.

**Cache on `(path, size, mtime)` → duration.** A re-validate after an edit that
touched no source then does zero I/O, so *"cheap enough to run after every write"* is
satisfied by a mechanism rather than a policy.

**Then report the cache miss, unprompted, at the top.** ADR-0005 says *"in a real
session nobody announces that a file got 0.8 s longer."* A mtime-keyed cache
announces it for free, as a side effect of being fast — and on the fixture that one
line **is** the entire defect.

**Scope the output, never the analysis.** Filtering what prints (`--group item-06`)
is useful mid-edit and safe: a checked-but-unprinted finding still exists; an
unchecked one silently does not. Every scoping of the *analysis* is a way of
reintroducing, under a new name, the cross-track blindness ADR-0005 names as the
worst bug its exercise produced. A filtered run must still let its exit code and its
one-line summary reflect the whole project.

Correction to how this was framed: **seconds were the wrong resource to measure.**
*"A check that is free to run and expensive to report is still expensive."* The
scarce thing is the agent's context.

## Frame alignment: check quantization damage, not alignment

ADR-0005 asks `validate` to *"note boundaries that are not frame-aligned."* **That
instruction is wrong as literally written, and the fixture is the proof:** 109 of its
120 time values — 47 of 48 distinct instants — are off the 40 ms grid at the
project's own 25 fps, the only aligned instant being `0`. A check with a 98% hit rate
on a correct, published project is not a check; it is the alarm fatigue this ADR
already identified as a safety problem.

Restate the requirement as **"report what quantization changes"**, and alignment
becomes a derived detail behind `--verbose`. On the fixture the honest answer is one
line — *nothing changes* — supported by three derived facts:

1. **No element is shorter than a frame** (the shortest is 1000 ms), so nothing can
   round out of existence.
2. **Every cut is an exact adjacency**, so both sides quantize to the same frame.
   Rounding can never manufacture an overlap or a gap — the only route by which
   misalignment becomes an *error*.
3. **Twelve instants are a picture cut and a speech boundary at the same number.**
   Audio is not quantized to the video grid, so at these the picture lands up to
   18 ms from the sound it was cut to. This is the only place in the project where
   the sub-frame times are observable.

Escalate to `review` only for the cases in (1) and (2), which are empty here.

## Checks that must not fire

- **Keyframes outside their element's range are legal** — a trimmed move — and must
  not be flagged. Seven elements in the fixture carry a scale keyframe past their own
  end; one is 13.8 s past the end of the project. See
  [#21](https://github.com/MBehtemam/Montagent/issues/21).
- **Gaps are never errors.** ADR-0005, restated because the severity rule above could
  be misread as overturning it. A visual gap uncovered by the union is `review`; it is
  still not an error.
- **Two tracks sharing a `layer` is not by itself a defect.** Five tracks in the
  fixture share two layer values and it is benign, because their boxes do not
  intersect. A naive same-layer check is a false positive on a legal file, and a
  geometry-aware one is a much larger check than it appears. That is
  [#24](https://github.com/MBehtemam/Montagent/issues/24), not this ADR — and #24
  turned out to be the anchor repair, not this check; it was finally given a home,
  as `error` rather than a report, by [ADR-0060](./0060-layer-tie-is-an-error-array-order-stays-meaningless.md).

## Two checks this exercise found that nothing had named

- **Animation discontinuity at a same-source cut.** Where two adjacent elements draw
  the **same source** and an animated property restarts, the image snaps mid-shot: in
  the fixture, scale `1.0161 → 1.0000` at 3018 and `1.0542 → 1.0000` at ~~64816~~
  **64016** (corrected by ADR-0033). The
  discrimination is load-bearing and is one field wide — the three other
  discontinuities in that track are cuts between *different* images, where resetting
  scale is correct. Depended on
  [#21](https://github.com/MBehtemam/Montagent/issues/21), now closed; fully specified
  by [ADR-0033](./0033-same-source-cut-continuity-is-a-review-check.md), which also
  draws the line against ADR-0012's separate group-keyframe-time check.
- **Text overflowing its own box, computed from the document.** ~~The fixture's
  `sentence-quiz` overhangs its card's top edge by ~6 px, derivable from `size`,
  `line_height` and a `y` that means centre.~~ **False against the committed
  fixture — see the amendment above.** *"Until `validate` does this, 'the text
  overflows its card' is a defect class that no check can see and that every restyle
  can produce."* Depends on the text model
  ([#13](https://github.com/MBehtemam/Montagent/issues/13),
  [#17](https://github.com/MBehtemam/Montagent/issues/17)) and on the map's open
  *literal size or fit rule* question.

  **Resolved by [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md), which
  this check was missing two things to be implementable at all.** First, an *input*:
  every term above is vertical and the fixture models the card as a **separate shape
  element**, so as written the check had nothing to measure against — ADR-0007 gives
  the text element a `box`. Second, a **width term**: with literal size and no
  auto-wrap settled, a hand-placed line that is simply too wide has no check
  anywhere, and four agents authoring against this format produced exactly that
  defect by three separate routes (lengthening a string, swapping the font,
  emphasising a word), each leaving every property individually valid. The height
  term also **generalises** rather than changing: `Σ over lines of (max run size on
  the line) × line_height`, which reproduces the ~1507–1567 figure above exactly in
  the single-size case.

## Consequences

- **`render` runs `validate` and refuses on `error`.** Not a flag.
- **MCP write tools return findings as their result**, so the tool-driven path never
  has an unchecked moment.
- **The check list is not closed by this ADR.** Per
  [ADR-0003](./0003-general-video-editor-not-channel-tooling.md), the fixture shows
  which checks *this* project needed and is never evidence that anything else is
  unneeded.
- **The probe cache is a gitignored sidecar**, consistent with
  [#4](https://github.com/MBehtemam/Montagent/issues/4)'s rule that probe results
  never live in the source of truth where they could go stale.
- **`validate` reads media but does not read git.** Comparing two versions is a
  different capability, folded into
  [#10](https://github.com/MBehtemam/Montagent/issues/10) — see below.
- **Severity depends on [#23](https://github.com/MBehtemam/Montagent/issues/23).**
  The union-of-visual-coverage computation is the same object as cross-track
  coverage; if #23 settles it differently, the severity rule follows it.

## What was ruled out of this ADR

**Comparing the project against a previous version.** Every agent asked for it, in
both of two exercises. It is **not `validate`** and it is not a new ticket: 4 of 5
placed it inside
[#10](https://github.com/MBehtemam/Montagent/issues/10), and the reason is that the
thing wanted is not a diff of *findings* but a comparison of *timelines* — kin to
`query` and `timeline`, which that ticket already owns. Designed separately, the
finding-identity contract gets designed twice and the two designs disagree.

`git diff` is **necessary and not sufficient**, and the fixture shows exactly why:
ADR-0005's one-element-per-line, sorted, stable-key-order convention makes the diff
unusually legible — one changed element is one changed line. But **the defect here is
an edit that did not happen, and an absent edit produces no hunk.** Nothing in the
diff marks the element the shift skipped. And had the audio genuinely been 800 ms
longer, both versions would validate clean and the skipped edit would appear in no
finding set on either side.

## The exercise

Ten agent sessions across four models (Opus, two Sonnets, Haiku, Fable), in isolated
sandboxes holding `CONTEXT.md`, the ADRs, the fixture media and one project file.

**Five were given a scenario rather than a questionnaire:** an editing session had
just finished on a real project file, they were the agent who had to decide whether
it was safe to render, and they were asked to produce *the verbatim text they wished
`montagent validate` had printed at them* — the literal output, not a description of
it — before answering any design question. The file was the genuinely defective one
an agent had shipped in
[#9](https://github.com/MBehtemam/Montagent/issues/9)'s exercise at *"very high"*
self-reported confidence. They were not told it was defective. One was briefed
hostile: to attack whether a validator should exist at all.

**Five then decided three deadlocked questions** with the options stated neutrally
and the author's preference removed; one was briefed to identify the likely
consensus and argue against it before committing. Results: **5–0** for no fast mode,
**4–1** for JSON-canonical/text-default, **4–1** for folding the version-comparison
capability into #10 — the last against the author's own recommendation. The sole
dissenter on both split votes was the model that had twice reported a correct edit as
broken.

**A setup flaw, recorded rather than glossed:** the brief stated that the audio file
had been re-recorded and replaced on disk. It had not been — the file was 2568 ms
throughout, and the project's declared 3368 was never true. This was inherited from
the previous exercise, where four agents converged on 3368 **on faith from the
brief** and not one probed the media. It turned the probe check from an argument into
a live demonstration by accident, not by design.

Every numeric claim repeated in this ADR was verified by script against the file
before being written down. Three judges returned three different track counts
(12, 14, 20) for a 154-line file, three different frame-alignment denominators, and
gap counts of 26 and 29 for the same document. The pattern from #9's jury holds:
**presence reads reliably; structure does not.**
