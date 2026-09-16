# Montaget

Montaget is a video editor whose project format is designed to be authored and
edited by an AI agent rather than dragged around in a GUI. A project states, by
being read, what is on screen at any given moment; Montaget renders it.

## Actors

**Montaget**:
The tool — a renderer and an MCP server. It is deterministic and contains no
model: given the same project and the same files it produces the same video
every time.
_Avoid_: calling Montaget an agent, or "the AI"

**Agent**:
An LLM client outside Montaget — Claude Code, or any other MCP client — that
authors and edits projects and calls Montaget's tools. Montaget never calls an
agent.
_Avoid_: user, bot, assistant, client (unqualified)

## Language

**Project**:
The whole video, as a single declarative document: frame size, frame rate,
background, output destination, an optional duration, and its tracks. Normally
stored as one JSON file in git, which is where its authority lives.
_Avoid_: composition, timeline, edit, movie

**Track**:
A named container holding elements, with an integer `layer` giving its place in
the stack. A track supplies *stacking*, never *timing*: it has no start, no
duration and no clock, its children carry absolute times on the project's one
timeline, and the order they appear in carries no meaning. Children of a single
track may not overlap in time — that is a validation error. Elements that should
overlap belong in different tracks.
_Avoid_: lane, channel, layer (as a container)

**Element**:
One thing placed on the timeline — an image, a video, an audio file, a piece of
text or a shape. Every element has a type, a required unique `id`, a time range
and an optional `group`, in that same shape whatever its type. It sits in a
track, which supplies its stacking position unless the element overrides it.
Audio is an element like any other; nothing owns it.
_Avoid_: clip, item, object, asset

**Id**:
A short, unique, author-chosen string naming one element, required on every
element regardless of type — `"id": "card-05"`. Its only job is to be a target,
for now an anchor's `below`/`above`
([ADR-0019](docs/adr/0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md)).
A separate namespace from a track's name: the two look alike as bare strings but
an anchor may only resolve against an element `id`, never a track name. Not a
tool-call argument — the write-tool invariant
([ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md)) forbids a
write tool taking a bare id as its argument, which is unrelated to an id
appearing as data on an element.
_Avoid_: key, name, index

**Layer**:
A place in the stack, as an integer — higher draws in front. Normally carried by
the track, so every element in it stacks together. An element may override its
track's layer with its own integer, or with an anchor.
_Avoid_: z-index, depth

**Anchor**:
An element's layer stated relative to another element's `id` rather than as a
number — `{"below": "title"}` resolves to that element's layer minus one,
wherever either of them sits. The target must itself carry a plain integer
layer — one hop only, never another anchor — so resolving one is a lookup, not
a walk. Written so a dependent element cannot drift out of sync when the thing
it depends on moves.
([ADR-0019](docs/adr/0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md))
_Avoid_: parent, constraint, binding; and do **not** use for an element's
positioning origin — that is `origin`. See the note under Origin.

**Origin**:
The point of an element's own box that its `x`,`y` places, and about which transforms
pivot. Exactly nine keywords, vertical component first:

    top-left      top-center      top-right
    center-left   center          center-right
    bottom-left   bottom-center   bottom-right

The grammar is `{top|center|bottom}-{left|center|right}` with one elision: the middle is
`center` alone, and **`center-center` is a schema error naming `center`** — two spellings
of one value break the write-read round trip, because `fmt` normalises on write and the
agent's next exact-string replace then finds nothing. It is `center-left`, never
`middle-left`: `top-center` needs a horizontal-middle word regardless, so a separate
`middle` would spell one concept two ways depending on axis. Defaults to `center`, so a
bare element is conspicuous rather than plausible.
_Avoid_: anchor, alignment, pivot

**Transform**:
Where a visual element sits and how it is drawn — `x`, `y`, `origin`, `scale`,
`rotation`, `opacity` — as flat fields on the element in absolute integer pixels. There is
exactly one per element and it is never nested, never inherited and never composed: no
element's transform is relative to another's. An element's size is declared, never
defaulted from the source file.
_Avoid_: matrix, layout, placement (as a field), position (as a field name)

**Keyframe**:
One `{"t","v","ease"}` record in a list that makes a transform property change over time.
Its `t` is written in timeline milliseconds but is **not a timeline time** — it is the
element's own animation geometry, so `shift` moves elements and their keyframes are
carried with them. A keyframe outside its element's range is legal and ordinary: it is how
a trimmed move is spelled.
_Avoid_: key, waypoint, stop, tween

