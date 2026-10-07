---
status: accepted
amends: 0040 (names the audio vocabulary its "Audio effects" boundary marker left for later), 0055 (fixes where `volume` sits against signal processing)
---

# Audio effects are an ordered list before `volume`; gain and routing stay flat

[Where does an audio effect sit on an element: an ordered list, flat fields, or both?](https://github.com/MBehtemam/Montagent/issues/799)
on the audio map ([#795](https://github.com/MBehtemam/Montagent/issues/795)) asked for the shape
every audio capability ADR fills in. This ADR decides only that shape. It adds no members.

## Decisions

### An ordered list for signal shaping; flat fields for gain and routing

An `audio` or `video` element carries **`audio_effects: [...]`**, an ordered list of
signal-shaping members: EQ, dynamics, loudness normalisation, restoration and creative effects.
Gain and routing controls stay **flat fields** at fixed points in the graph: `volume` today, and
pan/balance and channel operations when their ADRs land.

The test for a new capability: **a gain or routing control is flat; anything that shapes the
signal is a list member.** Order inside a processing chain changes the sound (EQ then compressor
differs from compressor then EQ), and the same kind may legitimately appear twice. A routing
control has one correct position, and a list would invite meaningless orders and duplicates.

This copies Premiere: a clip's fixed controls (Volume, Channel Volume, Panner) sit outside its
ordered effect rack. It also copies the format's visual side, where the `effects` list comes
before the flat `opacity` and blend mode (ADR-0147).

Rejected:
- **Only flat fields.** The renderer would fix one chain order that a later capability would have
  to reopen, and two EQs would have nowhere to go.
- **One list for everything.** This would put pan and channel operations where order and
  duplicates mean nothing.

### `volume` is post-fader: the list runs after `aloop` and before `volume`

The per-element graph becomes:

`atrim → atempo → aloop → [audio_effects] → volume → [pan/balance] → adelay → amix`

- **The list comes before `volume`.** The ducking write tool writes `volume` keyframes. With a
  compressor after `volume`, that compressor would push some of the dip back up, and the keyframes
  would no longer sound like what they say.

  The cost: a per-element limiter no longer holds its ceiling when `volume` is above `1`. The hard
  ceiling belongs to the master stage
  ([#801](https://github.com/MBehtemam/Montagent/issues/801)). A per-element limiter under
  `volume > 1` is a candidate `review` finding.
- **The list comes after `aloop`.** A looped bed is processed as one continuous stream at playback
  rate, so a tail isn't cut and a compressor's state isn't reset at every seam. Because the list
  runs after `atempo`, EQ frequencies are not scaled by `speed`.

  Whether a tail may run past the element's `end` is still open on the map.

### Where routing sits

**Pan/balance sits after `volume`,** as Premiere's Panner follows its Volume.

**Channel operations** (mono, swap, one channel only) get their slot from their own capability
ADR. The leading candidate is *before* the list. A channel operation decides what signal every
effect sees, for example a lav mic recorded on one channel. That is a restoration case, and
restoration is deferred. The precedent row for channel operations is also still open, so this
ADR does not fix that slot.

### The field is `audio_effects`, and a `video` carries it too

On a `video` it applies to the embedded track and sits next to the visual `effects`. ADR-0055
already ruled that a `video` is one element with intrinsic audio. The alternative is silencing
the video and adding a twin `audio` element, which would make an author keep trim, `speed` and
placement in step by hand.

ADR-0040 requires a separate vocabulary, so the two lists never share members. `validate` rejects
an audio member in `effects`, and a visual member in `audio_effects`, with a message naming the
right list.

Rejected names:
- `filters` collides with **Colour filter**, and with ffmpeg's word for every graph node.
- `sound` or `audio` reads as a container for `volume` too.
- `fx` is jargon.

### A member is `{"name": ..., <params>}`, the same tagged shape as `effects`

`name` discriminates a closed union, and each capability ADR adds one member and its parameters.
`name` and `enabled` (below) are reserved, so no parameter may use them. Whether a parameter is
animatable is decided separately.

### `"enabled": false` bypasses a member

Any member may carry `"enabled": false`. `fmt` drops `"enabled": true`, so `"enabled": false` is
the only spelling the field has. A disabled member is still validated, but it does not render.

Deleting a member to bypass it destroys its tuned parameters. A/B listening against the bypassed
source is a routine step under this map's evidence bar, not an edge case, and Premiere gives every
rack effect this toggle.

`review` flags any member left disabled, because in a finished project it is a leftover. Whether
visual `effects` should gain the same toggle is outside this ADR.

### Duplicates are allowed unless the member's own ADR makes it singular

Two members of the same `name` are ordinary, as in ADR-0040. Two high-passes in series and two
compressors are real practice.

A capability ADR may declare its member **singular**. A second enabled copy of a singular member
is then a `validate` error naming it. Loudness normalisation is the expected case, since a second
target can only override the first. Duplicate and singular checks count enabled members only.

### A member is singled out by its literal text

A member is singled out by its literal text within its element's block, as visual effects are.
`fmt`'s canonical key order keeps that text stable.

There is no per-member `id` and no index addressing. An id would be bookkeeping the renderer never
reads and that `validate` would have to keep unique. An index goes stale on every insertion.

Two byte-identical members can't be told apart by text. They are either interchangeable, or a
singular member's error.

## Consequences

- Every audio capability ADR now fills in:
  - its member's `name` and parameters;
  - whether the member is singular;
  - for a flat routing control instead, its slot in the graph.
- The schema gains `audio_effects` on `audio` and `video`: a list, defaulting to empty, of a union
  keyed on `name` that has no members yet.
- `CONTEXT.md` gains an **Audio effect** entry.
- `volume`'s position in the graph is now fixed as post-fader. Today's graph is unchanged until a
  first member exists.

## Evidence

Two three-juror courts (Opus, Sonnet, Fable), run blind, with the owner ruling with the Judge on
both. Ballots are in [`docs/research/juries/audio-effect-shape/BALLOTS.md`](../research/juries/audio-effect-shape/BALLOTS.md).

- **Round 1:**
  - list plus flat fields: 3/3;
  - after `aloop`: 3/3;
  - `video` carries it: 3/3;
  - post-fader: 2/1. Sonnet voted pre-fader so a per-element limiter holds its ceiling.
- **Round 2:**
  - `audio_effects`, the tagged shape, per-ADR singular members and text addressing: 3/3;
  - `enabled: false`: 3/3, overturning the Judge's recommendation against it;
  - routing after `volume`: 2/1. Opus split it, placing channel operations before the list. That
    split is recorded above as the leading candidate.
