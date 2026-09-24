# Repair report — en-halloween-decorating project

## What I changed, element by element, and why

`montagent validate` reported the same error on 8 elements: `` `gravity` is not
a field of this format. Which part of the source survives is determined by
`x`, `y`, `origin` and `clip`. ``

I removed the `"gravity":"..."` key from each of the 8 offending elements,
leaving every other key, value and the surrounding formatting untouched:

- `photo-05-intro` — removed `"gravity":"top"`
- `photo-05` — removed `"gravity":"top"`
- `photo-06` — removed `"gravity":"bottom"`
- `photo-07` — removed `"gravity":"bottom"`
- `photo-08` — removed `"gravity":"top"`
- `photo-05-quiz` — removed `"gravity":"top"`
- `photo-05-loop` — removed `"gravity":"top"`
- `handle-logo` — removed `"gravity":"center"`

No other field was touched, no element was reordered, and no line was
re-wrapped or re-indented.

**Why deletion, and not translating `gravity` into some other field:** the
validator's own error message tells you where the responsibility now lives —
`x`, `y`, `origin` and `clip` — and for every one of these 8 elements those
four fields are already fully specified and internally consistent with a
`gravity`-free reading of the element:

- All 7 photo elements place the source image at `x:0, y:0, origin:top-left,
  width:1080, height:1912` — i.e. the full declared rect is pinned to the top
  of the frame — and then apply the identical frame-space aperture
  `clip:[0,0,1080,1300]`. This is true whether the old `gravity` was `"top"`
  (`photo-05-intro`, `photo-05`, `photo-08`, `photo-05-quiz`,
  `photo-05-loop`) or `"bottom"` (`photo-06`, `photo-07`). Since the rect
  placement and the clip aperture are byte-for-byte identical across both
  groups, `gravity` was not distinguishing anything that `x`/`y`/`origin`/
  `clip` weren't already pinning down on their own — under the stated fact
  that `fit` never executes and the declared rect is authoritative, there is
  no remaining hook for `gravity` to act through. It was inert with respect
  to these four fields before I touched anything.
- `handle-logo` places a 68×68 image at `x:478,y:96,origin:top-left` and its
  `clip` is `[478,96,68,68]` — exactly equal to the rect. An aperture equal
  to the rect crops nothing, so `gravity:"center"` had no visible effect to
  preserve there either.

Given that, simple deletion is the only repair that satisfies "make it
validate without changing the video": every other repair I could imagine
(e.g. re-deriving a `clip` offset from `gravity`) would have *introduced* a
frame-space crop that isn't implied by anything else in the file, which risks
changing the video rather than preserving it.

## What I was unsure about

- **The biggest open question:** `gravity` historically (in whatever engine
  version wrote this file) selected which part of the *source pixels* would
  survive a `fit:"cover"` resize into `width`×`height`, independent of the
  frame-space `clip`. That is a source-space decision, and nothing in the
  current file records the source images' native pixel dimensions or aspect
  ratio. If `images/06.png` and `images/07.png` have a different aspect
  ratio from `images/05.png`/`images/08.png`, then under the *old* renderer
  `gravity:"bottom"` on `photo-06`/`photo-07` may have meant "keep the bottom
  of the source image, crop the top" — a real visual difference from
  `gravity:"top"` — and that distinction is not expressible in the new
  format at all (per the given facts, there is no field left that acts on
  the source before it's placed in `width`×`height`; only `x`/`y`/`origin`
  and the frame-space `clip` matter, and `fit` is now a non-executing label).
  If that reading of the old format is right, this migration is *lossy* —
  the difference between `photo-05`'s "top" framing and `photo-06`/
  `photo-07`'s "bottom" framing was baked into the old renderer's source
  crop, and it cannot be reconstructed from x/y/origin/clip alone without
  knowing the source images' true pixel size. I chose deletion anyway because
  (a) it's the only repair actually available given the stated facts, and
  (b) it's the one explicitly pointed to by the validator's own error text.
  But I could not confirm whether the pre-migration and post-migration videos
  actually look the same for `photo-06` and `photo-07`.
- A weaker, alternative reading is that `gravity` was already vestigial
  authoring metadata at the time this file was written — e.g. left over from
  an editor UI that recorded "which crop handle the author dragged" for
  provenance/undo purposes, with the actual crop already baked into
  `width`/`height`/`clip` by the time the file was saved. Under this reading
  the identical `clip` across `top` and `bottom` elements is expected (not a
  fluke), and deletion is exactly correct with no loss at all. I have no way
  to distinguish this from the lossy reading above without the source image
  dimensions or the old renderer's spec, so I flagged both.
- `fit:"cover"` remains on all 8 elements after my edit. It's declared inert
  by the facts I was given ("It never executes and the renderer never reads
  it"), and it wasn't flagged by the validator, so I left it untouched rather
  than speculating about whether it should be updated to reflect the loss of
  `gravity`. If `fit` is meant to be replaced/deprecated too, that's outside
  what `validate-output.txt` told me to fix.

## What I wanted and did not have

- The **native pixel dimensions of `images/05.png`, `images/06.png`,
  `images/07.png`, `images/08.png`**, which would tell me whether
  `fit:"cover"` into a `1080×1912` box ever needed to crop the source at all,
  and if so by how much — which is exactly the information `gravity` used to
  disambiguate. Without it I can't confirm whether removing `gravity`
  actually changes any rendered pixel for `photo-06`/`photo-07`.
- `graveyard.txt` (the format's retired-terms glossary) was mentioned as
  possibly present in some sandboxes but was not present in mine; an entry
  for `gravity` there might have stated explicitly whether its removal was
  designed to be a no-op (e.g. "gravity was already superseded by clip
  before this release") or a known lossy migration, which would have
  resolved the uncertainty above directly instead of leaving it inferred.

## Confidence

High confidence that the file is now syntactically and structurally correct:
it is valid JSON, keeps one element per line, keeps elements sorted by
`start` within each track, and keeps stable key order everywhere except the
removed key. I did not run `montagent validate` myself (I don't have the
binary), so I can't directly confirm the 8 errors are gone, but each removed
key exactly matches the field name and location the validator complained
about, and no other structural rule was touched.

Moderate-to-high confidence that the *video* is unchanged for 6 of the 8
elements (`photo-05-intro`, `photo-05`, `photo-08`, `photo-05-quiz`,
`photo-05-loop`, `handle-logo`), since their `clip`/rect values are
consistent with `gravity` already being inert there. Lower confidence for
`photo-06` and `photo-07` (the two `gravity:"bottom"` elements): I could not
verify against the source images' actual dimensions whether the old renderer
was actually applying a bottom-aligned source crop that the new file can no
longer express. That is the one thing in this repair I could not check.
