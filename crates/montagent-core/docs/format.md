# The Montagent project format: the rules the schema cannot express

The JSON Schema — served beside this document as `montagent://schema.json`, and in pieces
listed by `montagent://schema/index.json` — is the authority on **shape**: which keys exist
on which element type, which are required, and which values are admitted. It is generated
from the same Rust types that parse your file, so it cannot fall out of step with what
`validate` enforces.

This document is the other half. Everything below is a rule a schema has no way to say:
relationships between fields, between elements, and between the file and the bytes on disk.
Breaking one of these produces a legal-looking document that is the wrong video.

Read the schema for *what may be written*. Read this for *what it will mean*.

**Where these rules come from.** Every one of them is an accepted decision in Montagent's ADR
series, cited inline below by number. The ADRs are the specification and this document is
their agent-facing reading, not a second authority: where the two ever disagree, the ADR is
right. A test in `montagent-core` re-reads every ADR cited here and on the pages below, and
fails if one of them has stopped being accepted, so a rule that is quietly superseded cannot
go on being taught here.

## The format's pages

This page holds the rules every element shares. The rest are on pages of their own, each
served beside this one and each fitting one read. Read a page before you write what it covers:

- **`montagent://format/text.md`**: a `text` element. Runs and line breaks, direction and
  `align`, captions, `letter_spacing` and ligatures, `line_height`, fonts and their vendoring.
- **`montagent://format/compositing.md`**: how elements combine. Paint and gradients, effect
  order, `blend` and shadows, masks, `chroma`, and transitions.

## How you edit a project

**With your ordinary file tools** (ADR-0011). There is no CRUD API and there are no field-level edit
tools, deliberately: a tool that took an element id and a field name would cost you the
`diff` that proves your edit changed that element **and nothing else**. Montagent's tools
exist for the things a text editor cannot do — checking, measuring, querying, rendering.

That model has one prerequisite, and it is the reason for the writing convention below:
**your next exact-string replace must match exactly one element.**

### The canonical writing convention

A project file is written so that each element is a unique, matchable substring
(ADR-0005, ADR-0041, ADR-0007).

- Two-space indent. The header's keys one per line. `tracks` expanded, each track expanded.
- **Every element on exactly one line, whatever its length** — runs, keyframes, effects and
  all. There is no width threshold, deliberately: a formatter that reflowed long elements
  would be non-idempotent, and adding three characters would reflow an element into eight
  lines with a diff that has nothing to do with your edit.
- An element's own JSON is written *tight* — `{"id":"card-05","type":"rect",…}`, no spaces
  after `:` or `,`. The header and the track keys are written *spaced*. The difference is
  what makes an element line matchable; the same fields with spaces would be equally legal
  JSON and every replace you had written would find nothing.
- **Key order is the schema's property order** (ADR-0041). Not alphabetical, not the order you typed.
  Every type begins with the universal prefix `id, type, group, start, end, layer`, then
  its own fields, then `effects` last.
- One trailing newline, LF line endings, non-ASCII written raw rather than escaped.

`montagent fmt` restores all of this; `montagent fmt --check` reports what it would change
without touching the file. `validate` reports a departure from the convention under the
`LAYOUT` category on every run — which is **not** a severity and never blocks a render. A
file written outside the convention is unsafe to *edit*, not unsafe to *render*.

Two things `fmt` will never do to your file, both of which are content rather than
formatting: it never adds or removes a field that has a default (ADR-0030), and it never
rewrites a value — including choosing between `cover` and `contain` (ADR-0026) where an aspect ratio makes both
exactly true.

### Presence is content (ADR-0030)

Omitting a defaultable field and writing it explicitly at its default value are **two
different declarations**. Omission says *"give me whatever the default is"*; an explicit
`"opacity": 1` says *"I have pinned this to 1"*. They diverge the moment the default is
revisited. Nothing in Montagent will convert one into the other, `create_project` included:
the scaffold writes `background`, `duration` and `output` exactly when you asked for them.

## Time

- **One clock** (ADR-0005). Every time in the file is absolute integer milliseconds on the project's
  single timeline. There are no nested timelines, no per-track clocks and no relative times.
- **Ranges are half-open: `[start, end)`** (ADR-0005). An element whose `end` is 7500 is not on screen
  at 7500, and a neighbour starting at 7500 is. So a cut where one range ends and the next
  begins names a single instant — not an overlap, and not a one-millisecond gap.
- **There is no `duration` field on an element.** Its length is `end - start`, and a tool
  that wants the length computes it.
- **A keyframe's `t` is not a timeline time** (ADR-0012). It is the element's own animation geometry
  and travels with the element, which is why `shift` moves elements and carries their
  keyframes along. A keyframe outside its element's range is legal and ordinary: it is how
  a trimmed move is spelled.