**Easing**:
The shape of the interpolation **arriving at** a keyframe from the previous one — a name
published in the schema as its cubic bezier, or the four control points directly. The
first record of a list has nothing arriving at it and carrying an `ease` there is an error.
_Avoid_: timing function, curve, interpolation (as the field name), tween

**Clip**:
The static frame-space rectangle an element is drawn through. It does not rotate and does
not scale with the element, so `scale` moves the picture behind a window that stays put —
which is what a Ken Burns is. Shaped and soft masks are a different thing and are not this.
_Avoid_: crop, mask, viewport, bounds

**Run**:
One stretch of a text element's content, carrying its own text plus style deltas
over the element's base style. A text element's content is always an ordered array
of runs, even when there is only one. A run boundary is *style only* — it never
implies a line break; a line break is a `\n` character inside a run's text.
_Avoid_: span, segment, chunk

**Font**:
An ordered chain of font *files* the project declares under a semantic name, which
elements reference by that name. Always files, never a system family: a family name
is an entry in a table you cannot read, cannot commit, and that differs per machine.
The renderer opens nothing outside the chain.
_Avoid_: typeface, family, font stack

**Line height**:
A text element's line-height multiplier, restricted to one decimal digit — `1.0`, `1.1`,
`1.2`, … — always exactly representable as `n/10`. A line's height is the largest `size`
among its runs × `line_height`; the derived block height is `ceil` of that product over the
line count, evaluated in exact integer arithmetic, never IEEE double — the same ULP hazard
`fit`'s box sizing has (see Fit), on a field `fit`'s ADR never touched. Defaults to `1.2`
when omitted.
([ADR-0028](docs/adr/0028-text-block-arithmetic-is-exact-tenths.md))
_Avoid_: leading, line spacing (implies an additive gap, not a multiplier)

**Shape**:
A drawn primitive with no source file — `rect` or `ellipse`, each its own element type,
never a `shape` field inside a shared one. An ellipse inscribes its declared rect, so
neither needs a placement rule beyond the transform every visual element already carries.
A `rect` may declare a `radius`; an ellipse may not.
_Avoid_: rectangle (as the type name), box, figure, primitive (unqualified)

**Fill**:
The paint inside a shape's outline, as a single flat colour. Optional when a `stroke` is
present, giving an outlined shape; a shape with neither is a schema error naming both,
because an element that deliberately renders nothing and an element that forgot its paint
must not look alike. A gradient is not a fill — a flat colour is a value, a gradient is a
paint description, and descriptions belong in a closed vocabulary rather than an open
syntax.
_Avoid_: background, colour (for a shape), paint. Do not use for what a
time-based element does past the end of its source — that is `overrun`.

**Stroke**:
A second paint on the same outline — `stroke` and `stroke_width` — sitting on the
primitive rather than among the effects, and addressable per *run* on text. It never
enlarges the declared rect, and that is what separates it from an effect: a blur is not a
paint on the outline and a drop shadow is not addressable by run. On a shape it falls
**inside** the declared rect, so a stroked `card-05` still occupies exactly 984×169. On
text it falls **outside the glyph contour** — inside would thin the stems — and grows into
the box rather than past it, because a text element's `width`/`height` is a container
claim and not drawn geometry. It is in element space, so it scales with `scale`.
_Avoid_: outline, border, bord

**Colour**:
`#RRGGBB` or `#RRGGBBAA`, uppercase, and nothing else. No three-digit shorthand, no CSS
names, and `#RRGGBBFF` is an error naming the six-digit form — two spellings of one value
break the write-read round trip, as `center-center` does. Alpha lives here as well as in
`opacity` because `opacity` is the *animation* channel: a static per-run transparency, or
a translucent fill under an opaque stroke, has nowhere else to go. `fmt` never rewrites a
colour, including between the two forms. ASS writes `&HAABBGGRR` — byte-reversed, alpha
inverted — and every colour in the fixture was converted out of it by hand.
_Avoid_: color (in prose), rgba(), hex (as the field name)

**Group**:
An optional free-text label marking elements that belong together, such as every
element of one vocabulary item. It has no effect on rendering, timing or
stacking.
_Avoid_: scene, segment, section

**Source**:
The file an element draws on, written on the element itself as a path or a URL.
There is no table of files declared elsewhere and referred to by name.
_Avoid_: asset, resource, media reference

