---
status: accepted
amends: 0077 (a moving element carrying `motion_blur` is also resolved at exact rational sample instants around the floored frame instant; the frame instant still decides presence and a video's source frame), 0040 (motion blur is refused as an `effects` member; the order is written down: each sample takes its effects, mask and `opacity`, the samples are averaged, then the blend), 0011 (`query --at` reports `motion_blur` as written and whether the element is `moving` or `still` at that frame)
---

# Motion blur is a per-element field that accumulates the element over a centred shutter

[#702](https://github.com/MBehtemam/Montagent/issues/702), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663). Montagent paints one sharp image
per frame. A fast move shows as separate copies, so the craft skill sets speed limits on
every move, and the capability map lists motion blur as something it can't do yet. The
fakes an agent reaches for are both poor. A `blur` effect smears in every direction, and
copies at lowered opacity duplicate an element and its keyframes, then drift out of step
with every later edit.

**Precedent.** Row 8 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)): CapCut has a per-clip motion
blur. Premiere's shutter angle on its Transform effect is sourced only from Adobe Community
posts, so this ADR does not lean on it. A per-layer switch with a composition-wide shutter
is After Effects only. CapCut is enough under
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md),
so no accepted prototype is required for entry. The build still waits on a measuring
prototype (§6), because the cost and the byte-identity are unknown.