- **An animatable property is one the schema types as a literal or a keyframe list**
  (ADR-0146); a list on any other field is a schema error. Today: `x`, `y`, `scale`,
  `rotation`, `opacity` on every visual element; `volume` on `video` and `audio`; `width`,
  `height`, `fill`, `stroke`, `stroke_width` on a `rect` or `ellipse`, and a `rect`'s
  `radius`; a `path`'s `fill`, `stroke`, `stroke_width` and `points`; a `text` element's
  `color`, `stroke`, `stroke_width` and `letter_spacing`; every numeric and colour
  parameter inside `effects`, named `effects[1].radius (blur)` (see the compositing page).
  The box of an `image`, `video`, `text` or `path`, a path's `closed`, `clip`, enums, run
  and highlight paint, and a gradient's parameters stay static. A run's paint
  still beats the element's keyed value; where every
  run overrides it, `validate` says so (`R-TEXT-PAINT-OVERRIDDEN`).
- **A keyed colour blends in sRGB with premultiplied alpha** (ADR-0146), the CSS rule:
  each component clamped to its range, then rounded to the nearest byte; six digits are
  opaque. A fade from `#FF0000` to `#00000000` stays red, and `query --at` prints its
  midpoint as `#FF000080` — always a literal you can paste back.
- **A keyed size is continuous, and at or below zero draws nothing that frame**
  (ADR-0146). `width`, `height`, `radius` and `stroke_width` keyframes are non-negative
  integers (`0` is legal); between them the value is never rounded. Under an overshooting
  ease a `width` or `height` at or below zero draws nothing, as `opacity` 0 does; `radius`
  clamps to half the shorter side and `stroke_width` to `0`. A box with no positive size at
  **any** frame of its range is `E-NOT-PAINTED-NO-EXTENT` from both `validate` and `render`.
- **`shift` splits every keyframe list it cuts into literals** (ADR-0146): an integer to the
  nearest (ties away from zero), a colour to bytes. A split the field cannot hold — a size
  an overshoot carried below zero — is refused (`E-SHIFT-SPLIT-UNWRITABLE`), never clamped.
- **Slack is invariant** (ADR-0032). The distance from one boundary to the next is content, not
  padding a tool may absorb. `shift` refuses an edit that would change an existing slack's
  size rather than quietly taking up the difference.

## Tracks, elements and stacking

- **A track supplies stacking, never timing** (ADR-0004). It has no start, no duration and no clock.
- **Array order carries no meaning** (ADR-0060) — not for timing and not for stacking. Reordering the
  elements in a track changes nothing about the video.
- **Children of one track may not overlap in time** (ADR-0004). That is a validation error, and it is
  what tracks are for. Elements that should overlap belong in different tracks.
- **A gap in a track is legal and ordinary** (ADR-0006) — the silence between two narration lines is a
  gap. A gap is never an error, which is why it is reported apart from an overlap.
- **`layer` is an integer; higher draws in front.** It normally lives on the track. An
  element may override it with its own integer or with an anchor.
- **A layer tie is an error once the boxes actually overlap** (ADR-0060) in both time and space,
  because at that point the document does not say which draws in front. Two elements
  resolving to the same layer whose boxes never meet is fine.
- **An anchor is one hop, never a chain** (ADR-0019). `{"below": "title"}` resolves to that element's
  layer minus one. Its target must carry a plain integer `layer`; an anchor pointing at
  another anchor, at itself, or at a missing id is an error. A track's `name` and an
  element's `id` are separate namespaces — an anchor resolves only against an element `id`.
- **Every element carries a required, unique `id`,** whatever its type. Its only job is to
  be a target.
- **`group` is render-inert.** It says two elements belong to one authorial unit and
  nothing more — no timing, no stacking, no drawing consequence.

## Geometry

- **An element's size is declared, never inferred from its source file.** `width` and
  `height` are what the element occupies; Montagent will not read a PNG and fill them in.
- **`origin` is the point of the element's own box that `x`,`y` places** (ADR-0013), and the point
  transforms pivot about. Nine keywords, vertical first: `top-left`, `top-center`,
  `top-right`, `center-left`, `center`, `center-right`, `bottom-left`, `bottom-center`,
  `bottom-right`. The middle elides to `center` alone — `center-center` is an error naming
  it, because two spellings of one value break the write-read round trip.
- **There is exactly one transform per element, and it is flat** (ADR-0012). `x`, `y`, `origin`,
  `scale`, `rotation`, `opacity`, as fields on the element. No transform is nested,
  inherited or composed, and no element's transform is relative to another's.
