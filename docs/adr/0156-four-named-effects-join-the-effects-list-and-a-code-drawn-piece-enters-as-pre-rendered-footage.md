---
status: accepted
amends: 0040 (the `effects` vocabulary gains `grain`, `glow`, `posterize` and `directional_blur`; later members join one at a time by their own ADR under §2's tests), 0017 (reaffirmed, not reopened: an open shader or script file in the project stays refused, and §7 names the route a code-drawn look takes instead), 0146 (`seed`, `size` and `mono` are never animatable; until ADR-0146's effect slice lands, a keyframe list on any parameter of these four members is an error naming ADR-0146)
---

# Four named effects join the effects list, and a code-drawn piece enters as pre-rendered footage

> **Amended by [ADR-0170](0170-svg-and-lottie-do-not-enter-as-sources-and-each-is-pre-rendered-through-the-skill.md).**
> §7's pre-render route widens from a code-drawn piece to an SVG still and a Lottie animation,
> under shared size, text and recipe rules.

[#704](https://github.com/MBehtemam/Montagent/issues/704), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663). The painter already runs a
runtime shader (the chroma keyer), but the format reaches only eight `effects` members. The
capability map lists "generative effects" as something Montagent can't do. Its "Not by
design" list says to pre-render an open shader and bring it in as footage, but no skill
does that. An agent who wants grain, a bloom or a smear has nowhere to go.

**The question an agent asks** is "the look I want is not in the effects list: what now?"
There are three answers, and this ADR records them together so that none is read without the
others:

- use a named member (§1–§6);
- or pre-render the look as footage (§7);
- an open shader or script file in the project stays refused (§8).

**Precedent.** Row 9 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)): a named-effect vocabulary is
inside the reference class. The candidates come from
[#703](https://github.com/MBehtemam/Montagent/issues/703). Premiere names about 90 video
effects, including Noise, the Glows, Posterize and Directional Blur. CapCut publishes no
list. Every member below has Premiere precedent, so none takes the prototype entry test of
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md).
The build still waits on a measuring prototype (§6), because shaders are a named risk on the
map.

Settled by two `/court` rounds of three jurors each (Opus, Sonnet, Fable), with the owner
ruling with the Judge's read each time.

## The decision

### 1. Members sit in the `effects` list, and nowhere else

A named effect is an `effects` member, applied in list order like the others
([ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md)). It filters the element's
own pixels, plus a reach it declares.

There is no generator element that paints with no source. A texture is a `rect` carrying an
effect and a `blend`
([ADR-0147](0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md)),
for example a grey rect with `grain`, blended `overlay` over footage. This is the same
grammar the capability map already teaches for a vignette and a light leak.

### 2. How a member gets in

A member enters only if it passes three tests:

1. **The look cannot already be composed** from existing members, paints and blends.
2. **It is one built-in Skia filter, or one small runtime shader** written by Montagent, with
   typed parameters.
