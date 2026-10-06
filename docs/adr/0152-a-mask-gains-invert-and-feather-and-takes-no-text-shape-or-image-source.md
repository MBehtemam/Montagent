---
status: accepted
amends: 0040 (the `mask` member gains `invert` and `feather`; an image- or luma-sourced mask stays deferred for this ADR's own reasons, and a text or shape source is refused inside `mask`), 0084 (the parameter set is the rect, `radius`, `invert` and `feather`; the rect is still the one shape-independent set)
---

# A mask gains `invert` and `feather`, and takes no text, shape or image source

[#688](https://github.com/MBehtemam/Montagent/issues/688), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)
refused a matte taken from another element and named the element-local `mask` as the
smaller door. Today a `mask` is a `circle`, `rect` or `ellipse` cut in one rect
([ADR-0084](0084-the-mask-rect-is-one-shape-independent-parameter-set.md)). It keeps the
inside and has a hard, antialiased edge. Two things an agent asks for cannot be written:
keeping everything *except* a shape, and a soft edge. The nearest spelling of a soft edge,
`[mask, blur]`, softens the edge but also blurs the picture. That is a quiet wrong result
an agent will reach for.

**Precedent.** Row 7 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)): CapCut's per-clip mask takes
feather and inversion. Premiere's effect masks take Feather, Expansion, Opacity and Invert.
Premiere's "Mask with Text/Shape" lets a text or shape layer mask what lies beneath it.
`invert` and `feather` are inside the reference class, so
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
asks for no accepted prototype. Feather still waits on a measuring prototype for
byte-identity (§4).

Settled by two `/court` rounds of three jurors each (Opus, Sonnet, Fable), unanimous on
every question, with the owner ruling with the Judge's read both times.

## The decision

### 1. `invert`

A `mask` may carry `"invert": true`. With it, the mask keeps the pixels **outside** its
shape and erases the inside.

- Omitted means `false`. Writing `"invert": false` is legal and draws no finding.
- The field is static. A boolean is not an animatable type under
  [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md),
  so a keyframe list on `invert` is already refused by the schema check. Nothing new is
  written for it. A mask that flips mid-clip is two elements.
- **Masks in one list still intersect.** Each mask multiplies what is kept by its own kept
  alpha. So `[mask ellipse, mask smaller ellipse inverted]` keeps a ring, by the same rule
  that already makes two masks intersect, and no rule is written for it.
- The word is the mask's own field. It is not a colour inversion, which
  [ADR-0049](0049-v1-colour-filter-vocabulary-four-scalar-members.md) keeps out of the
  effect vocabulary.

### 2. `feather`

A `mask` may carry `feather`, a non-negative integer.

- It is in **unscaled element units**, and it rides the transform as the mask rect, a
  `blur` radius and a `stroke_width` do. A `feather` of 8 on an element at `scale: 2` is 16
  frame pixels wide.
- **The reading is `blur`'s.** The hard mask's coverage (1 inside the shape, 0 outside) is
  blurred by a Gaussian of σ = `feather` / 2. This is the CSS reading the painter already
  uses for a `blur` radius.
- **The softness is centred on the edge.** A feathered mask reaches a little past its
  stated rect and a little short of it. An inverted feathered mask keeps exactly
  1 − what the uninverted one keeps.
- Omitted means `0`. `0` is today's hard, antialiased edge, and a written `"feather": 0`
  paints the same bytes as none.
- It is an animatable property. ADR-0146 admits every numeric effect parameter, so a keyed
  `feather` is written in place and read by every tool that reads that one schema-derived
  list. Nothing new is written for it.
- The feather is part of the mask's own step. `[mask, blur]` keeps its present meaning:
  mask, then blur the result.

`invert` and `feather` sit beside the rect and `radius`. They are not per-shape fields, so
ADR-0084's refusal of a two-level lookup is untouched.

### 3. What does not enter

- **Expansion.** It moves the same edge the explicit mask rect already moves with literal
  numbers. A second knob for one edge gives two ways to say one thing.
- **Mask opacity** ("erase the outside only partly"). No case asks for it, and the element's
  `opacity` with a second element covers what is near. Not until a case asks.
- **A text source.** A string, a font and a text layout inside an effect would be a second
  text element hidden in a parameter list. Every check that text needs (fonts, overflow,
  captions) would then be owed to an effect. It is refused inside `mask`. The gesture
  behind it, footage showing through letters, belongs to **paint**:
  [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)
  made a paint a colour or a gradient, and an image or video paint on a text `color` is
  the natural third kind if a case asks.
- **A shape source.** The three closed shapes cover the shape elements' geometry. Any
  other figure is a path, and paths are still in the map's fog. A shape source waits on
  them.
- **An image or luma source.** ADR-0040's reasons stand: a second asset reference inside
  an effect, plus its fitting, luma or alpha, and a video source.

### 4. Byte-identical painting

`invert` changes which path is erased, not how. It needs no measurement beyond the
existing gating tests.

