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
_Avoid_: parent, constraint, binding

**Group**:
An optional free-text label marking elements that belong together, such as every
element of one vocabulary item. It has no effect on rendering, timing or
stacking.
_Avoid_: scene, segment, section

**Source**:
The file an element draws on, written on the element itself as a path or a URL.
There is no table of files declared elsewhere and referred to by name.
_Avoid_: asset, resource, media reference

**Source range**:
Which part of a file an element plays, as distinct from where the element sits on
the timeline. Only elements built on time-based media — video and audio — have
one; images, text and shapes have no insides.
_Avoid_: trim (as a noun), in/out point

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
that is templating, and it is out of scope for v1.
