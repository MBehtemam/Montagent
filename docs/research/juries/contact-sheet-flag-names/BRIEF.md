You are a juror. Answer the ten questions below from the facts given. Do not use tools, do not edit anything, do not make recommendations to the person who convened you beyond your ballot. Reply ONLY with the ballot block specified at the end.

# Background

Montagent is a tool for agents that edit a declarative video project (a JSON file of tracks and timed elements: images, text, rects, audio). Its verbs include `validate` (reports facts about the document as findings; reaches no verdicts), `query` (a "cut list": the half-open intervals over which the set of present elements is constant), `render`, `preview`, and `frame` (rasterizes a still at one instant, `--at <MS>`). Every verb exists on two surfaces: a CLI and an MCP tool, with the same meaning on both.

`frame` is gaining a range mode (`--from <MS> --to <MS>`, both required, half-open; the pair *is* the mode) that returns ONE contact sheet: one tile per *visual state*. Already ratified:

- **Selection (ADR-0094).** Take `query`'s cut list over the range, drop audio elements, re-merge adjacent intervals whose visual presence sets are equal. Each resulting run is a visual state; its tile is sampled at the first frame the project's grid actually paints inside it. This is the only selection rule — an earlier `--visual` switch was rejected, and a later panel rejected the names `--per-state` / `--each-cut` because they name a selection rule, advertising a sibling rule that does not exist.
- **ADR-0094 §3: "Keyframes are excluded by default and available behind a flag. The count of untiled keyframe change points is named in every answer regardless."** It was the jury's one split (2 jurors for keyframes-in-default, 1 against) and was decided "on an absence of evidence", pending a measurement.
- **ADR-0094 §5: "Uniform infill is a flag, never a default, and is a gap ceiling, not a count. It is strictly additive, can never displace a document-derived tile, is the first class evicted under budget, and its tiles are labelled in a different register."** A gap ceiling means: insert painted frames until no two consecutive tiles are further apart than this many milliseconds. A count (`--n`) was rejected because it "spends the same number of tiles on a 200 ms interval and a 20 s one". Its target is change the document does not state, such as a source clip's own cut.
- **Budget (ADR-0095).** The budget is *served tile width*: a 180 px target, a 140 px floor, exactly one degrade step from target to floor, and past the floor the call **refuses** — it may never thin, split, or reshape the sheet. On a 9:16 frame, 180 px ⇒ 18 tiles and 140 px ⇒ 30 tiles. Tile count is derived, never set by the caller.
- **Refusal (ADR-0105).** Overflow refuses as `E-SHEET-OVERFLOW`, exit 3, whose finding fields name sub-ranges that would fit, the visual-state count, the tile count the floor admits, and a `limit` field saying which floor bound (140 px tile width, or an 8 px served type floor on labels). A keyframe-flagged call pushed past the floor is already ratified to be this same code. Flag-combination errors (e.g. `--at` with `--from/--to`) are bare `E-INVOCATION`.
- **Disclosure (ADR-0094 §6, ADR-0097, ADR-0105).** Every answer carries an unconditional structured disclosure (rule, per-tile provenance, `skipped[]` runs with a reason, dropped audio-only boundaries, the untiled keyframe count, served tile width and rung) and a fixed `blind_to` list — six tokens, one being `between-keyframes`: "Keyframed values are shown only where a tile falls, and keyframe instants are untiled unless asked for; a wrong easing curve shows only if its endpoints are wrong." The plain-text answer must carry all of it in prose. `skipped[]` reasons are `no-grid-frame` (a state the grid never paints, which also raises a finding) and `infill-evicted` (an infill tile dropped under budget — disclosure only, no finding, "the instrument's own choice under budget, like the rung").
- **Tile label (ADR-0098).** `<index> <sigil> <instant>ms <±offset> <±id>`, one line fitted to tile width, type floored at 8 px served. If the line will not fit, content is elided — **sheet-wide, never per-tile** (so the busiest tile is never the one silently thinned); the identifying field is the first thing dropped, and if even the numeric core does not fit the sheet refuses. §6: tile class is carried twice, as a visual mark on the sheet and as an unabbreviated class token (`keyframe`, `infill`) on the tile's provenance line; document-derived tiles are unmarked, and the answer asserts zero counts (`0 keyframe, 0 infill`). §8: "The identifying field names what changed at this run's own boundary", e.g. `+word-08-bridge` (entered) / `-photo-06` (departed). The provenance list names the full presence set per tile; "the label is a pointer, the list is the census".
- **Blind-spot principle (ADR-0103, refusing crop-with-a-range).** A rule's blindness is safe to disclose because it is *constant and learnable*; a blindness that varies per call is not.

## Existing CLI conventions

Flags in use: `--at`, `--from`, `--to`, `--where`, `--crop`, `--full`, `--png`, `--all`, `--census`, `--elements`, `--json`, `--verbose`, `--release`, `--delta`, `--scope`, `--no-clobber`. Booleans are bare words (`--full`, `--all`, `--census`); there is no `--with-*` or `--include-*` form anywhere. Units are never in a flag's name — millisecond arguments carry `value_name = "MS"` in help text (`--at <MS>`, `--delta <MS>`). `frame` has no existing duration-valued argument. MCP parameters use the bare CLI word (`full`, `png`, `crop`).

The project glossary already defines **Gap**: "A stretch of a track with no element in it. Gaps are legal and ordinary — the silence between two narration lines is a gap."

## The measurement that just landed (#407)

It measured whether a keyframe tile catches anything the run-start tile misses.

