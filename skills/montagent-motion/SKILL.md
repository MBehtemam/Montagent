---
name: montagent-motion
description: "Motion graphics and titles in Montagent: launch spots, product and social ads, intros and outros, kinetic and typed text, logo reveals and loops, wipes, pops, colour changes, and trailer effects (slammed titles, glitch cuts, HUDs). Use when a Montagent video is built from graphics and text rather than footage or a character."
---

# Motion graphics and titles

Recipes for pieces built from shapes, stills and text. They slot into `montagent`'s loop at the build step; load `montagent` first if you haven't.

Open the reference file for the kind of piece before you plan it:

| The piece | Open |
|---|---|
| Launch spot, product or social ad, intro, outro | [the launch-spot structure](references/launch-spot.md) |
| Logo reveal, logo sting, anything that loops | [the logo loop](references/logo-loop.md) |
| Trailer, or any slammed, glitched, flashing or HUD effect | [trailer effects](references/trailer.md) |

## Plan on a beat sheet

Before you write an element, write the **beat sheet**: every entrance, colour change, cut and exit in one table, each on a beat.

- With music, a beat is `60000 / BPM` ms and a bar is four beats. Without music, use 500 ms (120 BPM): it reads as brisk and lands on whole frames at 24, 25, 30 and 60 fps.
- Main events (a cut, a word landing, a reveal) go on beats. Secondary motion (a settle, a stagger inside a group, a label after its bar) goes on half or quarter beats.
- Snap every time to a drawn frame, `floor(round(t * fps / 1000) * 1000 / fps)`, and confirm one with `measure --at` when in doubt.
- Build the project with a generator script from the beat sheet, so a retime is a one-line edit.

## Recipes

Every recipe gives starting ranges, not answers. Each ends in a **look step**: load `montagent-craft`, then look where the step says before you move on.

### Pop

An entrance with a small overshoot that settles. The overshoot is the curve: one keyframe pair with a bezier whose `y` goes above 1, not three keyframes.

```json
{
  "frame": {"width": 1920, "height": 1080}, "fps": 30, "background": "#101418", "duration": 2000, "output": "out/pop.mp4",
  "tracks": [
    {"name": "badge", "layer": 10, "elements": [
      {"id": "badge", "type": "ellipse", "start": 500, "end": 2000, "x": 960, "y": [{"t": 500, "v": 580}, {"t": 900, "v": 540, "ease": [0.34, 1.56, 0.64, 1]}], "origin": "center", "width": 240, "height": 240, "fill": "#FF5A36", "scale": [{"t": 500, "v": [0.5, 0.5]}, {"t": 900, "v": [1.0, 1.0], "ease": [0.34, 1.56, 0.64, 1]}]}
    ]}
  ]
}
```

- **Length:** 300–450 ms. Shorter snaps, longer floats.
- **Start scale:** 0.5–0.7 for text and cards; 0 for small shapes and badges.
- **Overshoot:** `[0.34, 1.56, 0.64, 1]` peaks about 10 % over. Raise its second number to 1.8 for something playful, or lower it to 1.3 for something corporate. Keep one curve for every pop in a piece.
- **Rise:** 20–60 px of travel on `y` with the same curve adds weight. Skip it on pops that sit in a row.
- **Stagger:** one beat when each item lands on its own beat; 60–120 ms between items inside one beat.
- **Look:** `frame` at the start plus 40 % of the length, where the overshoot peaks, and check nothing clips the frame edge or a neighbour. Then `preview` a span from just before the first pop to just after the last.

### Hold, then clear

An entrance's last keyframe already holds, so a hold costs nothing. An exit starts with a keyframe at the held value, at the moment the exit begins. `validate` reports that keyframe as `R-EASE-INERT`, and the findings guide explains why that's expected.

