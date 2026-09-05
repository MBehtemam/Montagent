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
text or a shape. Every element has a type, a time range and an optional `group`,
in that same shape whatever its type. It sits in a track, which supplies its
stacking position unless the element overrides it. Audio is an element like any
other; nothing owns it.
_Avoid_: clip, item, object, asset

**Layer**:
A place in the stack, as an integer — higher draws in front. Normally carried by
the track, so every element in it stacks together. An element may override its
track's layer with its own integer, or with an anchor.
_Avoid_: z-index, depth

**Anchor**:
An element's layer stated relative to another element rather than as a number —
`{"below": "title"}` resolves to that element's layer minus one, wherever either
of them sits. Written so a dependent element cannot drift out of sync when the
thing it depends on moves.
_Avoid_: parent, constraint, binding; and do **not** use for an element's
positioning origin — that is `origin`. See the note under Origin.

**Origin**:
The point of an element's own box that its `x`,`y` places, as one of nine keywords
(`top-left` … `center` … `bottom-right`), and about which transforms pivot. Defaults to
`center`, so a bare element is conspicuous rather than plausible.
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
essentially every text element, and layer anchoring appears rarely.

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

**Weight / bold**:
Not a field. A different weight is a different font file — with one declared file
and no family to search, `bold: true` could only mean synthetic emboldening, which
is renderer-specific and machine-dependent. Expect to reach for it out of CSS habit;
the schema rejects it and names the replacement.