Skia has no primitive for it ([#665](https://github.com/MBehtemam/Montagent/issues/665)).

Settled by three `/court` rounds of three jurors each (Opus, Sonnet, Fable), unanimous on
every question, with the owner ruling with the Judge's read each time.

## The decision

### 1. What it is: sub-frame accumulation

Motion blur paints the element N times, at N instants spread across a shutter interval
inside the frame, and averages the results. The smear is the element's own motion: a still
element gets none, a fast one gets a long smear, a rotating one an arc, and the smear falls
off as an eased move slows.

Nothing else is called motion blur. A static directional blur (a literal angle and length
that smear whether or not the element moves) is a different capability. It goes to the
named-effect ticket ([#704](https://github.com/MBehtemam/Montagent/issues/704)) as a
candidate under its own name, not to this field.

### 2. Where it is written

A visual element (`image`, `video`, `text`, `rect`, `ellipse`, `path`) may carry:

```json
"motion_blur": {"shutter": 180, "samples": 8}
```

- `shutter` is an integer angle in degrees, 1 to 360. The interval is
  `shutter / 360` of one frame.
- `samples` is an integer, 2 to 32. It is the number of paints on a moving frame, so the
  cost can be read from the file.
- Both keys are required. Neither is animatable. A keyframe list on either is a schema
  error.
- There is no `phase`. The interval is centred on the frame instant.
- It is a field, not an `effects` member. An effect is one step in a chain applied to one
  painted element. Motion blur repaints the element, chain included, so it has no place in
  the chain.
- There is no project-wide setting. The field is written on each element that should
  blur, so the cost and the look stay visible where the agent edits.

### 3. What the samples resolve

Frame n is painted at `instant(n) = ⌊n × 1000 / fps⌋` ms, as
[ADR-0077](0077-the-nine-render-readings-are-ratified.md) reads it. Its N sample instants
are exact rationals at the midpoints:

```
t_k = instant(n) + shutter/360 × (1000/fps) × ((k + ½)/N − ½),   k = 0 … N−1
```

They are evenly spaced and centred. At `shutter: 360`, no two frames share an instant.
The resolver already takes a rational instant. The painter's time is widened to it on this
path only, and the whole-millisecond path is unchanged.

**The whole element follows `t_k`.** That is every value its own keyframes resolve: the
transform, `opacity`, every animatable property
([ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)),
gradient stops, path `points` and a `units` stagger's values. Transitions do not follow it.
`slide` and `push` resolve before painting into whole-pixel offsets of a full frame
([ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)),
which is not the element's own motion.

**The frame instant decides presence and a video's source frame.** Samples only resolve
keyed values. A sample outside `[start, end)` does not make the element partly present, and
a `video` shows the one source frame held at the frame instant. Its footage is not
blurred, only its keyed values.

**The smear reaches only as far as the keyframes do.** Keyed values clamp past their first
and last record. On a frame where motion starts or stops at a keyframe, the smear is
therefore one-sided. That includes the frame just after a `shift` cut, where the keyframes
written at the cut clamp the later half's earlier samples. That is the intended reading.

### 4. Still, and the order of painting

An element is **still** at a frame when every value it resolves is equal at all N
instants. A still element is painted once, through today's path, and its bytes equal the
same element with no `motion_blur`.

A moving element is painted at each `t_k` with its `effects`, its `mask` and its
`opacity`. The N results are averaged with an integer average, in fixed order and correctly
rounded, so that N identical samples give back exactly the source bytes. Then
`blend` ([ADR-0147](0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md))
composites the average once.

### 5. What the tools say

- **`validate`.** Out-of-range values and missing keys are schema errors. One new `review`
  finding, **`R-MOTION-BLUR-STILL`**, fires when no resolved value of the element differs
  between any two instants inside `[start, end)`. A `units` stagger counts as motion.
  Motion lying wholly outside the element's life counts as still, so the finding fires only
  when the field is certainly useless. The repair is to remove the field. It is a review,
  never a refusal.
- **No check for `video`.** A video with keyed motion is a proper use. That its footage is
  not blurred is a rule in `format.md`.
- **No cost check.** `samples` is visible and capped in the file. A cost line in `render`'s
  report may follow once the prototype's numbers exist. It is not part of this ADR.
- **`query --at`** reports the field as written and one derived line: `moving` or `still`
  at the frame containing that instant. Values stay those at the frame instant. The sample
  instants are not listed.
- **`shift`** copies the field unchanged to both halves of a split and refuses nothing.

### 6. Byte-identical painting and cost

Each frame stays a pure function of the document and the frame number. The sample instants
are fixed by n, `fps`, `shutter` and `samples`. The average is integer and in fixed order.
Nothing is carried between frames, so a paint chunk's start is still indistinguishable from
`--from` ([ADR-0144](0144-render-paints-on-k-painters-over-chunks-and-the-spy-trailer-renders-in-a-minute.md)).
That is argued, not measured.

The averaged layer is a new kind of filter layer, so it falls under ADR-0144 §9. It is
either bounded by a bound measured byte-identical, or painted unbounded.

The build is gated on a measuring prototype
([#718](https://github.com/MBehtemam/Montagent/issues/718)). It must show:

- frames byte-identical across 1 to 10 painters, with the blur and shadow bounds hint on
  and off;
- a still element's bytes equal to no field;
- N identical samples averaging back to the source;
- the cost per moving element per frame at 1080p for `samples` 8, 16 and 32, against no
  blur.

If it fails the byte-identity run, motion blur is withdrawn, not excused.

## The four tests

| Invariant | How `motion_blur` passes |
| --- | --- |
| Literal values | Two integers. The sample instants and the average are fixed arithmetic, written down in §3 and §4. |
| Closed vocabulary | One field with two keys. Phase, an animatable shutter, adaptive samples, a project-wide setting and a directional blur are refused by name. |
| Checkable by `validate` | Bad values are schema errors, and `R-MOTION-BLUR-STILL` is decided from the file without painting a frame. |
| Exact-string replace | `"motion_blur": {"shutter": 180, "samples": 8}` is one string, added, changed or removed in place. |

## Why

**Accumulation, because the agent should not keep a second set of numbers in step.** A
directional blur would have to be re-derived from the keyframes after every edit, and it
cannot curve or fall off. `validate` could not catch a stale angle. Accumulation reads the
motion the agent already wrote.

**The whole element, because one rule beats a list.** If only the transform smeared, a shape
whose keyed radius grows would blur its position and keep a hard outline. Carving values
out would leave a list to keep in step with every future animatable property.

**Presence at the frame instant, because `query --at` must match the frame.** A sample that
decided presence would make an element's first frame partly transparent. `query` would then
call present an element that renders as a ghost. A video sample could also land on another
source frame, which sample-and-hold never produces.

**Midpoints, not end-points.** The `k/(N−1)` spacing includes both ends of the interval. At
`shutter: 360` it puts the last sample of frame n on the first sample of frame n+1.

**A static shutter, because an animated one is circular.** The shutter fixes the instants at
which values are resolved, so an animated shutter has no instant at which to be resolved.

**Rational instants, not whole milliseconds.** At 30 fps a 180° shutter spans about 16.7
ms, and 32 samples rounded down to whole milliseconds collapse to about 17 unevenly spaced
instants. `samples` would then not mean what it says, and the smear would band.

## Considered and refused

- **Not entering; faking it with `blur` or copies.** Refused in the opening and in "Why".
- **A static directional blur as motion blur, or both.** Refused in §1. A directional blur
  may enter on its own as a named effect.
- **Transform-only sampling.** Refused in "Why".
- **Blurring `slide` and `push`.** Refused in §3. If a transition smear is ever wanted, it
  is a transition's own option, decided separately.
- **Presence decided per sample.** Refused in "Why".
- **An `effects` member.** Refused in §2.
- **A project-wide setting.** Refused in §2. A default that element fields override may
  follow if a case asks.
- **A `phase` parameter.** It is one more value to misread, and no case asks for a trailing
  smear. It can be added later without breaking this field.
- **An adaptive sample count.** It would hide the cost and the output behind a renderer
  heuristic the file does not show.
- **An animatable shutter.** Refused in "Why".
- **Whole-millisecond sample instants.** Refused in "Why".
- **A review on every `video` carrying the field.** It would fire on correct projects.
- **A cost review or budget.** No measured basis yet.
- **`shift` refusing a split while moving, or a review of the cut frame.** It would block
  an ordinary edit for a one-frame, one-sided smear that §3 already states.
- **Two slices** (transform first, or renderer before tools). Either ships a state agents
  can see and would have to unlearn.

## Consequences

- `CONTEXT.md` gains a **Motion blur** entry.
- `format.md` gains the field, its bounds, the sampling rule, the order, the video rule and
  the one-sided smear.
- The capability map gains one line under **Motion**, and the can't-do item marked with
  #702 is deleted.
- The craft skill keeps its speed limits for elements without blur. Its opening line is
  replaced by the rule that, for a fast move, you set `motion_blur` (180°, 8–16 samples)
  rather than slowing the move. It is not a default. The sample advice may be revised once
  the prototype's cost numbers exist.
- The hand-off spec is one slice,
  [#719](https://github.com/MBehtemam/Montagent/issues/719), gated on
  [the measuring prototype](https://github.com/MBehtemam/Montagent/issues/718).