- **`scale` is always `[sx, sy]`** (ADR-0012), never a bare number.
- **`clip` is a static frame-space rectangle** and never animates (ADR-0025): it is simultaneously the
  aperture the source is drawn through and the fixed denominator a `fit` claim is checked
  against. It does not rotate and does not scale with the element — which is exactly what
  makes a Ken Burns move work: `scale` moves the picture behind a window that stays put.
- **`fit` is a claim about how you computed `width`/`height`, not a layout mode** (ADR-0015). Montagent
  checks your arithmetic against the source's real dimensions; it does not do the fitting
  for you. Where an aspect ratio makes `cover` and `contain` exactly equivalent, both
  spellings stand and nothing will normalise one to the other. **No renderer reads it**: the
  declared rect is authoritative, so a source is resampled to exactly `width`×`height` and
  `frame` draws the same picture whether `fit` says `cover`, says `contain`, or is absent.
- **`radius` is a field on `rect` and not on `ellipse`** (ADR-0014). A single integer,
  defaulting to 0 — one corner radius, not four. An ellipse inscribes its declared rect and
  has no corners to round, so `radius` on one is an unknown key rather than a field that
  quietly does nothing.
- **`stroke` never enlarges the declared rect** (ADR-0014). On a `rect` or `ellipse` it falls inside
  it; on text it falls outside the glyph contour and grows into the box rather than past it;
  on a `path` it is centred on the outline, and `validate` checks the box contains it (below).

### Paths (ADR-0154)

- **A `path` draws a list of vertices in integer pixels from its declared box's top-left
  corner.** It takes a `rect`'s fields except `radius`, plus a required `closed` (a boolean)
  and a required `points` list. The transform places the box exactly as a `rect`'s; the box
  **bounds** the drawing and never stretches it, so `width`, `height` and `closed` are
  static. Resize a path by editing its points, or animate `scale`.
- **A vertex is `{"at": [x, y], "in": [dx, dy], "out": [dx, dy]}`**, every value an
  integer, and no other key. `in` and `out` are optional **handles**: offsets from their
  own vertex. A missing handle is a zero offset, so a vertex with neither is a corner, and a
  written `[0, 0]` draws the same.
- **Each segment is a cubic Bezier** from one vertex's `at` through `at + out`, then the next
  vertex's `at + in`, to that `at`. A closed path adds the segment from the last vertex back
  to the first, using the last `out` and the first `in`. There is no `line` or `polygon`
  type: **a line is a two-vertex open path** with no handles, and a polygon is a closed path
  of corners. An open path needs two vertices and a closed one three
  (`E-PATH-TOO-FEW-POINTS`); an `in` on an open path's first vertex or an `out` on its last
  shapes nothing (`E-PATH-DANGLING-HANDLE`).
- **From SVG:** a `C c1 c2 p` segment from the vertex `prev` sets `prev.out = c1 − prev.at`,
  then adds a vertex `at = p` with `in = c2 − p`; an `L p` segment adds a vertex at `p` with
  no handles. A `d` string is not a value.
- **`fill` needs `"closed": true`**; on an open path it is a schema error. Fill uses the
  nonzero winding rule, so a self-intersecting outline fills its overlap. A gradient `fill`
  or `stroke` is measured against the declared box, as on every shape.
- **The stroke is centred on the outline**, open or closed, with a round join and a butt
  cap. **The box contains it:** with the inset `m = ceil(stroke_width / 2)` (0 with no
  `stroke`, the largest keyed value where `stroke_width` is keyed), every `at`, `at + in`
  and `at + out` lies in `[m, width − m] × [m, height − m]`, in every literal value of
  `points`. Outside is `E-PATH-OUTSIDE-BOX`, naming the vertex, the record, the absolute
  position and `m`. Nothing is clipped to the box.
- **Keyed `points` is a whole list per keyframe**, interpolated number by number. Every
  value has the same vertex count and each vertex the same handles
  (`E-PATH-KEYFRAME-SHAPE`). An overshooting ease clamps each absolute vertex and handle
  into the inset box at that instant. `shift` cuts a keyed `points` only where every
  resolved number is an integer, and refuses elsewhere.
- **`query --at` prints a path's `path`**: its resolved vertices with absolute control
  points in box pixels. `NOT COVERED` counts the declared box, as for an `ellipse`.

## Values

- **Colours are `#RRGGBB` or `#RRGGBBAA`, uppercase, and nothing else** (ADR-0014). No three-digit
  shorthand, no CSS names. `#RRGGBBFF` is an error naming the six-digit form, because a
  fully-opaque eight-digit colour is a second spelling of a value the six-digit form already
  says. Alpha lives here as well as in `opacity` because `opacity` is the *animation*
  channel.
