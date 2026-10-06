---
status: accepted
amends: 0005 (an element carrying `source_time` has no source range: `source_start` and `source_end` are refused on it, and the curve names the source instead), 0020 (reverse playback is settled for video as a falling segment of `source_time`, not a future explicit field; `speed` and `overrun` are refused on an element carrying `source_time`), 0055 (a `video` carrying `source_time` must carry `volume` as the literal `0`), 0146 (`source_time` joins the one derived list of animatable properties), 0011 (`query --at` reports `source_time` and a derived `rate` for a remapped element)
---

# A speed ramp is a time-remap curve of source times on a video element

[#706](https://github.com/MBehtemam/Montagent/issues/706), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663). `speed` is one static multiplier
([ADR-0020](0020-speed-overrun-hold-loop.md)), so a clip cannot play at normal rate, slow to
0.3× and come back. The capability map lists speed ramps as something to keep out of the
design. Reverse playback was deferred by ADR-0020 to "a future explicit field".

**Precedent.** Row 11 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)): both editors ramp speed over a
clip. Premiere keys a speed curve with Bezier handles, and CapCut has curve points and named
presets. Premiere also reverses inside a ramp: a speed keyframe dragged with Ctrl/Cmd shows
"the speed as a negative percentage of the original speed"
([Adobe, archived January 2022](https://web.archive.org/web/20220125121626/https://helpx.adobe.com/premiere-pro/using/duration-speed.html)).

**The departure from precedent.** Both editors key the *rate*. This ADR keys the *source
time*, as After Effects' Time Remap does. The two are one-to-one: the remap curve is the
running sum of the speed graph, and the speed graph is the remap curve's slope. Every
Premiere ramp is exactly one remap curve. The capability enters under
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
on that equivalence alone, so no accepted prototype gates it. That ground is narrow: it
admits a different spelling of a mechanism the class has, not a different mechanism.

Settled by two `/court` rounds of three jurors each (Opus, Sonnet, Fable), unanimous on every
question, with the owner ruling with the Judge's read each time.

## The decision

### 1. The field

A `video` may carry `source_time`, an animatable property under
[ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md).
Its value is a **source time in integer milliseconds**: which moment of the file is on
screen. A negative value is a schema error.

```json
"source_time": [
  {"t": 12000, "v": 0},
  {"t": 13000, "v": 1000, "ease": "linear"},
  {"t": 15000, "v": 1600, "ease": "linear"},
  {"t": 16000, "v": 2600, "ease": "linear"}
]
```

That reads: normal rate for a second, 0.3× for two seconds, then normal rate again. Each
key's `t` is timeline milliseconds, like every keyframe's. A softer change of rate is written
with more keys, or with an `ease` on the keys around it.

- **The rate is the slope.** A steep segment plays fast, a shallow one slow, a flat one
  freezes, and a falling one plays in reverse. There is no `reverse` field.
- **An eased segment never plays at a constant rate.** A stretch at one steady rate is two
  keys joined by a `linear` ease.
- **A literal is a freeze frame.** `"source_time": 4200` shows source 4200 ms for the whole
  element. That is Premiere's Frame Hold. It replaces the fake of a short source range with
  `overrun: "hold"`.
- `source_time` is on `video` only. On `audio` it is a schema error (§4).

### 2. The curve is the only author of the source

On an element carrying `source_time`, each of `source_start`, `source_end`, `speed` and
`overrun` is a `validate` error, `E-REMAP-FIELD`. Its fix says to remove that field. The
curve already says which source moment shows at every instant, so a second statement of it
would be one more place for one key edit to leave stale.

ADR-0020's agreement rule, `end - start == round((source_end - source_start) / speed)`, does
not apply to a remapped element. Its replacement is:

> At every painted frame instant in `[start, end)`, the resolved source time (§3) lies in
> `[0, file duration)`.

It is checked at the instants the render paints, so an overshooting ease is caught exactly
where it would show. A failure is `E-SOURCE-OVERRUN`, in a new remap arm. It names the first
offending instant, the source time resolved there, and which side was crossed: below `0`, or
at or past the duration.

`loop` has no spelling on a remapped element. A jump back to the start needs two keys at
least 1 ms apart ([ADR-0082](0082-a-keyframe-list-must-be-written-in-ascending-t.md)), which
is a hack, not a seam. Looping stays a feature of un-ramped elements.

### 3. Which source frame a frame instant shows

At frame instant `instant(n)`, as
[ADR-0077](0077-the-nine-render-readings-are-ratified.md) reads it:

1. `source_time` is evaluated by the one number interpolation every animatable property uses
   (ADR-0146), under each key's `ease`.
2. Before the first key, the first key's value holds. After the last key, the last key's
   value holds. That is the ordinary keyframe rule, so the picture freezes there.
3. The value is rounded half-up to an integer millisecond.
4. The frame shown is the last source frame starting at or before that millisecond
   ([ADR-0096](0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md)).

`validate`'s check in §2 calls the same function the painter calls, so the check and the
render cannot disagree. There is no exact-rational path for linear segments. A remap written
to mimic a static `speed` is not promised to match that element to the millisecond, and
nothing compares them, because the two never sit on the same element.

### 4. Audio

A remapped `video` must carry `volume` as the literal `0`. Anything else is a `validate`
error, `E-REMAP-AUDIBLE`: a number other than `0`, a keyframe list (even one of all zeros),
or no `volume` at all, since the default is `1`. The fix says to put the sound on a separate
`audio` element.

`atempo` takes one fixed rate per filter, and the mix is built on exact sample counts at
48 kHz ([ADR-0077](0077-the-nine-render-readings-are-ratified.md)). A varying rate has no
exact spelling there. Premiere does not ramp audio either: under time remapping linked audio
"remains at 100% speed" and "does not remain synchronized with the video" (the archived page
above). Montagent refuses that desync rather than playing it. After Effects varispeeds the
audio instead, and that is the alternative left for later (see "Considered and refused").

### 5. What the tools report

- **`query --at`** prints, for a remapped element, `source_time` (the rounded millisecond of
  §3) and **`rate`**, both marked derived. `rate` is the change in resolved source time from
  this frame instant to the next, divided by the frame interval, printed signed with an `×`
  (`-0.5×`). It is what the viewer sees, and it has one value at a key, where the curve's
  slope may jump. Neither is ever accepted as input.
- **`R-REMAP-HELD-END`**, a new `review`, fires when the element's range holds a painted
  frame instant before its first key or after its last. Each end is reported separately,
  with the length of the frozen stretch. A literal `source_time` never fires it. Its message
  says that a deliberate freeze is written as a flat pair of keys, which silences it.
- **`shift`** changes nothing. A cut inside a remapped element is the existing straddle
  refusal, `E-SHIFT-STRADDLE`, and a moved element carries its `source_time` keys like any
  keyframe list. Their values are source times, so they do not move.

## The four tests

| Invariant | How `source_time` passes |
| --- | --- |
| Literal values | Every key's `t` and `v` is an integer millisecond. The rate is derived and printed, never written. |
| Closed vocabulary | One field. `reverse`, an animatable `speed`, a separate speed graph, ramped audio and a loop on a ramp are refused by name. |
| Checkable by `validate` | The agreement rule is evaluated at the painted instants with the painter's own function. `E-REMAP-FIELD`, `E-REMAP-AUDIBLE` and the remap arm of `E-SOURCE-OVERRUN` are decided from the file and the probed duration. |
| Exact-string replace | A key is one record. Moving where the slow-down lands is one `v` changed in place, with nothing else to keep in step. |

## Why

**Source time, because the file should say what is on screen.** Under keyed speed, which
source moment shows at an instant is the integral of the speed curve from `start`. Under an
eased segment that integral has no exact closed form, so `validate`'s agreement could only be
approximate, against [ADR-0045](0045-speed-invariant-is-evaluated-in-exact-arithmetic.md).
Keyed speed also fails two of ADR-0146's three tests. It does not resolve from the file and
the instant alone, and `E-SPEED-MISMATCH` relies on `speed` as one fixed value. And an agent
landing a slow-down on a given source frame would have to solve the integral backwards.
Under `source_time`, each key states the answer.

**One author, because one edit should be one edit.** Kept as bounds, `source_start` and
`source_end` would be a second spelling of the curve's extent. Moving a key could push the
curve past them, and the agent would learn of the second edit only from `validate`.

**One interpolation, because a check is only as good as its agreement with the render.** A
second, exact path for linear segments would make two code paths that can disagree by a
millisecond at a rounding boundary. `validate` could then pass a frame the render resolves
differently.

**Reverse here, because the curve already says it.** Refusing a falling segment would need a
monotonicity rule with no authoring benefit. A `reverse` field would have to compose with the
curve, and every way of composing it is a second spelling of a falling segment.

## Considered and refused

- **`speed` made animatable.** Refused in "Why".
- **Keyed `speed` with linear segments only.** Its integral is exact (piecewise quadratic),
  but it bans eases on one property against
  [ADR-0038](0038-ease-is-required-on-every-non-first-keyframe-record.md), and landing a
  source moment still means solving a quadratic.
- **A `reverse` field.** Refused in "Why".
- **`source_start` and `source_end` kept as required bounds.** Refused in "Why".
- **`source_start` and `source_end` required to equal the curve at `start` and `end`.** It
  breaks under reverse and says nothing about an overshoot in the middle.
- **Exact rational arithmetic on linear segments.** Refused in "Why".
- **Audio that follows the ramp, by varispeed (pitch moves with the rate).** It is After
  Effects' behaviour and a real effect (a tape stop), but it needs a continuously varying
  resampler Montagent does not have. It is a separate capability with its own precedent
  question.
- **Pitch-preserving variable stretch.** Not sample-exact, and a new dependency.
- **Linked audio left at 100% speed, as Premiere does.** It is a desync no one writes on
  purpose.
- **`volume` as a keyframe list of zeros on a ramped video.** One later edit to one key would
  bring the desync back, and nothing would check it.
- **The names `remap` and `time_remap`.** They name the operation, not the value, so they
  carry neither the unit nor what the number refers to.
- **A list-only field, refusing the literal.** It would make `source_time` the one animatable
  property that cannot take a literal, and a still would need two identical keys.
- **A prototype gate.** Refused in the opening. The spec's acceptance still requires a
  rendered ramp, a reverse and a freeze, byte-identical across painters.

## Consequences

- `CONTEXT.md` gains a **Time remap** entry. **Speed**, **Source range** and **Overrun** say
  that they do not apply to a remapped element.
- `format.md` gains the field, the four refused fields, the agreement rule, the resolution
  chain of §3, the audio rule and the held-end review.
- The capability map gains one line for time remapping (ramps, reverse, freeze frames), and
  the can't-do item marked with #706 is deleted.
- Every tool that reads a video's source range (`query --at`, the painter and its feeds,
  `frame`, the contact sheet, `measure`'s keyed-alpha series, `verify`) gains a remap branch.
  A falling segment cannot ride a forward-decoding feed, so it takes the per-frame
  `frame_at` path, and the spec reports that cost.
- The hand-off spec is one slice,
  [Time remap: `source_time` on video, with ramps, reverse and freeze frames](https://github.com/MBehtemam/Montagent/issues/745).
