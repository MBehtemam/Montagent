---
status: accepted
amends: 0106 (decision 8's `untiled` is a count with its members named beside it in `untiled_points`, apart from `skipped[]`; an untiled point's reason is `no-grid-frame` or, new here, `not-requested`; `volume` is not in the population; decision 9's fields are named and its prose is a clause printed only under the flag; decision 7's keyframe label measures its offset from its earliest change point), 0125 (a provenance entry gains `keyframes`, `why` and `class` gain `keyframe`, the sheet gains `keyframes {tiled, untiled, untiled_points}`, and `E-SHEET-OVERFLOW`'s `states` counts run tiles alone), 0126 (a state carrying keyframe tiles weighs more than one tile, and the sub-range cut is made on those weights)
---

# An untiled keyframe point is named in the census, and never in `skipped[]`

[#491](https://github.com/MBehtemam/Montagent/issues/491), a ticket under spec
[#486](https://github.com/MBehtemam/Montagent/issues/486). It builds `--keyframes`: ADR-0106's
keyframe tiles, and the keyframe census on every range answer. ADR-0106 fixed where a keyframe
tile is sampled, what the census counts and what an overflow must say. #486's Further Note 4 lists
what it left open: **where untiled change points live**, and **whether `untiled` is a count, a
list, or both**. Under ADR-0031 each is a spec gap until an ADR ratifies it. This one ratifies
what the build chose, and the few choices it had to make that no note anticipated.

## Decision

### 1. `untiled` is a count, and `untiled_points` names every member (Note 4)

Every range answer carries

```json
"keyframes": {"tiled": 1, "untiled": 1, "untiled_points": [
  {"point": "a.x@1020", "element": "a", "property": "x", "at": 1020,
   "sample_ms": 1040, "run": {"start": 500, "end": 1030}, "reason": "no-grid-frame"}
]}
```

- **Both.** `tiled` and `untiled` are ADR-0106 D8's two numbers, so `0 tiled, 0 untiled` reads
  the same in the JSON and the text. An untiled point is exactly the one a caller cannot find by
  looking at the sheet, so the count alone would be a census whose members cannot be searched
  for. ADR-0111 says such a census names them. A tiled point is already named, on the provenance
  line of the tile that shows it.
- **`sample_ms`** is where the point would be seen: `frame --at <sample_ms>` shows it, which is
  the next call an untiled point asks for. It is `null` only where `t` is so far out that its
  frame cannot be counted in 64 bits. Such a point stays in the census as `no-grid-frame`, so it
  never drops out of the count.
- **`run`** is the visual state the point is interior to, with the same key a skipped state and a
  provenance line use.

### 2. Untiled points never share a list with `skipped[]` (Note 4)

A `no-grid-frame` entry in `skipped[]` is a declared visual state the video never shows, and
raises `N-QUANTIZATION` (ADR-0118). A `no-grid-frame` change point is disclosure only, never a
finding (ADR-0106 D12). One list would need a type field to keep those two apart, and a reader who
skipped the field would count an untiled point as an unpainted state. So the untiled points live in
the census that counts them, `skipped[]` keeps meaning "a state with no tile", and **nothing in
`keyframes` raises a finding**. The text prints each untiled point on its own line under the
census, in the `DISCLOSED` block, beside `skipped` and never in it.

### 3. Without the flag an interior point is untiled as `not-requested`

ADR-0106 D8 puts the census on every answer, flag or not, and defines `no-grid-frame` as the only
reason. Without `--keyframes`, a point whose sample frame is inside its own state, and is not that
state's run tile's frame, has no tile only because none was asked for. Calling that
`no-grid-frame` would be false: the grid paints it. So it is **`not-requested`**. A point with no
painted frame left in its state is `no-grid-frame` whether or not the flag was passed, because the
flag would not add a tile for it.

A point that samples at a run tile's frame is **tiled without the flag** (ADR-0106 D8's grid-aware
split) and is listed on that run tile's provenance line. The coincidence fixture prints
`1 tiled, 3 untiled` without the flag and `4 tiled, 0 untiled` with it.

### 4. The population leaves out `volume`

The population is every keyframe record's `t` on `x`, `y`, `scale`, `rotation` and `opacity`, on
an element with an id that is in the visual presence set of the state holding `t`, with
`s < t < e`. ADR-0012's sixth animatable property, `volume`, is left out. A change in `volume` is
audible, the selection drops audio members, and the sheet's `audio` blind spot says nothing audible
is on it. A tile for a `volume` change on a video element would show a frame whose pixels do not
change there. The fixture is unaffected: it has 0 interior points with or without `volume`.

An element without an id is left out as well, for ADR-0128 §2's reason: the provenance line would
have nothing to call it.

### 5. The record

- A provenance entry gains **`keyframes`**: the change points the tile shows, as
  `element.property@t` in clock order. It is empty on a run tile that shows none.
- A keyframe tile has **`why: keyframe`** and **`class: keyframe`**. Its `run` is the state it
  samples inside, and `classes.keyframe` counts it.
- A run tile that shows a change point keeps `why: boundary`, `class: run`, its offset and its
  label (ADR-0106 D11). Only its `keyframes` list says so.
- `coverage` still counts states: a keyframe tile stands for no state of its own.

### 6. A keyframe tile's label measures its offset from its earliest change point

ADR-0128 §5 gave the form: `12 K 42840ms +27`. The offset field is the tile's *"signed boundary
offset"* (ADR-0098 §3), and a keyframe tile's boundary is its change point, not its state's start.
So the offset is **the painted millisecond less the earliest change point the tile shows**. It says
how far past the change the frame is, which is ADR-0106's honest cost ("up to one frame period past
`t`"). ADR-0106 D10's point at 1013 ms, painted at 1040 ms, reads `… K 1040ms +27`. Where points
share the tile, the earliest gives the largest offset, which is the point the frame is furthest
from.

The label has no identifying field, so it ends after the offset: that is how ADR-0106 D7's blank
slot is drawn, as ADR-0128 §3 already reads it, and no trailing space stands in for it. Its two
forms are one string, and it counts in the label fit at that one length. It can never force the sheet-wide elision, and a keyframe tile is still
fitted: the one type size must hold it.

### 7. The overflow's keyframe fields, and the clause printed only under the flag

Under `--keyframes`, `E-SHEET-OVERFLOW` gains three fields:

| field | value |
| --- | --- |
| `keyframe_tiles` | the keyframe tiles the flag adds |
| `fits_without_keyframes` | whether the run tiles alone draw a sheet, sized as a call without the flag would size them |
| `keyframe_tiles_admitted` | the most tiles the sheet holds, less the run tiles: how many keyframe tiles fit beside them |

`states` counts the visual states needing a run tile, as it did before keyframe tiles existed.
`fits` stays the most tiles the sheet holds.

The template gains clauses that print only when those fields are present. A template marks such a
clause `{?field}…{/field}` (printed when the field is present and not `false`) or
`{!field}…{/field}` (printed when it is present and `false`). An absent field prints neither. On
#407's 18-state document with 13 keyframe tiles, the refusal reads: *"… its 18 visual states need a
tile each and `--keyframes` adds 13 keyframe tiles, and the sheet holds 30 … Without `--keyframes`
the range fits; with it, the sheet holds 12 keyframe tiles beside the run tiles and never drops one
to fit. Ask for these instead, each a sheet that fits: [0, 1600) and [1600, 3600)."*

Without the flag none of the three fields is present, and the text never mentions `--keyframes`.

### 8. A sub-range keeps every keyframe tile of its states

A keyframe tile lies inside one state (ADR-0106 D10), so the tiles a range needs are a sum over its
states: one run tile, plus that state's keyframe tiles. A state with no painted frame weighs
nothing. ADR-0126's cut is made on those weights.

- It uses the same count, `⌈n / fits⌉` upward, and the same even shares, larger first.
- A sub-range after the first opens at the first tiled state reached once the sub-ranges before it
  hold their shares.
- A count is accepted only when **every sub-range's own tile count** fits.

With one tile per state this is ADR-0126's split exactly. A state is never cut, so a heavy one can
carry a sub-range past its share, and the check on each count is what keeps "each sub-range fits"
true. A single state with more tiles than a sheet holds names no sub-range, as a frame too tall for
one tile does. Each sub-range, asked with the flag, draws every keyframe tile counted for it, so an
overflow never thins by proposing ranges that drop some.

A point sampled across a sub-range's end (ADR-0106 D12) is `no-grid-frame` in the earlier
sub-range and shows on no tile of it. In the whole range it shows on the next state's run tile.
Either way it adds no tile, so the weights agree.

## Consequences

- `frame`'s range mode has a `keyframes` module: the population, where each point samples, and
  whether a tile of this sheet shows it.
- `Ask` gains `keyframes`. Without `from`/`to` it is refused as `E-INVOCATION` in the verb, naming
  `--from`/`--to`. The CLI and MCP flags stay with #493.
- `sizing::size` takes a keyframe-tile count per state.
- `E-SHEET-OVERFLOW`'s template gains the flag-only clauses, and the text renderer reads
  `{?…}`/`{!…}` clauses.
- The keyframe tile's strip is inverted as ADR-0128 §5 fixed, and its label carries `K`.
- Two committed fixtures: `fixtures/keyframe-coincidence` (ADR-0106 D11) and
  `fixtures/keyframe-cross-boundary` (D12, both branches), which ADR-0106 made a shipping condition.

## Evidence

- `crates/montagent-core/tests/frame_range.rs`:
  - the coincidence fixture's four tiles, labels, change points and census, with and without the flag;
  - the cross-boundary fixture's tiled and untiled branches, over the whole document and with the
    next run outside the range, with no finding and nothing in `skipped[]`;
  - the main fixture's `0 tiled, 0 untiled` with and without the flag;
  - twelve keyframe tiles on 18 states drawn at the degraded rung, and the thirteenth refused with
    the three fields and the clause, each sub-range drawing its keyframe tiles;
  - an overflow without the flag carrying none of them;
  - the inverted strip;
  - the refusal without a range.
- `tests/cross_verb.rs`: each keyframe tile is `frame --at <its painted ms>`, byte for byte.
- `src/verbs/frame/sizing.rs`: 12 keyframe tiles fit and 13 split `[0, 1600)`/`[1600, 3600)`; a
  heavy state moves the cut; one state past a sheet names no sub-range.
- `src/text.rs`: the conditional clause prints on `true`, on `false`, and on neither when absent.
