# Survey: how declarative JSON video APIs model a timeline

Gathered while resolving [Name the core domain model](https://github.com/MBehtemam/Montagent/issues/4).
Companion to [the renderer survey](./renderer-survey.md), which covered *renderers*
and *interchange formats*; this one covers **authoring APIs** — products whose
input is a JSON document describing a video.

All claims below come from the vendors' own documentation, fetched September 2026.
Items marked **NOT CONFIRMED** could not be established from an official source and
are recorded as gaps rather than guesses.

## Shotstack

`Edit → timeline → tracks[] → clips[] → asset`.

- **Tracks are a constraint.** *"Do not place clips on a track so that their start
  or end times overlap. This will result in the clips flickering because the API
  does not know which clip to display."*
  ([core concepts](https://shotstack.io/docs/guide/getting-started/core-concepts.md))
  Enforced by consequence, not by validation. **NOT CONFIRMED:** whether the API
  rejects overlapping clips.
- **Z-order: lower array index draws in front** — the inverse of most intuitions.
  *"Tracks are an array within the timeline, the first element is the top track and
  the last element is the bottom most track."*
- **Timing: absolute, with opt-in sequencing.** `start`/`length` are seconds; both
  accept `"auto"`/`"end"`. *"For multiple clips, the `auto` setting stitches clips
  sequentially according to their order in the track."*
  ([smart clips](https://shotstack.io/docs/guide/architecting-an-application/smart-clips.md))
  Ripple exists but only for `auto` clips.
- **No grouping noun.** Track → clip is the whole hierarchy.
- **Assets inline** per clip (`clip.asset.src`). Its `alias` mechanism references
  *clips for timing*, not assets.
- **NOT CONFIRMED:** the normative `Track`/`Timeline` schema text; the API
  reference page truncates before the Schemas section.

## Creatomate (RenderScript)

Root object → `elements[]`; `track` is an integer *property on each element*.
Compositions nest and carry their own timeline.

- **Tracks are a sequencing mechanism — the strongest of the four.** *"Tracks allow
  you to arrange elements to play in sequence. When multiple elements share the
  same track number, they play one after another."*
  ([the timeline](https://creatomate.com/docs/api/render-script/the-timeline))
  The engine enforces it even against explicit times: *"An element followed by
  another on the same track is capped at the follower's start."*
  ([durations](https://creatomate.com/llms/durations.md))
- **Z-order:** `z_index` overrides definition order, with a sharp edge for
  generated JSON — *"ALL z-indexed children draw above every non-z-indexed
  sibling, whatever the value"* ([elements](https://creatomate.com/llms/elements.md)).
  **NOT CONFIRMED:** that a higher track number draws in front. Strongly implied by
  the mask wording (*"the element is used as a mask for the element one track
  below it"*) and a watermark recipe, but never stated.
- **Timing: both.** A number is absolute; `null` sequences after the previous
  element on the same track.
- **Grouping noun: yes** — the `composition` element. *"Scenes are compositions
  placed in sequence on track 1."*
- **Assets inline** (`source`). No table.
- Creatomate publishes an **LLM-facing reference** at
  [llms.txt](https://creatomate.com/llms.txt), more detailed than its human docs —
  evidence that models authoring this JSON directly is the expected usage.

## Editly

Edit spec → `clips[]` → `layers[]`. *"track"* refers only to audio.

- **Non-overlap is structural, not a rule.** Clips have no `start` at all; the
  array *is* the timeline. Confirmed against
  [src/types.ts](https://github.com/mifi/editly/blob/master/src/types.ts).
- **Z-order: array order within a clip, last on top.** No integer.
- **Timing:** clips purely sequential; layers relative to their parent clip via
  `start`/`stop`. Ripple is total and unavoidable.
- **Grouping noun: yes** — the clip is both the segment and the sequencing unit.
- **Assets inline** per layer (`path`).
- **The cost of the model, visible in the API:** nothing can outlive its clip, so
  independent audio needed a separate `audioTracks[]` array and a `detached-audio`
  layer type to bridge the two systems.

## JSON2Video

`movie → scenes[] → elements[]`, plus a movie-level `elements[]` overlaying every
scene.

- **No track concept.** Every occurrence of "track" in the full documentation
  corpus refers to audio or to job tracking.
- **Z-order: array order, later paints on top**, with an integer override where
  *higher* draws in front — the opposite direction to Shotstack.
  ([basic concepts](https://json2video.com/docs/v2/getting-started/basic-concepts))
- **Timing: absolute within the container, sequential between containers.**
  *"This time is relative to the beginning of the scene it's in or, if the element
  is part of the movie's elements array, relative to the beginning of the movie."*
  Ripple is scene-level only.
- **Grouping noun: yes** — `scene`, with its own clock, background and duration.
- **Assets inline** (`src`).

## What the four agree and disagree on

**Nobody uses a track as a pure z-lane.** Every track-like concept carries a
non-overlap constraint. Shotstack and Creatomate put that constraint on a
horizontal lane spanning the timeline; Editly and JSON2Video have no tracks and
hoist it onto a vertical segment spanning the canvas. These are transposes: a lane
model makes "music across everything" trivial and "swap scene 3" hard; a segment
model does the reverse. Both patch the gap — JSON2Video with movie-level elements,
Creatomate with top-level tracks.

**Z-order direction has no convention.** Three answers among four products.
Shotstack's first-is-top is the outlier and the most likely thing for a generator
to get backwards.

**Assets are inline, four for four.** Not one has a table of files referenced by
id — despite all four being HTTP services, which is the case where a table would
help most.

**Three of four have a grouping noun** (Creatomate `composition`, Editly `clip`,
JSON2Video `scene`); only Shotstack has none.

## What an LLM finds hard in each

- **Shotstack** — absolute seconds are easy to compute; the reversed track order
  contradicts the near-universal last-drawn-wins convention, and the overlap rule
  fails as silent flicker rather than an error.
- **Creatomate** — omitting timing gives a very short prompt-to-JSON path, but the
  model must then simulate a multi-rule duration cascade to predict output at all.
  The vendor's own docs call this *"the number-one source of broken renders"* and
  warn that smaller models *"produce broken RenderScript without warning."*
- **Editly** — smallest and least ambiguous; almost any valid spec is sane. But
  anything spanning clip boundaries has no expression at all.
- **JSON2Video** — maps onto HTML/CSS models the LLM already holds, so intuitions
  transfer with little correction. But nothing catches an element timed against the
  movie when the docs specify the scene: the render succeeds and is simply wrong.
