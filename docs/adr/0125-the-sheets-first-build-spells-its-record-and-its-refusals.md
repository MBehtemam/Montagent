---
status: accepted
amends: 0094 (section 6's structured disclosure is spelled — the JSON key names, what `coverage` counts, and a range with no painted frame), 0095 (the degraded rung is named `degraded` and drawn at what the near-square grid serves, not at exactly 140 px; both widths are capped at the project's own width; the sheet is drawn at its served size), 0097 (a range on `frame` is read by `render`'s rule, and the refusals are given in a fixed order in which `--crop` outranks `--full`), 0105 (`E-SHEET-OVERFLOW`'s fields are named and `limit`'s first value is `tile-width`; a refused range answer carries the report alone)
---

# The sheet's first build spells its record and its refusals

[#488](https://github.com/MBehtemam/Montagent/issues/488), the tracer bullet under spec
[#486](https://github.com/MBehtemam/Montagent/issues/486). It builds `frame --from --to`: one
contact sheet with one tile per visual state, its disclosure, and its refusals. The ADRs from
0094 to 0106 fix what the sheet is. They leave some things a caller can observe unspelled, and
#486's Further Notes 5, 6, 7, 10 and 11 list them. Under ADR-0031 each is a spec gap until an
ADR ratifies it. This one ratifies what the first build chose, and the few choices it had to
make that no note anticipated.

Labels and the READER CHECK are #490's. Keyframe tiles are #491's, infill is #492's, the
sub-ranges an overflow names are #489's, and the CLI and MCP surfaces are #493's. Nothing
here decides any of those.

## Decision

### 1. The record's key names (Note 5)

A range answer carries the report, `frame: null`, `query: null`, and one `sheet` object:

| key | value |
| --- | --- |
| `from`, `to` | the range, half-open |
| `rule` | `{name, version, sentence}`: `first-painted-frame-of-each-visual-state`, version `1`, and the rule in one sentence |
| `picture` | `{encoding, width, height, columns, rows, served_tile_width, served_tile_height, rung, rung_px, path, bytes}`, or `null` (section 5) |
| `rasterized` | `{width, height}`: the true pixels each tile was painted at |
| `provenance[]` | one per tile: `index` (from 1, reading order), `instant_ms`, `run {start, end}`, `why` (`boundary`), `class` (`run`), `present` (element ids, in full on every line), `not_painted`, `painted_partially` |
| `classes` | `{run, keyframe, infill}`, zeros asserted (ADR-0098 §6) |
| `skipped[]` | `{run {start, end}, reason, present}`; `reason` is `no-grid-frame` in this build |
| `audio_boundaries_dropped` | `{count, boundaries[]}`, each `{at, entering, leaving}` naming the audio elements |
| `coverage` | `{range_ms, states, tiled, skipped, depicted_ms, not_depicted_ms}` |
| `blind_to[]` | `{token, sentence}`, the six of ADR-0105 §5 with `between-keyframes` as ADR-0106 words it |
| `sources`, `fonts` | every file the tiles opened |

`why` and `class` are two fields because they answer two questions. `why` says what put the
instant on the sheet (ADR-0094 §6). `class` is the token ADR-0098 §6 prints on the provenance
line. A run tile is `boundary` and `run`; #491 and #492 add the other two pairs.

`not_depicted_ms` is the total length of the skipped states: the milliseconds of the range
that no tile stands for. It is not "every millisecond but the sampled one". That is the
`inside-run` blind spot, which is stated on every answer and would make the number the range
length less a few frames every time.

The plain text prints every one of these (ADR-0097 §6) in a `SHEET` block, a `PROVENANCE`
block, a `DISCLOSED` block and a `BLIND TO` block, in that order, above the findings and
NOT CHECKED. Each blind spot's sentence prints on one line, unwrapped, so the text and the
JSON hold byte-identical strings.

### 2. The degraded rung is what the grid serves at 140 px or more (Note 6)

The rungs are named **`target`** and **`degraded`**. `CONTEXT.md` avoids an unqualified
*floor*, and the 140 px width is the **tile-width refusal**, so a rung named `floor` would
be a fourth thing called that.

A degraded sheet is drawn at **the near-square grid's own served width**, which lies in
`[140, 180)`. It is not shrunk to exactly 140 px. Drawing narrower than the budget allows
would give up legibility for nothing, and ADR-0095 §2 derives count from width, not the other
way round. `rung_px` says which width the rung guarantees. The fixture's 19-state case serves
at 173 px.

**Both widths are capped at the project's own width.** A tile served at the project's true
width loses nothing to a downscale, and ADR-0095's widths measure what a downscale loses.
Without the cap a 100 px project would be refused on every range.

### 3. The sheet is drawn at its served size, and a tile is a function of its frame alone

The sizing function ports `docs/research/contact-sheet-budget/check_tile_budget.py`'s model
line for line. Its table tests are that script's numbers: 18 portrait tiles are 6 × 3,
served 1107 × 1092 at 184 px, and 30 is the most the refusal width admits. The drawn picture
is `columns × served_tile_width` by `rows × ⌊served height / rows⌋`. That is inside the
served size, so the tier does not downscale it again and the disclosed width is the width
looked at.

Each tile is painted at true pixels, resampled **with mipmaps** into a tile-sized canvas
over opaque black, and placed on the sheet 1:1. A bilinear reduction of five or six times
samples four pixels out of thirty, and a one-pixel stroke survives or not by where it falls.
Resampling straight into the tile's cell made its pixels depend on its position: Skia differs
by a few pixels between offsets. The canvas of its own makes each tile a function of
`frame --at <its instant>` alone, which the cross-verb test asserts byte for byte. The black
underneath is `frame --at`'s own. Without it a translucent `background` would show the
previous tile through it.

The strip beneath each tile, and any cell with no tile, is `#1A1A1A` until #490 draws labels
in it.

### 4. A range on `frame` is `render`'s range, and the refusals come in a fixed order (Notes 7 and 11)

`frame` reads `--from`/`--to` through the function `render` reads its pair with, so the two
cannot disagree about a range. A missing half, `from ≥ to` and a negative `from` are bare
`E-INVOCATION`, in `render`'s words.

A range call gives the first reason that applies, in this order:

1. `--at` with either half of the pair: the modes are mixed.
2. The pair itself: a missing half, empty or backwards, or before 0.
3. `--crop`: permanent, naming the two-step loop and "whole frames by design" (ADR-0103).
4. `--full`: the tier fact (ADR-0097 §4).

**`--crop` outranks `--full`.** Its refusal is permanent and names the loop the caller
actually wants. `--full` on its own is a flag with nothing to buy. With `--crop` beside it,
it is legal on the single frame the loop sends the caller to (ADR-0101), so its reason would
teach nothing the crop's does not.

A document with no positive integer `fps` has no grid to sample. It is `E-NOT-A-PROJECT`,
as a `frame` with no integer `width`/`height` already is.

### 5. What a refused range answer carries, and what an empty one does (Note 10)

**A refusal carries the report alone**, with no `sheet` key. There is no sheet, so there is
no tile 1 for #490's reader check to quote, and `blind_to` describes the blind spots of a
sheet that was not drawn. A refusal is about the invocation (ADR-0105 §1). The same shape
holds for `E-INVOCATION`, `E-SHEET-OVERFLOW`, and a document that does not parse or is not a
project.

`E-SHEET-OVERFLOW` carries `from`, `to`, `states` (visual states needing a tile), `fits` (the
most tiles the sheet holds), `limit` and `limit_px`. `limit`'s one value in this build is
**`tile-width`**, at 140 px. #489 adds the sub-ranges and its field, spells the type floor's
value with #490, and replaces the template's closing "ask for a narrower range" with the ranges
themselves. The refusal is sized before a tile is painted, so no picture is ever made.

**A range with no painted frame is not a refusal.** An example is `[1010, 1030)` on #437's
document. Its only state is unpainted, which is a fact about the document, so the answer is
the sheet record with `picture: null`, no image, the state in `skipped[]`, its
`N-QUANTIZATION`, and every disclosure. The text says *no tile: no frame is painted in
[from, to)*.

### 6. A tile's painting findings go to the report once

Each provenance line carries its own `not_painted` and `painted_partially`, as `frame --at`'s
picture does. The findings behind them are pushed once per `(element, code)` over the whole
sheet. This is `render`'s rule for a span: the same element declining for the same reason on
every tile is one fact.

## Consequences

- `render::extent` and `render::instant_of` move to `crate::exact`, which ADR-0118 left to this
  ticket. Four verbs and a check call them.
- `E-SHEET-OVERFLOW` goes from Declared to Live.
- `montagent-render`'s `Canvas` gains `snapshot` and `composite`.
- Nothing is on the CLI or MCP yet (#493).

## Evidence

The numeric claims are asserted by tests that run on every build:
`crates/montagent-core/src/verbs/frame/sizing.rs` (the rung edges, the grid shapes, 30
admitted, all against `check_tile_budget.py`'s numbers), `tests/frame_range.rs` (18 tiles at
184 px on the fixture, 28 dropped audio-only boundaries named, 0 skipped, the degraded rung at
173 px, the refusal at 31, every refusal's reason), and `tests/cross_verb.rs` (the sheet's
states equal `query --from --to` filtered and re-merged, and every tile equals
`frame --at <instant>` composited, byte for byte).
