---
status: accepted
amends: 0040 (writes the `...shape params` ellipsis it left unwritten, and states the coordinate space and transform behaviour its effect model implied but never said), 0068 (discharges the full-parameter-surface ticket it graduated; its param-less form is preserved exactly, as the identity value of the set written here), 0075 (extends its lesson — the two readings it warned were unratified are ratified here, with the measurement it asked for commissioned rather than asserted)
---

# The mask rect is one shape-independent parameter set, element-local, and it rides the transform

**Ticket:** [#185](https://github.com/MBehtemam/Montagent/issues/185), graduated from
[ADR-0068](0068-the-bare-mask-key-retires-masks-are-effects-members.md).

[ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md) wrote the mask union member as
`mask{shape: "circle"|"rect"|"ellipse", ...shape params}`. ADR-0068 found that the ellipsis
was an unmade decision rather than shorthand, stated **the minimum its migration forced** —
the param-less form is the inscribed shape — and graduated the rest. This is the rest: the
explicit parameter set, the coordinate space, the non-square answer, and what a mask does
under `scale`, `rotation` and a non-`top-left` `origin`.

## Decisions

### The parameter set is one rect, shared by all three shapes

```json
{"name": "mask", "shape": "circle", "x": 40, "y": 120, "width": 200, "height": 200}
{"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": 320, "height": 180, "radius": 16}
{"name": "mask", "shape": "ellipse", "x": 0, "y": 0, "width": 400, "height": 240}
```

**`x`, `y`, `width`, `height` name the rect the shape is inscribed in.** They are not
per-shape geometry. `circle` is the largest circle inscribed in that rect — diameter
`min(width, height)`, centred on it; `rect` is that rect; `ellipse` is the ellipse inscribed
in it. All integers, in element-local pixels (below).

**The obvious alternative — per-shape fields — is refused, and ADR-0049 is why.** The
natural sketch is `circle{cx, cy, r}`, `ellipse{cx, cy, rx, ry}`, `rect{x, y, width, height,
radius}`: fields that vary with `shape`. That makes `shape` a real discriminator narrowing a
field set, and
[ADR-0049](0049-v1-colour-filter-vocabulary-four-scalar-members.md) rejected precisely that
construction when it refused a single `color{mode, ...}` effect —

> it would be the only two-level lookup in v1's vocabulary (an agent must learn "what
> effects exist" and, for this one member, "what modes exist inside it")

— and resolved it by making four flat members instead. `mask` cannot take that resolution:
splitting it into `mask_circle`/`mask_rect`/`mask_ellipse` would retire
`{"name":"mask","shape":"circle"}`, which ADR-0068 ratified and the committed fixture
carries. So the two-level lookup is avoided on the other axis — **one field set, and `shape`
selects which figure is drawn in it, never which fields exist.** An agent that has learned
`mask` has learned all of `mask`.

This is also the only formulation under which `shape` keeps the meaning ADR-0040 gave it. In
the per-shape sketch, `shape` is a schema-narrowing tag; here it is what it reads as — the
figure.

### The rect's identity value is the element's own rect, and ADR-0068's form is preserved exactly

**The four fields are all-or-none.** Either all four are absent, or all four are present. A
partial tuple — `x` without `width` — is a schema error naming the other three. Two arities
per member, never 2⁴ spellings, and no field defaults independently of its siblings.

**Absent, the rect is the element's own rect: `(0, 0, width, height)` in element-local
space.** Substituting that into the rule above yields ADR-0068's sentence verbatim — a
param-less `circle` is the largest circle inscribed in the element's rect, `rect` and
`ellipse` take that rect itself. **ADR-0068's form is not merely left legal; it is this
ADR's identity value, reached by the same arithmetic.** The committed fixture does not
migrate and `handle-logo` renders the same pixels.

That is the ADR-0049 four-clause standard ADR-0068 asked this ticket to be held to, met on
all four: fixed arity, bounded integer parameters, a documented identity value, and not
reproducible by composing members that already exist — nothing else in the vocabulary cuts a
shape.

**Under [ADR-0030](0030-defaultable-field-presence-is-content.md) the two arities are two
different declarations**, not two spellings of one. The bare form says *the mask follows the
element's rect* — it re-derives when the rect changes. The explicit form says *this rect*,
and keeps saying it after a resize. That is the substantive difference, and it is the
strongest argument for admitting the explicit form at all: the bare form's meaning changes
silently when `width`/`height` change, and before this ADR there was no way to pin it.

### `radius` is a field of `shape: "rect"` only

A single integer, identity `0`, rounding the mask rect's corners. On `circle` or `ellipse`
it is an **unknown key**, with a message naming the reason rather than a field that quietly
does nothing.

This is not a new exception; it is
[ADR-0014](0014-stroke-is-paint-the-text-box-is-required.md)'s rule applied to the same word
for the same reason. Its heading is *"`radius` is a field on `rect`"*, and the `shape`
element's `radius` is already *"a single integer, defaulting to 0 — one corner radius, not
four."* A mask rect rounds its corners the way a drawn rect does, with the same name, the
same arity and the same identity. An inscribed ellipse has no corners to round, and ADR-0007
has ruled twice that *"a field the renderer cannot honour is worse than no field."*

It is the one field whose legality depends on `shape`, and it is admitted as such rather
than pretended away. The alternative — deferring corner radii — was rejected because a
rounded picture-in-picture is the most ordinary masking gesture in the CapCut/Premiere
reference class, and ADR-0014 already paid this exact cost with this exact argument: *"a
rounded rectangle is unremarkable in the CapCut/Premiere reference class, and admitting it
now costs one clause where admitting it later is a schema change."*

**There is no rotation term on `ellipse`.** The element's own `rotation` already rotates the
mask (below), so an ellipse angle would be the format's only nested rotation — a second
angular quantity on an element that ADR-0012 gave exactly one. A tilted ellipse is spelled
by rotating the element.

### Coordinate space: element-local, `(0,0)` at the rect's top-left, independent of `origin`

Mask coordinates are measured against the element's own rect, in **unscaled** element units,
with `(0, 0)` at that rect's top-left corner **whatever the element's `origin` keyword is**.

**Frame-absolute was refused because `clip` already occupies that slot.** A frame-space,
non-rotating aperture is definitionally what `clip` is
([ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md),
[ADR-0025](0025-clip-stays-static.md)), and `CONTEXT.md` goes out of its way twice to keep
the two apart — *"Shaped and soft masks are a different thing and are not this"* under Clip,
and `_Avoid_: clip` under Mask. Giving the mask frame coordinates would make the two
numerically interchangeable in the common case and would mean every mask needs rewriting
whenever `shift` or a layout change moves its element.

**`origin` does not participate.** It is a placement anchor — it says which point of the box
lands on `(x, y)` — and it is not a re-parameterisation of the box's interior. Letting it
move the mask's coordinate origin would give an element nine possible mask frames, make a
mask unreadable without cross-referencing an unrelated field, and mean that changing
`origin` to reposition an element silently rewrites its mask geometry. The rect is
`(0, 0, width, height)` under every origin.

### The mask rides `scale` and `rotation`

The mask is carried by the element's transform: a keyframed Ken Burns `scale` grows the
masked figure with the picture, and a rotated element's `rect` mask paints a **rotated
rectangle** on the frame.

Stated in the drawing order rather than by adjective, because *"pre-transform"* and
*"post-transform"* name this same behaviour in opposite directions depending on which end of
the pipeline the speaker is standing at, and the words are worse than useless here:

> `clip` is applied first, in frame space, outside everything. Then the element's box is
> placed — translate to `(x, y)`, rotate, scale, then shift back by the origin's fraction of
> itself. **The mask is declared inside that box, in its unscaled units, and is therefore
> transformed along with the pixels it cuts.**

Two independent reasons, either sufficient. First, **it is the rule the rest of the effect
vocabulary already follows**: a `blur` of radius 8 on an element at `scale: 2` is sixteen
frame pixels wide, and a text `stroke_width` scales the same way. A mask that alone refused
the transform would be the single exception in a seven-member closed list, and the exception
would be invisible at the schema level — nothing in `mask{shape, ...}` would tell a reader
it lives in a different space from the `blur` beside it in the same array. Second, **the
alternative is `clip` again**: a fixed, non-rotating, non-scaling porthole the picture moves
behind is what ADR-0025 deliberately made `clip` and kept permanently static. Admitting a
second one would be two spellings of one concept, which this project has retired three times
(ADR-0012's `scale` union, ADR-0013's `center-center`, ADR-0068's bare `mask`).

**Indirect animation is not a violation of "no effect parameter is keyframable."** That rule
governs what may carry a keyframe list, and the mask's fields stay literal integers on every
frame — `validate` and `fmt` see no keyframe list inside an effect. The mask's on-screen
geometry moves because the transform moves, exactly as a blur's footprint already grows
under a Ken Burns. This is stated because it is an implicit consequence a reader would
otherwise have to infer from the renderer, and inferring rendering facts is how ADR-0075
happened.

**What this costs, named rather than glossed:** a *shaped* porthole — a non-rectangular
window the picture moves behind — is not expressible with `mask` alone in v1. `clip` gives
the static window but only rectangular. The route ADR-0040 assigns to vignettes (a shape
element overlaid) reaches it, and the honest future answers are a shape option on `clip` or
the deferred soft/alpha mask. Recorded in Not Yet Specified rather than left to be
discovered.

### `R-MASK-CIRCLE-NON-SQUARE`: a `review` finding when a circle's rect is not square

A `circle` whose mask rect has `width != height` — **derived or explicit** — raises a
`review`-severity finding, naming as repair: use `ellipse` to fill the rect, or give the
mask a square rect.

`circle` is the one shape whose meaning *discards* part of its rect. `rect` is the rect and
`ellipse` fills it; both are total, and neither gets a finding — an ellipse inscribed in a
non-square rect is an ordinary oval and nobody is surprised by it. A circle on a 1080×1912
rect erases 832 px that the author never typed a number for, which is the signature of a
footgun rather than a decision.

`review` and not an error: the behaviour is determinate, ADR-0068 ratified it, and the
committed fixture legitimately relies on it. Redefining `circle` on a non-square rect to
mean anything else was refused outright — it would either make `circle` a second spelling of
`ellipse` or fork the meaning of an already-ratified default across geometries.

**The finding is honest because the repair exists**, which it does only because of this
ADR's first decision. Under a param-less-only vocabulary the sole available repair would be
"resize the element", i.e. move the picture to satisfy a checker — and a finding that can
only name that is noise. The dependency ran the other way too: it is part of why explicit
geometry is admitted here.

**Writing the explicit rect is itself the acknowledgement.** Under ADR-0030 presence is
content, so an author who genuinely wants a `1080×1080` circle on a tall element spells the
square rect and the finding falls silent — no suppression mechanism, no annotation, and the
file ends up recording the intent rather than leaning on arithmetic.

### Key order within the member

`name, shape, x, y, width, height, radius`.

[ADR-0041](0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md) hands
a new field's position to the ADR that introduces it, and this one takes it explicitly
rather than leaving it to be read off a struct. The rect fields follow `shape` in the order
ADR-0012 already fixed for every other rect in the format (`x, y, width, height`), and
`radius` trails them exactly as it trails the `shape` element's own fields under ADR-0014.

## Consequences

- The `mask` member gains five fields: `x`, `y`, `width`, `height` (all-or-none, integers,
  element-local, identity = the element's rect) and `radius` (integer, identity `0`, legal
  only when `shape` is `rect`).
- **No fixture migration.** `{"name":"mask","shape":"circle"}` is the identity value of the
  new set, not a legacy form beside it, and `handle-logo` renders unchanged.
- The schema gains one conditional: `radius` is an unknown key unless `shape` is `rect`.
  Expressible under ADR-0009's full 2020-12 enforcement.
- `validate` gains `R-MASK-CIRCLE-NON-SQUARE`, `review` severity, firing on derived and
  explicit rects alike. The fixture's square badge triggers nothing.
- Canonical key order for the member is `name, shape, x, y, width, height, radius`.
- ADR-0040's `...shape params` ellipsis is written; ADR-0068's graduated ticket is
  discharged.
- **The two coordinate/transform readings are now ratified rather than inherited from the
  renderer.** The shipped code already behaves this way, which is corroboration and not
  authority — see Evidence.
- A shaped static porthole remains inexpressible; recorded in Not Yet Specified.
- Implementation — schema, renderer, the new check, and the goldens below — is a separate
  ticket. This ADR decides, and does not land code.

## Evidence

**A three-juror court** (Opus 5, Sonnet 5, Fable 5.1 — via `/court`), each given the same
question packet cold, blind to each other and to the author's recommendations. Full ballots
and the Judge's read are committed at
[`docs/research/juries/mask-parameter-set/`](../research/juries/mask-parameter-set/README.md).

- **Unanimous 3/3** on the coordinate space (element-local, `(0,0)` at the rect's top-left,
  `origin` not participating), on the mask riding `scale`/`rotation`, on indirect animation
  not violating the keyframe rule, and on the non-square finding being `review`-severity and
  `circle`-specific.
- **Split 2–1** on whether explicit geometry belongs in v1 at all. Sonnet 5 would have
  ratified param-less-only and deferred pending a forcing case, on ADR-0049's
  don't-admit-until-forced discipline and ADR-0068's warning against guessing the explicit
  form's shape. Resolved here against that ballot, on a precedent no juror was given:
  **ADR-0014 admitted `rect`'s `radius` with zero fixture evidence**, on exactly this
  argument — *"a rounded rectangle is unremarkable in the CapCut/Premiere reference class,
  and admitting it now costs one clause where admitting it later is a schema change."* The
  dissent's strongest point is preserved in the design rather than overruled by it: the
  all-or-none rule and the shape-independent rect exist to keep the guessed surface as small
  as a guess is allowed to be.
- **No juror proposed the shape-independent rect.** The panel's concrete sketch (Fable 5.1)
  was per-shape — `circle{cx,cy,r}`, `ellipse{cx,cy,rx,ry}` — and the ADR-0049 two-level
  lookup objection that rules it out was found after the court, in review of the ballots.
  Recorded because it is the substantive respect in which this ADR is not what the jury
  described.

**Commissioned, not asserted.** Two of the three jurors independently warned that agreeing
with the shipped renderer is weak corroboration given ADR-0075, and asked for measurement
instead. The implementation ticket carries that obligation:

- a golden on a **rotated, scaled, non-`top-left`** masked element, which is the frame no
  committed fixture can produce and the one that distinguishes every candidate answer to the
  coordinate-space and transform questions;
- a golden on an **explicit mask rect** that differs from its element's rect;
- a test asserting the param-less form and the explicit form spelling the element's own rect
  render **identically**, which is the identity-value claim above stated as an executable
  assertion rather than a sentence.

ADR-0075 exists because ADR-0068 asserted a rendering fact that measurement later refuted.
The three readings above are the ones this ADR would be wrong about in the same way, so they
are named as tests rather than as prose.