`feather` is a blur on the eraser. The blur and shadow bounds hint of
[#652](https://github.com/MBehtemam/Montagent/issues/652) already treats a mask as "only
erases, so what it holds bounds what it passes on". A feathered erase still only erases, so
the hint's reasoning should hold. That is inferred, not measured. The `feather` build is
gated on a measuring prototype: frames byte-identical across 1 to K painters, with the
bounds hint on and off, for every shape, inverted and not, under rotation, non-uniform
scale and sub-pixel motion. Which Skia mechanism softens the eraser is the prototype's to
find. The rule it must meet is §2's. If `feather` fails the run, it is withdrawn, not
excused. `invert` does not depend on that run.

### 5. What `validate` says

**A new `review` finding, `R-MASK-ERASES-ALL`:** a mask that erases every pixel of its
element on every frame. It is decided from the file alone, and fires when all of these hold:

- `shape` is `rect`, `invert` is `true`, and `radius` is `0` or omitted;
- `feather` is `0` or omitted (with a feather, the edge keeps a ramp);
- none of the mask's rect fields, `radius` or `feather` is keyed;
- the mask's rect contains the element's rect. An omitted rect *is* the element's rect
  (ADR-0084), so it always does. A written rect is compared with the element's rect only
  where the element's size is not keyed. Where it is keyed, the finding stays silent.

An inverted `circle` or `ellipse` keeps the corners, and never fires. A keyed inverted rect
that grows to cover the element is a wipe-out, and never fires.

The repair is to write the rect you meant, or to drop `invert`. Writing the covering rect
explicitly does not silence the finding. That costs nothing, because a static full erase is
never the clearest way to hide an element: `opacity: 0`, or removing the element, says it
better.

**No finding for a wide feather.** A `feather` past half the mask rect's short side keeps
no pixel fully. That is a soft-glow vignette, which is an ordinary look. The value is
already an explicit literal, so there is nothing further to write as an acknowledgement.
The only "repair" would be changing a value the author meant. `frame` shows it.

Schema errors cover the rest: `invert` that is not a boolean, `feather` that is negative or
not an integer, and either field on any effect but `mask`.

## The four tests

| Invariant | How `invert` and `feather` pass |
| --- | --- |
| Literal values | One boolean, and one integer or a keyframe list of integers. §2's arithmetic is fixed and documented. |
| Closed vocabulary | Two named fields on one effect. Expansion, mask opacity and every non-shape source are refused or deferred by name. |
| Checkable by `validate` | Bad types are schema errors, and `R-MASK-ERASES-ALL` is decided from the file without painting a frame. |
| Exact-string replace | `"invert": true` and `"feather": <n>` are each one string, flipped or changed in place. |

## Why

**`invert`, spelled as the reference class spells it.** Both tools call it Invert, so an
agent's first guess validates. The format already writes on/off state as a boolean (`loop`,
`caption`). A keyword such as `keep: "inside" | "outside"` was considered: it reads a
little better in a diff, but it is a coined word with no precedent and invites a third
value nobody has asked for.

**`feather` with `blur`'s reading.** Agents get one rule for both radii. A centred edge
keeps the inverted mask the exact complement of the plain one. An edge feathered inward
only would keep the stated rect as a hard outer bound, but it breaks that symmetry and
differs from both reference tools.

**The finding is narrow on purpose.** An inverted bare `rect` mask is the likely slip. It
reads as "flip the mask", and the agent forgets that the omitted rect is the whole element,
so the element vanishes without a word. `frame` on an empty element shows nothing, which is
the weakest signal an agent can get. Keyed rects and keyed element sizes are left out so
that the finding never fires on a wipe-out, and so that it can be predicted from the file.

**Two slices.** `invert` and its finding do not depend on the byte-identity run, so the run
does not hold them back.

## Considered and refused

- **`keep: "inside" | "outside"`.** Refused above.
- **`invert` not entering.** Refused: "everything except the circle" cannot be written
  today, and it has precedent in both tools.
- **An inward-only or outward-only feather.** Refused above.
- **`feather` without a prototype gate.** Refused: a Gaussian on an eraser is a new
  floating-point path, and byte-identity is a rule on this map.
- **Expansion, mask opacity, and a text, shape or image source.** §3.
- **`R-MASK-ERASES-ALL` as an error.** Refused: the construct is determinate. A static
  covering rect is also a legal halfway state while an agent writes a wipe, before the rect
  is keyed.
- **No erase-all check.** Refused: an element that vanishes silently is the failure an
  agent trusting `validate` is least able to see.
- **A review for a wide feather.** Refused in §5.

## Consequences

- `CONTEXT.md`'s **Mask** entry gains `invert` and `feather`, and says that a text source
  is refused and that an image source stays deferred.
- `format.md` and the feature map describe both fields, the ring that two masks make, and
  the `[mask, blur]` difference.
- The hand-off spec is two slices:
  [slice 1](https://github.com/MBehtemam/Montagent/issues/695) (`invert` and
  `R-MASK-ERASES-ALL`, workable once this ADR is merged) and
  [slice 2](https://github.com/MBehtemam/Montagent/issues/696) (`feather`, gated on
  [the measuring prototype](https://github.com/MBehtemam/Montagent/issues/697)). Slice 2
  extends `R-MASK-ERASES-ALL` with its `feather` condition.
