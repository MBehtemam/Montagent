# Repair report — en-halloween-decorating.montagent.json

All 8 errors were the same one: `gravity` is no longer a field of the format.
The validator's own hint says its job — *which part of the source survives* — is
now carried by `x`, `y`, `origin` and `clip`. So the repair was not "delete the
field" everywhere; it was "re-express each `gravity` value in geometry, then
delete the field", which is a no-op for some elements and a real move for others.

## The reading I repaired under

For the seven photos: the element rect is 1080x1912 placed on a 1080x1920 frame,
with a static aperture `clip:[0,0,1080,1300]`. `fit:"cover"` is (per the format
facts) only a *claim* that 1080x1912 is the source scaled to cover the 1080x1300
aperture at its own aspect ratio; it never executes, so the declared rect holds
the **whole** scaled source and the 612 px of overflow is disposed of purely by
where the rect sits relative to the aperture. That makes `gravity` exactly a
statement about which 1300 px band of the 1912 px rect lands inside the aperture:

- `gravity:"top"` -> the rect's top edge flush with the aperture's top -> the
  existing `y:0, origin:"top-left"` already says this. Nothing to move.
- `gravity:"bottom"` -> the rect's **bottom** edge flush with the aperture's
  bottom (frame y = 1300) -> the existing geometry says the opposite, so the
  element had to be re-anchored.
- `gravity:"center"` on `handle-logo` -> its `clip` (`[478,96,68,68]`) is exactly
  its rect, so nothing is cropped at all and gravity had nothing to decide.

## What I changed, element by element

| Element | Change | Why |
|---|---|---|
| `photo-05-intro`, `photo-05`, `photo-08`, `photo-05-quiz`, `photo-05-loop` | deleted `"gravity":"top"` only | Geometry already expresses top gravity: rect top at y=0, aperture top at y=0. Deletion is pixel-for-pixel a no-op. |
| `photo-06`, `photo-07` | deleted `"gravity":"bottom"`; `"y":0 -> "y":1300`; `"origin":"top-left" -> "origin":"bottom-left"` | Bottom gravity means the bottom 1300 px of the source is what shows. Anchoring the rect's bottom-left at (0,1300) puts the rect at frame y -612..1300, so the aperture `[0,0,1080,1300]` now frames the source's bottom band instead of its top band. `x`, `width`, `height`, `clip`, `scale` untouched — the on-screen window is in the same place, only the picture behind it is the half the author asked for. |
| `handle-logo` | deleted `"gravity":"center"` only | Clip equals the rect exactly; no crop exists for gravity to steer. No-op. |

Nothing else was touched: key order, one-element-per-line layout, `start` ordering,
indentation, and every other field are byte-identical to the input. The file parses
as JSON (checked) and contains no remaining `gravity`.

## What I was unsure about — exhaustively

**1. Whether `gravity:"bottom"` was ever visible at all (the biggest risk).**
There is a rival reading in which, in the *old* format, `fit` did execute: the
source was cover-cropped into the 1080x1912 rect and `gravity` chose which part of
the *source* was discarded by that crop, entirely independently of `clip`. Under
that reading my repair is wrong, and the correct repair is not expressible without
the source images' pixel dimensions — which I do not have. I rejected it because
the format facts state plainly that `fit` never executes and the renderer never
reads it, and because the validator explicitly hands gravity's job to
`x`/`y`/`origin`/`clip`. But this is the assumption my whole repair rests on, and
I could not test it.

**2. `y:1300 + origin:"bottom-left"` vs. `y:-612 + origin:"top-left"`.**
These describe the *identical* rect at scale 1.0, so both are legal and both
satisfy the errors. I chose the bottom-left anchor because it dominates: the two
differ only if `scale` is applied about the `origin` point, and in that case the
bottom-left anchor keeps the bottom edge flush through the whole 1.0->1.08 zoom —
the mirror image of what the top-gravity photos do, where the top edge stays
flush. If `scale` is applied about the rect centre or the rect's top-left instead,
the two encodings are indistinguishable, so I lose nothing. The format facts do
not say what `scale` scales about, so I could not confirm this; it is an
inference, not a fact.
Residual risk of this choice: it assumes `"bottom-left"` is a legal `origin`
value. The file only ever uses `top-left`, `center`, `center-left`, so I am
inferring a 3x3 vocabulary (the retired `gravity` values `top`/`bottom`/`center`
support that). If the validator rejects `bottom-left`, the drop-in fallback is
`"y":-612,"origin":"top-left"` on those same two lines. I also weighed that
`y:-612` needs a negative coordinate, which is nowhere stated to be legal either;
I preferred the risk that sits on an enum I can reason about.

