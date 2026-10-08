# The Montagent project format: transitions

A page of `montagent://format.md`, which holds the rules every element shares and lists the
other pages; read it first. This page holds the `transition` element that bridges two others,
for the picture and for the sound. Like the rest of the format docs, every rule here is an
accepted decision in the ADR series, cited inline by number, and the ADR is right where the
two disagree.

## Transitions

- **A transition is its own element bridging `from` and `to` by id** (ADR-0059): two
  distinct visual elements on separate tracks, its `start`/`end` exactly the time they share.
- **`kind`** (ADR-0150): `crossfade` ramps their opacity linearly; in a `slide` `to` moves in
  over a still `from`; in a `push` `to` moves in and pushes `from` out, joined; in a `wipe` a
  straight edge crosses the frame, `from` keeping the side not yet reached and `to` the side
  passed.
- **`direction`** is the way the motion travels: `left` means the content moves leftward;
  the incoming element enters from the right. Required on `wipe`, `slide` and `push`.
- **`ease`** takes the keyframe vocabulary; omitted means `linear`. Neither `direction` nor
  `ease` is allowed on a `crossfade`.
- **Offsets travel a full frame width or height, outside the element's transform,** in whole
  pixels: a lower-third slides as far as full-screen video, its own keyed motion, effects
  and `clip` moving with it. A wipe's hard edge is cut against each element's own `clip`.
- **In a slide, `to` must paint above `from`**; a transition never changes a layer.
- **`audio` carries the sound across the same window**: `cut`, `constant_power`
  or `constant_gain`. `from` fades out and `to` fades in over `start`..`end`, so a trim of
  either element moves the one window. Optional on `crossfade`, `wipe`, `slide` and `push`;
  absent means `cut`, the hard cut under the picture. Write `constant_power` on every new
  transition between sounding clips, and `constant_gain` only when both sides are the same
  source (a cutaway back into one take). An element's `volume` multiplies with the fade, and
  `R-TRANSITION-VOLUME-STACK` reviews a `volume` that changes inside the window.
- **`audio_crossfade` is a fifth `kind` for sound alone**: it paints nothing and
  bridges `audio` or `video` elements (a video's embedded audio fades, its picture is
  untouched), over the same exact window. `audio` is required on it and is `constant_power`
  or `constant_gain`; `cut` is refused, and `direction` and `ease` are not fields of it.
  `ease` is picture-only: it never bends the sound. A J-cut or L-cut, whose window is not the
  overlap, stays hand-written `volume` keyframes.
