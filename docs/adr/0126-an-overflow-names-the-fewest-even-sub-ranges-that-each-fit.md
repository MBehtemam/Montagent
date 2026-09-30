---
status: accepted
amends: 0095 (section 3's "names sub-ranges that would fit" is given its algorithm — the fewest sub-ranges, tiles shared as evenly as they go, cut at a state's start and covering the range exactly), 0105 (`limit`'s two values are spelled `tile-width` and `type-floor`, and the sub-ranges' field is `sub_ranges`, rendered as a prose list), 0125 (section 5's template closes with the sub-ranges rather than "ask for a narrower range")
---

# An overflow names the fewest even sub-ranges that each fit

> **Amended by [ADR-0129](0129-an-untiled-keyframe-point-is-named-in-the-census-and-never-in-skipped.md).** A state carrying keyframe tiles weighs more than one tile, and
> the cut is made on those weights, each sub-range's own count checked.

[#489](https://github.com/MBehtemam/Montagent/issues/489), a ticket under spec
[#486](https://github.com/MBehtemam/Montagent/issues/486). ADR-0095 §3 says a range that does not
fit at the tile-width refusal is refused, *"and the refusal names sub-ranges that would fit"*.
ADR-0105 says those sub-ranges are finding fields and that ADR-0095 owns how they are found.
ADR-0095 never says. #486's Further Notes 1 and 2 list the gap, and under ADR-0031 it is a spec
gap until an ADR ratifies it. This one does.

## Decision

### 1. The fewest sub-ranges, tiles shared as evenly as they go

Let `n` be the visual states that need a tile, and `fits` the most tiles a sheet of this frame
holds (ADR-0125 §5). The refusal names **`k` sub-ranges**, where `k` is the least count from
`⌈n / fits⌉` up for which a sheet of `⌈n / k⌉` tiles and a sheet of `⌊n / k⌋` tiles both fit.
The first `n mod k` sub-ranges take `⌈n / k⌉` tiles, and the rest take `⌊n / k⌋`.

In practice `k` is `⌈n / fits⌉`: under the research model a sheet of fewer tiles never serves
narrower. The check makes *every sub-range fits* a property of the construction rather than of
that model. `sizing.rs` scans for `fits` rather than solving for it, for the same reason.

**Even rather than greedy.** Greedy packing gives the same count of sub-ranges and worse sheets.
The 31 portrait states of the refusal's test split greedily into 30 and 1: a degraded sheet at
141 px and a sheet of one tile. Split evenly, they are 16 and 15, and both are drawn at the
180 px target. The caller makes the same number of calls either way. The budget's currency is
tile width (ADR-0095 §1), so the split shares tiles, not milliseconds.

### 2. Cut at a state's start, and cover the range exactly

A sub-range after the first **starts where its first tiled state starts**. The first starts at
`from`, and the last ends at `to`. So the sub-ranges are consecutive, half-open, and their union
is `[from, to)`:

- **No state is cut.** Each boundary is one the selection already made. Requesting a sub-range
  gives the same visual states, because neighbouring states already differ and nothing new
  merges. So each sub-range draws the tiles this answer counted for it, and each one fits.
- **No boundary is dropped.** Every visual state lies in exactly one sub-range. Overflow never
  shrinks the boundary set, even silently through the remedy it proposes (ADR-0094).
- **A state no frame paints goes with the sub-range before it.** It needs no tile, so it moves
  no count. The first sub-range also keeps any such state before its first tile. Its
  `N-QUANTIZATION` is raised by whichever sheet contains it.

The boundaries are the document's milliseconds, not frame instants. A state's start is where
the cut list changes, which is the one place the selection reads it the same way twice.

The sub-ranges give up ADR-0095 §3's cross-tile relation between states that land on different
sheets. That is the `across-sheets` blind spot, stated on every answer. The caller can still
narrow the range another way. The sub-ranges are the one next call that is certain to fit.

### 3. The spellings

- **`sub_ranges`**: an array of `{from, to}`, in order. `states` and `fits` keep ADR-0125's
  names. They are the visual-state count and the tiles admitted that #486 asks for.
- **`limit`** is **`tile-width`** when ADR-0095's 140 px served tile width bound, and
  **`type-floor`** when ADR-0098's 8 px served type bound. `limit_px` carries the bound in
  served pixels either way. This build raises only `tile-width`. The label's numeric core is
  #490's to size, and #490 raises `type-floor` under this spelling.
- The plain text renders `sub_ranges` as prose: `[0, 3200) and [3200, 6200)`. The renderer
  keys this on the field's name, not its shape, because other templates interpolate the
  document's own JSON, and a value there that happened to look like a list of ranges must
  still print as written. Every other array stays compact JSON. An empty `sub_ranges` reads
  `none`. That happens only when a single tile of the frame
  cannot serve at the limit, for example a frame so tall that one tile is too narrow. No range
  of that document fits, and the refusal says so rather than naming a range that would also be
  refused.

The template ends *"Ask for these instead, each a sheet that fits: {sub_ranges}."* in place of
ADR-0125's *"ask for a narrower range."*. The keyframe-only clause (ADR-0106) is #491's.

## Consequences

- `sizing::size` takes the visual states and the fps and counts the tiled states itself. An
  `Overflow` carries its sub-ranges, so the sizing module's table tests cover them across state
  counts and fps values.
- `E-SHEET-OVERFLOW` gains `sub_ranges`, and its template names them.
- The text renderer reads `sub_ranges` as half-open ranges in prose.

## Evidence

`crates/montagent-core/src/verbs/frame/sizing.rs` holds the table tests: the split's counts,
its boundaries, the unpainted states it carries at 25, 24 and 30 fps, and the empty list.
`tests/frame_range.rs` requests every named sub-range in turn and asserts that each draws a
sheet, and it checks the refusal's plain text.