- **The repo's one real project (65 s, 25 fps, 18 visual states) has no keyframe population.** Of 14 declared keyframe change points: 7 sit on a run boundary the run-start rule already tiles; 7 lie outside the lifetime of the element declaring them (a "trimmed move", legal and documented — a 15 s Ken Burns ramp whose element is cut short). Interior to a run on a visible element: **0**. So the keyframe flag adds zero tiles there, and ADR-0094 §3's mandated "count of untiled keyframe change points", read against the declared list, prints **14** where the honest count is **0** — "noise read as a gap". The measurement proposes the counted set be *keyframes interior to a run, on an element visible at that keyframe*.
- **On a constructed project with an interior population**, two of four planted defects were caught by keyframe tiles at 138 px: a 1.85× scale overshoot mid-run cropping a face, and a text block drifting off canvas only between its endpoints. `validate` deliberately never flags the latter (partial off-canvas is "ordinary, legal animation vocabulary"), so a keyframe tile is the only view reaching it. An 8 % `step`-eased snap was **not** caught, nor (correctly) an honest linear ramp: "the tile sees amplitude, not shape."
- **The flag can make a working call refuse.** On an 18-state document the keyframe budget is 12 extra tiles; 31 tiles is 138.4 px (under the 140 px floor) and 7.9 px type (under the 8 px floor). The 13th keyframe tile turns a working call into `E-SHEET-OVERFLOW`.
- **A keyframe tile has no ADR-0098 §8 identifying field** (it has no boundary change). The prototype used `element.property` (e.g. `photo-05-loop.scale`, 19 chars vs a boundary field's ~15), which overran; since elision is sheet-wide, one long keyframe id strips the identifying field from *every* tile. An infill tile likewise has no boundary.
- **An option it measured but did not settle:** a *width-conditional default* — include keyframe tiles whenever the sheet still clears 140 px, exclude them otherwise. Free on the fixture and on lightly-animated projects; never reaches the refusal.

# The questions

**Q1 — Does keyframe tiling stay off by default?**
(a) Off by default, opt-in flag; retire ADR-0094's "absence of evidence" caveat.
(b) Width-conditional default (on while the sheet clears 140 px, off otherwise).
(c) On by default, opt-out flag.
(d) Something else.

**Q2 — The keyframe flag's spelling and arity** (for the answer to Q1; if you chose (c), the opt-out spelling). Candidates include `--keyframes`, `--with-keyframes`, `--include-keyframes`, or something else. Bare boolean or valued? The name must not read as "switch to a different selection rule".

**Q3 — The infill flag's spelling** (its value is a ceiling in milliseconds, and the name must not read as a count — the mistake `--n` made).
(a) `--infill <MS>`
(b) `--infill-ceiling <MS>`
(c) `--infill-every <MS>`
(d) `--max-gap <MS>`
(e) Something else.

**Q4 — The domain term for the quantity.** ADR-0094 calls it a "gap ceiling". Keep that term, or coin another for the glossary (e.g. "infill ceiling": the longest span of the range the sheet may leave unsampled)? Say which and why.

**Q5 — Infill versus the degrade step.** When infill tiles would not fit at the 180 px target:
(a) The document-derived tiles alone fix the rung; infill fills only the slots left over at that rung, so requesting infill can never degrade the sheet.
(b) Infill may push the sheet to the 140 px floor before any infill tile is evicted.
(c) Something else.

**Q6 — What a partly-honoured infill request promises.** If only some infill tiles fit:
(a) Drop all infill; disclose it.
(b) Keep a subset chosen by some ordering; the ceiling then holds in some places and not others; disclose the evicted ones in `skipped[]`.
(c) Honour the smallest ceiling ≥ the requested one that fits, uniformly; disclose "requested X ms, achieved Y ms"; the tiles that the requested ceiling would have added appear in `skipped[]` as `infill-evicted`.
(d) Refuse.
(e) Something else.

**Q7 — The label of a tile with no boundary (keyframe and infill tiles).**
(a) Such tiles carry no identifying field; the class mark and sigil say what they are, the provenance line names e.g. `photo-05.scale`, and they are excluded from the sheet-wide elision fit.
(b) Carry `element.property` and accept that sheet-wide elision fires earlier.
(c) A shortened form (say what).
(d) Something else.

**Q8 — The disclosed keyframe count.** Adopt the measurement's definition — keyframes interior to a run, on an element visible at that keyframe — for ADR-0094 §3's count? And what does the answer report when the flag *is* passed (e.g. an `untiled` count only, or `tiled` and `untiled` both)?

**Q9 — The overflow refusal when the keyframe flag caused it.** When the range fits without the flag but not with it:
(a) Add a finding field to `E-SHEET-OVERFLOW` saying the range fits without the keyframe flag (and how many keyframe tiles would fit), rendered as prose.
(b) No new field; the existing sub-ranges suffice.
(c) Something else (e.g. degrade by dropping keyframe tiles instead of refusing).

**Q10 — Surfaces and invalid values.**
(a) Both flags on both CLI and MCP with identical meaning, MCP names the bare CLI words in snake_case.
(b) Either flag without `--from/--to` is `E-INVOCATION`.
(c) An infill ceiling ≤ 0, or below one frame period, is refused as `E-INVOCATION` rather than clamped.
Agree or disagree with each, and why.

# Ballot

Reply with exactly this block and nothing else. You may reject a question's framing outright; that is a valid answer.

```
🗳️ **Juror <n>** (<the model backing you>) — **VOTE: Q1 … ; Q2 … ; Q3 … ; Q4 … ; Q5 … ; Q6 … ; Q7 … ; Q8 … ; Q9 … ; Q10 …**

**Reasoning:** <why, per question>
**Trade-offs:** <what each choice costs, or why not the other options>
```
