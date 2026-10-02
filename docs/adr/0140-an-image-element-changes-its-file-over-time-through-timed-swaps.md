---
status: accepted
amends: 0002 (an image's file is no longer one file for its whole range: `swaps` names others, inline, each for a timed window), 0012 (keyframes stay transform-only; a swap is a separate timing construct, and `shift` moves it as it moves a keyframe)
---

# An image element changes its file over time through timed `swaps`

[#621](https://github.com/MBehtemam/Montagent/issues/621), on the map
[#591](https://github.com/MBehtemam/Montagent/issues/591), answering
[#518](https://github.com/MBehtemam/Montagent/issues/518). Until now an element drew one
file for its whole range ([ADR-0002](0002-inline-source-no-asset-table.md)). An image that
changes what it shows, such as a mouth or a phone screen stepping through screenshots, had
to be split into one element per file. Each piece then carried its own copy of the box and
of any motion the run shared. A cut-out owl's mouth was 39 elements.

## The decision

**An `image` element accepts an optional `swaps` list.** Each swap is
`{"start", "end"?, "source"}`: a timed window in which the element draws a different file in
the same box. Outside every swap it draws its `source`, which stays a plain string.

```jsonc
{"id":"screen","type":"image","start":0,"end":6000,"source":"screens/home.png",
 "x":960,"y":540,"origin":"center","width":498,"height":1080,"fit":"contain",
 "scale":[{"t":0,"v":[0.6,0.6]},{"t":5960,"v":[0.898,0.898],"ease":"linear"}],
 "swaps":[{"start":1200,"source":"screens/tap.png"},
          {"start":1360,"source":"screens/loading.png"},
          {"start":3360,"source":"screens/done.png"}]}
```

- **Times** are absolute milliseconds, like keyframe `t`. A swap's window is half-open,
  like a timeline range.
- **`end` is optional.** Without it, a swap runs until the next swap's `start`, or the
  element's `end` if it is the last. With it, the base `source` shows again after `end`.
- **Each swap's `source` is inline,** a path or URL resolved exactly as `source` is
  ([ADR-0053](0053-asset-path-resolution-no-assetroot.md)). There is no table of files.
- **Images only.** `swaps` on any other type is `E-SCHEMA-UNKNOWN-KEY`. Changing a video's
  file is a cut, and a source range per window would be a different feature.
- **Its key position** in `image`'s canonical order is after `opacity` and before
  `effects` ([ADR-0041](0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md)).

**The limit.** A swap changes the file, never the box. It is stepped, never blended or
interpolated, and nothing drives it: no expression, no index, no reference to another
element. Every frame a swap list draws can already be drawn by abutting image elements, so
this adds no capability ([ADR-0003](0003-general-video-editor-not-channel-tooling.md)
is not engaged). It only re-spells a run of hard cuts. A proposal to blend, deform or drive
swaps is a new decision, not a widening of this one.

**There is no "nothing drawn" state.** A swap list always has a base image. Omitting
`source` so that only the swaps draw, as an overlay that is blank between windows, stays
open on the map's fog. It would be a backward-compatible widening.

### `validate`

- **`E-SWAP-ORDER`:** the list is not sorted by `start`. There is no silent sort and no
  "later wins". An implicit `end` is the *next* swap's `start`, so an unsorted list would
  resolve to windows nobody wrote.
- **`E-SWAP-OVERLAP`:** two swaps share an instant, which only an explicit `end` can cause.
  `end == next.start` is not an overlap and is not flagged.
- **`E-SWAP-RANGE`:** a swap is empty, or falls outside the element's own `[start, end)`.

All three are refuse-class, on `E-HIGHLIGHT-OVERLAP`'s reasoning: the document does not say
which number is the wrong one. Every check that reads an image's file reads **every** swap's
`source` too. That includes the missing-file check, `E-FIT-DEVIATION` (the renderer stretches
whatever file it gets into the declared box), the media digest and `R-SOURCE-CUT-POP`, which
compares the file showing as one element ends with the file showing as the next begins.

### Every other reader of `source`

Every tool that reads an image's file must see its swaps: `frame`, `render`, contact sheets,
`query --at` (which reports the file showing at that instant), `timeline`, `chroma`, `verify`
and `keyed`. A reader that still reads only the base draws or reports the wrong file in
silence, so the implementation is not complete until all of them do.

### `shift`

`shift` moves a swap as it moves a keyframe
([ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md)):

- a swap wholly after `at` moves by `delta`;
- on an element that straddles `at`, a straddling swap lengthens (its explicit `end`
  moves, or it lengthens implicitly), and later swaps move;
- a swap starting exactly at `at` is treated as a keyframe at `t == at` is.

**Removing time** is specified now and binding when it ships: a swap inside the removed span
is dropped, and a straddling swap shortens, or is dropped if its `end` would fall before its
`start`. Today `shift` refuses every negative `delta` for every element, so swaps open no
hole that keyframes have not already got. The routine is meant to be shared with
`highlight` windows, whose own `shift` gap is
[#613](https://github.com/MBehtemam/Montagent/issues/613).

## Why this shape

1. **A window over a base, not a keyframe.** ADR-0012 keeps keyframes transform-only, and
   [ADR-0048](0048-per-word-highlighting-is-a-timed-window-on-the-run.md) already refused to
   copy the keyframe shape for a value that isn't a transform. Interpolation exists only for
   numbers, so a source keyframe would need `"ease":"step"` on every record. A swap is
   ADR-0048's `highlight` construct moved from a run to an image.
2. **Not a list in `source`.** Every consumer reads `source` as a string, so a list there
   would be silently skipped by the missing-file and fit checks.
3. **Not a sprite sheet.** A keyframed cell index is the opaque id ADR-0002 rejects, and it
   needs a packing step.
4. **`end` optional rather than required.** The two variants tied on the edit test. The
   optional form writes less, and no run hit the `end == next.start` hazard the required
   form was there to prevent.

[#595](https://github.com/MBehtemam/Montagent/issues/595) records these rejections in full,
with its jury's ballots.

## What it buys, and what it doesn't

**The gain is in edit size only.** It is not in cost, turns or reliability, which were
equal across spellings. The evidence is in `docs/research/prototypes/swaps/`, and
`python3 docs/research/prototypes/swaps/edit_size_check.py` re-derives every figure below
and exits non-zero if one stops reproducing.

The request was to insert an error screen into a phone mock-up that steps through four
screenshots under one push-in (`P-edit-error.md`). Nine seeded agent runs
([#597](https://github.com/MBehtemam/Montagent/issues/597)) landed 3/3 in every spelling
(`runs/scores.txt`):

| spelling | changed lines, agent runs | minimum correct edit |
|---|---|---|
| split | 68, 68, 68 | 66 |
| split inside a container carrying the push-in | not run | 35 |
| swaps, `end` optional | 30, 26, 26 | 24 |
| swaps, `end` required | 29, 29, 29 | 27 |

**Most of the split's excess is the container's to remove, not this ADR's.** The split
copies the push-in's keyframes onto every screen. A container that carries the push-in
(#499's question, [#632](https://github.com/MBehtemam/Montagent/issues/632), undecided) takes
the minimum from 66 lines to 35. Swaps owns the rest, 35 to 24: the box each new screen would
otherwise copy, and one element in place of one per screen. If the format stays flat, swaps
keeps the whole 66 to 24.

On the owl, swaps folds the 39 mouth elements into one. It does not remove the baker, because
the owl's body is baked. That is #499's to remove.

**When to revisit.** If a container lands and a re-score with agent runs shows swaps saving
less than a fifth of the container spelling's lines, or the readers above cost more to keep
in step than this ADR expects, a later ADR may supersede this one and remove `swaps`.

## Consequences

- **Glossary.** **Swap** is added to `CONTEXT.md`, and **Source** points to it.
- **The prototype is not the implementation.** `prototype/swaps` (pinned at `c6e7b285`)
  showed the shape builds and renders pixel-identically to the split. The work that lands it
  on `main` is a separate spec slice, and it covers every reader listed above.
