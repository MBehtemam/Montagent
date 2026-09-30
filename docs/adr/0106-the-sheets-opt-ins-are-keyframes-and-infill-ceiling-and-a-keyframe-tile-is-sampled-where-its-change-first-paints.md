---
status: accepted
amends: 0011 (the `frame` row's range arguments gain two opt-ins, `--keyframes` and `--infill-ceiling <MS>`, on both surfaces), 0094 (decision 3's default is confirmed and its "absence of evidence" caveat retired; its disclosed count is redefined as keyframe change points interior to a run on a visible element, reported as `tiled` and `untiled`; decision 5's "gap ceiling" is renamed the infill ceiling and specified as uniform, fitted into leftover slots, and never degrading the sheet), 0098 (section 8's identifying field is absent on keyframe and infill tiles, which are excluded from the sheet-wide elision fit), 0105 (`E-SHEET-OVERFLOW` gains a field saying the range fits without `--keyframes`; the `between-keyframes` sentence states where keyframe tiles are sampled)
---

# The sheet's opt-ins are `--keyframes` and `--infill-ceiling`, and a keyframe tile is sampled where its change first paints

> **Amended by [ADR-0128](0128-the-tile-label-is-fitted-at-one-size-per-sheet-and-names-the-topmost-entrant.md).** Decision 7's blank slot stands beside ADR-0098 §2's placeholder:
> `=` fills a run tile's slot where nothing nameable changed, and a keyframe or infill tile's
> label ends after its offset.

> **Amended by [ADR-0129](0129-an-untiled-keyframe-point-is-named-in-the-census-and-never-in-skipped.md).** Decision 8's `untiled` is a count whose members are named in
> `untiled_points`, never in `skipped[]`; a point with no tile only because the flag was not passed
> is `not-requested`; `volume` is not in the population. Decision 9's fields are `keyframe_tiles`,
> `fits_without_keyframes` and `keyframe_tiles_admitted`.

