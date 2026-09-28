---
status: accepted
amends: 0074 (names `frame`'s range mode as the caller its `type` filter anticipated, specifies the re-merge that filter implies, and turns the audio-only boundary loss it measured into a mandatory disclosure on the sheet)
---

# The sheet's instants are the document's visual states, sampled at the first frame the grid paints

> **Amended by [ADR-0097](0097-the-range-is-from-to-on-both-surfaces-and-the-caption-becomes-an-attribution-obligation.md).**
> Decision 6's *"unconditional structured disclosure... plus one sentence of prose restating
> it"* is specified: **the plain-text form carries the disclosure's full content**, not a prose
> summary of it, because the no-`--json` path is the one an agent reaching for pixels actually
> takes and hiding the disclosure there would make the default answer the untrustworthy one.
> `--json` changes the disclosure's form, never its presence. The **per-tile provenance** this
> decision mandates is also named as what discharges ADR-0011's amended attribution
> obligation, jointly with the fitted label — because a label fitted to a 140 px tile cannot
> name the presence set.
>
> **Also amended by [ADR-0098](0098-the-tile-label-is-a-floored-fitted-line-naming-the-change-at-its-own-boundary.md).** Three of this ADR's clauses are specified rather
> than changed. Decision 2's *"the label prints both the sampled instant and the run's
> boundary"* is satisfied by a **signed offset** from the instant, not a second absolute
> millisecond — the two always differ by less than one frame period (37 ms against a 40 ms
> frame on the fixture), so the offset is lossless at three characters instead of seven, and
> `+0` prints so an on-grid boundary is an assertion rather than an absence. Decision 5's
> *"labelled in a different register"* is a **visual mark on the sheet plus an unabbreviated
> class token on the provenance line**, with document-derived tiles unmarked **and the zero
> counts asserted**. Decision 6's per-tile provenance **repeats its full presence set on
> every line** — no delta — because a delta would force the reader to reconstruct state by
> accumulation, which is where misattribution enters. This ADR's open question *"what the
> label carries"* is closed there.

[#398](https://github.com/MBehtemam/Montagent/issues/398), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Resolved by a jury of three
independent models (Opus 5, Sonnet 5, Fable 5.1) put to six sub-questions; ballots verbatim
in [`docs/research/juries/contact-sheet-instant-selection/`](../research/juries/contact-sheet-instant-selection/README.md).
The legibility numbers this ADR spends come from
[`docs/research/contact-sheet-legibility/`](../research/contact-sheet-legibility/FINDINGS.md)
([#396](https://github.com/MBehtemam/Montagent/issues/396)), whose geometry claims carry a
re-executable check.

This ADR fixes **which instants the sheet shows and what it must say about the ones it does
not**. It does not fix the flag spelling or the verb table — that is
[#401](https://github.com/MBehtemam/Montagent/issues/401)'s, and the amendment to ADR-0011
lands there.

## Decision

**A tile is one visual state, sampled at the first frame the project's grid actually paints
inside it.**

1. **The collapse is a `type` filter inside `frame`, followed by a re-merge.** `frame`'s
   range mode takes `query`'s cut list, filters members to visual types, then **merges
   adjacent intervals whose filtered presence set is equal**. No `--visual`, no second
   `query` mode, no new term. Equality is on the *set*, not on "has any visual element".
2. **The instant is the least frame index `n` whose painted millisecond
   `⌊n × 1000 / fps⌋` (ADR-0077) falls inside the run.** Not the midpoint, not the boundary
   millisecond. The label prints **both** the sampled instant and the run's boundary, which
   are different numbers whenever the boundary is off-grid.
3. **Keyframes are excluded by default and available behind a flag.** The count of untiled
   keyframe change points is named in every answer regardless.
4. **A run containing no painted frame gets no tile** and is named in `skipped` with reason
   `no-grid-frame`. Brevity alone never skips a run — only the absence of a paintable frame
   does.
5. **Uniform infill is a flag, never a default, and is a gap ceiling, not a count.** It is
   strictly additive, can never displace a document-derived tile, is the first class evicted
   under budget, and its tiles are labelled in a different register.
6. **Every answer carries an unconditional structured disclosure** — the rule, the per-tile
   provenance, what was skipped and why, the audio-only boundaries dropped, and a fixed
   `blind_to` enumeration — plus one sentence of prose restating it.

## Why

### 1. The filter is ADR-0074's own prescribed ritual, and the re-merge is the work

ADR-0074 rejected a `--visual` flag in terms that decide this: *"a caller that wants the
visual cut list filters one field of an answer it already has."* `frame`'s range mode **is**
that caller. Reaching for a new `query` mode here would reopen a question ADR-0074 closed on
a cost asymmetry that has not changed.

All three jurors reached this independently, and all three then noticed the same thing the
ADR did not have to say: dropping audio members makes adjacent intervals *identical*, so the
filter alone changes nothing. It is the **re-merge** that turns the fixture's 46 intervals
into ~18 visual states, and it is the hand-operation the trial agents complained about. The
filter is one line; the merge is the feature.

Equality is on the filtered set, per Juror 3's wrinkle: two adjacent intervals differing only
in *which* text card is up are two states, not one.

### 2. A midpoint is an instant no other verb computes

Unanimous, and on a ground the brief did not supply. The tile must be a frame the renderer
would actually emit, so an agent can re-call `frame --at <that ms>` and `query --at <that
ms>` and get the same pixels and the same stack. A midpoint is synthetic — nothing else in
the tool produces it, so nothing else can confirm it.

ADR-0035 supplies the other half: an off-grid boundary time is meaningful data but is *not a
renderable frame*. So the boundary and the sampled instant are two different numbers and the
label prints both. The tile answers *what is settled here*; the label preserves *what changed
here*.

The known cost, named by Juror 3: on a run that opens with a fade, the first painted frame is
the first frame of the fade, and the agent sees a near-empty tile that is not a defect. This
is accepted deliberately. The alternative is a "settled" heuristic, and choosing when a state
has settled is a verdict ADR-0006 keeps out of this layer.

### 3. The keyframe split, and why the disclosure decides it

This is the one question the jury split on: Jurors 1 and 2 put keyframe endpoints in the
default set; Juror 3 put them behind a flag.

Juror 1's argument is the serious one, and it is this map's own thesis: a rule watching only
presence goes **silent** across the fixture's zooming photos, and silence read as coverage is
the failure the whole effort exists to prevent.

**That objection is answered by the disclosure all three jurors independently demanded, not
by the default.** An answer that says *"14 keyframe change points on visible elements were not
tiled — pass the flag"* is not silent. It is a named, counted omission with a remedy attached.
The failure mode Juror 1 is defending against is an *absence*, and the fix for an absence is
to name it, which point 6 does unconditionally.

Once the silence is gone only the measured cost remains, and it is one-directional. #396
established that legibility is governed by **served tile width**, that fine detail dies at
~140 px, and that the working point is ~18 tiles at ~180 px. Keyframes roughly double the
fixture's tile count. Paying that on every call — to catch a defect class nobody has yet
shown exists, since a linear zoom produces nothing at its keyframe that is invisible at the
run's first frame — is the wrong default.

**This is decided on an absence of evidence and the ADR says so.** Nobody has measured whether
a keyframe tile catches anything a run-start tile misses. That is a cheap experiment of
exactly #396's shape, filed as [#407](https://github.com/MBehtemam/Montagent/issues/407), and
it may flip this default. The flag ships now; the default is provisional and labelled as such.

**Easing midpoints get nothing — 3–0.** An easing midpoint is derived from the curve, not
stated by the document. Inventing instants is the guessing this feature exists to replace.

### 4. A state the renderer never paints is a fact worth reporting

The fixture's 4 ms run at 56112–56116 contains no painted frame at 25 fps — frames land at
56080 and 56120. There is no frame to show, so there is no tile.

Jurors 1 and 3 are preferred over Juror 2's "merge it into the neighbour": merging quietly
asserts the state was shown, when the renderer never paints it. It enters `skipped` with its
bounds and reason instead. This is ADR-0074's *named, never silently dropped* applied to
instants rather than elements — and it is independently interesting, because a visual state
the rendered video never shows is exactly the structural consequence ADR-0035 says to report
as fact rather than judge.

### 5. Infill is a gap ceiling because a count is not proportional to time

All three confirmed infill as a flag. Juror 1's shape is adopted over Juror 2's `--n` and
Juror 3's per-run count, on his own argument: a count *"spends the same number of tiles on a
200 ms interval and a 20 s one."* A gap ceiling — *insert painted frames until no two
consecutive tiles are further apart than this* — expresses what the caller actually wants,
which is a bound on how long the sheet can be blind for, and it is aimed squarely at the
defect class infill exists for: change the document does not state, such as a source clip's
own cut.

Four constraints make it safe, and all four are part of this decision: off by default;
strictly additive, never displacing a document-derived tile; first evicted under budget; and
labelled in a different register. The last is not cosmetic. Juror 2 put it best — ship
uniform samples wearing the same label as document-derived ones and the trial's failure is
rebuilt *inside* the sheet.

### 6. The disclosure is unconditional, and ADR-0074 already proved why

Prose alone is the `preview` ladder's precedent (ADR-0021/0046/0065/0067/0078) and it is not
enough, because this reader is a machine that will summarise. So the answer carries a
structured field: the `rule` and its version; per-tile `instant_ms`, source run, `why`
(`boundary` | `keyframe` | `infill`) and the elements that produced it; `skipped[]` with
reasons; `coverage` including milliseconds not depicted; the **served tile width**; and a
fixed `blind_to` enumeration. One sentence of prose restates it, because Juror 3 is right
that the agent reads the narrative and the trial's failure was silence in the narrative.

Juror 1's unconditionality argument is adopted in full: **a caveat that appears only when
something went wrong teaches the reader that its absence is an all-clear.** That is the trial
failure rebuilt in our own tool, so `blind_to` is emitted on every answer, including perfect
ones.

**The strongest support for this is in ADR-0074 itself, and no juror was shown it.** That ADR
measured the fixture's boundaries — **47 with every element, 19 visual only, 28 reachable only
through audio** — and recorded exactly what the visual view does wrong:

> The visual cut list does not report a narration boundary as missing; it reports an interval
> it believes is constant, and is wrong about, with nothing in the output to suggest
> otherwise.

That is this sheet's own operation, and an accepted ADR has already called it a failure. The
sheet performs the narrowing ADR-0074 permits, so it inherits the obligation ADR-0074
implies: **the answer names the audio-only boundaries it dropped and their count.** Without
that, a sheet of 18 tiles over the fixture silently asserts constancy across 28 boundaries it
cannot see. This is the one disclosure clause that is not a design preference but a direct
consequence of an accepted decision — and it is why this ADR carries an `amends` on 0074.

Juror 3's line on codes is adopted: **no finding code for a blind spot** — a finding says
something about *this document*, and the blindness is a property of the *rule*. A skipped run
does earn one, because it is a fact about this document. Nothing here judges anything
(ADR-0006): `blind_to` describes the rule, `skipped` describes the document.

## What this ADR does not decide

- **Flag spelling and the verb table** — #401. This ADR fixes that a keyframe knob and a
  gap-ceiling knob must exist and what they mean, not what they are called.
- **The tile budget and overflow behaviour** — [#399](https://github.com/MBehtemam/Montagent/issues/399).
  Juror 1 proposed that an over-budget sheet degrade tile scale to the floor and then **refuse
  with a code naming sub-ranges that would fit**, rather than serve a thinned sheet that looks
  complete. That is the strongest idea in the panel and it belongs to #399; it is recorded
  there, not decided here. This ADR contributes one constraint: **silently shrinking the
  boundary set is the one thing overflow must never do.**
- **What the label carries** — [#400](https://github.com/MBehtemam/Montagent/issues/400).
  Noted there: the label is load-bearing for detection, not cosmetic. #396 found the invisible
  text legible only because the label says the card should carry text.
- **Per-tile crop** — [#406](https://github.com/MBehtemam/Montagent/issues/406), blocked by
  [#402](https://github.com/MBehtemam/Montagent/issues/402).

## Trade-offs, and what this rule is blind to

- **Inside a run.** A source clip's own cut, content motion in a still-looking element,
  a zoom ending on a badly cropped face. Only the gap ceiling reaches these, and only if the
  caller asks.
- **Between keyframes, and at them by default.** A bad easing curve shows only if its
  endpoints are wrong.
- **Below the served tile width.** #396 measured fine detail dying at ~140 px. The invisible
  text survived only because absence-of-text-on-a-card is a large-area cue; a *subtly*
  miscoloured caption would not.
- **Across sheets.** The wrong-photo defect was caught because both photos landed on one
  image. A range split across two sheets loses exactly that, and the sheet performs no
  comparison of its own (ADR-0006).
- **Audio entirely**, and motion, which stays with `preview` by design.
- **Complexity cost.** The re-merge is real logic in `frame` that must agree with `query`'s
  presence-set semantics or the two verbs will disagree about what a boundary is. That is the
  price of honouring ADR-0074 rather than bypassing it, and it needs a cross-verb test.
- **The residual risk, named by Juror 1 and not designed away.** An agent can still paraphrase
  a disclosed sheet as "I checked the whole range." The structured `coverage` and `blind_to`
  fields make that paraphrase contradicted by the answer it came from. That is the most a tool
  can do; it does not make it impossible.