- **Exit length:** 200–350 ms, `ease-in`. Exits run faster than entrances.
- **Exit move:** a fade plus 20–40 px of travel, or scale down to 0.9. For a hard cut, use a single `step` keyframe to the new value (see the starter project's title).
- **Hold length:** at least one beat longer than reading takes: about 250 ms per word, and never under 1 s for a line.
- **Group exits:** the whole group leaves together, or staggered 30–60 ms in reading order.
- **Look:** `frame` on the last drawn frame before the exit, and check the whole group is fully in and at rest.

### Colour change on a word

Recolour a word inside its line with a `highlight` on that word's run, starting on the beat. This keeps one element: there's no second copy to crossfade, and the element's own colour stays the line's colour.

```json
{
  "frame": {"width": 1920, "height": 1080}, "fps": 30, "background": "#101418", "duration": 3000, "output": "out/accent.mp4",
  "fonts": {"title": [{"file": "fonts/Inter-Bold.ttf"}]},
  "fontVendor": {"fonts/Inter-Bold.ttf": {"licence": "OFL-1.1", "source": "https://github.com/rsms/inter", "sha256": "288316099b1e0a47a4716d159098005eef7c0066921f34e3200393dbdb01947f"}},
  "tracks": [
    {"name": "line", "layer": 20, "elements": [
      {"id": "line", "type": "text", "start": 0, "end": 3000, "x": 960, "y": 540, "origin": "center", "width": 1600, "height": 140, "font": "title", "size": 110, "color": "#F5F0E6", "align": "center", "runs": [{"text": "Video your agent can "}, {"text": "read.", "highlight": {"start": 2000, "end": 3000, "color": "#FF5A36"}}]}
    ]}
  ]
}
```

- **Timing:** the change lands on a beat, never on the word's entrance. Give the eye at least half a beat to read the word first.
- **Emphasis:** when the word is its own element, a scale bump on the same beat (1.0 to 1.06–1.1 and back over 200–300 ms) makes the change land.
- **Look:** `frame` one frame before and one frame after the change, and check only the one word changed.

### Kinetic and typed text

Lines that arrive a word or a letter at a time are set with the bundled script, not written by hand. `python3 scripts/type_on.py <project> <spec>` (run it with `--help` for the spec) measures the line with `measure`, places each word or letter where the whole line would set it, and prints the project with those elements added:

- **Words landing on beats:** `"by": "word"`, `"enter": "pop"`, and one time per word in `times`. A `highlights` entry recolours one word on its beat.
- **Typing:** `"by": "letter"`, 60–150 ms between letters (irregular reads more human: vary ±30 ms), and a `cursor` that blinks at 500–600 ms per cycle while idle, stays solid while typing, and steps ahead of each letter.
- **Tracked titles:** `tracking` adds pixels after each letter. <!-- workaround: #510 · replaced by: a letter-spacing key on text --> <!-- guard-ok: times highlights cursor tracking -->

It writes one element per unit, so `validate` raises the caption checks on every one of them; the findings guide covers that.

- **Look:** `preview` the span of the line coming on. Then `frame` it at rest, and check the spacing reads as one line, with no gap wider at a word or letter boundary.

### Reveal by wipe

A wipe uncovers something by sliding an edge across it. Put an **occluder** above the thing: a rect filled with exactly what is behind the thing. Then shrink the occluder about its far edge, so its near edge sweeps across. An accent bar riding that edge makes the wipe read as deliberate.

```json
{
  "frame": {"width": 1920, "height": 1080}, "fps": 30, "background": "#101418", "duration": 2000, "output": "out/wipe.mp4",
  "tracks": [
    {"name": "mark", "layer": 10, "elements": [
      {"id": "mark", "type": "image", "start": 0, "end": 2000, "source": "media/still.png", "x": 960, "y": 540, "origin": "center", "width": 360, "height": 360, "fit": "contain"}
    ]},
    {"name": "occluder", "layer": 11, "elements": [
      {"id": "occluder", "type": "rect", "start": 0, "end": 1000, "x": 1160, "y": 540, "origin": "center-right", "width": 400, "height": 400, "fill": "#101418", "scale": [{"t": 400, "v": [1.0, 1.0]}, {"t": 900, "v": [0.0, 1.0], "ease": [0.65, 0, 0.35, 1]}]}
    ]},
    {"name": "edge", "layer": 12, "elements": [
      {"id": "edge", "type": "rect", "start": 400, "end": 1000, "x": [{"t": 400, "v": 760}, {"t": 900, "v": 1160, "ease": [0.65, 0, 0.35, 1]}], "y": 540, "origin": "center", "width": 16, "height": 420, "fill": "#FF5A36", "opacity": [{"t": 900, "v": 1.0}, {"t": 1000, "v": 0.0, "ease": "ease-in"}]}
    ]}
  ]
}
```

- **Length:** 400–600 ms, `[0.65, 0, 0.35, 1]`: a sweep that starts and ends gently.
- **Occluder:** 20–40 px larger than the thing on every side, and the same fill as the ground behind it. The start keyframe sits where the wipe begins, so the thing stays covered until then.
- **Edge bar:** 8–24 px wide, 5–10 % taller than the occluder, moving on the same curve and time. It leaves over 100–200 ms once the sweep is done.
- **Direction:** wipe in the reading direction; to wipe off, grow the occluder back from the near edge.
- **Busy ground:** an occluder only works over a flat colour. Over a photo, a gradient or footage, reveal with a pop or a push instead. <!-- workaround: #463 · replaced by: an animated mask -->
- **Look:** `frame` at the middle of the sweep and check the edge bar covers the occluder's edge exactly; then `frame` just after the sweep and check nothing of the occluder is left.

### Grow from an edge

Bars, underlines, dividers and progress lines grow out of the edge they stand on. Set `origin` to that edge (`bottom-center` for a bar on a baseline, `center-left` for an underline) and scale the one axis from 0 to 1.

- **Length:** 400–600 ms, `[0.16, 1, 0.3, 1]` (fast out, long settle).
- **A set of bars:** stagger by a quarter to half a beat, in reading order. The heights tell the story, so keep them honest to the data or the claim.
- **Labels:** each label arrives when its own bar settles (a 150–250 ms fade or a small pop), not with the whole set.
- **Baseline:** it draws first, growing from its centre over 200–300 ms, and the bars start as it lands.
- **Look:** `frame` halfway through the tallest bar's growth and check every bar is anchored to the baseline; then `frame` at rest.

### Push-in on a still

Screenshots and product stills get a slow push while on screen. A crossfade between two stills keeps the push going across the join.

```json
{
  "frame": {"width": 1920, "height": 1080}, "fps": 30, "background": "#F5F0E6", "duration": 5000, "output": "out/push.mp4",
  "tracks": [
    {"name": "shot-1", "layer": 10, "elements": [
      {"id": "shot-1", "type": "image", "start": 0, "end": 3000, "source": "media/still.png", "x": 960, "y": 540, "origin": "center", "width": 720, "height": 720, "fit": "contain", "scale": [{"t": 0, "v": [1.0, 1.0]}, {"t": 3000, "v": [1.036, 1.036], "ease": "linear"}], "effects": [{"name": "mask", "shape": "rect", "radius": 28}, {"name": "shadow", "dx": 0, "dy": 24, "radius": 48, "color": "#101418", "opacity": 0.3}]}
    ]},
    {"name": "shot-2", "layer": 11, "elements": [
      {"id": "shot-2", "type": "image", "start": 2500, "end": 5000, "source": "media/still.png", "x": 960, "y": 540, "origin": "center", "width": 720, "height": 720, "fit": "contain", "scale": [{"t": 2500, "v": [1.03, 1.03]}, {"t": 5000, "v": [1.06, 1.06], "ease": "linear"}], "effects": [{"name": "mask", "shape": "rect", "radius": 28}, {"name": "shadow", "dx": 0, "dy": 24, "radius": 48, "color": "#101418", "opacity": 0.3}]}
    ]},
    {"name": "joins", "layer": 12, "elements": [
      {"id": "join", "type": "transition", "start": 2500, "end": 3000, "kind": "crossfade", "from": "shot-1", "to": "shot-2"}
    ]}
  ]
}
```

- **Push:** 4–8 % over the whole run of stills, `linear`. A push that eases in or out reads as a camera move, not a drift.
- **Across a crossfade:** plan one push from the first still's start to the last still's end. Each still's keyframes are that push's values at its own `start` and `end`, so both stills match through the crossfade.
- **Crossfade:** one beat (400–600 ms), ending on the beat the second still owns.
- **Frame:** a `mask` with a 20–32 px `radius`, plus a `shadow` with `dy` 16–32, `radius` 40–64 and `opacity` 0.25–0.4, in the ground's darkest colour.
- **Look:** `preview` across the crossfade and check the two stills hold the same size through the join, with no jump.
