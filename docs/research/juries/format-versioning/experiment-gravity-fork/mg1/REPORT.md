# Repair report — `project.json`

## Summary

Eight elements carried a `gravity` field, which the format no longer has. I deleted
the field from all eight and changed nothing else. The file is valid JSON, all 60
elements are still one-per-line, key order is untouched, element order is untouched,
and a structural diff confirms the two files are identical once `gravity` is removed
from both.

## What I changed, element by element

| Element | Track | Removed | Why |
|---|---|---|---|
| `photo-05-intro` | photo (10) | `"gravity":"top",` | field retired; no replacement field exists |
| `photo-05` | photo (10) | `"gravity":"top",` | same |
| `photo-06` | photo (10) | `"gravity":"bottom",` | same — **see uncertainty #1** |
| `photo-07` | photo (10) | `"gravity":"bottom",` | same — **see uncertainty #1** |
| `photo-08` | photo (10) | `"gravity":"top",` | same |
| `photo-05-quiz` | photo (10) | `"gravity":"top",` | same |
| `photo-05-loop` | photo (10) | `"gravity":"top",` | same |
| `handle-logo` | handle-logo (31) | `"gravity":"center",` | same |

Each deletion removed the key, its value and its trailing comma, leaving
`"fit":"cover","clip":[...]` adjacent. No whitespace, indentation, ordering or other
key was touched. 8 lines changed, 0 lines added or removed.

### Why deletion rather than migration

`gravity` meant *which part of a source survives a crop*. In this format the visible
region is fully determined by `x`, `y`, `origin`, the declared `width`/`height`, and
the static frame-space `clip`; `fit` is only a claim about how `width`/`height` were
derived and never executes. There is therefore no field that `gravity` maps onto —
the quantity it named has no remaining degree of freedom. The only way to express
"show me the lower part of this source" in the current format is to change the rect
itself (`y`, or `height`, or `clip`), which is a different repair and is discussed
below.

## What I was unsure about

Listed most-to-least consequential.

### 1. `photo-06` and `photo-07` had `gravity:"bottom"`, the other six did not

This is the only place where deletion might not be behaviour-preserving, and it is
the thing I would most want a human to look at.

Reconstructing the old semantics: a source of W×H was cover-scaled into the declared
1080×1912 rect at `s = max(1080/W, 1912/H)`, the overflow was cropped, and `gravity`
chose which edge the surviving band was anchored to. The frame-space `clip`
`[0,0,1080,1300]` then showed the top 1300 rows of that rect.

Two readings, and I cannot distinguish them from inside this directory:

- **(a) `gravity` was inert.** If the sources are exactly 1080×1912 (or exactly that
  aspect ratio, 0.5649), cover is an exact fit, the overflow is zero pixels, and
  `top` / `bottom` / `center` all render identically. Deletion is then an exact
  no-op for all eight elements.
- **(b) `gravity` was load-bearing for 06 and 07.** If the sources are taller than
  0.5649 the crop is real, and deleting `bottom` moves those two photos' content
  down inside the clip by the overflow height. The magnitude is whatever the
  overflow was: for a 1080×1920 source it is 8 px (visible only under scrutiny);
  for a materially taller source it could be large.

I chose deletion for both anyway, for four reasons:

1. There is no legal way to say "bottom" in the current format without editing the
   rect, and **every rect edit needs the source's pixel dimensions, which I do not
   have.** Guessing dimensions to compute a `y` offset would be fabrication, and a
   wrong guess damages a photo that is currently correct-or-nearly-so.
2. The seven photo elements are byte-identical in `x`, `y`, `origin`, `width`,
   `height`, `fit` and `clip` — they differ *only* in id/group/start/end/source and
   in this one word. If `bottom` were deliberate art direction for two specific
   images, I would expect the framing fields to differ too. They never do.
3. The glossary excerpt I was given says agents reach for `gravity` by **copying a
   neighbouring element**, and that its eight occurrences left the fixture in the
   same change that retired it. Both the "copied, not authored" provenance and the
   "removed wholesale" precedent point at deletion.
4. Deletion is uniform. Deleting six and compensating two would produce a file where
   two photos were framed by a rule I invented and could not verify.

### 2. Whether the correct new-format expression of these elements is a *different* rect

Arguably the format-native way to write "a cover-cropped photo" now is: declare
`width`/`height` as the **full cover-scaled source size** (which overflows the frame),
position it with `x`/`y`/`origin`, and let `clip` be the aperture that does the
cropping. Under that reading, `width:1080, height:1912` is suspicious — it is the
*post-crop* rect, a leftover of the old model, and `fit:"cover"` as a derivation
claim would be false unless the source really is 0.5649 aspect.

