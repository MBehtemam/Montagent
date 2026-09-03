# Track vs `layer`: re-testing the no-track decision against the CapCut/Premiere reference class

Research for [#20](https://github.com/MBehtemam/Montaget/issues/20). Companion to
[the declarative video API survey](./declarative-video-api-models.md), which covered the
*authoring API* side (Shotstack, Creatomate, JSON2Video, Editly). This file covers the gap:
the **desktop NLE / interchange** side — OpenTimelineIO, Premiere Pro, CapCut, DaVinci
Resolve, FCPXML — and the agent-authoring argument.

All claims cite a primary source inline. Anything not established from a primary source is
marked **NOT CONFIRMED**. Sources that could not be fetched are marked **NOT REACHED**.

Status: **complete.** All five formats in §2 were reached, with the provenance caveats noted
in §2.3 (CapCut has no published schema) and §2.4 (Resolve's reference ships inside the
application, so it is quoted from a verbatim mirror).

## 1. Summary of what was established

1. **"A track constrains its contents to play in sequence without overlapping" is confirmed,
   five for five, from primary sources.** OTIO `Track` (§2.1), FCPXML `spine` — *"a container
   for elements ordered serially in time"* (§2.5), CapCut — `SegmentOverlap` on any
   overlapping insert (§2.3), Premiere — you must choose insert *or* overwrite (§2.2),
   Resolve — append/insert verbs only (§2.4). ADR-0001 asserted this without citations; it is
   now the best-evidenced claim in either survey. Two formats independently add the *same*
   single escape hatch: overlap within a lane is permitted only as a typed transition.
2. **But tracks do *not* imply ordinal timing, and the first draft of this research got that
   wrong.** Only the interchange formats (OTIO `Track`, FCPXML `spine`) derive position from
   order. **All three shipping editors store an absolute time range on each item** —
   Premiere's `getStartTime` *"the starting sequence time of this track item"*, Resolve's
   `GetStart() # Returns the start frame position on the timeline`, CapCut's
   `target_timerange` — plus a *separate* source range and an integer lane index. Field for
   field, that is Montaget's element. **The constraint lives in the edit operations, not in
   the storage.** (§3 Q1.)
3. **This reframes the whole question.** It is not "flat absolute times vs tracks" — every
   tool in the reference class is flat and absolute underneath. It is: **should the
   non-overlap rule be expressible, and if so does it live in a container, a validator, a
   tool, or nowhere?**
4. **Ripple edit, the classic argument for tracks, largely dissolves.** In Premiere it is a
   `ripple` **boolean argument** to a removal action, alongside a separate `shiftOverLapping`
   flag (§2.2) — an opt-in operation that recomputes absolute times, which is exactly what an
   agent rewriting downstream `start` values would do. It is not a free consequence of track
   storage. It is also only ever free *within* a lane; cross-lane sync is a UI affordance
   Montaget has no place for, and ADR-0001's own fixture (a 14.5s still over six unrelated
   audio events, three attached to no visual) is the case a per-lane ripple silently
   desynchronises. (§3 Q3.)
5. **The one real cost of the flat model is the diff, not the edit** — a conceptually
   one-line insertion touches every downstream element in git. That is a review cost, and it
   is addressable by tooling without touching the time model. (§3 Q3.4, option b1.)
6. **FCPXML is near-direct vindication of `layer`'s shape.** Apple shipped `spine` for "plays
   after" and a per-item signed integer `lane` for "draws over" — *"0 = contained inside its
   parent, >0 = anchored above its parent, <0 = anchored below its parent"* — refusing to
   overload one for the other (§2.5). Across both surveys, z-order direction otherwise has no
   convention at all (OTIO bottom-first, Shotstack first-is-top, Creatomate `z_index` with a
   sibling trap, Premiere and CapCut unstated). An explicit integer with a documented
   direction removes a class of silent generator error rather than picking a side of it.
   **Nothing found argues against `layer` as the stacking mechanism.** (§3 Q2.)
7. **Two genuine gaps in the flat model**, both independent of tracks: no way to declare that
   a run of elements is meant to be contiguous (so accidental overlap fails silently and
   visually), and no way to say "this element always sits directly under that one". A
   constrained track closes the first; **FCPXML's anchoring closes the second**, and neither
   requires giving up absolute times. (§3 Q4; options b3 and b5.)
8. **The strongest published evidence on machine authors and optional sequencing is
   negative**: Creatomate, the vendor with a dedicated LLM-facing reference, calls its own
   sequencing/duration cascade *"the number-one source of broken renders"*. Evidence against
   option (b2) specifically, not against tracks in general. (§3 Q5.)
9. **Two incidental findings that touch other parts of ADR-0001.** Resolve and CapCut both
   partition tracks **by media kind**, a mild counter-example to the ADR's rejection of
   per-kind collections — though how CapCut resolves z-order between `text` and `sticker`
   could not be established (§2.3, §2.4). And FCPXML pairs a clip with its audio, then needs
   `audioStart`/`audioDuration` *"to define J/L cuts"* to un-pair them again — which is the
   ADR's own argument against clip-owned audio, confirmed from the other side (§2.5).

Eight options are laid out in §4 — (a) unchanged; (b1–b5) five different opt-in affordances,
including one (**b5**, anchoring) that #20 did not anticipate; (c1–c2) two track shapes —
each with its cost, its value to an agent author, and the exact ADR-0001 text it would amend.
**No recommendation is made; the choice is the human's.**

## 2. Per-tool findings

### 2.1 OpenTimelineIO — `Track` vs `Stack`

OTIO is the strongest source available: an interchange schema, published as source, that
needed *both* a sequential container and an overlapping one and named them separately. The
reason it needed both is visible in ~10 lines of C++.

