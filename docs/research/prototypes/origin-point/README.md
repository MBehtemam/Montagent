# Prototype: `origin` accepts a point (#612)

Throwaway branch `prototype/origin-point`, built for [Score the free pivot against the
baker](https://github.com/MBehtemam/Montagent/issues/598). It is never merged. It builds
the shape decided in [Does origin accept a free pivot point?](https://github.com/MBehtemam/Montagent/issues/596).

## What the branch builds

- **`origin` on image, video, rect and ellipse** is a keyword or `[px, py]`: integer pixels
  in the unscaled box, measured from its top-left. The point may lie outside the box, and
  it is static. This is `BoxOrigin` in `crates/montagent-core/src/model/mod.rs`.
- **Text stays keywords-only.** Its schema error names only the nine keywords.
- **Schema errors** for a non-integer, a wrong-length or an unknown value name the nine
  keywords and the `[px, py]` form. No finding is raised for a point outside the box.
- **The renderer** gets the origin as box pixels (`Transform::origin`), so keywords and
  points resolve through one function (`geometry::origin_offset`). `query`'s rect maths and
  the aperture-coverage checks read the same function.
- **Same-source cut:** two keywords compare as keywords, as before. Once a point is
  involved, the box-local resolved points are compared, so `"top-left"` against `[0, 0]`
  is not a trigger.
- **`fmt`** leaves both forms as written.
- **`format.md`** documents the point. **The baker** (`bake_rig.py`) takes an optional
  `joint` per part and writes it as the point.

Tests are in `crates/montagent-core/tests/origin_point.rs`. The rest of the suite passes
unchanged, golden frames included.

## The unpadded owl

| What | Where |
|---|---|
| Unpadded parts for the score, 5-px grid | `docs/research/skills-eval/assets/character/unpadded/` (`rig.json` + `parts/`) |
| Its bake, from the pinned `hoot.base.json` and `hoot.spec.json` | `hoot.unpadded.montagent.json` |
| Parts for the equivalence pair, even grid with a 4-px margin | `docs/research/skills-eval/assets/character/unpadded-even/` |
| The equivalence pair | `hoot.padded.exact.montagent.json`, `hoot.unpadded-even.exact.montagent.json` |

The bake commands and the counts:

```sh
python3 unpad.py ../../skills-eval/assets/character/rig.json <out> 5 0
python3 unpad.py ../../skills-eval/assets/character/rig.json <out> 2 4 zero
# in a workspace holding brand/, character/, stills/ and the pinned run's hoot.*.json:
python3 bake_rig.py hoot.base.json character/unpadded/rig.json hoot.spec.json
```

- **The score's owl counts exactly what #592 counted.** The body has 6 elements and 594
  keyframes, the face 44 elements and 109 keyframes, the scene 6 and 6. `validate` gives
  76 `R-EASE-INERT` and 6 `N-TRACK-GAP`, as before.
- **Only `origin`, `width`, `height` and `source` differ.** The body is 21,766 bytes against
  21,715, and the face 11,888 against 11,604.
- **The baker's joint gap is unchanged** at 1.21 px (forearm_left at 2566).

## The done-check, and why it is a pair

**The pinned owl can't be matched pixel for pixel by any integer point.** The reason is
arithmetic:

- Its boxes are `round(source × 0.6)`. So the torso's 1408 px becomes 845, a ×0.60014
  stretch, and its centre falls at 422.5, a half pixel.
- A trimmed part can't reproduce that stretch: 845 and 1408 share no factor.
- An integer point can't reach the half pixel.

**So the render is proven equal on a pair that differs only in padding.** The human ruled
for this on #612. Both owls draw each part at its source size with `scale: [0.6, 0.6]`
(`exact.py`):

- the padded one keeps `"center"`, which lands on whole pixels because every padded PNG has
  even sides;
- the unpadded one uses the joint point.

The cut is even-aligned from the PNG's top-left, so Skia's mip level 1 lines up, and it
keeps a 4-px transparent margin, so edge clamping doesn't bleed art outward.

`compare.py`, 60 instants (every 4th frame of 0–8000 ms), full scale, PNG
(`compare.log`):

| Pair | Pixels differing per frame (of 2,073,600) | Largest difference |
|---|---|---|
| **Equivalence:** padded vs unpadded, both exact-scale | median 161, max 311 | **1 / 255** |
| Score owl (`hoot.unpadded`) vs the pinned owl | median 78,637, max 81,938 | 64 / 255 |
| Encoding alone: the pinned owl vs the padded exact-scale owl | median 62,931, max 65,699 | 58 / 255 |

What the rows show:

- **The equivalence pair differs only by f32 rounding** in the composed matrix, nothing
  larger.
- **The score's owl differs from the pinned owl about as much as the padded owl does when
  only its box encoding changes.** That difference is the box rounding: sub-pixel
  antialiasing on edges, with no art misplaced.
- **The one identical frame in each row is 0 ms**, where the owl is off screen.

## What this prototype found on the way

- **A tight crop is not enough.** Odd offsets misalign Skia's mip level 1, which cost up
  to 27/255. A crop with no margin lets edge clamping bleed art outward, which cost up to
  12/255. An agent trimming its own art would hit both, so it matters for any skill that
  teaches the point form.
- **A coarse grid cut into the art.** Clamping the crop to the PNG first cut a few pixels
  off `forearm_left`. Now the crop may run past the PNG edge, and `unpad.py` asserts that
  the art is kept.