I rejected rewriting the rects. It needs source dimensions I do not have; it would
touch seven elements the validator did not flag, against the instruction not to
change what I was not repairing; and if reading (a) above is true it is unnecessary.
But I want it on record that a maintainer with the images might decide this file
needs that larger migration and that my minimal repair only silences the error.

### 3. Whether `fit:"cover"` should also have changed

Candidates: leave it, change to `"contain"`, change to `"fill"`, or delete it. I left
it. `fit` never executes and the renderer never reads it, so no choice here can change
the video; the validator does not flag it; and `"cover"` remains a defensible claim
about how 1080×1912 was derived. Changing it would have been a cosmetic edit to an
unflagged field.

### 4. `handle-logo`'s `clip`, and whether `gravity:"center"` mattered there

`handle-logo` is a 68×68 rect at (478, 96) with `clip:[478,96,68,68]` — an aperture
exactly equal to the rect, i.e. a no-op clip — plus `mask:"circle"`. For any square
logo, cover into a square crops nothing and `center` was inert; deletion is exact.
Only a non-square `logo-en.png` would make it matter, and a non-square logo in a
circle-masked square slot is unlikely to be what was intended anyway. I also
considered deleting the redundant `clip`, and did not: it was not flagged and removing
it changes nothing.

### 5. Things that look odd but validate, which I deliberately left alone

- Every photo's `scale` ramp ends well past the element's `end` (e.g. `photo-05-loop`
  runs 64016–65216 but ramps to t=79016; `photo-05-intro` ends at 3018 but ramps to
  15000). This is a consistent 15-second Ken-Burns ramp truncated by the cut,
  applied identically to all seven photos. Clearly deliberate, not flagged, untouched.
- `photo-05-intro` and `photo-05` are the same source and the same rect split across
  a cut at 3018 with separate restarting ramps — the scale visibly snaps back to 1.0
  at 3018. Intentional-looking (the hook lands there), not flagged, untouched.
- `validate-output.txt` names the file `en-halloween-decorating.montaget.json`, while
  the file I was given is `project.json`. I assumed these are the same file renamed
  and did not rename anything.

## What I wanted and did not have

1. **The pixel dimensions of `images/05.png`, `06.png`, `07.png`, `08.png` and
   `brand/logo-en.png`.** This is the one missing fact that would settle uncertainty
   #1 outright. With them I could compute the cover overflow and state definitively
   whether `gravity:"bottom"` was inert or worth compensating. I had no image files
   and no manifest of source dimensions.
2. **ADR-0015 itself** (which the glossary excerpt cites, and which I did not fetch,
   per the rules). I wanted it for two specific things: the exact *old* semantics of
   `fit` + `gravity` — in particular whether `gravity` anchored the source inside the
   rect (my assumption above) or the rect inside the frame, which changes the analysis
   completely — and whether the retirement change prescribed any mechanical rect or
   clip compensation for elements whose `gravity` was not the default.
3. **The old format's default `gravity`.** If the default was `"top"`, the five
   `"top"` elements are provably no-ops and the blast radius shrinks to exactly two
   elements. I assumed nothing about the default and so had to treat all eight as
   potentially meaningful.
4. **A renderer, or before/after frames.** Nothing in this report is visually
   verified; I could not render either version and could not diff pixels.

## Confidence

- **That the file now validates:** high. The eight reported errors were all the same
  single-field error and all eight occurrences are gone; the validator reported
  nothing else ("8 errors, 0 review, 0 notes").
- **That it is well-formed and convention-preserving:** certain, checked
  mechanically. It parses; 60 elements; one element per line; elements still sorted
  by `start` within each track; key order preserved; a field-stripped structural
  comparison against the original is exactly equal.
- **That the video is the video the author meant:** high for six of the eight
  elements — the five photos that said `"top"` (where `top` agrees with a clip that
  already takes the top 1300 rows, so the two rules point the same way) and
  `handle-logo` (square slot, no-op clip).
  **Medium and conditional for `photo-06` and `photo-07`.** If the sources are
  1080×1912 they are untouched; if they are taller, those two photos now show a band
  from higher up the image than before, by an amount I cannot compute.

**What I could not check, plainly:** I never saw the images, so I could not compute
the cover overflow, could not confirm that `fit:"cover"` was ever a true claim about
these rects, and could not verify that `gravity` was inert rather than load-bearing.
I could not run the validator or the renderer — I am reporting an expected clean
validate, not an observed one. And I could not confirm my reconstruction of the old
`fit`+`gravity` pipeline against the ADR that retired it.