**A `Track` child has no start time. Its position is derived by summing what came before
it.** From `Track::range_of_child_at_index`
([src/opentimelineio/track.cpp](https://github.com/AcademySoftwareFoundation/OpenTimelineIO/blob/main/src/opentimelineio/track.cpp)):

```cpp
RationalTime start_time(0, child_duration.rate());
for (int i = 0; i < index; i++) {
    Composable* child2 = children()[i];
    if (!child2->overlapping()) {
        start_time += children()[i]->duration(error_status);
    }
}
```

This is the whole argument in one loop. A track is not a lane that things are *placed on*;
it is a list whose *order is the timing*. Answering "where does child 7 sit?" is an O(n)
accumulation over children 0..6 — arithmetic, not a read.

**A `Stack` child's start time is 0 — all children begin together.** From
`Stack::range_of_child_at_index`
([src/opentimelineio/stack.cpp](https://github.com/AcademySoftwareFoundation/OpenTimelineIO/blob/main/src/opentimelineio/stack.cpp)):

```cpp
return TimeRange(RationalTime(0, duration.rate()), duration);
```

and `available_range` takes the **max** of children's durations, not the sum. So `Stack` is
the parallel/overlapping composition and `Track` is the sequential one. **The two schema
names exist precisely to split "plays after" from "draws over".**

**Z-order comes from position in the `Stack`, bottom-first.** *"The layers in a stack are
iterated from the bottom (the first entry in the stack) towards the top (the final entry in
the stack). Images in a stack overlay lower images using an alpha composite operation."*
([Timeline Structure](https://opentimelineio.readthedocs.io/en/stable/tutorials/otio-timeline-structure.html))
Note this is the *opposite* direction to Shotstack's first-is-top (see the
[API survey](./declarative-video-api-models.md)) — three of the formats surveyed so far
disagree on which end of a list is in front, which is exactly the kind of thing a generator
gets backwards. Montaget's explicit integer `layer` sidesteps that class of error entirely.

**Overlap *within* a track exists, but only as a special case.** *"Within a track, clips may
overlap via a `transition`. In that case, the contribution of track is the linear blend of
the elements joined by the transition."* (Timeline Structure, above.) The mechanism is the
`overlapping()` flag skipped in the accumulation loop and a `transition->in_offset()`
subtraction. So OTIO does not permit free overlap in a track; it permits a bounded,
typed exception for cross-dissolves, and pays for it with a special case in the
position calculation.

**Because a track has no absolute times, empty space needs an object.** `Gap` exists to
occupy time so following items land where intended — *"a `Gap` is meant to be
transparent"*, and in the worked example *"the `Gap` in 'Track-001' is 4 frames long, and
the track below, 'Track-002', has frames 102-105 of 'Clip-003' aligned with the `Gap`
above."* This is a direct cost of the sequential model that an absolute-time model does not
pay: **to leave a hole you must write an object into the file, and to read what is on
screen you must know that an object is deliberately nothing.**

**Nesting is uniform:** *"By nesting a `Composition` (either `Track` or `Stack`) we can
refer to a `Composition` as though it was just another `Clip` in the outer
`Composition`."* A `Timeline` *"always has a top-level `Stack` object to hold its `Track`
children"* — i.e. the standard NLE picture is literally *stack-of-tracks*: overlap between
lanes, sequence within a lane.

**What this establishes for Montaget.** OTIO is evidence *for* ADR-0001's premise, not
against it: the format that thought hardest about this refused to make one container mean
both things. Montaget's `elements[] + layer` is the *Stack* half — parallel composition
with explicit stacking — with each element carrying its own absolute time range instead of
being aligned at 0. What Montaget does not have is the *Track* half, and OTIO shows exactly
what that half is: **ordinal timing, plus `Gap` objects, plus a transition special case.**

### 2.2 Adobe Premiere Pro — UXP API

Premiere's project file is undocumented binary/gzipped XML, so the published **UXP API** is
the primary source for its data model. It contradicts the naive picture in a way that
matters, and it is the most directly relevant finding after §2.1.

**A Premiere track item carries its own absolute sequence time.** From
[`VideoClipTrackItem`](https://developer.adobe.com/premiere-pro/uxp/ppro_reference/classes/videocliptrackitem/):
`getStartTime` *"Returns a TickTime object representing the starting sequence time of this
track item"*, and `getEndTime` the ending sequence time. Source trimming is a **separate**
pair: `getInPoint` / `getOutPoint` *"representing the track item in point relative to the
start time"*. There is also `getTrackIndex`.

This is a genuinely important result for #20, and it cuts *for* Montaget's shape rather than
against it: **Premiere is not OTIO-shaped.** A Premiere clip is not positioned by summing
what precedes it — it holds an absolute sequence start, an absolute sequence end, a separate
source range, and an integer track index. That is, field for field, Montaget's element:
absolute time range, plus separate source range, plus an integer lane. The distinction
CONTEXT.md draws between **Element** time range and **Source range** is exactly Premiere's
start/end vs in/out split.

**The sequencing constraint lives in the *edit operations*, not in the data.** From
[`SequenceEditor`](https://developer.adobe.com/premiere-pro/uxp/ppro_reference/classes/sequenceeditor/),
adding media is either `createInsertProjectItemAction` (*"Create insert ProjectItem into
Sequence Action"*) or `createOverwriteItemAction` (*"Create overwrite Sequence with
ProjectItem Action"*), and `createCloneTrackItemAction` describes itself as duplicating
*"using an insert or overwrite edit method."* **Ripple is a boolean argument to a removal,
not a property of a track**: `createRemoveItemsAction` takes a `ripple` parameter and a
`shiftOverLapping` parameter.

That is the sharpest thing in this whole document for option **(b1)**. In the tool the
reference class is named after, ripple is **an opt-in argument to an operation performed on
absolute-time data** — not an emergent consequence of ordinal storage the way it is in OTIO.
Premiere gets ripple *without* paying OTIO's read cost, because it stores absolute times and
recomputes them on edit. An agent doing arithmetic over a JSON array and rewriting downstream
starts is doing precisely what Premiere does; it is not working around the absence of a
feature, it is implementing the feature the same way Premiere does.

**The insert/overwrite pair is itself the evidence that a track cannot hold overlapping
items.** If a lane permitted free overlap there would be nothing to choose between: you would
simply place the clip. The API forces the caller to say whether the incoming item pushes the
existing ones aside or replaces them, which is only a question if two items cannot occupy the
same time on the same track.

**Empty space is addressable as a kind of track item.**
[`VideoTrack.getTrackItems`](https://developer.adobe.com/premiere-pro/uxp/ppro_reference/classes/videotrack/)
takes parameters filtering by item type and including or excluding *empty* items — so, as in
OTIO's `Gap`, a hole in a Premiere track is a thing the model can hand you, not an absence.
**NOT CONFIRMED:** whether an empty item is materialised in the stored project or synthesised
on query; the reference does not say, and the constants page for track-item types 404s.

**A track carries lane-level state.** `VideoTrack` exposes `name`, `id`, `getIndex()`,
`isMuted()` / `setMute()` and lock-changed events. This is the one thing §4's option (c2)
buys that the flat model has no home for.

**NOT CONFIRMED:** that a higher video track index draws in front. Universally believed and
almost certainly true, but the UXP reference nowhere states a compositing direction, which is
itself consistent with §3 Q2 — no format surveyed makes this explicit except by side effect.

### 2.3 CapCut / JianYing `draft_content.json`

**There is no primary source.** ByteDance publishes no schema, no format documentation and no
scripting API for the CapCut project file; `draft_content.json` is an internal file that
third parties have reverse-engineered. Everything in this subsection is therefore **at best
second-hand**, and is flagged as such. It is included because CapCut is half the reference
class named in #20 and its absence would be a bigger hole than its uncertainty.

The strongest available evidence is
[pyJianYingDraft](https://github.com/GuanYixuan/pyJianYingDraft), a library that *writes*
draft files the application then opens successfully — so its model is empirically validated
even though it is not authoritative.

**Segments carry an absolute time range on the track.** `target_timerange` is documented in
the source as *"片段在轨道上的时间范围"* — the time range of the segment on its track — as a
`start` plus a `duration`. `source_timerange` is separate: *"截取的素材片段的时间范围, 对贴纸而言
不存在"* — the range taken from the source material, *which does not exist for stickers*.

That is the **third** independent confirmation (with §2.2 and §2.4) of two things Montaget
already does: timeline position stored absolutely on the item, and source range as a separate
optional concern that only time-based media has. CONTEXT.md's **Source range** entry — *"Only
elements built on time-based media — video and audio — have one; images, text and shapes have
no insides"* — is the same rule CapCut applies to stickers.

**Overlap within a track is prohibited.** `Track.add_segment` requires that a segment *"不与
现有片段重叠"* — must not overlap existing segments — and raises `SegmentOverlap` with *"New
segment overlaps with existing segment"*. So CapCut agrees with every other format in both
surveys: **a lane is a non-overlap constraint.** ADR-0001's premise holds for the tool the
reference class is most identified with. **NOT CONFIRMED** whether the constraint is enforced
by the application or only by this library, but a library that generates files the app opens
would have no reason to invent it.

**Tracks are typed by media kind**, six of them: `video`, `audio`, `effect`, `filter`,
`sticker`, `text`. Track ordering is an integer `track_order`, documented as *"内部顺序, 值越大
越靠后导出"* — internal ordering, larger values exported later.

This is a **second and sharper counter-example to ADR-0001's rejection of per-kind
collections** than Resolve's (§2.4). Resolve's kinds (video/audio/subtitle) never compete for
z-order; CapCut's `text` and `sticker` plainly do. So a shipping product in exactly Montaget's
class does partition by kind *and* has to stack across kinds. **NOT CONFIRMED — and this is a
real gap — how CapCut resolves z-order across kinds.** Segments export a
`track_render_index` field, but the library sets it to `0` and offers no explanation, and no
primary source describes it. Whether cross-kind stacking is by `track_order`, by
`track_render_index`, or by a fixed kind precedence could not be established.

The honest summary: **the CapCut evidence supports the non-overlap finding and the
absolute-time finding, and leaves the z-order question open.** Anyone relying on the per-kind
observation to argue against ADR-0001 should first resolve `track_render_index` from a real
export.

### 2.4 DaVinci Resolve scripting API

*Provenance caveat: Blackmagic ships its scripting reference as a `README.txt` inside the
application rather than publishing it on the web. The text below is quoted from a verbatim
mirror of that README
([b3n0y/ResolveDevDoc](https://raw.githubusercontent.com/b3n0y/ResolveDevDoc/main/docs/source/readme_resolveapi.rst)),
cross-checked against a second mirror
([ResolveDevDoc on readthedocs](https://resolvedevdoc.readthedocs.io/en/latest/readme_resolveapi.html)).
It is vendor text at one remove, not a vendor-hosted page.*

**Resolve agrees with Premiere, not with OTIO: a timeline item holds an absolute frame
position.**

- `GetStart() --> int # Returns the start frame position on the timeline.`
- `GetEnd() --> int # Returns the end frame position on the timeline.`
- `GetLeftOffset() --> int # Returns the maximum extension by frame for clip from left side.`
  — i.e. source handles, a separate concern from timeline position, again mirroring the
  Element / Source-range split in CONTEXT.md.

**A track is an addressing coordinate, not a container that owns times.** Items are reached
by `(trackType, index)`, never by walking an ordered child list:

- `GetTrackCount(trackType) --> int # Returns the number of tracks for the given track type
  ("audio", "video" or "subtitle").`
- `GetItemListInTrack(trackType, index) --> [items...] # Returns a list of timeline items on
  that track (based on trackType and index). 1 <= index <= GetTrackCount(trackType).`
- `GetTrackName(trackType, trackIndex) --> string` — again, lane-level state.

Combined with §2.2, this makes **two of the three named desktop NLEs store absolute times per
item and use the track only as a lane index plus an edit-time constraint.** The ordinal model
of §2.1 is specific to OTIO's *interchange* abstraction; it is not how the editors themselves
represent a timeline. This weakens the "tracks mean ordinal timing" framing considerably —
the honest statement is **tracks mean a non-overlap constraint enforced at edit time over
data that is stored absolutely.**

**Sequencing again lives in the verbs.** `AppendToTimeline(clip1, clip2, ...)` appends;
`InsertGeneratorIntoTimeline(generatorName)` inserts. The API offers no way to state that two
items overlap on one track — you choose an edit verb and the application computes positions.

**One point that cuts against ADR-0001, and it should be recorded:** Resolve's tracks are
**typed by media kind** — `"audio"`, `"video"`, `"subtitle"` are not track *names*, they are
part of the addressing scheme, and there is no way to ask for "everything at frame 150"
across kinds without iterating three type-partitioned spaces. This is a mild real-world
counter-example to ADR-0001's rejection of **per-kind collections**, whose stated objection
was that *"cross-kind stacking becomes unanswerable: if a text and a shape live in different
arrays, nothing in the document's structure says which draws in front."* Resolve does exactly
that and lives with it — because subtitles and audio never compete with video for z-order, so
the ambiguity ADR-0001 fears does not arise for *those* particular kinds. It would arise for
Montaget's text/shape/image, which Resolve keeps together on video tracks. **The ADR's
argument survives, but its scope is narrower than stated.**

### 2.5 FCPXML `spine` / `lane`

All quotes below are DTD comments from Apple's own hosted copy of the
[FCPXML v1.5 DTD](https://developer.apple.com/library/archive/documentation/Miscellaneous/Conceptual/LegacyDTDsFinalCutPro/FCPXMLDTDv1.5/FCPXMLDTDv1.5.html).
This is the single best source in the file after §2.1, because **Apple split sequencing and
stacking into two orthogonal mechanisms and documented both in one place.**

**Sequencing is a container.** *"A 'spine' is a container for elements ordered serially in
time. Only one story element is active at a given time, except when a transition is
present."*

That second sentence is §2.1's finding restated by a different vendor: a serial container,
plus a single typed exception for transitions. **Two independent formats reached the same
design, including the same escape hatch.** The convergence is strong evidence that "sequential
container with a transition special case" is what a track *is*, not an accident of one
implementation.

**Stacking is an integer attribute, and it is signed.** From the `ao_attrs` entity comments:

```
<!-- The 'lane' attribute specifies where the object is contained/anchored relative to its parent: -->
<!--    0 = contained inside its parent (default) -->
<!--    >0 = anchored above its parent -->
<!--    <0 = anchored below its parent -->
<!-- The 'offset' attribute defines the location of the object in the parent timeline (default is '0s'). -->
```

So FCPXML has **an integer stacking coordinate with an explicitly stated direction — higher is
in front — carried as a property of the item, exactly like Montaget's `layer`**, plus a
separate `offset` giving position in the parent's timeline. Apple did not overload one concept
to mean both things; it shipped `spine` for "plays after" and `lane` for "draws over".

This is the most direct vindication available of the *shape* of ADR-0001's decision. It is
also the fourth distinct answer to §3 Q2's z-order question, and — with OTIO's stated
bottom-first — one of only two formats surveyed across both documents that says which way is
front at all.

**`gap` is confirmed in a second format.** *"A 'gap' element defines a placeholder with no
associated media. Gaps cannot be anchored to other objects."* `duration` is `#REQUIRED` on it.
Empty time again costs an object. Note the second sentence: a gap is a serial-container
citizen only — it is meaningless in the stacking dimension, which is precisely the asymmetry
§2.1 predicted (`Stack` needs no gaps; `Track` does).

**`lane` is *anchoring*, not just z-order — and this answers a question §3 Q4 called
inexpressible.** A lane value positions an item *relative to its parent*, and FCP's connected
clips move with the item they are anchored to. That is a published, shipping design for
**"this element always sits directly under that one"**: make the relationship structural
rather than numeric. See the revision note in §3 Q4.

**Timing attributes distinguish timeline position from source position**, as in Premiere and
Resolve: `%clip_attrs;` carries `offset` (position in parent), `start` (source in-point) and a
`#REQUIRED duration`. `clip` additionally has `audioStart` / `audioDuration` *"to define J/L
cuts (i.e., split edits) on composite A/V clips"* — worth noting against ADR-0001's rejection
of **a clip owning its audio**: FCP does pair them, and then needs two extra attributes to
un-pair them again for the commonest edit in the craft. That is the ADR's argument, confirmed
from the other side.

**NOT CONFIRMED:** whether Final Cut writes an explicit `offset` on every child of a `spine`
or relies on the serial ordering plus the `0s` default. The DTD makes `offset` optional and
declares the spine serial, so both are permitted by the schema; determining what the
application actually emits needs a sample export, which was not obtained.

## 3. The five questions from #20

### Q1 — What a track actually *is* in the data model

*Revised after §2.2–§2.5. The first draft of this answer, written from OTIO alone, was
wrong in an instructive way and the correction is the most useful thing in this document.*

**A track is a non-overlap constraint. It is *not*, in the shipping editors, ordinal
timing.** Those are two separable things and only the interchange formats conflate them.

Split the five sources into two groups:

**Group 1 — interchange formats, where order *is* timing.** OTIO's `Track` derives a child's
position by summing prior durations (§2.1); FCPXML's `spine` is *"a container for elements
ordered serially in time"* (§2.5). Both add exactly one escape hatch for transitions — OTIO's
`overlapping()` skip, Apple's *"Only one story element is active at a given time, except when
a transition is present."* Two vendors, same design, same exception.

**Group 2 — the actual editors, where every item stores its own absolute time.**

- Premiere: `getStartTime` *"the starting sequence time of this track item"*, `getEndTime`,
  plus separate `getInPoint`/`getOutPoint` and a `getTrackIndex` (§2.2).
- Resolve: `GetStart() # Returns the start frame position on the timeline`, `GetEnd()`, items
  addressed as `(trackType, index)` (§2.4).
- CapCut: `target_timerange` = *"片段在轨道上的时间范围"*, a start plus a duration, with
  `source_timerange` separate (§2.3).

**Three for three, the editors store what Montaget stores**: an absolute time range on the
item, a separate source range, and an integer lane coordinate. Nobody in Group 2 makes a
clip's position depend on its neighbours.

So where does the constraint live? **In the edit operations.** Premiere makes you choose
`createInsertProjectItemAction` or `createOverwriteItemAction`; Resolve gives you
`AppendToTimeline` and `InsertGeneratorIntoTimeline`; CapCut rejects an overlapping segment
outright with `SegmentOverlap` (§2.3). The application computes new absolute times and writes
them down. **A track is a rule applied at write time to data that is stored flat and
absolute.**

**What this does to ADR-0001.** Its load-bearing sentence is that *"in every comparable tool
a track constrains its contents to play in sequence without overlapping."* That is now
confirmed five for five, from primary sources, and is the best-evidenced claim in either
survey. What is *not* confirmed — and what the ADR never actually claimed, though it is easy
to read in — is that adopting a track would force ordinal timing on Montaget. It would not.
Premiere is proof that you can have the constraint and keep absolute times.

This makes option (c2) in §4 (tracks as containers over absolutely-timed children) a
materially stronger candidate than the OTIO-only reading suggested, and it makes the choice
less dramatic than #20's framing: the real question is not "flat and absolute vs tracks" —
every editor examined is flat and absolute underneath — but **"should the non-overlap rule be
expressible, and if so, where does it live: in the data, in a validator, or in a tool?"**

### Q2 — How z-order is expressed

**No convention exists.** Counting both surveys, four formats give at least three answers:

- OTIO: derived from position in the `Stack`, **bottom-first** — *"iterated from the bottom
  (the first entry in the stack) towards the top (the final entry)"* (§2.1).
- Shotstack: derived from track index, **first-is-top** — the exact opposite (see
  [API survey](./declarative-video-api-models.md)).
- Creatomate: an explicit `z_index` that overrides definition order, with the trap that
  *"ALL z-indexed children draw above every non-z-indexed sibling, whatever the value"*
  (API survey).
- Editly / JSON2Video: no lanes at all; document order within a scene.
- **FCPXML: a signed integer `lane` on the item, direction stated outright** — *"0 = contained
  inside its parent (default), >0 = anchored above its parent, <0 = anchored below its
  parent"* (§2.5). Higher is in front.
- Premiere: **NOT CONFIRMED** — the UXP reference never states a compositing direction (§2.2).
  CapCut: **NOT CONFIRMED** — an unexplained `track_render_index` (§2.3).

FCPXML is worth dwelling on, because it is the closest thing to independent confirmation of
Montaget's design that this research found: **Apple, facing exactly this question, shipped
`spine` for "plays after" and a per-item integer `lane` for "draws over", and did not overload
one for the other.** Montaget's `layer` is FCPXML's `lane` with the anchoring dropped.

This is the clearest finding of the whole exercise and it does **not** depend on the
sequencing question. A generated document that gets the direction backwards renders
successfully and looks wrong. Montaget's explicit integer `layer`, with a stated direction
(*"higher draws in front"*, CONTEXT.md), removes the class of error rather than picking a
side of it. **Nothing in this research argues against `layer` as the z-order mechanism.**
The whole live question is about *sequencing*, which is a separate axis that these formats
happen to have welded onto the same field.

### Q3 — Does the sequencing constraint buy anything an *agent* wants? (ripple edit)

Ripple edit is the strongest argument for tracks, so it deserves the sharpest statement:
under a sequential container, "insert a 2s shot at 0:12" changes **one** record. Everything
downstream re-times for free, because position *is* order. Under absolute times the same
edit rewrites every downstream `start` and `end`.

**§2.2 substantially settles this question, and not in the direction the ripple argument
assumes.** In Premiere, ripple is **an argument to an operation, not a property of the
data**: `createRemoveItemsAction` takes a `ripple` boolean and a `shiftOverLapping` boolean
(§2.2). Premiere stores absolute times per item (Q1 above) and *recomputes them when you ask
for a ripple*. So the tool the reference class is named after does exactly what an agent
editing Montaget JSON would do: hold absolute times, and rewrite the downstream ones on
demand. **Ripple is not something a track model gives you for free; it is something an editor
implements over flat absolute data, and it is opt-in even there.** That removes the premise of
the classic argument. The remaining four points stand on their own:

**1. If the cheap write is bought with ordinal storage, it trades a cheap write for an
expensive read — and an agent reads far more often than it writes.** (This applies to the
Group 1 formats of Q1 — OTIO `Track`, FCPXML `spine` — and to option (c1) only; per Q1 the
editors do not pay this.) §2.1 is precise about the price: the position of child *n* is a loop over
children 0..n-1. An agent asking "what is on screen at 6.2s" — the question ADR-0001 is
built around — must run that accumulation for every lane, while also recognising `Gap`
objects as deliberate nothing and applying the transition special case. The agent asks that
question on every turn, to know the state of the thing it is editing. It performs the
insertion once. Optimising the once at the expense of the every-turn is the wrong trade for
this author.

**2. The write cost is mechanical; the read cost is not.** Recomputing downstream times is a
single pass of addition over a JSON array — an agent can do it with a two-line script and
verify it by re-reading the file, and the result is fully checkable after the fact. There is
no equivalent escape for the read: resolving positions in a track model requires evaluating
the document, and "the project is inert data understood by reading, not evaluating" is the
project's stated premise (CONTEXT.md: a project *"states, by being read, what is on screen
at any given moment"*).

**3. Ripple is only free *within a lane*, and Montaget's own fixture is the case where that
is wrong.** In a track-based NLE, rippling one track shifts that track only; audio,
captions and overlays on other tracks stay put unless the human engages a sync/ripple-all
affordance — which is a **UI** feature. Montaget has no UI. ADR-0001's fixture is exactly
the shape that breaks: *"one still is on screen for 14.5s with six unrelated audio events
under it, three belonging to no visual at all."* A per-lane ripple through that produces
silent desynchronisation — a render that succeeds and is wrong. The agent would have to
reason about cross-lane scope regardless, which is most of the work the ripple was supposed
to save. **Partly confirmed since:** Premiere's removal action takes *both* a `ripple` flag
and a separate `shiftOverLapping` flag (§2.2) — two knobs, precisely because "what else
moves" is a second decision that the track structure does not answer by itself. **NOT
CONFIRMED:** the exact semantics of either flag; the UXP reference names the parameters
without describing their effect.

**4. What the flat model actually loses is the *diff*, not the edit.** This is the one real
cost and it should not be waved away. Under absolute times, a conceptually one-line change
produces a diff touching every element after the insertion point. A human reviewing the
commit cannot see at a glance that only one thing was intended, and a second agent reading
the diff has the same problem. That is a genuine, concrete disadvantage of (a) — and note
it is a *review* cost, not an authoring cost, and it is addressable by tooling (option b1)
without changing the time model.

**Provisional answer:** ripple is largely a mouse affordance. Its value comes from the
human's inability to retype twenty numbers, and it is bought with a read-time cost that a
human pays with their eyes for free (they are looking at a rendered timeline, not at the
file) and an agent pays with arithmetic every turn. The asymmetry runs the opposite way for
the two authors, and Montaget has only one of them.

### Q4 — What the flat model makes hard

- **"Insert 2s here and push everything back."** O(n) edit, O(n) diff, and — the sharp edge
  — the agent must decide *what* "everything" means. Everything after 0:12? Everything on
  layer 0? Everything except the background music? The format cannot express the answer, so
  it cannot check the answer either. In a track model the format answers "everything in this
  lane" for you, which is sometimes right and sometimes exactly wrong (see Q3.3).
- **"Swap these two segments."** Under absolute times this is two blocks of time-range
  rewrites, and it is *easier* than under tracks in one respect — a segment is identified by
  its `group`, and groups are kept contiguous in the array by convention (ADR-0001), so the
  edit is local. But if the two segments have different durations, everything between and
  after them shifts too, and the "everything" ambiguity above returns.
- **"This element always sits directly under that one."** Genuinely inexpressible. `layer`
  is an absolute integer, so "directly under X" must be maintained by hand, and inserting a
  new element between them means renumbering. Creatomate has the same problem and hits it in
  its mask feature (*"the element is used as a mask for the element one track below it"*,
  API survey), which is relational and therefore fragile. **A track model does not solve this
  either** — it converts renumbering into reordering, which is a different edit, not a
  cheaper one.

  **Revised after §2.5 — FCPXML does solve this, and not with tracks.** Apple's `lane` is
  *anchoring*: *"the 'lane' attribute specifies where the object is contained/anchored
  **relative to its parent**"*, `>0` above and `<0` below. A connected title is attached to
  the clip it annotates and travels with it. The mechanism is **a relationship between two
  items, orthogonal to both sequencing and absolute stacking** — neither a track nor a global
  z-index. That is a live design option for Montaget that #20 does not list, and it is added
  as **(b5)** in §4. It is not free: it makes `layer` relative, so "what draws in front"
  requires walking to the parent — a small, bounded dose of exactly the arithmetic ADR-0001
  exists to avoid.
- **Accidental overlap on a lane meant to be sequential.** The flat model has no way to say
  "these are meant to be back-to-back", so it has no way to warn that an edit made them
  overlap by 0.3s. The failure is a silent visual one. This is the gap option (b3) targets.

### Q5 — Prior art in agent/LLM-facing formats

Covered in full by the [API survey](./declarative-video-api-models.md) and not repeated
here. The one finding that bears directly on this ticket: **Creatomate is the vendor with
the most explicitly LLM-facing documentation** (a dedicated `llms.txt` reference more
detailed than its human docs — *"evidence that models authoring this JSON directly is the
expected usage"*), and it is also the vendor whose docs describe its own optional-sequencing
mechanism as *"the number-one source of broken renders"*, warning that smaller models
*"produce broken RenderScript without warning."*

That is the closest thing to direct evidence anyone has published on the exact question #20
asks — a track/sequencing model, aimed at machine authors, self-reported as the primary
failure mode. It is evidence against option (b2) specifically, not against tracks in
general, and it should be read as one data point rather than a proof.

## 4. Options

Presented as options, not a verdict. The choice is the human's.

Every option below is scored on the same three things: **what it costs**, **what it buys an
LLM agent authoring this JSON with ordinary file tools** (Read/Edit/Write over a file in
git — *not* a human dragging with a mouse, and not necessarily an MCP call), and **exactly
which text in [`docs/adr/0001-flat-element-list.md`](../adr/0001-flat-element-list.md) would
have to be amended**.

The pivot for all of them is the OTIO mechanism in §2.1: a sequential container buys a
cheap *write* (insert one thing, everything after it re-times for free, because position
*is* order) by selling a cheap *read* (position becomes an O(n) prefix sum, plus `Gap`
objects, plus a typed exception for overlap). ADR-0001's driving requirement is a cheap
read. So the real question in every option is: **how much read-time arithmetic is it
willing to buy back, and what does it get for it?**

### Option (a) — keep `layer` + free overlap, unchanged

No format change. Absolute `start`/`end` on every element, integer `layer`, free overlap,
`group` as an inert label.

**What it costs.**

- No format help for the three operations in question 4 (§3). "Insert 2s at 0:12" rewrites
  every downstream `start` and `end`: an O(n) *edit*, and — the concrete cost — an O(n)
  *diff*. A change that is conceptually one insertion shows up in git as a change to every
  element after it, so review can't see at a glance that only one thing was intended.
- **No expression for sequencing intent, therefore no validation surface.** Two elements
  meant to be back-to-back on `layer: 0` that accidentally overlap by 0.3s after a bad
  edit are indistinguishable, to the format and to the validator, from two elements
  deliberately cross-fading. The failure is silent and visual. This is the same class of
  failure the prior survey found in Shotstack (*"the overlap rule fails as silent flicker
  rather than an error"*, [API survey](./declarative-video-api-models.md)) — except
  Shotstack at least has a rule to fail.
- `layer` is an unfamiliar word to a reader arriving from CapCut or Premiere, who will look
  for lanes. (This is a docs cost, not a format cost.)

**What it buys the agent.**

- "What is on screen at 6.2s" stays a single filter over a single array — no prefix sums,
  no `Gap` objects to recognise as deliberate nothing, no transition special case. The
  agent answers it inside its own turn, from the text, with no tool call.
- Every element's record is self-contained: its absolute time is *in it*. A targeted edit
  is a targeted edit; a partial read of the file still yields true facts.
- Z-order is an explicit integer, so the first-is-top/last-is-top coin-flip that the two
  surveys together found four different answers to (OTIO bottom-first, Shotstack
  first-is-top, Creatomate `z_index` with its non-z-indexed sibling trap) is not a class of
  error that can occur here.

**ADR text to amend: none — and the supporting text gets *stronger*, not weaker.** ADR-0001's
Considered-options paragraph asserts *"A survey of four shipping declarative video APIs found
that no product uses a track as a pure stacking lane"*, and CONTEXT.md's rejected-term entry
generalises it to *"Premiere, Resolve, OpenTimelineIO, Shotstack, Creatomate"*. That
generalisation was, before this research, an assertion. It is now **confirmed five for five
from primary sources**: OTIO `Track` (§2.1), FCPXML `spine` (§2.5), CapCut's `SegmentOverlap`
(§2.3), Premiere's insert-or-overwrite pair (§2.2), Resolve's append/insert verbs (§2.4). The
citations should simply be added, and CapCut added to the list.

**One correction is owed, though.** ADR-0001 and CONTEXT.md are worded so as to suggest tracks
and absolute times are alternatives. Per §3 Q1 they are not: **all three shipping editors store
absolute per-item times *and* have tracks.** The wording should be tightened to say what is
actually true and actually sufficient — a track constrains its contents not to overlap — rather
than implying a time model that only the interchange formats use. Leaving it as-is risks the
next reader rejecting a good option (c2) for a reason that is not real.

### Option (b) — `layer` plus an opt-in sequencing affordance

The interesting middle, and the one with the most internal variation. "Opt-in sequencing in
an absolute-time format" is not one design; it is at least four, and they differ sharply on
whether the read cost comes back.

#### (b1) Sequencing as an *operation*, not a field — materialised into absolute times

Montaget grows an editing operation (an MCP tool, or a documented script) — `insert_at`,
`ripple`, `swap` — that rewrites absolute times and hands back a normal project file. The
document's shape never changes. There is no new field, no new noun, nothing new to read.

- **Cost:** two authoring paths that must agree. The stated target agent edits JSON with
  ordinary file tools; it gets no help unless it chooses to call the tool, and an agent
  that hand-edits after a ripple can produce something the tool would not have produced.
  Also, the tool has to guess scope — see the sync problem in §3 question 3: rippling
  "everything after 0:12" and rippling "everything after 0:12 on layer 0" are different
  operations and the format cannot say which was meant. Whichever it picks is wrong half
  the time, so the scope has to be an argument, which means the agent has to have thought
  about it — which is most of the work.
- **Buys:** the write cost drops to one call; the diff is still O(n) but is now
  machine-generated and uniform, which is easier to review than hand-arithmetic. The read
  cost stays exactly zero. Nothing about the inert-data principle is touched.
- **ADR text:** the Consequences bullet *"Ordinary cuts — 'insert a shot at 0:12 and shift
  everything after' — get no help from the format. Whether that is solved by relative
  authoring materialised into absolute times, or another way, is deferred to the time-model
  decision."* This option is that deferral resolving to **"solved outside the format, by an
  operation"**. The bullet would be rewritten to say so, and to name the scope-argument
  problem. Nothing else in the ADR changes; no container is introduced, so *"There is no
  track, no scene, and no container of any kind between the project and an element"*
  survives verbatim.

#### (b2) Relative timing in the document — `start: null`, or `{"after": "elem-7"}`

The Creatomate design: an element may omit its start and be sequenced after the previous
element sharing its track (see [API survey](./declarative-video-api-models.md)).

**This is the variant that reintroduces the O(n) read cost, and it reintroduces something
worse than arithmetic.** Under OTIO's `Track` the position of child 7 is at least a
*mechanical* sum of six durations sitting right there in order. Under a reference-based
relative start, the position of an element is the head of a dependency chain that must be
resolved — and if the referent is itself relative, resolved transitively. The document
stops being readable and starts being *evaluable*, which is precisely what CONTEXT.md's
opening sentence rules out: a project *"states, by being read, what is on screen at any
given moment"*.

- **Cost:** the driving requirement of ADR-0001 is gone. There are now two ways to say when
  something starts, so every reader must handle both, and every generator must choose. The
  prior survey already has the vendor's own verdict on this exact design: Creatomate's docs
  call the duration cascade *"the number-one source of broken renders"* and warn that
  smaller models *"produce broken RenderScript without warning"* — from the vendor with the
  most explicitly LLM-facing documentation of the four. That is close to a controlled
  experiment on the question this ticket asks, and it came out badly.
- **Buys:** genuinely short authoring for the common case — "these five clips, back to back"
  needs no arithmetic at write time at all, and stays correct when one of them changes
  duration. This is a real benefit and it is why Creatomate did it.
- A **bounded variant** — relative starts permitted only one level deep, only against an
  element that carries an absolute start — caps the read at O(1) per element and kills the
  transitivity problem. It still costs: two ways to express one thing, and a reader still
  doing arithmetic to answer the 6.2s question.
- **ADR text:** the driving-requirement paragraph — *"an agent must be able to answer 'what
  is on screen at 6.2s' by reading, with no arithmetic and no evaluation. Flat and
  absolute, that question is a single filter over a single array — a check the agent can run
  inside its own turn, without a tool call."* — must be weakened or deleted. That paragraph
  is the ADR's entire justification, so amending it is not an amendment; it is a new
  decision. CONTEXT.md's **Project** and **Element** entries would also need the second
  timing form added.

#### (b3) Declared sequencing intent that the renderer ignores — a lint, not a constraint

Elements may carry a free-text sequencing name, exactly parallel to `group`: the renderer
ignores it completely, times stay absolute, but Montaget's validator warns when two
elements sharing that name overlap in time or leave an unintended hole.

This is the only variant that adds *sequencing* without adding *sequencing semantics*.
Nothing is computed from it; it is a machine-checkable assertion about times that are
already fully written down.

- **Cost:** a fourth vocabulary word next to `group`, and the two are confusable — `group`
  means "these belong together, probably simultaneous", the new one means "these belong
  together, strictly one after another". Overloading `group` itself is cheaper in
  vocabulary but wrong in meaning: ADR-0001's own example of a group is *"every element of
  one vocabulary item"*, which is simultaneous by construction. There is also a real risk
  the field decays into decoration nobody maintains, since nothing breaks if it is absent
  or stale.
- **Buys:** the agent gets to *state* that a run of elements is meant to be contiguous, and
  then gets told when an edit broke it — converting §3's silent-overlap failure into a
  caught one, at zero read cost. It also gives the ripple operation of (b1) its missing
  scope argument: "ripple this sequence" is well-defined in a way "ripple layer 0" is not.
  And it costs nothing to ignore: a project that never uses it reads exactly as it does
  today.
- **ADR text:** the opening paragraph, *"Every element … carries the same `type`, time
  range, `layer` and optional `group`"* — the new optional field joins that list. A new
  Consequences bullet is needed stating that the field is inert and advisory. Everything
  else stands: no container is added, times stay absolute, the driving requirement is
  untouched. This is the smallest amendment of any option that changes the format at all.

#### (b5) Anchoring — a relative `layer` against a named parent element

*Added after §2.5; not one of the options #20 anticipated.*

An element may state its stacking position **relative to another element** rather than
absolutely — FCPXML's `lane`, where *"0 = contained inside its parent (default), >0 = anchored
above its parent, <0 = anchored below its parent"* (§2.5). A caption anchored `+1` to a shot
is in front of it and stays in front of it whatever renumbering happens elsewhere.

- **Cost:** the one thing §3 Q4 flags as inexpressible becomes expressible, but `layer` stops
  being a plain readable integer. Answering "what draws in front at 6.2s" requires resolving
  each anchor to its parent — bounded if anchors may not chain, unbounded if they may. It is
  the same *shape* of cost as (b2), applied to the z-axis instead of the time axis, and it
  needs the same guard rail (one level only). It also adds a second meaning to `layer`,
  which is currently the least ambiguous field in the format.
- **Buys:** relationships that survive edits. "The subtitle sits under the logo" stops being a
  fact an agent must re-derive and re-maintain after every insertion, and becomes something the
  document states. For an agent doing repeated targeted edits this is exactly the class of
  invariant that silently rots under (a).
- **Note the asymmetry worth flagging to the decider:** FCPXML's anchoring also carries
  *timing* (a connected clip moves in time with its parent). A Montaget version could take
  the z-order half and leave the timing half — which would be a genuinely novel split, and
  therefore unevidenced. **NOT CONFIRMED that anyone ships z-anchoring without time-anchoring.**
- **ADR text:** the opening paragraph's element field list, plus a Consequences bullet. The
  driving-requirement paragraph (*"no arithmetic and no evaluation"*) needs a carve-out for
  z-order resolution — smaller than (b2)'s carve-out, since it concerns stacking rather than
  the "what is on screen at 6.2s" question the sentence is actually about, but a carve-out all
  the same. CONTEXT.md's **Layer** entry would need rewriting.

#### (b4) Convention and recipes only — no format change, no tool

Document the idiom ("contiguous runs on one layer, kept adjacent in the array") and leave it
there. Costs nothing, buys nothing that (a) does not already have, and is listed only
because it is the honest floor of what "opt-in affordance" can mean.

### Option (c) — something genuinely track-shaped

#### (c1) Full OTIO-style tracks: order *is* timing

`tracks[]`, each an ordered list of children; a child has no start time; holes are `Gap`
objects; in-lane overlap is a typed transition or is impossible.

- **Cost:** everything §2.1 documents, paid in full. Position becomes an O(n) prefix sum, so
  ADR-0001's driving requirement is abandoned outright. Empty space stops being empty and
  becomes an object that a reader must recognise as deliberate nothing. Overlap becomes a
  special case with its own type. Two or three new nouns (`track`, `gap`, and probably
  `transition`) enter a vocabulary that currently has six. And **an element's absolute time
  is no longer written on the element**, so a partial read of the file no longer yields true
  facts — which is a specific, sharp loss for an agent that reads files in windows.
  ADR-0001's own fixture is the stress case: *"one still is on screen for 14.5s with six
  unrelated audio events under it, three belonging to no visual at all"* — under (c1) each
  of those events needs a lane, and the silences between them need `Gap` objects.
- **Buys:** the one-line insertion diff, structural impossibility of accidental overlap,
  familiarity to a reader who pictures CapCut, and free ripple *within a lane* (with the
  cross-lane sync caveat from §3 question 3 — which is not a caveat the format solves, it
  is one every track-based NLE solves in its UI, and Montaget has no UI to solve it in).
- **ADR text:** effectively a rewrite, and the new ADR would supersede rather than amend.
  The **title** (*"A project is a flat list of uniform elements, not tracks or scenes"*),
  the **opening paragraph** in full, the **driving-requirement paragraph**, and the
  **Considered options → Tracks** paragraph all invert. In CONTEXT.md, the **Layer** entry
  (whose *Avoid* list literally contains "track") and the whole **Track** rejected-term
  entry are deleted. Note one live tension: ADR-0001's argument against *scenes* is *"nesting
  implies a local clock whether or not one exists. You cannot put elements inside a
  container and then ask readers not to infer that their times are relative to it."* A
  `Track` is a container **whose children's times genuinely are relative to it**, so that
  argument does not merely fail to block (c1) — it becomes an argument that (c1) is at least
  *honest* where scenes were a trap. Whoever writes the superseding ADR has to deal with
  that sentence either way.

#### (c2) Tracks as pure z-lanes: containers whose children keep absolute times

`tracks[]` exists, children still carry absolute `start`/`end`, and the track is a named
bucket plus a stacking position — no sequencing constraint at all.

*Upgraded after §2.2–§2.4: this is closer to the shipping editors than the first draft of
this section assumed. Premiere, Resolve and CapCut all pair tracks with absolute per-item
times (§3 Q1), so (c2) plus a non-overlap rule is essentially the real CapCut/Premiere data
model.*

- **Cost:** as a *pure* z-lane with no constraint, this is the shape no surveyed product uses
  — every track-like concept in Shotstack, Creatomate, Editly and JSON2Video carries a
  non-overlap constraint, OTIO calls the unconstrained container a `Stack` rather than a
  `Track` (§2.1), and CapCut enforces the rule explicitly (§2.3). So the word would promise
  sequencing to every reader arriving from any of them and not deliver it — ADR-0001's stated
  objection, unchanged and now better evidenced. Calling it `lane` or keeping `layer` avoids
  the false promise but then it is barely (c) at all.
- **The variant that actually matches the reference class** is (c2) *with* the constraint:
  containers, absolute times on children, and a rule that children of one track may not
  overlap — enforced by the validator, not by the storage. That is Premiere and CapCut,
  faithfully. It costs a container (and so must answer ADR-0001's *"nesting implies a local
  clock"* objection — answerable here, since children keep absolute times and there is no
  local clock to infer, but it must be answered in writing or readers will infer one anyway).
  It buys the reference-class mental model outright, plus the validation that (b3) reaches for
  by other means. This is the strongest form of (c) and the one a decider should compare (a)
  against.
- **Buys:** structural locality. ADR-0001 currently recovers this by convention —
  *"Elements sharing a `group` are kept contiguous in the array by convention. This is a
  formatting rule only; nothing depends on it"* — and a container would make it structural.
  It also gives per-lane properties (mute, hide, lane-wide opacity) somewhere to live, which
  the flat model has no home for. Read cost is unchanged: absolute times stay on elements.
- **ADR text:** the sentence *"There is no track, no scene, and no container of any kind
  between the project and an element"* is directly contradicted and must go. The
  **Considered options → Tracks** paragraph must be rewritten, since its argument is
  specifically about the word, not the structure. The **group-contiguity Consequences
  bullet** becomes moot. The driving-requirement paragraph survives — the 6.2s question
  becomes a filter over two nested arrays instead of one, which is still a read.

### The axis these sit on

| | read cost for "what's on screen at 6.2s" | write cost of "insert 2s at 0:12" | new nouns |
|---|---|---|---|
| (a) | filter one array | rewrite N elements, O(n) diff | 0 |
| (b1) | filter one array | one operation call, O(n) diff | 0 |
| (b2) | resolve a dependency chain | rewrite 1 element | 0–1 |
| (b3) | filter one array | rewrite N elements, O(n) diff, **validated** | 1 (inert) |
| (b4) | filter one array | rewrite N elements, O(n) diff | 0 |
| (b5) | filter one array; z-order needs an anchor walk | rewrite N elements, O(n) diff | 0–1 |
| (c1) | prefix-sum per lane, skip gaps | insert 1 child | 2–3 |
| (c2) | filter two nested arrays | rewrite N elements, O(n) diff | 1 |

The table is the argument in miniature: **only (b2) and (c1) buy the cheap write, and both
pay for it in the read.** Everything else is a choice about vocabulary, validation and where
the editing logic lives — not about the time model at all.

And per §3 Q1, (c1) is the *interchange-format* shape, not the reference class. **Premiere,
Resolve and CapCut are all in the (c2)-with-a-constraint row.** So the decision #20 poses is
narrower than it looks: nobody is asking Montaget to give up absolute times, because none of
the tools it is being compared to have. The live question is whether the non-overlap rule
should be expressible at all, and if so whether it lives in a container (c2), an inert label
checked by a validator (b3), an editing operation (b1), or nowhere (a).
