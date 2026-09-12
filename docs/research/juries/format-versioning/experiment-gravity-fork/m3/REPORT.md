# Repair report — en-halloween-decorating

## What I changed, element by element

All 8 validator errors were the retired `gravity` field. The validator's own hint —
"which part of the source survives is determined by `x`, `y`, `origin` and `clip`" —
says the replacement mechanism is positioning the element rect relative to the static
`clip` aperture. Every photo element declares a 1080×1912 rect with a frame-space
aperture `clip:[0,0,1080,1300]`, i.e. only 1300 of the 1912 pixels of height are
visible; `gravity` was choosing *which* 1300.

- **photo-05-intro, photo-05, photo-08, photo-05-quiz, photo-05-loop**
  (`gravity:"top"`): deleted the `gravity` field only. With `origin:"top-left"`,
  `x:0, y:0`, the rect's rows 0–1300 already sit inside the clip — the top band is
  what shows, which is exactly what `gravity:"top"` asked for. No geometry change
  needed; removal is behavior-preserving.

- **photo-06, photo-07** (`gravity:"bottom"`): deleted `gravity` **and changed
  `y` from `0` to `-612`**. To show the *bottom* 1300px of a 1912px-tall rect in
  the aperture spanning frame rows 0–1300, the rect's bottom edge must land at
  row 1300: `y = 1300 − 1912 = −612`. Deleting the field alone would have
  silently flipped these two shots from a bottom crop to a top crop — a changed
  video that still validates, the worst outcome.

- **handle-logo** (`gravity:"center"`): deleted the `gravity` field only. Its
  clip `[478,96,68,68]` is exactly its declared rect, so the aperture crops
  nothing and center-gravity is a no-op in rect-vs-clip terms (see caveat under
  "unsure" — this is the one element where the *old* meaning of gravity may have
  been a source-level crop I cannot reproduce).

Formatting: one element per line, key order, indentation and element ordering
are untouched; the only edits are the field deletions and the two `y` values.
The file parses as valid JSON.

## What I was unsure about

1. **What `gravity` used to mean — rect-in-clip alignment vs source-in-rect crop.**
   Two readings were possible:
   - (A) *Rect vs clip*: gravity aligned the element's rect inside the clip
     aperture. This is what the validator's error text points at, and it is fully
     expressible with `y`. I repaired under this reading.
   - (B) *Source vs rect*: gravity chose which part of the source survives a
     `fit:"cover"` crop into the declared rect (CSS `object-position` style).
     Under the current format this is inexpressible directly — `fit` never
     executes and the source is drawn into the declared rect as-is.
   For the photos the two readings almost certainly coincide: the declared
   1080×1912 rect looks like the *already-cover-scaled* source size (source
   scaled to width 1080 yields height 1912), so there is no residual
   source-in-rect crop and all the cropping is done by `clip`. If instead the
   photos' native aspect were wider than 1080:1912, reading (B) would have
   implied a crop I cannot reconstruct without the source pixel dimensions.
   I chose (A) because the validator message states it, and because the numbers
   (1912 vs the 1300 aperture, and top/bottom values on otherwise identical
   elements) only make sense if gravity was selecting the visible band.

2. **`y:-612` vs alternatives for the bottom-gravity photos.** Equivalent
   encodings existed: keep `y` positive with `origin:"bottom-left"` (`y:1300`),
   or `y:-612` with `origin:"top-left"`. I kept `origin:"top-left"` and moved
   `y`: the smaller diff, and consistent with the sibling photo elements. I also
   assumed negative coordinates are legal ("absolute integer pixels" does not
   forbid them, and off-frame placement plus clip is the natural idiom); I could
   not run `montaget validate` to confirm.

3. **Interaction with the `scale` keyframes (1.0 → 1.08 Ken Burns zoom).** I do
   not know the format's scale anchor point (the origin point? rect center?).
   Whatever it is, for photo-06/07 the anchor's frame position now differs from
   before (the rect moved up 612px), so the drift direction of the zoom within
   the aperture may differ slightly from what the old renderer did with
   `gravity:"bottom"` (which plausibly kept the bottom edge pinned during zoom).
   At 8% zoom the divergence is at most ~46px of band drift over the shot. There
   is no field left to express "keep bottom pinned while scaling," so this is
   the closest legal encoding.

4. **handle-logo's center gravity.** If reading (B) applied to it and
   `brand/logo-en.png` is not square, the old renderer center-cropped it into
   the 68×68 circle and the new renderer will draw it into the 68×68 rect
   (possibly distorted). Logos of this kind are almost always square, and the
   `mask:"circle"` plus exact-fit clip suggest the author treated it as square.
   No repair expressible in this format could hedge against this without the
   source dimensions.

5. **Stale `scale` keyframe end-times.** Several elements' second keyframe `t`
   lies beyond the element's `end` (e.g. photo-05-intro: keyframe at t=15000,
   element ends 3018). The validator did not complain, so I treated it as the
   intended idiom (a shared 15s zoom curve sampled only while the element is
   live) and left it alone — repairing it was not asked and might change timing.

## What I wanted and did not have

- **`graveyard.txt`** — named as possibly present; not in this sandbox. I wanted
  the glossary's retirement note for `gravity` to confirm reading (A) vs (B) and
  the prescribed migration recipe.
- **The source images' pixel dimensions** (`images/05..08.png`,
  `brand/logo-en.png`) — to confirm 1080×1912 is the cover-scaled source size
  and that the logo is square.
- **A runnable `montaget validate`** — to confirm the repaired file passes,
  that negative `y` is accepted, and that nothing else regressed.
- **The scale-anchor rule** — to judge how far the zoom drift on photo-06/07
  deviates from the old bottom-pinned behavior, if it was bottom-pinned.

## Confidence

High (~90%) that the static composition is the one the author meant: the
gravity:"top" and gravity:"center" removals are provably no-ops under the
rect/clip model, and the −612 offset is the unique translation that keeps the
bottom band of photos 06 and 07 in the aperture. The residual risks I could not
check: (a) whether the photos' native aspect makes `fit:"cover"` a real crop
(reading B), which would change what "bottom" meant; (b) the exact zoom drift
on photo-06/07, which depends on the unknown scale anchor and may differ from
the old renderer by up to ~46px late in each shot; (c) that the validator
actually accepts negative `y`. I could not run the validator or open any source
asset, so "validates" is asserted from the error list (all 8 errors were
`gravity`, all 8 occurrences removed, JSON re-parsed clean) rather than
observed.
