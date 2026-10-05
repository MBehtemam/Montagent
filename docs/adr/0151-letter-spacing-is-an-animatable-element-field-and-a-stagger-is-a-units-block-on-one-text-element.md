---
status: accepted
amends: 0148 (its three binding requirements are discharged and the stagger is specified in full), 0146 (`letter_spacing` and the `units` lists join the one derived animatable list; its open item "per-letter and per-run animation" is closed), 0007 (a text element gains `letter_spacing` and `units`, and a run gains `unit`; optional ligatures are switched off by a rule read from the file), 0044 (`R-OFF-CANVAS` widens the declared rect by the unit offsets), 0106 (a stagger's keyframe change points are named by unit index, and only the first and last scheduled units count)
---

# Letter spacing is an animatable element field, and a stagger is a units block on one text element

[#682](https://github.com/MBehtemam/Montagent/issues/682), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663). Two capabilities are decided here.
**Letter spacing** answers [#510](https://github.com/MBehtemam/Montagent/issues/510): a tracked
title today needs one element per letter, which breaks shaping, kerning and alignment.
**Per-letter animation** is the stagger that
[ADR-0148](0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md)
admitted in principle: one text element's animation runs once per unit, each unit started a
fixed delay after the one before. ADR-0148 bound it to three requirements: a split run's values
replace the derived ones, the unit is named exactly, and the tools report per unit.

**Precedent.** Row 4 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)). Premiere's Tracking and CapCut's
character spacing are both in thousandths of an em, and Premiere keyframes Tracking, so letter
spacing and its animation are inside the reference class. Free per-letter animation is in
neither tool, so under
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
the stagger enters only once its prototype
([#683](https://github.com/MBehtemam/Montagent/issues/683)) is accepted by the owner.

**How it was decided.** Two grilling rounds, each put to a court of three jurors (Opus, Sonnet
and Fable) judging as the agent that writes and edits the file. The jurors agreed on every
question but one (§6, the caption note), and the owner ruled with the Judge's read both times.

## The decision

### 1. Letter spacing

A `text` element gains **`letter_spacing`**: an integer in thousandths of an em, negative
allowed, default 0.

- **Element-level only.** A run may not override it.
- **Measured against each run's own size.** The space added after a grapheme is
  `size × letter_spacing / 1000` pixels, where `size` is the size of the run the grapheme sits
  in. A `size` edit therefore never quietly changes the tracking.
- **Where it is added.** After every grapheme cluster of a line, spaces included and across run
  boundaries, **except the line's last**. A centred or end-aligned line stays where `align`
  puts it.
- **Layout follows it.** The spaced advance is the advance: the typographic block, `align`,
  `origin` and the ink box all measure the spaced line. Lines still break only at `\n`, so a
  change of spacing never re-breaks a line.
- **Animatable.** It meets ADR-0146's three criteria. It is integer-typed, so its keyframe
  values are integers and the resolved value is continuous and never rounded, as with `x`.

**Optional ligatures are decided from the file, not from the instant.** If any `letter_spacing`
value or keyframe on the element is non-zero, or the element has a `units` block with
`by: letter`, the element is shaped with `liga`, `clig` and `dlig` off throughout its whole
range. Otherwise it is shaped as today. A spacing animation that passes through 0 therefore never
swaps glyphs mid-shot, and `validate` can say from the file how the element shapes. This is the
CSS rule for non-zero letter spacing, made static.

### 2. The stagger is a `units` block

A `text` element may carry one **`units`** block:

```json
"units": {"by": "letter", "every": 40, "order": "forward", "origin": "center",
          "y": [{"t": 1000, "v": 20}, {"t": 1300, "v": 0, "ease": "ease-out"}],
          "opacity": [{"t": 1000, "v": 0}, {"t": 1300, "v": 1, "ease": "linear"}]}
```

| Field | Value |
| --- | --- |
| `by` | Required. `letter`, `word` or `line` (§3). |
| `every` | Required. A positive integer of milliseconds: the step between one unit's start and the next. Static. |
| `order` | `forward` (default) or `reverse`. |
| `origin` | One of the nine `origin` keywords, default `center`: each unit's pivot (§3). |
| `x`, `y`, `rotation`, `scale`, `opacity` | The unit lists. Each is a literal or a keyframe list, with the same record shape and `ease` rules as an element's own. At least one is required. |

**Each unit runs the lists late by its delay.** The lists are written for the first unit, on the
absolute clock like every keyframe. A unit's **delay** is its position in `order` times `every`:
with `forward` the first unit in reading order has delay 0, and with `reverse` the last does.
Unit *n*'s value at instant *t* is the list's value at *t − delay*.

**The unit values are offsets inside the element.** `x`, `y` and `rotation` add to the unit's
laid-out pose, and `scale` and `opacity` multiply it. Their defaults are 0, 0, 0, `[1, 1]` and 1.
`x` and `y` are in the element's own space, before the element's `scale` and `rotation`. The
element's transform still moves the whole block, so a title can slide in while its letters rise.
The element's own keyframes keep the meaning they have today, whether or not a `units` block is
present.

**No unit paint.** A unit cannot vary `color` or `stroke`. A run would otherwise have two timed
paint sources, the unit lists and its highlight, and ADR-0146 refused exactly that. A
letter-by-letter colour change is a highlight window, or a second element.

**A fading unit fades as one group.** While a unit's resolved opacity is below 1, its fill and
stroke are drawn together into one layer, and that layer takes the unit's opacity. Multiplying
alpha into the fill and stroke separately would show the stroke through the fill, more so under
negative spacing. At opacity 1 no layer is made, so an idle `units` block draws the same bytes as
no block. The prototype measures what the layer costs.

**The rest of the element's pipeline is unchanged.** `effects`, `blend` and the element's
`opacity` apply to the element after its units are drawn. A highlight still changes a run's paint,
and the unit lists still move it. The two do not interact.

### 3. What one unit is

| `by` | One unit is |
| --- | --- |
| `letter` | A grapheme cluster that is not whitespace. Punctuation and emoji are letters. |
| `word` | A UAX #29 word segment that contains a letter, a digit or an `Extended_Pictographic` character. Punctuation attaches to the word before it on the same line, and otherwise to the word after it, so an opening quote belongs to the word it opens. |
| `line` | One `\n`-separated line that contains at least one letter. |

- **Counting.** Units are counted in logical (reading) order across the whole element, ignoring
  run boundaries, because a run boundary is style only. In right-to-left text the first unit is
  the rightmost one on screen.
- **Whitespace and empty lines take no step.** With `by: letter`, `"A B"` is units 0 and 1.
- **The count comes from the text alone.** It uses the same grapheme rule as the caption pace
  check. Shaping never changes how many units there are or which index a letter has.
- **Merged clusters share timing, not a count.** Where required shaping merges clusters into one
  glyph (Arabic joining, Indic conjuncts), the glyph moves on the timing of the first of them in
  reading order. Each merged unit still occupies its index and its step, so the units after it
  keep their schedule. `R-UNIT-MERGED` (review) names the merged clusters.
- **A unit's box.** For a letter it is the grapheme's own advance, without the added letter
  spacing, by the line's slot. The pivot therefore does not drift while spacing animates. A word's
  or a line's box runs from its first grapheme's leading edge to its last grapheme's trailing edge,
  by the line's slot. `origin` on the `units` block picks the pivot in that box.

### 4. Singling out a unit

A unit is singled out by splitting it into its own run. That run carries a **`unit`** object:

```json
{"text": "O", "unit": {"delay": 300, "y": [{"t": 1200, "v": 60}, {"t": 1700, "v": 0, "ease": "ease-out"}]}}
```

- **`delay`** is a literal that replaces the derived *position × every* for this unit.
- **A list** (`x`, `y`, `rotation`, `scale`, `opacity`) replaces that property's derived list for
  this unit wholesale. A list on a run is on the absolute clock and is **never** shifted by any
  delay: what is written is what runs.
- A property the run does not name keeps the element's list, run at the run's delay: the derived
  one, or `delay` if given.
- The singled-out unit keeps its index and its position in `order`, so its neighbours keep their
  schedule.
- At least one of `delay` or a list is required.

Three `validate` errors keep the replace rule honest:

| Code | Fires when |
| --- | --- |
| `E-UNIT-RUN-NOT-ONE-UNIT` | A run carrying `unit` holds anything other than exactly one unit's graphemes: part of a unit, more than one, or whitespace. With `by: word` the unit includes its attached punctuation, and with `by: line` it is the whole line. The finding names the unit count found. A run carrying `unit` on an element with no `units` block is also this error. |
| `E-UNIT-RUN-UNDECLARED` | A run's `unit` names a list the element's `units` block does not declare. Naming a new property would add, not replace. |
| `E-UNIT-RUN-MERGED` | A run's `unit` singles out a unit that shaping merges with a neighbour (§3). Its timing cannot be its own. |

### 5. The tools

**`query --at`.** A staggered element gains:

- A summary: `by`, the unit count, and the **stagger window**, from the earliest start of any unit
  list to the latest end, after delays and including run overrides.
- A `units` array listing **every** unit, whatever its state: `index`, `text`, `run`, `delay`,
  `overridden` per property (for example `{"y": true, "opacity": false}`, plus `delay`),
  `merged_with` (the indexes it shares a glyph with), and the resolved `x`, `y`, `scale`,
  `rotation` and `opacity` at the instant. There is no cap. A missing row would leave "settled",
  "not started" and "missing" looking alike.

**`R-OFF-CANVAS`.** The check asks whether an element's declared rect ever intersects the frame
([ADR-0044](0044-off-canvas-is-a-standing-review-check-not-a-frame-change-census.md)). For a
staggered element, the rect is first widened separately in each direction: left by the most
negative unit `x` value, right by the most positive, and likewise for `y`. The values come from
every unit list, the element's block and every run override. That is the furthest a unit can
reach, so the check stays document-only and needs no fonts. A finding names the list that set each
widened edge. Unit `scale` and `rotation` are not included: when the `units` block or an override
carries either, the finding says so instead of skipping it silently.

**The contact sheet.** A stagger contributes keyframe change points
([ADR-0106](0106-the-sheets-opt-ins-are-keyframes-and-infill-ceiling-and-a-keyframe-tile-is-sampled-where-its-change-first-paints.md))
for two units only:

- the first scheduled unit;
- the **last scheduled** unit, the one whose lists end latest after delays and overrides, which is
  not always the highest index.

Every run override's own lists also contribute. Points are named by the unit's reading-order index,
`title.units[39].y@2860`, so each name maps back to a unit an agent can find. Every unit in between
is left to `query --at`. Eighty tiles for a forty-letter title would bury every other change point.

**`shift`.** `shift` carries the `units` lists and every run override's lists like any keyframe
list. A delay is relative, so it is carried unchanged. A cut that falls inside the stagger window
is **refused**, naming the window. A split of the lists at instant *T* would be right for one unit
only, and writing a correct split for every unit would turn one block into one run per unit.
This is ADR-0146's rule: `shift` refuses what it cannot write as legal literals.

**The one derived list.** ADR-0146's schema-derived list of animatable properties gains
`letter_spacing` and the five unit lists, both on the `units` block and on a run's `unit`. Every
tool reads it.

### 6. Captions

`R-CAPTION-PACE` is unchanged: graphemes divided by the element's duration.

A staggered element that the caption checks run on gets one note, **`N-CAPTION-SETTLES`**: the text
is fully readable at *T*. *T* is the latest last keyframe over every unit list, after delays and
including run overrides. When *T* is at or past the element's end, the note says the caption never
settles. The agent sees when the cascade lands without the pace check being redefined. Redefining
pace from *T* would overstate the problem, because readers follow letters as they arrive.

## The four tests

| Invariant | Letter spacing | The `units` block |
| --- | --- | --- |
| Literal values | An integer or a keyframe list of integers. | `every`, every delay and every list record is a literal. A unit's start is fixed arithmetic: position times `every`. |
| Closed vocabulary | One field. | `by`, `order` and `origin` are closed enums. The lists are the five transform names. |
| Checkable by `validate` | Its sign and type come from the schema. The ligature rule is read from the file. | The unit count, the override errors and the merged-cluster review come from the text and the fonts. `R-OFF-CANVAS` widens from the file alone. `query --at` reports every unit. |
| Exact-string replace | One value or one keyframe record. | Retiming is one edit, `"every": 40`. A unit is singled out by splitting its run, and that run's own strings replace the derived values. |

## Why

**Thousandths of an em.** It is the unit in both tools of the reference class, so a value can be
copied from either. It also scales with `size`: in pixels, every `size` edit would need a second
edit to the spacing.

**No spacing after a line's last grapheme.** CSS adds it, which pushes a centred line half a step
off centre. An agent cannot see why from the file.

**A separate block, not the element's own keyframes run per unit.** If a `stagger` flag
reinterpreted the element's keyframes, one edit would silently change what every existing keyframe
means. It would also leave no way to move the title while its letters stagger.

**Replace per property, with absolute times.** "This letter lands late" is one `delay`, and "this
letter bounces instead" is one list. Requiring an override to restate every property would put a
second copy of each list in the file, and nothing could tell a deliberate difference from a
forgotten edit.

**Refuse inside the window.** The alternative to the refusal is a file rewritten into one override
per unit. That destroys the edit surface an agent comes back to.

## Considered and refused

- **Letter spacing in pixels.** Refused: it breaks under a `size` edit, and neither reference tool
  uses it.
- **A run-level `letter_spacing` override.** Refused for now: a static run value beside an animated
  element value needs a precedence rule, and a word with its own tracking can be its own element.
- **Ligatures switched by the spacing value at each instant.** Refused: glyphs would swap as an
  animation passed through 0, and `validate` could not predict it.
- **A `stagger` flag on the element's own keyframes.** Refused: §Why.
- **Unit paint** (`color`, `stroke` per unit). Refused: two timed paint sources on one run.
- **Refusing `by: letter` on text whose shaping merges clusters.** Refused: it would bar whole
  scripts from letter staggers. **Merging silently** is refused too: the review names the merge.
- **Centre-out and edges-in orders.** Not now: no precedent and no prototype. A tie rule exists
  (units at an equal distance start together), so this can enter later by its own ADR.
- **A random order, even with a literal seed.** Refused: no unit's start can be read from the file
  without running the generator. If a non-monotone order is ever wanted, the admissible shape is a
  literal permutation list, which can be read, edited by exact string, and checked against the unit
  count. That is recorded here, not decided.
- **An override that restates every property**, and **a delay-only override.** Refused: §Why. A
  delay-only override cannot say "this letter rises higher", which is the usual reason to single
  one out.
- **`query --at` printing only units that are moving**, or a cap on the rows. Refused: an absent
  row is ambiguous, and a cap is one more printing rule to learn.
- **Laying out every unit inside `R-OFF-CANVAS`.** Refused: it would make the check depend on fonts,
  and the widened rect already bounds where a unit can go.
- **A change point for every unit.** Refused: it floods the sheet.
- **`shift` splitting each unit separately.** Refused: §Why.
- **Measuring caption pace from the settle instant.** Refused: it would penalise captions whose
  letters are legible while they move. The `N-CAPTION-SETTLES` note gives the number instead.

## Consequences

- **Two slices.**
  - **Slice 1, letter spacing**, is inside the reference class and needs no prototype gate.
  - **Slice 2, the `units` block**, starts once the prototype
    ([#683](https://github.com/MBehtemam/Montagent/issues/683)) is accepted by the owner.

  Each slice shows byte-identical output across parallel painters
  ([ADR-0144](0144-render-paints-on-k-painters-over-chunks-and-the-spy-trailer-renders-in-a-minute.md)).
- **The painter needs a unit index per glyph.** Today a placed glyph knows only its run. Slice 2
  carries the grapheme cluster each glyph came from, and from that its unit.
- `CONTEXT.md` gains **Letter spacing** and **Unit**, and **Stagger** and **Run** are reworded to
  them.
- `format.md` and the feature map gain both fields, the unit rules, the replace rule with its
  one-sentence warning that a run's lists are never delayed, and the blind spots: `R-OFF-CANVAS`
  ignores unit scale and rotation, and the contact sheet shows only the first and last scheduled
  units.

## Not settled here

- **The fading unit's place in the draw order.** Today the painter draws every glyph's stroke and
  then every fill. Where a fading unit's group layer sits in that order, and what it costs, is the
  prototype's to measure and slice 2's spec to state.
- **`R-LINE-INK-COLLISION` under animated spacing.** It measures glyph ink between lines. Slice 1's
  spec states at which instants it resolves `letter_spacing`, after reading how the check samples
  today.
