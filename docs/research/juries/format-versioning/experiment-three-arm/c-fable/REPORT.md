# Render fix — en-halloween-decorating

## What I did, and why

**Diagnosis.** The build box runs `montaget 0.9.2`. Its validator rejected exactly two keys — `fit` and `clip` — on all 8 image elements (7 photos + the handle logo), and refused the render. Nothing else in the file was flagged, so every other key in the project (`mask`, `speed`, `scale` keyframes, `runs`, `origin`, etc.) is known-good on this binary. The build box hasn't changed in months; the project file evidently comes from a newer authoring vocabulary than the renderer installed here. With no network I can't upgrade montaget, and I don't reliably know what 0.9.2's older spelling of these concepts was — guessing legacy key names risks a second failed night or, worse, a silently wrong video. So I re-expressed the same picture using only vocabulary this exact binary has already accepted.

**The fix, in two moves:**

1. **Removed `fit:"cover"` and `clip:[...]` from all 8 image elements.**
   - On the 7 photo elements, `clip:[0,0,1080,1300]` served one purpose: hide the photo below y=1300 so the cream `#FBF3E3` background forms the bottom panel behind the captions and sentence cards (the intro title's dark-blue-on-cream styling only works if that band is cream), and contain the 1.08 Ken Burns zoom inside the same window. The clip rectangle's left/right/top coincide with the element and frame bounds, so its *only* visible effect is the y=1300 bottom edge.
   - On `handle-logo`, `clip:[478,96,68,68]` exactly equals the element's own bounds — a geometric no-op. The circular look comes from `mask:"circle"`, which 0.9.2 accepts and which I kept.
2. **Added a `photo-mask` track at layer 15** — a single full-duration rect, x=0 y=1300, 1080x620, fill `#FBF3E3` (the project background color). It sits above the photo track (layer 10) and below everything else (cards at 20, text at 21–23, header at 30+). Since photos are the only visual elements below layer 20, this rect reproduces the clip's one visible effect pixel-for-pixel, including containing the zoom, for the full 0–65216 ms.

The result validates as JSON and uses no element key beyond the set the 0.9.2 validator already accepted in the failed run's own output.

## What I considered and rejected

- **Translating `fit`/`clip` into presumed 0.9.2-era keys** (e.g. some `gravity`/`crop` spelling). Rejected: I have no spec on this box, and a guessed key is either rejected again (another lost night) or — far worse — accepted with different semantics and ships a wrong video without an error.
- **Upgrading montaget.** No network; nothing to install from.
- **Just deleting the keys and rendering.** Deleting `clip` alone would paint photo all the way down to y≈1912, putting the caption text on photo instead of the cream panel — legible-ish but visibly *not* the published video. The mask rect closes that gap.
- **Editing photo geometry instead** (height 1300 instead of 1912). Rejected: it changes how `fit`-less scaling maps the source into the box (1080x1300 is a very different aspect than 1080x1912), so the visible crop of each photo would shift. The overlay rect changes nothing about how photos are sampled.
- **Doing nothing / waiting for the author.** The video has to go out today; the change is minimal, reversible, and documented here.

## What I would tell the team

The overnight render failed because the project file uses the newer `fit`/`clip` vocabulary but the build box is still on montaget 0.9.2, which rejects unknown keys. I shipped tonight's video by removing those two keys and adding a background-colored masking rect at layer 15 that reproduces the photo clip exactly, using only 0.9.2-accepted primitives — the rendered pixels should be identical. Please either upgrade montaget on the build box or pin the authoring side to the 0.9.2 vocabulary, and eyeball tonight's output before it goes wide; the temporary `photo-mask` track can be deleted once the box understands `clip` natively.

## Confidence, and what I could not check

**High (~90%) that the output matches the previously published video.** The clip reproduction is exact by construction: the mask rect covers precisely the region the clip hid, in the same color the clip revealed, on a layer that occludes only photos. Residual risks I could not check, because the binary, source images, and old spec are not available to me:

1. **`fit:"cover"` removal on the photos.** If the source PNGs are exactly 1080x1912 (or that aspect), dropping `fit` is a no-op. If they're 1080x1920, a stretch-to-box default would distort by ~0.4% vertically — imperceptible — but a substantially different source aspect would visibly distort. I could not open `images/*.png` to confirm.
2. **`fit` removal on the 68x68 logo** — same reasoning; a non-square `brand/logo-en.png` would squash slightly inside its circle mask.
3. **0.9.2's default when `width`/`height` are given without `fit`** — I assume stretch-to-box (the near-universal default); if it letterboxes instead, photos could show cream gutters.
4. **I could not run `montaget render` at all** — the binary is not present in my working directory, so the fix is verified against the validator's own error list (only `fit`/`clip` were unknown; every key I now use was accepted), not against an actual render. First action on the real box: run the render and eyeball frame ~1s (photo/panel boundary at y=1300), ~11s (first sentence card), and the header logo.