**3. Moving the element vs. moving the aperture.** `clip:[0,612,1080,1300]`
would also expose the source's bottom band — but `clip` is in *frame* space, so
that moves the visible window 612 px down the screen, over the sentence card.
Rejected as changing the video.

**4. The top-gravity elements: is deletion really a no-op?** Only if "top gravity"
meant flush-to-aperture-top, which the current `y:0` already encodes. If gravity
had instead been measured against the frame or against some pre-crop box, deletion
could silently change them. I have no way to distinguish these, and deletion is
the only change that leaves the file self-consistent with its top-gravity
siblings.

**5. `handle-logo`'s `fit:"cover"` + `mask:"circle"`.** A 68x68 rect declared
`cover` from a presumably non-square logo is a slightly suspicious claim, but
`fit` is inert and the clip crops nothing, so the gravity deletion is safe
regardless. I left the `fit` claim alone: it is not an error, and TASK.md says not
to repair what I was not asked to.

**6. Out-of-scope oddities I deliberately did NOT touch.** Every photo's `scale`
keyframe pair ends far past the element's own `end` (e.g. `photo-05-intro` ends at
3018 but its second keyframe is at t=15000), so every zoom is truncated mid-flight.
`photo-05-quiz` (68856) and `photo-05-loop` (79016) even keyframe past the
project `duration` of 65216. This looks like a deliberate slow-Ken-Burns idiom
rather than a bug, the validator does not flag it, and it is not part of the
gravity repair — but it is the reason the zoom anchor in point 2 actually matters
on screen.

## What I wanted and did not have

- **`graveyard.txt`.** TASK.md says some sandboxes get an excerpt of the glossary;
  mine did not. It would presumably contain the retirement note for `gravity` —
  the one document that would say authoritatively what `gravity` used to do and
  what the sanctioned mechanical rewrite is. Everything in section 2 above is me
  reconstructing that note from the validator's one-line hint.
- **The pixel dimensions of `images/05-08.png`** (and `brand/logo-en.png`). With
  them I could confirm that 1080x1912 is the true cover-scale of each source
  against the 1080x1300 aperture, which would settle uncertainty 1 outright. I
  have no image files in my directory.
- **The rule for what `scale` scales about** (origin point, rect centre, or rect
  top-left). This decides uncertainty 2.
- **The legal `origin` vocabulary.** I inferred `bottom-left` exists.
- **The validator itself.** I have its previous output but cannot re-run
  `montagent validate`, so "the repaired file validates" is reasoned, not observed.
- I did not consult any Montagent repository, ADR, spec or website.

## Confidence

**High that the file now validates** — the only reported error class is gone,
the file is well-formed JSON, and the writing convention is preserved. The single
way it could still fail is if `"bottom-left"` is not a legal `origin` (fallback
given above).

**Moderate-to-high that it is the video the author meant.** Six of the eight
elements were provable no-ops. For `photo-06` and `photo-07` I am confident about
the *intent* (the author asked for the bottom of those two images and the old file
said so in a word the format no longer has) and about the frame-1.0 geometry;
I am less confident about the zoom anchor over their 13-second and 12-second lives.

**What I could not check, plainly:** I never ran the renderer or the validator and
never saw the images. I cannot confirm that 1080x1912 is the uncropped scaled
size of each source rather than an already-cropped one — if it is already cropped,
`photo-06` and `photo-07` are now showing the wrong band of their source and my
repair made the video worse, not better, in a way no amount of reading this file
would reveal. Someone with the image files should eyeball those two shots.