[#418](https://github.com/MBehtemam/Montagent/issues/418), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Put to **two juries** of three models —
Opus 5.5, Sonnet 5.5 and Fable 5.1 — on the same day. The first took ten questions about the two
flags and was unanimous on eight, with a three-way split on the infill flag's spelling. The second
took four questions about where a keyframe tile is sampled, which the first round had exposed
but not asked. It was unanimous on three and split 2–1 on the fourth. The human took the judge's
read on every split. Ballots are recorded verbatim in
[`docs/research/juries/contact-sheet-flag-names/`](../research/juries/contact-sheet-flag-names/README.md)
and
[`docs/research/juries/contact-sheet-keyframe-instant/`](../research/juries/contact-sheet-keyframe-instant/README.md).

The evidence this spends is [#407](https://github.com/MBehtemam/Montagent/issues/407)'s,
re-derived by
[`docs/research/keyframe-tiles/classify_keyframes.py`](../research/keyframe-tiles/classify_keyframes.py),
which passes as committed: the fixture declares 14 keyframe change points, 0 of them interior to a
run on a visible element, and 30 tiles is the last count the 140 px floor admits.

## The question

ADR-0094 mandated two flags on `frame`'s range mode and named neither. ADR-0097 settled every
*existing* flag's behaviour under a range and left these two alone on purpose: *"this ADR
deliberately does not invent them."* The ticket asked four things:

- what each flag is called;
- what each takes;
- whether either composes with the refusal;
- whether both carry over to both surfaces.

It was ordered after #407 because that measurement *"may flip this default"*.

It did not flip it. It did expose a question none of the earlier ADRs had asked: ADR-0094 §2
fixed which frame a *run* is sampled at, and nothing fixed which frame a *keyframe* is sampled at.

## Decision

1. **Keyframe tiling stays off by default, and ADR-0094's "decided on an absence of evidence"
   caveat is retired.** The evidence now exists, and it points the same way the jury did.
   **A width-conditional default is rejected**: on while the sheet clears 140 px, off otherwise.
   *(Round 1 Q1, 3–0.)*
2. **The keyframe opt-in is `--keyframes`, a bare boolean.** It is `keyframes` on MCP.
   *(Round 1 Q2, 3–0.)*
3. **The infill opt-in is `--infill-ceiling <MS>`.** Its value is a ceiling in milliseconds, and the
   unit lives in `value_name` as it does for `--at` and `--delta`. It is `infill_ceiling` on MCP.
   *(Round 1 Q3, split three ways; the judge's read.)*
4. **The quantity is the infill ceiling**, and "gap ceiling" is retired. It is the longest span, in
   painted time, the sheet may leave between two consecutive tiles **of any class**. Infill is what
   closes the span; the span is measured across every tile. *(Round 1 Q4, 2–1; the judge's read.)*
5. **The document-derived tiles fix the rung, and infill fills only the slots left over.**
   Document-derived means run tiles, plus keyframe tiles when asked for. Requesting infill can
   never degrade or refuse a sheet. *(Round 1 Q5, 3–0.)*
6. **A partly-honourable infill ceiling is honoured uniformly at the smallest ceiling that fits.**
   The answer says `requested X ms, achieved Y ms`, or `achieved: none` when no slot is left. The
   tiles the requested ceiling would have added are listed in `skipped[]` as `infill-evicted`,
   grouped into runs. A ceiling longer than the range is a legal request that adds no tile.
   *(Round 1 Q6, 3–0.)*
7. **A tile with no boundary carries no identifying field.** Keyframe and infill tiles have no
   ADR-0098 §8 change to name. Their class mark and sigil say what they are, their provenance line
   names the change point, and they are **excluded from the sheet-wide elision fit**. Where the field
   would print, the label leaves a blank, never a placeholder that could read as an id.
   *(Round 1 Q7, 3–0.)*
8. **The disclosed keyframe count covers keyframe change points interior to a run, on an element
   visible at that change point**, and is reported as **two numbers, `tiled` and `untiled`, on
   every answer**, whether or not the flag is passed.
   - **Membership is a raw-millisecond test anyone can read off the document:** `s < t < e`, on a
     visible element.
   - **The split is grid-aware:** a change point is `tiled` exactly when some tile of *this* sheet
     sits at its sample frame (decision 10).

   *(Round 1 Q8, 2–1 on "always"; round 2 Q2, the judge's read of a split in how the jurors said it.)*
9. **When `--keyframes` is what pushed a range past the floor, `E-SHEET-OVERFLOW` says so.** A
   finding field records that the range fits without the flag, and how many keyframe tiles the
   floor would admit. The template renders it as prose. Keyframe tiles are never dropped to make a
   sheet fit. *(Round 1 Q9, 3–0.)*
10. **A keyframe change point at `t` is sampled at the first frame the grid paints at or after
    `t`:** the least `n` with `⌊n·1000/fps⌋ ≥ t`. This is ADR-0094 §2's rule for a run, applied to
    the other kind of boundary the document states. The label prints the **painted** millisecond,
    and the provenance line prints the change point as `element.property@t`. So a keyframe at 1013
    ms on a 25 fps grid is labelled `1040ms` and listed as `photo-05.scale@1013`, and `frame --at
    1040` reproduces the tile exactly. *(Round 2 Q1, 3–0.)*
11. **One tile per distinct painted frame.** Change points that share a sample frame share a tile,
    and its provenance line lists them all. **A change point whose sample frame is a run tile's
    frame adds no tile:** the run tile keeps its class (unmarked), its boundary field and its label,
    and the change point joins its provenance line as `tiled`. *(Round 2 Q2, 3–0.)*
12. **A change point with no painted frame left in its own run is sampled by the rule as written.**
    That frame is the next run's tile frame, because both are the least painted frame at or after a
    millisecond with no painted frame between them. If the element is visible there, the change
    point is `tiled` by that tile. If it is not visible there, or the next run lies outside the
    range, it is `untiled` with reason `no-grid-frame`. That is **disclosure only, never a finding.**
    *(Round 2 Q3, 2–1; the judge's read.)*
13. **Keyframe tiles sample change points only, never curve-derived instants.** ADR-0094's 3–0
    refusal of easing midpoints extends to extrema: a `back` ease overshooting between two correct
    endpoints stays behind the `between-keyframes` blind spot. A peak tile, if one is ever wanted,
    would be a fourth opt-in class, never a reinterpretation of `keyframe`. *(Round 2 Q4, 3–0.)*
14. **Both flags are on both surfaces with the same meaning.** Either one without `--from`/`--to`
    is bare `E-INVOCATION`, the way `--full` and `--crop` with a range already are the other way
    round. An infill ceiling of zero or less, or below one frame period, or not a whole number of
    milliseconds, is refused as `E-INVOCATION` rather than clamped. *(Round 1 Q10, 3–0.)*

## Why

### 1. The default: #407 answered the caveat, and the knob's third shape fails ADR-0103

ADR-0094 §3 ran the jury's one split in the dissenter's favour on the ground that the disclosure
removes the silence the majority feared. That left only a measured cost against a benefit nobody
had measured. #407 measured both.

- **On the repo's one real project the flag adds zero tiles.** Its 14 keyframe change points
  are 7 on run boundaries already tiled and 7 outside the lifetime of the element declaring
  them.
- **On a constructed population the flag catches two large-area geometric defects**, and one of
  them is a class `validate` declines to check by ADR-0044's design.
- **Its thirteenth tile on an 18-state document turns a working call into a refusal.**

That is an argument for the flag existing and against its being the default, and the caveat is
retired because it is no longer true.

The **width-conditional default** was #407's own idea, and it was a new shape rather than a choice
between the two the jury debated, so it had to be refused on its own terms. All three jurors
refused it on the same ground: ADR-0103's.

- A blind spot is safe to disclose when it is **constant and learnable**.
- A default that includes keyframes while the sheet clears 140 px, and drops them otherwise, is
  blind to keyframed interiors on a 20-state range and sighted on a 12-state range of the same
  document.
- So narrowing `--from`/`--to` would *add* tiles the wider call did not show.

Juror 3's line: *"the same document is blind on a 20-state range and sighted on a 12-state one.
That is exactly the per-call-varying blindness ADR-0103 refuses."*

### 2. `--keyframes`: a class, not a rule

ADR-0097 §1 recorded all three jurors rejecting `--per-state` and `--each-cut` because those name a
selection rule, and ADR-0094 had made the run-start rule the only one. The same trap was the one
this name had to avoid. `--keyframes` avoids it because it names a **population the sheet adds**,
and it is literally ADR-0098 §6's class token. It follows the CLI's bare-word booleans (`--full`,
`--all`, `--census`), and the CLI has no `--with-*` or `--include-*` form anywhere.

It takes no value because there is nothing to size. Tile count is derived and the budget is width
(ADR-0095).

### 3. `--infill-ceiling`: the name must not read as a count, and the ticket said so

This was the one real split: `--infill`, `--infill-ceiling` and `--max-gap` each got one vote.

- **`--max-gap`** falls to the glossary. **Gap** already means *"a stretch of a track with no
  element in it"*, and Juror 3 named the misreading exactly: *"the longest track gap to
  tolerate."* Juror 2, who chose it, conceded the collision and proposed a glossary entry to
  disambiguate it. The judge's read was that a flag whose name needs a glossary entry to stop it
  meaning something else in the same tool is the wrong trade.
- **`--infill`** had the strongest consistency argument. It is one word across flag, class token
  and skip reason (`infill-evicted`), and it is the only candidate that adds no hyphenated flag to
  `frame`. Juror 3 conceded the cost: *"bare class-token names put the 'ceiling, not a count'
  semantics entirely in help text."* The ticket's own constraint was that **the name** must not
  read as a count, *"which is the mistake `--n` made"*, and `--infill 5` reads as five tiles.
- **`--infill-ceiling`** meets that constraint without relying on help text, and costs length.

The repo already has hyphenated flags (`--no-clobber`, `--no-probe`, `--enable-gpl`), though none
on `frame` until now.

`--infill-every` was refused by all three for a reason worth keeping: a ceiling is **not a
period**. Infill is inserted only where the tiles the document produced sit further apart than
the ceiling, so a busy range gains none and an empty one many. That is ADR-0094 §5's own argument
against a count, and *"every"* would rebuild the uniform cadence a count implies.

### 4. "Infill ceiling", measured across every tile

Juror 2's dissent on the term is answered in the definition rather than the name. It argued that
*"coining 'infill ceiling' ties the concept to one tile class, but the ceiling is defined over
consecutive tiles of any class"*. That is true, and the glossary entry says the span is measured
across tiles **of any class** while infill is what closes it. The alternative, keeping "gap
ceiling", would put **Gap** into the glossary in a second sense inside the same tool.

### 5–6. Infill can never cost the document's tiles, and its promise is uniform

ADR-0094 §5 made infill *"strictly additive, can never displace a document-derived tile, first
evicted under budget."* ADR-0095 then made width the only currency. Put together, letting infill
trigger the degrade step would take 40 px from every document-derived tile, which is displacement
in the only unit the budget counts. Juror 1: *"displacement by resolution."* So the rung is fixed
first, without infill.

**The consequence is stated rather than discovered later.** An 18-state whole-document call on the
fixture fills all 18 slots at 180 px, so **`--infill-ceiling` adds nothing there**. Infill earns
tiles on narrower ranges. All three jurors accepted this cost; Juror 3 was the one who spelled it
out.

When only part of a request fits, there were three options:

- Keep a subset of the infill tiles. The ceiling then holds in some stretches and not others:
  **a blindness that varies within one answer**, the worst case of ADR-0103's principle.
- Drop everything, which throws away a fit that was available.
- Refuse, which contradicts *"first evicted"* and ADR-0105 §4's reading of eviction as *"the
  instrument's own choice under budget, like the rung."*

The smallest uniform ceiling that fits keeps a single statable promise, `achieved Y ms`
everywhere, with the difference disclosed. All three chose it.

### 7. A field that is not there has no length

#407 found that a keyframe tile has no ADR-0098 §8 identifying field, because it has no boundary
change. The prototype's `element.property` stand-in ran long (`photo-05-loop.scale`, 19
characters against a boundary field's 15). Because ADR-0098 §5 makes elision **sheet-wide**, one
long keyframe id stripped the identifying field from **every** tile.

So the least important tile on the sheet degraded the most important ones, which inverts the
reason elision was made sheet-wide in the first place. An absent field has no length and cannot
drive the fit. The tile's class is still asserted twice (mark and sigil), and the provenance line
is where naming happens: *"the label is a pointer, the list is the census."* Juror 2 added the
blank-not-placeholder rule, so an empty slot can never be misread as an id.

### 8, 10–12. Where a keyframe tile is sampled, and what the counts claim

Round 1 exposed this and did not ask it. Juror 1 noticed that `untiled` can be non-zero even with
the flag on, when two change points share a painted frame or one falls between grid frames. So
the sample instant had to be fixed before the counts could mean anything, and it went to a second
panel.

**The first painted frame at or after `t`**, unanimously, for the reason that decides it.

- A keyframe is a boundary between two easing segments exactly as `s` is a boundary between two
  visual states. The frames before `t` are governed by the segment still approaching `v`, and the
  first frame at or after `t` is the first frame governed by the segment that starts at `v`.
- **For a `step` ease it is the only frame that shows the change at all.** Under "last at or
  before", the flag would tile the instant and show the old value.
- "Nearest" mixes both behaviours in a way that depends on where `t` falls against the grid.
- The cost is bounded and always runs in one direction: the tile shows the value up to one frame
  period past `t`. The `between-keyframes` sentence now says so.

**Coincidence falls out of that rule.** A change point a few milliseconds after `s` samples at the
run tile's own frame, and all three jurors kept that tile a run tile. The class records *why the
tile exists*, and a run tile exists without any flag. Marking it `keyframe` would make the same
pixels change class depending on a flag.

**The counts are grid-aware, and the population is not.** The two remaining jurors on this point
put the line in different places.

- Juror 1 would drop from the population any change point that lands on a run tile's frame.
- Juror 3 keeps membership a raw-millisecond test and makes only the `tiled`/`untiled` split a
  claim about the sheet.

The judge took Juror 3's version. The population stays derivable from the document alone, the
counts become statements about what the sheet shows, and reporting a change point `untiled` when
the run tile *is* its sample frame would be a false count. The cost, as Juror 3 named it: `tiled`
can be non-zero **without** the flag, and *"the 'zero is asserted without the flag' expectation
becomes 'the census is asserted'."* On the fixture both versions print `0 tiled, 0 untiled`.

**Decision 12 is the rule applied across the boundary, not a special case**, which is why the 2–1
went the way it did. If no painted frame lies in `[t, e)`, the least painted frame at or after `t`
**is** the next run's tile frame. So sampling there adds no tile and invents no instant.

Juror 2's case for "untiled with a reason" rested on *"the video also never shows it"*. That is
false whenever the element is still visible in the next run, because the frame right after `t` is
on screen and is that run's tile. It is true only when the element is not visible there, and
decision 12's fallback says `untiled` for exactly that case.

There is no finding, because an unpainted **run** is a declared visual state the video never shows.
An off-grid keyframe in an element's last partial frame period suffers only the ordinary
quantization every off-grid keyframe suffers, and the trimmed-move precedent already says an unseen
keyframe is not a finding.

### 9. The refusal must point at the cheaper remedy

`E-SHEET-OVERFLOW`'s sub-ranges point a caller at a narrower range. When the flag caused the
overflow, the cheaper remedy is to drop the flag, and nothing in the existing fields tells those two
cases apart. Juror 3: *"the alternative is a correct refusal that steers callers to the wrong fix."*
Degrading by dropping keyframe tiles was refused by all three as ADR-0095's forbidden thinning by
another name.

### 13. Change points only

Both of #407's catches sat at change points. The overshoot peaked at a keyframe because the author
*stated* 1.85 there, not because a curve overshot. A curve-derived extremum is an instant computed
from the easing function rather than stated by the document, which is exactly what ADR-0094
refused 3–0. #407's measurement strengthens that: *"the tile sees amplitude, not shape"*, and a
standard ease's overshoot is a few percent, under the width floor anyway.

### 14. Surfaces and values

ADR-0097 put the range on both surfaces, and nothing here is surface-specific.

A ceiling below one frame period asks for a spacing the grid cannot paint, so it is a malformed
request rather than a tight one. Clamping it would answer a question the caller did not ask.

## Honest costs

- **`--infill-ceiling` rests on one juror's vote**, taken as the judge's read of a three-way split.
  The consistency argument for `--infill` was the stronger *aesthetic* one, and it lost only to the
  ticket's explicit constraint.
- **Infill is a no-op on a whole-document call on the repo's one fixture.** That is correct, but it
  means the flag's only exercised path on committed data is the null one.
- **A keyframe tile shows the value up to one frame period past `t`,** not `v` itself. A fast
  segment after a keyframe can look slightly advanced.
- **A change point absorbed into a run tile appears only on the provenance line,** not in the
  label or the mark.
- **Decision 12 makes a run tile's census depend on keyframes from the previous run**, which Juror 3
  called *"a small conceptual leak across the boundary."*
- **Neither panel was shown a keyframe population at a real frame rate other than 25 fps,** and
  every worked example here is on that grid.
- **Three models from one family, again.** The unanimous questions were the ones where the brief
  leaned on a precedent (ADR-0094 §2's run rule, ADR-0103's constant-blindness test), and unanimity
  there is weak evidence of anything beyond the precedent being clear.

## Consequences

- `frame`'s range mode takes `--keyframes` and `--infill-ceiling <MS>` on the CLI, and
  `keyframes: bool` and `infill_ceiling: integer` on MCP. The `frame` row of ADR-0011's verb table
  gains both. The verb count is unchanged: these are arguments, not a verb.
- ADR-0094 §3's disclosure field becomes `keyframes: { tiled, untiled }` over the interior-on-visible
  population, on every answer. Untiled entries carry a reason (`no-grid-frame` is the only one this
  ADR defines).
- The disclosure gains `infill: { requested_ms, achieved_ms }` when `--infill-ceiling` is passed,
  with `achieved_ms: null` rendered as `achieved: none`.
- `E-SHEET-OVERFLOW`'s fields gain `fits_without_keyframes` (bool) and `keyframe_tiles_admitted`,
  rendered as a template clause present only when `--keyframes` was passed. The registry template
  changes when the range mode is built, not here.
- ADR-0105's `between-keyframes` sentence becomes: *"Keyframed values are shown only where a tile
  falls; keyframe change points are untiled unless asked for, and a keyframe tile shows the first
  painted frame at or after the change, so a wrong easing curve shows only if its endpoints are
  wrong."*
- The glossary gains **Infill ceiling** and **Keyframe change point**, and retires "gap ceiling".
- The map's fog patch on a width-conditional default is closed by decision 1.

## Not decided here

- **The implementation.** This ADR decides observable behaviour (ADR-0031); the range mode itself is
  unbuilt.
- **A constructed fixture exercising decisions 11 and 12.** #407's `doctored/` project places every
  keyframe on the grid, so coincidence and the cross-boundary case have no committed instance.
  Following ADR-0105's precedent, it is a shipping condition for the implementation, not for this
  ADR.
- **Whether infill tiles need an identifying field of their own.** Decision 7 gives them none, for
  the same reason as keyframe tiles, and #422's cold-observer check is the place that would show a
  cost.
