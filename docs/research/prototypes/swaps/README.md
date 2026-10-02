# Prototype: `swaps` (#614)

Throwaway. This branch (`prototype/swaps`) builds the shape [Which shape lets an element change its image over time?](https://github.com/MBehtemam/Montagent/issues/595) shortlisted, so that [Score the image-change shape against the baker](https://github.com/MBehtemam/Montagent/issues/597) has something to score. Nothing here merges to `main`. The ADR and the **Swap** glossary entry wait for the score.

## The shape

`swaps` sits on `image` elements only: a sorted list of `{start, end?, source}` over a base `source`, in absolute milliseconds. A swap without `end` runs until the next swap's `start`, or the element's `end` if it's the last. A swap with `end` falls back to the base after it. Every swap draws in the element's one declared box. Both variants share one schema; the project file picks the variant, not a build flag.

| Where | What it does now |
|---|---|
| renderer (`frame`, `render`, sheets) | draws the swap holding the instant, else the base |
| `validate` | missing-file check and `E-FIT-DEVIATION` on every swap's `source`; `E-SWAP-ORDER`, `E-SWAP-OVERLAP`, `E-SWAP-RANGE`; `swaps` on any other type is `E-SCHEMA-UNKNOWN-KEY` and nothing else; `end == next.start` is not flagged |
| `R-SOURCE-CUT-POP` | compares the file showing as A ends with the file showing as B starts |
| media digest | hashes every swap's file |
| `query --at` | a `source` field on raster rows: the file showing |
| `shift` | swaps move as keyframes do (see below) |

**`shift`:** an element wholly after `at` moves every swap. On a straddler, every `start` and `end` after `at` moves by `delta`. So a swap holding `at` lengthens, and a swap starting exactly at `at` stays and lengthens, as SPLIT holds a keyframe at `t == at`. An `end` exactly at `at` stays (half-open). Swaps sitting at `at` are listed in the coincident preamble. **Removal isn't built:** `shift` refuses a negative `delta` outright, so the drop-and-shorten half of the rule has nothing to attach to. `highlight` windows still don't move ([shift does not move a run's highlight window](https://github.com/MBehtemam/Montagent/issues/613) hasn't landed), so no routine is shared yet.

**Not taught about swaps:** `timeline`'s source column, `chroma`, `verify`, `keyed` and the audio mix still read the base `source` only. None of them touches the score's cases.

## Fixtures

Regenerate with `python3 make_fixtures.py`, then `montagent fmt` each `.montagent.json`. The committed files are what get scored.

- `owl/hoot-swaps.montagent.json` (`end` optional) and `owl/hoot-swaps-ended.montagent.json` (`end` required): the pinned baked owl (sha256 `93c1b627…`) with its 39 mouths as one element `owl-mouth`, base `mouth_closed`, and 36 swaps. A baked mouth that *is* `mouth_closed` becomes the end of the swap before it. The blinks stay split and the body stays baked. `brand`, `character` and `stills` are symlinks to `skills-eval/assets`.
- `pip/pip-split.montagent.json`, `pip/pip-swaps.montagent.json` and `pip/pip-swaps-ended.montagent.json`: a phone mock-up at 25 fps stepping home → tap (160 ms) → loading (2 s) → done, under one linear push-in. `done.png` is 1080×2400 where the others are 1080×2340, so `E-FIT-DEVIATION` fires in every spelling (the split's `screen-done` shares the box, so the spellings stay pixel-equal). `screens/error.png` is there for the score's insert-a-screen edit.
- `pip/pip-group.hypothetical.json`: the split inside a group carrying the push-in. **No build reads it.** The `group` type and its `elements` key don't exist; it's there to be counted, so the swap isn't credited with #499's container.

## Pixel equivalence

`python3 check_pixels.py <montagent> A B FROM TO` draws every painted frame of both files with `frame --full --png` and compares all four channels exactly.

| Pair | Result |
|---|---|
| baked owl vs `hoot-swaps` (0–8000) | 225 / 240 identical |
| baked owl vs `hoot-swaps-ended` (0–8000) | 225 / 240 identical |
| `pip-split` vs `pip-swaps` (0–6000) | 150 / 150 identical |
| `pip-split` vs `pip-swaps-ended` (0–6000) | 150 / 150 identical |

**The 15 owl frames that differ are the two pauses,** 1933–2333 ms and 4500–4533 ms. The baked owl draws no mouth there, and one element with a base draws `mouth_closed`. The head paints only a beak, so there's no rest mouth for the base to coincide with. That's the "no nothing-drawn state" the resolution accepted, and its fog patch (a swap list with no base image) is what would close it.

**The owl's rotation is the baked mouths' own records, joined** (100 records, 23 of them `step` at the old cuts), not the head's. The baker rounds each mouth's copy, so the copies drift from the head by up to 0.007° at painted frames. That's sub-pixel, but it moves anti-aliased edges in 77 frames. Joining the copies keeps the frames equal. An author following the head would write the head's 56 records instead.

## Face counts (the score's job, given for orientation)

| | elements | keyframes | bytes | `R-EASE-INERT` |
|---|---|---|---|---|
| baked | 44 | 109 | 11,604 | 18 |
| `swaps`, `end` optional | 6 | 110 | 7,096 | 12 |
| `swaps`, `end` required | 6 | 110 | 7,437 | 12 |

Measured with #592's split; the body is unchanged (58 reviews).
