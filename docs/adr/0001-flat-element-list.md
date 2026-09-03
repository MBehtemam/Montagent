---
status: partially superseded by 0004
---

# A project is a flat list of uniform elements, not tracks or scenes

> [!IMPORTANT]
> **The track rejection below is superseded by
> [ADR-0004](./0004-tracks-as-constrained-lanes.md).** Elements now live in
> `tracks`, each a named lane with an integer `layer`, whose children keep
> absolute times and may not overlap. The objection recorded here — that "track"
> promises sequencing it would not deliver — was confirmed correct five for five
> from primary sources; ADR-0004 resolves it by *delivering* the constraint rather
> than avoiding the word.
>
> **The rest of this ADR stands unchanged**: no scene, no local clock, audio as a
> peer element, no per-kind collections, and the driving requirement that "what is
> on screen at 6.2s" be answerable by reading. Specifically superseded: the
> "no container of any kind" sentence below, the **Considered options → Tracks**
> paragraph, and the group-contiguity bullet under **Consequences**.

A project holds one `elements` array. Every element — image, video, audio, text,
shape — carries the same `type`, time range, `layer` and optional `group`, and
elements overlap freely in both time and space. There is no track, no scene, and
no container of any kind between the project and an element. Audio is an element
like any other; no visual element owns a sound.

The decision is driven by one requirement: an agent must be able to answer "what
is on screen at 6.2s" by *reading*, with no arithmetic and no evaluation. Flat
and absolute, that question is a single filter over a single array — a check the
agent can run inside its own turn, without a tool call.

## Considered options

**Tracks** (Premiere, Resolve, OpenTimelineIO, Shotstack, Creatomate). A survey
of four shipping declarative video APIs found that *no* product uses a track as a
pure stacking lane: in every one, a track constrains its contents to play in
sequence without overlapping. "Track" therefore means *sequencing*, not
*stacking*. Montaget imposes no such constraint, so adopting the word would
promise behaviour it does not have — and a reader arriving from any of those
tools would expect ripple edits and not get them.

**Scenes** (JSON2Video `scene`, Editly `clip`, Creatomate `composition`; three of
the four surveyed). A grouping noun does two separable jobs: giving related
elements a shared identity, and giving them a shared clock. The identity is
valuable and free; the clock is where the damage is. JSON2Video's docs describe
the exact failure — an element's `start` is relative to its scene, a model
computes it against the movie instead, and the render succeeds and is silently
wrong. Editly shows the other cost: nothing can outlive its clip, so a header or
watermark spanning the whole video is simply inexpressible.

The trap is that *nesting implies a local clock whether or not one exists*. You
cannot put elements inside a container and then ask readers not to infer that
their times are relative to it. So the identity job is served by a `group`
string, which the renderer ignores entirely, and no container exists to imply the
rest.

**Per-kind collections** (`images[]`, `audio[]`, `texts[]`). Rejected mainly
because cross-kind stacking becomes unanswerable: if a text and a shape live in
different arrays, nothing in the document's structure says which draws in front.

**A clip owning its audio.** Rejected because the pairing is a coincidence of the
slideshow case. In the project's own fixture one still is on screen for 14.5s
with six unrelated audio events under it, three belonging to no visual at all.
Background music, narration outliving a cut, and deliberate silence are all
inexpressible under that model — Editly grew a separate `audioTracks` array as an
escape hatch for exactly this, and now has two audio systems.

## Consequences

- Nothing keeps a group internally consistent. Changing one vocabulary item's
  layout does not change its siblings, and they can drift. This is the templating
  cost the map already accepted; it is paid here.
- Ordinary cuts — "insert a shot at 0:12 and shift everything after" — get no
  help from the format. Whether that is solved by relative authoring materialised
  into absolute times, or another way, is deferred to the time-model decision.
- Elements sharing a `group` are kept contiguous in the array by convention. This
  is a formatting rule only; nothing depends on it. It recovers the locality that
  nesting would have given, so a targeted edit lands on a contiguous block.
- The flat, uniform shape is a clean expansion target should templating arrive
  later: a template expands into N elements sharing one `group`.