**Timeline range**:
Where an element sits on the project's one absolute clock, as a `start` and an
`end` in whole milliseconds. The range is half-open — an element is on screen from
its `start` up to but not including its `end` — so a cut where one range ends and
the next begins names a single instant, not an overlap and not a gap.
_Avoid_: duration (as a stored field), offset, timecode

**Source range**:
Which part of a file an element plays, as distinct from where the element sits on
the timeline. Only elements built on time-based media — video and audio — have
one; images, text and shapes have no insides and are simply held for their whole
timeline range. A time-based element's two ranges are the same length unless it
says otherwise.
_Avoid_: trim (as a noun), in/out point

**Speed**:
A time-based element's playback-rate multiplier — `0.645` plays its source at
0.645× normal rate, slower. Strictly greater than zero; `0` and negative
values are schema errors. Changes what the source range's two ranges are
allowed to disagree by: `end - start` must equal `source range / speed`,
rounded to the nearest millisecond. Reverse playback is not `speed`'s job and
is undecided, not ruled out.
([ADR-0020](docs/adr/0020-speed-overrun-hold-loop.md))
_Avoid_: rate, stretch factor (reads as the reciprocal and gets the direction
backwards), tempo

**Overrun**:
What a time-based element does once its (possibly speed-adjusted) source runs
out before its timeline range does: `"hold"` freezes the source's last frame
(video only — a schema error on audio, where the correct spelling of "then
silence" is a shorter element and a gap) or `"loop"` restarts the source from
its beginning with a hard cut, no crossfade. Composes with `speed` rather than
excluding it. Present only when needed — there is no `"none"` value.
([ADR-0020](docs/adr/0020-speed-overrun-hold-loop.md))
_Avoid_: fill (spent — see Fill), extend, pad

**Gap**:
A stretch of a track with no element in it. Gaps are legal and ordinary — the
silence between two narration lines is a gap. A gap is never an error, which is
why it is reported apart from an overlap rather than alongside one.
_Avoid_: hole, blank, silence (as a name for the general case)

**Shift**:
Moving every time at or after some instant by an offset, so that inserting or
removing time carries the rest of the project with it. It is named because it is
the one edit that is arithmetic rather than authorship, and therefore the one
Montaget performs instead of the agent.
_Avoid_: ripple (as the primary term), slide, nudge

## Rejected terms

These words are deliberately absent. Each is standard vocabulary in a comparable
tool, which is exactly why using it here would mislead.

**Scene**:
Elsewhere a scene owns its own clock, so its children's times are relative to it.
A `track` is the one container Montaget has, and the distinction is exactly this:
a scene owns a clock, a track owns a stacking position. A track has no start, no
duration and no origin, so there is nothing for a child's time to be relative to.
Nesting still *invites* the assumption — JSON2Video documents the silent failure
that follows — which is why the times stay absolute and why the schema says so
loudly rather than relying on this paragraph.

**Clip**:
Elsewhere a clip pairs a visual with its audio, and its duration follows that
audio. In Montaget audio is an ordinary element with its own independent time
range, so no such pairing exists.

**Asset**:
Elsewhere an asset is an imported file declared once and referenced by id.
Montaget writes the file's location on the element that uses it. Note also that
an asset is *not* a reusable configured object in the sense of a Unity prefab —
that is templating, and it is out of scope for v1. The one carve-out is **Font**,
for reasons recorded in [ADR-0002](docs/adr/0002-inline-source-no-asset-table.md);
media sources never get a table.

**Anchor, for text positioning**:
Every comparable tool — ASS's `\an`, CSS, every GUI editor — calls the nine-way
positioning point an *anchor*. Montaget spends that word on layer-relative stacking
instead, so the positioning point is **`origin`**. The collision is recorded here
because it is near-certain to be rediscovered: the positioning concept appears on
essentially every text element, and layer anchoring appears rarely. The schema catches
the rediscovery **on shape, not presence**: `anchor` carrying a *string* is an error
naming `origin`; `anchor` carrying a below/above *object* is the ordinary feature above.

**Box**:
Retired, having meant two different things at once: a literal `[x,y,w,h]` rect on an image
and, in [ADR-0007](docs/adr/0007-text-runs-literal-size-declared-fonts.md), *the id of the
element you must fit inside*. A field that is sometimes a rect and sometimes a reference
cannot even produce a good error message. The rect is now `x`,`y`,`origin`,`width`,`height`
plus `clip`; the containment reference is now literal `width`/`height`, because fifteen of
the fixture's twenty-two text elements have no element behind them to name.

**Align, for images**:
`align` means one thing: how a text element's lines align to each other
(`start`/`center`/`end`). On an image the same word meant *which part of the source survives
the crop*, which is not alignment at all — that is `gravity`. The collision is recorded
because it caused a live misreading: the fixture's `align:"left"` on text was transcribing
ASS `\an4`, **left-middle**, so its `y` was a centre and nothing in the file said so.

**Path, line, polygon**:
Every visual element is placed by `x`, `y`, `origin`, `width`, `height`. A point list is
not — it has **no declared extent**, and giving it one needs either a second unit system
or an extent derived from its own content, both of which are already closed. So these are
rejected rather than merely absent, and the reopening condition is stated so it can be
met: admitting a point-list shape is a new ADR about *placement*, not a schema addition.
Do not reach for "a thin rotated `rect` draws a line" as the reason — it is false in the
way that matters, because the endpoints never appear in the file. A `path` is additionally
a mini-language inside a JSON string: unreadable by reading, unmatchable by exact-string
replace. Commit an SVG or a PNG instead.

**Gravity**:
Retired entirely by [ADR-0015](docs/adr/0015-fit-is-a-derivation-claim-and-gravity-retires.md).
It meant *which part of a source survives a crop* — a quantity that does not exist once the
declared rect is authoritative at render: the rect's `x`, `y`, `origin` and the static
frame-space `clip` already determine it, with no freedom left to spend. A schema error on
every element type; on an image the message names `x`/`y`/`origin` and `clip`, on text and
shapes it names `origin`. Expect to reach for it by **copying a neighbouring element** — that
is how eight of eight agents met it — which is why its eight occurrences left the fixture in
the same change that retired it.

**Fit**:
A **derivation claim, not a layout mode**. `fit` records the rule by which the author computed
`width`/`height` from the source and the aperture; no renderer reads it, because the declared
rect is what gets drawn. Its only consumer is `validate`. The closed set is `cover`, `contain`
and `literal`, and the box a rule works against is `clip`'s `w`/`h`.
Defined in [ADR-0015](docs/adr/0015-fit-is-a-derivation-claim-and-gravity-retires.md).

**Source dimensions**:
The input to `fit`'s arithmetic. One type-generic pipeline: decode, resolve rotation
(the container's track-level display transform; codec-level orientation metadata is not
consulted), apply pixel aspect ratio, round to one integer pair. Images are the
degenerate case — PAR is `1:1` and EXIF orientation is the only rotation signal — so
this is the same rule ADR-0015 stated for images, generalised, not replaced.
Defined in [ADR-0023](docs/adr/0023-video-source-dimensions-par-and-rotation.md).

**PAR (pixel aspect ratio)**:
Applied, not ignored — silence here would repeat the EXIF-orientation divergence ADR-0015
already legislated against, and would leave `validate` disagreeing with any renderer whose
decode path applies it. Declared explicitly on the element as `par: [num, den]`, an exact
integer-pair rational populated by an ingest/authoring tool, never silently re-probed from
whichever of the container/bitstream layers that can disagree with each other. Defaults to
`[1, 1]`; a schema error on any non-video element. Defined in
[ADR-0023](docs/adr/0023-video-source-dimensions-par-and-rotation.md).

**Fill / none / stretch, for `fit`**:
`fill` is spent — it is a shape's paint colour ([ADR-0014](docs/adr/0014-stroke-is-paint-the-text-box-is-required.md)).
`none` is a **false friend**: CSS `object-fit: none` means *intrinsic size*, which this format
cannot express, so it is a schema error naming `literal`. `stretch` was rejected as naming an
effect that is constant across the vocabulary — every value resamples to the declared rect —
and so distinguishes nothing. The collisions are recorded because they are the CSS-habit
reach: three of eight agents tried `none` or `fill` before finding the legal value.

**Literal, versus declared**:
The escape value is `literal`, continuing [ADR-0007](docs/adr/0007-text-runs-literal-size-declared-fonts.md)'s
*"Literal `size`. No fit-to-box."* — this format's established opposition between a literal
number and a fit rule. `declared` was rejected despite being a jury plurality: the corpus
already spends it generically, 14 times as "declared rect" and 6 as "declared extent", so it
describes all three values and distinguishes none of them.

**Weight / bold**:
Not a field. A different weight is a different font file — with one declared file
and no family to search, `bold: true` could only mean synthetic emboldening, which
is renderer-specific and machine-dependent. Expect to reach for it out of CSS habit;
the schema rejects it and names the replacement.
