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
background, output destination, an optional duration, and its elements. Normally
stored as one JSON file in git, which is where its authority lives.
_Avoid_: composition, timeline, edit, movie

**Element**:
One thing placed on the timeline — an image, a video, an audio file, a piece of
text or a shape. Every element has a type, a time range, a `layer` and an
optional `group`, in that same shape whatever its type. Audio is an element like
any other; nothing owns it.
_Avoid_: clip, item, object, asset

**Layer**:
An element's place in the stack, as an integer — higher draws in front, and
elements sharing a layer draw in the order they appear in the file, later in
front.
_Avoid_: z-index, depth, track

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

**Track**:
In every comparable tool — Premiere, Resolve, OpenTimelineIO, Shotstack,
Creatomate — a track constrains its contents to play in sequence without
overlapping. Montaget has no such constraint: elements stack by `layer` and
overlap freely. Borrowing the word without its meaning would set an expectation
Montaget does not meet.

**Scene**:
Elsewhere a scene owns its own clock, so its children's times are relative to it.
Montaget's `group` deliberately carries no timing meaning, and nesting elements
inside a container would imply one whether or not it existed.

**Clip**:
Elsewhere a clip pairs a visual with its audio, and its duration follows that
audio. In Montaget audio is an ordinary element with its own independent time
range, so no such pairing exists.

**Asset**:
Elsewhere an asset is an imported file declared once and referenced by id.
Montaget writes the file's location on the element that uses it. Note also that
an asset is *not* a reusable configured object in the sense of a Unity prefab —
that is templating, and it is out of scope for v1.
