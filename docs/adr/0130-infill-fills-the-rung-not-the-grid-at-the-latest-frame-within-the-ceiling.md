---
status: accepted
amends: 0106 (decision 4's span after the last tile runs to the first frame painted at or after the range's end; decision 5's slots are the tiles the rung admits, not the grid's empty cells, and infill keeps the identifying field as well as the rung; decision 6's `achieved: none` is spelled for a sheet with slots left; decision 14's frame-period refusal is placed after the document is read, and a whole number written with a zero fraction is that number), 0125 (`why` and `class` gain `infill`, the sheet gains `infill {requested_ms, achieved_ms}` when the ceiling is passed, and a `skipped[]` entry gains `evicted`, an `infill-evicted` entry's count)
---

# Infill fills the rung, not the grid, at the latest frame within the ceiling

[#492](https://github.com/MBehtemam/Montagent/issues/492), a ticket under spec
[#486](https://github.com/MBehtemam/Montagent/issues/486). It builds `--infill-ceiling <MS>`:
ADR-0106's infill tiles, fitted into the slots the document-derived tiles leave and honoured
uniformly. ADR-0106 fixed what the ceiling bounds, that infill never degrades a sheet, and what a
partial fit discloses. #486's Further Note 8 lists what it left open: **the ceiling at non-integer
frame periods, and a ceiling of exactly one period**. The ticket adds a third: **which painted
frames infill picks inside a span**, which no ADR fixes. Under ADR-0031 each is a spec gap until an
ADR ratifies it. This one ratifies what the build chose, and the few choices it had to make that
no note anticipated.

## Decision

### 1. A slot is a tile the rung admits, not an empty cell of the grid

ADR-0106 D5 says infill *"fills only the slots left at the rung"* the document-derived tiles
fixed. There were two readings, and the build takes the second.

- **The grid's empty cells.** The document-derived tiles' near-square grid is kept, and infill
  goes into the cells it leaves over. This changes nothing about those tiles. It also makes
  infill nearly useless. From 1 to 20 portrait tiles the grid leaves **at most one** empty cell
  (5 tiles on a 3 × 2 grid, 17 on a 6 × 3), and **a single state is one tile on a 1 × 1 grid,
  with none**. #486's user story 23 is *"so that long still-looking states get looked inside"*,
  and under this reading the longest still state has no slot at all.
- **The tiles the rung admits.** Infill may add tiles while the sheet still serves every tile at
  the rung's width. One state at the target rung has **17 slots**, because 18 tiles still serve at
  184 px. ADR-0106's own worked example, *"an 18-state whole-document call on the fixture
  fills all 18 slots at 180 px"*, speaks of the rung's capacity. It cannot decide between the
  readings on its own, though: 18 tiles on a 6 × 3 grid leave no empty cell under either.

Under the second reading the grid is re-derived for all the tiles, so the document-derived tiles
are served **narrower** than without the ceiling. ADR-0106 §5–6 forbids *"displacement by
resolution"* at the rung step, and the rung is kept, so this stays inside it. What else is kept:

- **The identifying field.** Infill tiles leave the sheet-wide elision fit (ADR-0106 D7), and a
  narrower grid could still elide the document-derived tiles' ids by narrowing them. So a count is
  admitted only where the labels keep the field the document-derived tiles had: carried stays
  carried. Losing the id is content giving way (ADR-0098 §4), which is a degradation of its own.
- **The type floor.** Every label on the sheet, infill ones included, is fitted at one size that
  is at least 8 px (ADR-0128 §4). An infill label that cannot be drawn there takes its slot away,
  never the document-derived tiles' ids.
- **Every smaller count.** A count is admitted only where every count up to it keeps the rung, so
  the uniform fit below can use fewer tiles than the most without passing the rung on the way.
  The research model does not assume served width falls monotonically with count, so this is
  checked rather than assumed.

The type size may shrink, and the tiles do. Both are what the rung already permits a sheet of
that many document-derived tiles.

### 2. An infill tile is the latest painted frame no more than the ceiling after the tile before it

A span runs from one tile's painted instant to the next tile's, **of any class** (ADR-0106 D4).
**The last tile's span runs to the first frame painted at or after `to`**, the frame the video
paints next once the range's own frames end. Without that closing span a single long state would
be one tile with no span, and a ceiling would never add anything to it.

Inside a span longer than the ceiling `C`, infill walks from the tile that opens it. Each infill
tile is **the latest painted frame at or before the previous tile plus `C`**, until the span's end
is within `C`. So:

- **It is exact.** No span on the sheet is longer than `C`, because every step is at most `C` and
  lands on a frame the grid paints. A rule that spaced the tiles evenly and then snapped each to
  the grid could pass `C` by up to a frame period at every snap.
- **It adds the fewest tiles.** Walking as far as the ceiling allows at each step is the smallest
  count any placement on that grid can reach.
- **It can be checked by hand.** Each tile's instant follows from the one before it and `C`. On
  the committed long-state fixture at 2000 ms the tiles after the run tile at 1000 ms are at
  3000, 5000 and 7000 ms. With a keyframe tile at 4040 ms they are at 4000 and 7040: the walk
  restarts at every tile, whatever its class.

The cost is a **short remainder at the end of a span**. A 1000 ms span at a 400 ms ceiling is
tiled at 400 and 800, not at 333 and 666. That is the price of the bound being exact.

### 3. Uniform honour, and when nothing is achieved

When the requested ceiling's tiles fit, it is achieved as asked. Otherwise the sheet is drawn at
**the smallest whole-millisecond ceiling whose tiles fit**, in every span at once (ADR-0106 D6).
A larger ceiling never needs more tiles, so this is a search and not a scan.

- **The achieved ceiling is then the sheet's longest span.** If a longer span were left
  shorter than it, a smaller ceiling would have fit with those same tiles. On the long-state
  fixture at 200 ms the achieved ceiling is **520 ms**, the sheet has 18 tiles, and its longest
  span is 520 ms. The step to 520 comes from the grid: at 25 fps, 500 ms walks in 480 ms steps
  and needs two tiles more than the slots hold.
- **`achieved_ms` is `null` exactly when the request needed tiles and the sheet draws none.**
  ADR-0106 D6 names the case *"no slot is left"*. It is not the only one. Sixteen equal 200 ms
  spans on a sheet with two slots cannot be closed uniformly by any ceiling under 200 ms, since
  every span needs a tile of its own. The smallest ceiling that fits is the spans' own length,
  with no tile added. So `achieved: none` means **no ceiling better than the document's own
  tiles**, whether or not a slot was free.
- **A ceiling longer than every span is achieved as asked** and adds no tile, which is ADR-0106
  D6's *"legal request that adds no tile"*. The sheet is then the one drawn without the flag.

If only fewer tiles' labels fit (section 1), the search is run again for the next smaller count,
down to none.

### 4. The frame-period edge cases (Note 8)

- **There is no 29.97 fps project.** The format's `fps` is an integer (the schema says so and the
  sheet refuses anything else as `E-NOT-A-PROJECT`). So Note 8's non-integer period is 24, 30 or
  60 fps, where `1000 / fps` is not a whole millisecond.
- **A ceiling is under one frame period exactly when `C × fps < 1000`**, in integers. That is
  refused as `E-INVOCATION`, never clamped (ADR-0106 D14), and the reason names the least ceiling
  the grid can honour, `⌈1000 / fps⌉`.
- **A ceiling of exactly one frame period is legal.** At 25 fps, 40 ms tiles every frame: it asks
  for the spacing the grid paints, and nothing finer.
- **At 30 fps the least whole ceiling is 34 ms**, and it tiles every frame too. The grid paints at
  `⌊n · 1000 / 30⌋`, so painted frames are 33 or 34 ms apart, and 34 is the least whole number
  every such spacing fits under. 33 is under the period of 33.3 ms and is refused, although many
  frames on that grid are 33 ms apart. A whole-millisecond ceiling can never equal a non-integer period.
- **The period check runs once the document is read**, because it needs the document's `fps`.
  Every other refusal of the flags is settled before the file is opened (the verb's rule), so an
  unreadable file is reported before a ceiling under its period.
- **A whole number is whatever reads as one.** `40` and `40.0` are both 40 ms. `40.5`, a number
  too large to read exactly as a whole number, and anything that is not a number are refused as *"not a whole number of
  milliseconds"*. Zero and negatives are refused as *"must be more than zero"*. The ceiling reaches
  the core verb as the caller wrote it, as `--crop` does, so both surfaces refuse the same
  spellings.

### 5. An infill tile's label

`3 I 3000ms +2000`. The sigil is ADR-0128 §5's `I`. The offset is from **the start of the state
the tile samples inside**: an infill tile stands for no change, so its state's opening is the only
boundary it has (ADR-0098 §3). A keyframe tile measures from its change point for the same reason
(ADR-0129 §6). There is no identifying field, so the label ends after the offset. The strip is
inverted, as for a keyframe tile.

### 6. Evictions are one `skipped[]` entry per run, counted

The tiles the requested ceiling would have added and the sheet does not draw are grouped **by the
visual state they fall in**, one `skipped[]` entry each:

```json
{"run": {"start": 1000, "end": 9000}, "reason": "infill-evicted",
 "present": ["bg", "b"], "evicted": 36}
```

- **A count, not a list of instants.** How many instants a ceiling asks for is bounded only by
  the range: an hour at 40 ms is 90,000. Section 2's rule reproduces every one from the ceiling
  and the tiles, and `frame --at` looks at any instant.
- **Only the requested ceiling's own tiles that are not drawn count.** A tile the achieved
  ceiling happens to place at the same instant is drawn, and is not evicted. On the long-state
  fixture, 200 ms asks for 43 tiles, and 3 of them (3600, 6200 and 8800) are on the 520 ms walk
  too. So 40 are evicted: 4 in the first state and 36 in the second.
- **`evicted` is present only on an `infill-evicted` entry.** A `no-grid-frame` entry is a state
  with no tile, and has nothing to count.
- **`coverage.skipped` still counts states with no tile**, so it is not `skipped[]`'s length once
  an eviction is listed. An evicted state has its run tile.
- **An eviction is never a finding** (ADR-0105 §4). It is the instrument's choice under budget,
  and the document's finding count does not move with the flag.

### 7. An infill tile shows the change points sampled at its frame

A `not-requested` change point (ADR-0129 §3) samples inside its own state, where its element is
on screen. If an infill tile lands on that frame, the point is **tiled**, by ADR-0106 D8's
grid-aware split. It joins the infill tile's `keyframes` list, and the tile keeps its class, as a
run tile does under D11. On the long-state fixture a 3040 ms ceiling puts an infill tile at
4040 ms, where `b.x@4010` first paints, and the census reads `1 tiled, 0 untiled` without
`--keyframes`. A `no-grid-frame` point samples at a run tile's frame or past the range, and an
infill tile is at neither.

### 8. The record

- `infill: {requested_ms, achieved_ms}` is on the sheet **only when the ceiling was passed**
  (ADR-0106 D6). The text prints `infill      requested 200 ms, achieved 520 ms`, or `achieved:
  none`, in the `DISCLOSED` block after the keyframe census.
- A range with no painted frame has no span to bound, so its ceiling is achieved as asked.
- An infill tile is `why: infill`, `class: infill`, its `run` the state it samples inside, and
  `classes.infill` counts it.
- An overflow is decided on the document-derived tiles alone, before infill is placed, and never
  mentions the ceiling: infill cannot cause one.

## Honest costs

- **Asking for infill makes the document-derived tiles smaller.** One long state draws at 784 px
  alone and at 184 px with 17 infill tiles. That stays on the target rung, which is all ADR-0106
  promised. It is still a real change of the picture those tiles are seen in, made by a flag.
- **The placement front-loads a span.** Its tiles sit every `C` from the tile that opens it, and
  the short remainder falls at the end.
- **"The smallest ceiling that fits" is a search over counts, and labels can shorten it.** If the
  most tiles fit the rung but their labels do not fit the type floor, the answer is drawn at the
  next smaller count's ceiling. A different set of tiles at an intermediate ceiling might have
  fit. This is only possible where labels are at the type floor, and it is not searched for.
- **Evictions are counted, not listed.** A caller that wants an evicted instant recomputes it
  from section 2's rule.
- **The ceiling is not measured to the range's start.** The first tile is the first frame painted
  in the range, so there is no painted time before it to bound. A range that opens on an unpainted
  state starts its first span at the first painted one.

## Consequences

- `Ask` gains `infill_ceiling`, the ceiling as written. Without `from`/`to` it is refused as
  `E-INVOCATION`, naming `--from`/`--to`. The CLI and MCP flags stay with #493.
- `sizing` gains `infill`: given the document-derived tiles' fit and instants, the range's end,
  the ceiling and a label measure, it returns the sheet, the infill instants, the achieved ceiling
  and the evicted instants. `size` is unchanged.
- The infill tile's strip is inverted and its label carries `I`, as ADR-0128 §5 fixed.
- One committed fixture, `fixtures/long-state`: a one-second state and an eight-second state with
  one interior change point.

## Evidence

- `crates/montagent-core/src/verbs/frame/sizing.rs`:
  - a ceiling that fits, on and off the grid;
  - 20 seconds at 1000 ms honoured at 1120 on 17 slots, with its evictions;
  - no slot, and two slots over sixteen equal spans, both achieving none;
  - a ceiling longer than the range;
  - a keyframe tile restarting the walk;
  - a ceiling of one period at 25 fps, and 34 ms at 30 fps;
  - the degraded rung kept, and a label that does not fit taking the slot away.
- `crates/montagent-core/tests/frame_range.rs`:
  - the long-state fixture's tiles, labels, marks and disclosure at 2000 ms;
  - the 200 ms partial fit at 520 ms with its two `infill-evicted` entries and no finding;
  - infill with `--keyframes`, and an infill tile tiling a change point;
  - a ceiling longer than the range drawing the sheet drawn without it;
  - the main fixture gaining no infill tile at 40 ms (`achieved: none`) or at 1,000,000 ms;
  - every refusal: without a range, zero, negative, fractional, too large, not a number, and
    under one period at 25 and 30 fps, with one period and 34 ms at 30 fps legal.
- `crates/montagent-core/tests/cross_verb.rs`: each infill tile is `frame --at <its painted ms>`,
  byte for byte.