- **`speed` is strictly greater than zero** (ADR-0020, ADR-0045), and it changes what the two ranges may
  disagree by: `end - start` must equal the source span divided by `speed`, rounded to the
  nearest millisecond, evaluated exactly rather than in floating point.
- **`overrun` has no `"none"` value** (ADR-0020) — it is present when needed and absent otherwise.
  `"hold"` is video-only; the correct spelling of "then silence" on audio is a shorter
  element and a gap.
- **`ease` is required on every keyframe record except the first, and forbidden on the
  first** (ADR-0038). Easing describes the interpolation *arriving at* a keyframe, and nothing arrives
  at the first one. Presence is a pure function of position.
- **`t_from` records where a keyframe's `t` came from, and no renderer reads it** (ADR-0086).
  It is optional, it sits immediately after the `t` it annotates, and it is the one field
  whose only consumer is `validate`: the literal `t` beside it stays the sole author of what
  renders. Two rules, and the set is closed —
  `{"rule": "element-start"}`, which takes no argument, and
  `{"rule": "after-previous", "ms": <non-negative integer>}`, which means *the previous
  record's `t` plus `ms`*. `after-previous` on the **first** record of a list is a schema
  error, because nothing comes before it. Writing a rule the schema does not publish, an `ms`
  on `element-start`, no `ms` on `after-previous`, or a negative one, are schema errors too.
  If the arithmetic stops holding, `validate` says so as `R-DERIVED-T` — an `error` stating
  the integer the rule now derives. **Omitting `t_from` is not a claim that a value is
  independent**; it is no claim at all, and nothing infers one for you.

## Sources

- **A `source` is written on the element itself** (ADR-0002) — a path or a URL. There is no table of
  assets declared elsewhere and referred to by name, and no top-level field that changes
  how a `source` resolves.
- **A relative path is always relative to the project file's own directory** (ADR-0053). An absolute
  path is permitted and resolved as-is. There is no shared base to edit: a project refers to a
  remote store by writing each `source` as a URL.
- **A URL `source` is probed but not rendered** (ADR-0131). `validate` probes it, and reports it
  as an `error` (`E-NOT-MIXED-REMOTE`, `E-NOT-PAINTED-REMOTE`), because `render` and `frame`
  draw and mix local sources only and refuse it. A remote store is for reference, not
  rendering: to render, fetch each file and point `source` at the local copy by path. A
  `file:` URL is a local path and renders. Fetching inside `render` is deferred, not ruled out.
- **The document records no source duration** (ADR-0011). What the file on disk actually is gets
  established by probing it, which is why `validate` always probes and why the cache-miss
  line is printed unprompted rather than treated as an optimisation detail.

## Versioning

**There is no format version field** (ADR-0016, ADR-0017). An older binary meeting a file authored against a
newer schema discovers it through the unknown-key error, which is why the schema is closed
at every object level — including inside `run` and `keyframe`. An unknown key is an error
naming the key, never a silently ignored field, and there is no escape hatch object for
arbitrary extra data.

Likewise, a retired spelling is an error that names its replacement rather than being
quietly accepted: `gravity`, `box`, `align` on a non-text element, `anchor` as a string,
`center-center`, `#RRGGBBFF`, `bold`, and `none`/`fill` as a `fit` value are all errors you
can repair from the message without reading an ADR.

## What the tools will and will not tell you

`validate` reports **facts** (ADR-0006) — findings with stable codes, each carrying the numbers it
rests on. It never reports a verdict that would require knowing what the video is for.
Every finding has a class: `error` (the render is refused or is guaranteed wrong), `review`
(legal, renders, and you must look at a frame to know whether it was meant), `note` (a fact
you may want), plus `UNCHECKED` (the question could not be answered) and `LAYOUT` (the
writing convention), neither of which is a severity.

Two consequences worth holding onto:

- **`render` runs the identical checks and refuses on any `error`** (ADR-0006). Validation is not an
  optional step you can skip; it is what the render does before it draws anything.
  A `LAYOUT` finding never gates it. The video is written to the project's `output` via a temp
  path and an atomic rename, always at the declared frame size, and after it succeeds the
  `review` findings it did not refuse on print beneath the result together with the `NOT
  CHECKED` footer — exit 0 never means the video is right. `render --from --to` writes one
  half-open range to `out/<name>.<from>-<to>.mp4` and can never land on the deliverable.
- **Every write tool returns the new state's findings, never `ok`** (ADR-0011). You do not run
  `validate` after a write — the write hands you `validate`'s answer.

And the boundary every report ends with, which is not modesty but the most important
sentence in the tool: **Montagent verifies that the file is internally legal. It cannot tell
you whether the file says what you meant it to say.**