3. **It stays inside the element's box, or inside a reach it declares** as a formula of its
   parameters, so the bounds hint of [#652](https://github.com/MBehtemam/Montagent/issues/652)
   can stay on.

The first batch is the four members below. Later members join one at a time, each by its own
ADR, under the same tests. The vocabulary stays closed and can grow.

### 3. Time and seed

A member that is random by nature takes a required literal `seed`. It **re-rolls on every
output frame**, by a fixed integer hash of the seed and the element's **local instant**. That
instant is an exact rational measured from the element's `start`.

- The hash uses no floats, so every painter draws the same value.
- Measuring from the element's own start means that when `shift` moves an element, its grain
  moves with it rather than changing.
- The re-roll rate follows the output `fps`, so a 24 fps render and a 60 fps render re-roll at
  different rates.

No other member reads time. Motion of the wave or ripple kind takes a keyed phase, so the
file still shows it. There is no general time input to a shader.

### 4. The four members

**`grain`**: `{seed, amount, size, mono}`

- `seed` is an integer from 0 to 2³¹−1, required and static.
- `amount` runs from 0 to 1 and is animatable. It is a symmetric offset per colour channel.
- `size` is an integer from 1 to 8, static. Each `size`×`size` cell shares one draw.
- `mono` is a boolean, static. `true` gives one draw applied to R, G and B (luma grain);
  `false` gives three independent draws (colour grain).
- Cells sit in **element-local space**, anchored at the box origin, so they move and scale
  with the element's transform.
- The offset is applied to non-premultiplied colour and clamped to 0–1. Alpha is never
  changed, so a transparent pixel stays transparent.

**`glow`**: `{threshold, radius, intensity}`, a threshold bloom

- `threshold` runs from 0 to 1, read against the Rec.709 luma of the non-premultiplied sRGB
  values.
- `radius` is in pixels, with σ = radius / 2, read exactly as `blur` reads it.
- `intensity` runs from 0 to 4 and is the gain on the glowing part.
- All three are animatable.
- **The bright-pass** scales each pixel's colour by
  `max(0, luma − threshold) / (1 − threshold)`. At `threshold: 1` the bright-pass is defined
  as empty, so nothing is ever divided by zero.
- The bright part is blurred and added (`Plus`) back over the element, inside the element's
  own layer and before its `blend`.
- The reach is `blur`'s 3σ.
- There is no `color`. A glow takes the element's own colours. A coloured alpha glow is
  already a zero-offset `shadow`, and the bright-pass is what `shadow` cannot do.

**`posterize`**: `{levels}`

- `levels` is an integer from 2 to 256, and animatable.
- Applied per channel on non-premultiplied sRGB values from 0 to 1:
  `q = round(v × (levels − 1)) / (levels − 1)`, rounding to nearest with ties away from zero.
  So `levels: 256` is exactly the identity on 8-bit values.
- An animated `levels` resolves to a continuous value
  ([ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)).
  Posterize rounds it the same way, the rule `shift` already uses.
- Alpha is untouched, and it has no reach.
- **It keeps the bound.** A colour filter that maps transparent black to transparent black
  keeps the bound, and posterize does. Whether `tint`, `saturation`, `brightness` and
  `contrast` meet the same criterion is measured by the prototype and is not decided here.

**`directional_blur`**: `{angle, length}`

- `angle` is in degrees. 0 smears horizontally, and positive is clockwise, as with
  `rotation`. The blur is centred, so 0 and 180 give the same smear.
- `length` is the total smear in pixels, centred on each pixel.
- Both are animatable. A keyed `angle` interpolates literally, so 350 → 10 sweeps the long
  way round.
- **It is one runtime shader**, which takes `ceil(length) + 1` samples (at least 1) evenly
  spaced along the line. They are read bilinearly and weighted equally. `length: 0` is the
  identity.
- The reach is `(|cos θ| × length / 2, |sin θ| × length / 2)`.
- An animated `length` steps the sample count, and `format.md` says so.

This is not motion blur. It smears whether or not the element moves
([ADR-0155](0155-motion-blur-is-a-per-element-field-that-accumulates-the-element-over-a-centred-shutter.md) §1).

### 5. What the tools say

- **Schema errors** cover:
  - every range and type above;
  - a `grain` without `seed`;
  - a keyframe list on `seed`, `size` or `mono`.

  Until ADR-0146's effect slice lands, a keyframe list on **any** parameter of these four
  members is also an error, and its message names ADR-0146.
- **One new `review`, `R-GRAIN-SEED-SHARED`.** It fires when two `grain` members with the same
  `seed`, `size` and `mono` are visible at the same instant with the same local instant (their
  elements' starts coincide on the frame grid). They then draw the same pattern, which shows
  as a locked texture. The repair is to change one seed.
- **No review flags an identity value** such as `amount: 0`, `levels: 256` or `length: 0`.
  This matches `blur` with `radius: 0` today.
- **`query --at`** reports each member's resolved values.

### 6. Byte-identical painting and cost

Each member is a pure function of the document and the frame number. `grain` hashes integers.
`directional_blur`'s sample count and weights are fixed by its parameters. Every painter
builds its own `RuntimeEffect`, so nothing is shared between threads. That is argued, not
measured.

The build is gated on a measuring prototype
([#722](https://github.com/MBehtemam/Montagent/issues/722)). It must show four things:

- frames byte-identical across 1 to 10 painters, with the bounds hint on and off;
- `grain`'s pixels unchanged when its element is shifted by a whole number of frames;
- each declared reach covering the pixels the member actually touches;
- the cost per member per frame at 1080p. This is recorded for information and is not a pass
  or fail, because no budget was set beforehand.

A member that fails the byte-identity run is withdrawn to the map's **Not yet specified**. It
does not hold back the others.

### 7. A code-drawn piece enters as pre-rendered footage

A look outside the vocabulary, drawn in code with any tool, enters through a skill and never
through the file:

- The skill renders the piece to **one video file with alpha, in a lossless codec** that
  `probe` reads as alpha and the decoder returns pixel-exact. The project references it as an
  ordinary `video`. Nothing in the format changes.
- Beside the footage, **outside the project**, the skill writes a **recipe**: the source code,
  the exact command, tool versions, size, fps, frame count, and a hash of the decoded frames.
  This follows ADR-0017's sidecar route. Re-running the recipe rebuilds the footage, and the
  skill compares decoded-frame hashes, not file bytes.
- A PNG sequence may be an intermediate step, but only the video enters the project.
- The codec is chosen by [#723](https://github.com/MBehtemam/Montagent/issues/723): the first
  candidate that round-trips pixel-exact with alpha. **There is no lossy fallback.** If none
  passes, fixing the decoder becomes the skill's precondition.

This section records only the decision. The commands, the recipe's layout and the codec are
in the skill's hand-off spec.

### 8. An open shader or script file stays refused

A project may not name an SkSL, GLSL or script file, or carry such code inline.
[ADR-0017](0017-closed-schema-no-escape-hatch.md) stays closed, and ADR-0145's table says why:
"a path may name media to decode, never a program to run." This ADR reopens neither.
Reopening it would be a separate later decision, and it would need evidence that the named
vocabulary plus pre-rendering proved too narrow.

## The four tests

| Invariant | How the members pass |
| --- | --- |
| Literal values | Every parameter is a number or a boolean. `grain`'s variation is a fixed hash of a written seed and the element's local instant, written down in §3. |
| Closed vocabulary | Four named members with typed parameters. A generator element, a time input, an open shader file and a coloured glow are refused by name. |
| Checkable by `validate` | Ranges, the required seed and the static parameters are schema errors, and `R-GRAIN-SEED-SHARED` is decided from the file without painting a frame. |
| Exact-string replace | Each member is one object in the `effects` list, added, changed or removed in place. |

## Why

**Effects only, because one rule beats two ways to do it.** A generator element would need
its own answers for size, fill, opacity, keyframes and transitions, duplicating `rect`. It
would also give agents two spellings of the same frame. A `rect` plus an effect plus a blend
reuses parts agents already know.

**A seed plus a fixed re-roll, because grain must move and the file must stay literal.**
Static grain reads as dirt on the lens. Keying a new seed on every frame would fill files with
hundreds of keyframes. A general time input would let motion happen that the file does not
show, a step toward a `draw(t)` script. A seed is written, and the rule is written here.

**The element-local instant, because a moved element should look the same.** With the
project instant, moving an element would change its grain. With the element's own instant,
its pixels move with it, and a single-frame preview agrees with the full render.

**A glow without colour, because `shadow` already makes the coloured one.** Two members
with overlapping meanings would make agents choose between them.

**Every formula written down, because "a function of length" is not byte-identical.** The
sample count, the quantiser and the bright-pass each have more than one common definition.
Left to the implementation, they could change between builds.

**A lossless codec with decoded-frame hashes, because the recipe should rebuild the pixels.**
A lossy codec ties the hash to one encoder build. File bytes also change with container
metadata while the pixels stay the same.

## Considered and refused

- **A generator element** (`noise` and the like). Refused in "Why". It may be revisited by its
  own ADR if a member truly has no source to filter.
- **No time input at all**, so grain stays static unless its seed is keyed. Refused in "Why".
- **A general time input** to every shader. Refused in §3 and "Why".
- **The project instant** for grain's re-roll. Refused in "Why".
- **Rotate, blur, rotate back** for `directional_blur`. It resamples twice, its byte-identity
  is unknown, and it softens across the axis.
- **A `color` on `glow`.** Refused in "Why".
- **The distortions** (wave warp, ripple, twirl, spherize, mirror, magnify). Deferred, not
  refused: they sample outside the pixel, and their edge rule (clamp, transparent or wrap) is
  undecided.
- **Sharpen, emboss, mosaic and RGB split.** Deferred to keep the first batch small. This is
  not a judgement against them.
- **Displacement Map.** Refused: it reads another element's pixels
  ([ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)).
- **Camera shake and wiggle.** Refused: they are procedural motion the file would not show.
  Use keyed position or rotation instead.
- **Corner Pin.** It belongs with the 2.5D decision
  ([#705](https://github.com/MBehtemam/Montagent/issues/705)).
- **Two ADRs** (format and skill). Refused: the refusal of the open file would end up in one
  of them, and a later member's ADR could re-argue it without seeing the other routes.
- **A PNG sequence, or the agent's choice, as the skill's output.** No element takes a
  sequence, and a choice doubles what must be supported for no gain.
- **ProRes 4444 or another lossy fallback.** Refused in §7.
- **A review on identity values.** It would nag every fade-in from `amount: 0`.
- **A cost gate.** No budget was set beforehand. Cost is recorded.

## Consequences

- `CONTEXT.md`'s **Effect** entry names the twelve members, and gains **Seed** and
  **Pre-rendered footage**.
- `format.md` gains the four members, their formulas, the seed and re-roll rule, and the
  bound criterion. Its line "no effect parameter is keyframable" is reworded to say what is
  true: not yet, until ADR-0146's effect slice lands.
- The capability map adds one line per member under its area. It deletes the can't-do item
  marked with #704, and adds the pre-render route. The "Not by design" line on open shader
  files points to the skill and to this ADR.
- The hand-off specs are three slices:
  - `posterize`, `glow` and `directional_blur`
    ([#724](https://github.com/MBehtemam/Montagent/issues/724));
  - `grain` ([#725](https://github.com/MBehtemam/Montagent/issues/725));
  - the pre-render skill ([#726](https://github.com/MBehtemam/Montagent/issues/726)).

  The first two are gated on [the prototype](https://github.com/MBehtemam/Montagent/issues/722),
  and the third on [the codec task](https://github.com/MBehtemam/Montagent/issues/723).
