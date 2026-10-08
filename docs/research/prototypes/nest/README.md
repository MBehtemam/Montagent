# Prototype: `nest` (#780)

Throwaway. This branch (`prototype/nest`) builds the container [Which shape does a group transform take?](https://github.com/MBehtemam/Montagent/issues/632) ruled provisionally, so that [Score the nest against the baker](https://github.com/MBehtemam/Montagent/issues/781) has something to score. Nothing here merges to `main`. The ADR and the **Nest** glossary entry wait for the score.

## The shape, as built

`"type": "nest"`: `start`, `end`, a required static `pivot: [px, py]` (integer pixels, the parent's space), animatable `x`, `y`, `rotation`, `scale`, and `tracks[]`. Matrix = `translate(x, y) · about(pivot)(rotate · scale)`, composed before each child's own transform. A nest at rest is the identity.

| Where | What it does now |
|---|---|
| model, schema | `Body::Nest`; `layer`, `opacity`, `blend`, `effects`, `mask`, `clip`, `motion_blur` on a nest are `E-SCHEMA` with a message saying why (a pure matrix, group compositing is not part of it); a missing or keyword `pivot` is `E-SCHEMA` |
| `crates/montagent-core/src/nest.rs` | the one resolver: `composed(element, t)`, `matrix_of`, and `flatten`, which gives every reader of `Loose::elements_in_tracks` the leaves of every nest, each carrying its chain as `_nest` |
| renderer | `Transform.nest`, applied after `clip` (frame space) and before the element's own `translate`/`rotate`/`scale`; motion blur samples evaluate the chain at each sample instant |
| `validate` | `E-NEST-OUTSIDE-WINDOW`, `E-NEST-EMPTY` (outermost only), `E-NEST-TOO-DEEP` (cap 4), `E-NEST-TARGET`, `R-CLIP-IN-MOVING-NEST`; `R-MOTION-BLUR-STILL` sees a moving chain; layer tie, coverage, retired, cut and the anchor check read the flat view; the track overlap rule reads the written tree, so a nest counts in its parent track by its window and each nested track obeys it |
| `query --at` | a child's row gains `nest` and `composed` (`x`, `y`, `rotation`, `scale`); a `nests` section lists every live nest; `NOT COVERED` and the visible rectangle compose, and answer "rotated" where a chain turns; a text's ink box composes when the chain only moves it |
| projection | `quad` composes the chain, so `corners`, ink and the layer bound read the composed quad |
| transitions | a slide or push offset is frame space and is added *after* the chain (a camera that turns never turns a slide) |
| `shift` | a nest is an element of its track, shifted whole or split; its keys are keyframes; its children are shifted by the same rule |
| `fmt`, `L-KEY-ORDER` | canonical form recurses through nests; a nest is written expanded (its keys one per line, its tracks below), a leaf stays one line |
| `bake_rig.py --nest` | the owl as nests, no copied transform (see below) |

## Fixtures

- `fixtures/nest/basic`, `bad-*`: the smallest nest, and one project per `validate` finding.
- `fixtures/nest/camera.montagent.json`: a whole-scene camera (pan, zoom, turn) over a rig of two levels, with `motion_blur` on three children, `blur` and `shadow` effects, and a projected (`swivel`) child. Gated across painters below.
- `fixtures/nest/pip/pip-nest.montagent.json` against `pip-flat.montagent.json`: [#597](https://github.com/MBehtemam/Montagent/issues/597)'s phone mock-up, the push-in as `x: 0, y: 0, pivot: [960, 540]`, against the same `scale` keyframes copied onto each child.
- `owl/`: the pinned baked owl (`hoot.baked.montagent.json`, sha256 `93c1b627…`, reproduced by the unchanged baker from `hoot.base.json` + `hoot.spec.json`) and `hoot.nest.montagent.json`, from `python3 skills/montagent-character/scripts/bake_rig.py --nest hoot.base.json character/rig.json hoot.spec.json` then `montagent fmt`. `character`, `brand` and `stills` are symlinks into `skills-eval/assets`. `count.py` counts either.
- Arm B's skill teaches the nest bake in `skills/montagent-character/SKILL.md` ("Or bake it as nests").

## What was proved

`cargo test -p montagent-core --test nest` (20 tests):

- the matrix (rotation about a pivot, scale, offset, identity at rest);
- **a nest at rest is a no-op**: identical PNG bytes at four instants, for an identity nest, and for a nest in a nest whose keys hold the identity;
- **every reader sees a child's composed position**: the painter's pixel box, `query --at`'s `composed`, and the rectangle `NOT COVERED` is built from agree for a moving, scaling nest; a turning nest leaves rectangles unanswered, with a reason; nests compose outermost first; a projected child's corners hold what the painter drew; a slide's offset is not scaled or turned by the chain;
- **byte-identical across painters**: `camera.montagent.json` renders the same frames and MP4 at K=1, (3, C=2), (2, 5), (4, 1) and (10, 1), and the same frames with the filter-layer bound off;
- the phone push-in as a nest differs from the flat spelling by at most 1/255 on a handful of pixels (18 at 5900 ms);
- `validate` for each finding, and `shift` splitting a nest's rotation with a hold and stretching its child.

## The owl

| | baked (pinned) | nests |
|---|---|---|
| elements | 56 | 62 (56 + 6 nests) |
| tracks | 8 on the character | 14 |
| keyframes, body | 594 | 271 |
| keyframes, face | 109 | 0 |
| keyframes, whole file | 709 | 277 |
| bytes | 36,450 | 26,066 |
| `R-EASE-INERT` | 76 | 0 |
| deepest nest | | 3 (torso, upper arm, forearm) |

Pixels, all 240 drawn frames, `frame --full --png`: the torso is identical, 2 frames are byte-identical, and every other frame differs on the **edges** of the head, arms and hands only (best integer shift 0). The cause is the one #612 found: the baker rounds every part's position to a whole pixel at every frame from its continuous parent chain, and the nest rounds each joint once, at rest. Neither is exact; the nest's joints coincide exactly with their pivots.

## What the build contradicts or leaves open

Open questions go back to the map; this build settles none of them.

1. **One track per sibling nest.** A nest counts in its parent track by its window, with no exemption, so two nests that are present together cannot share a track. The owl's 6 nests cost 6 extra tracks (8 → 14). The ruling's rule is a real cost for every rig.
2. **`R-CLIP-IN-MOVING-NEST` fired on every screen of the phone mock-up**, whose clips are `[0, 0, 1920, 1080]`. A clip that contains the frame cuts nothing. Built: it skips such a clip. The ruling's wording ("a clipped child whose chain is not the identity") does not.
3. **A transition's slide offset** was added inside the chain by the first build (and so scaled and turned by it). The ruling says it acts outside; built that way, tested.
4. **The projection's geometry ignored the chain** until `quad` was taught it; the layer bound, `corners` and the ink checks all read `quad`. A projected nest stays in the fog, but a projected *child* needed this.
5. **`pivot` is whole pixels, so a rig's joints round.** The baker uses the same rounded point for the joint and the image, so the joint is exact, but each part's rest position is off the continuous layout by up to 0.5 px. A rig at a non-integer scale has no exact pivot.
6. **A keyword or missing `pivot` is `E-SCHEMA` with serde's text** ("invalid type: string, expected an array of length 2"). It does not say a keyword is refused. The message wants writing.
7. **`shift --scope <track>`** names a top-level track; nested tracks have their own names. Scoping to a child's track stretches the child past its nest's window, and the file `shift` writes then fails `E-NEST-OUTSIDE-WINDOW`. `shift`'s stagger-window refusal and coincident preamble walk the top level only.
8. **`timeline` and `compare` read the flat view.** A nest's own keys (its rotation) do not appear in `compare`; `timeline` has no row for a nest.
9. **Duplicate ids** between a nest and a leaf are not reported (`validate` has no uniqueness check on any element, and the nest sits outside the stack).
10. **Text under a turning or scaling chain** has no ink box (refused, as a rotated or scaled text is); only a translation carries it. The resolution's "local box labelled local" slot is not built.
11. **The phone push-in and the owl are not byte-identical** to their flat spellings (1/255 on 18 pixels; edges on the owl), because the nest composes in `f32` and the baker rounds per frame. Only a nest at rest is exactly a no-op.
12. **The format needs a layout for a nest**: a nest cannot be one line, so `fmt` writes it expanded. An exact-string edit against a child still matches one line; against a nest it matches its header line.
13. **Two-level cap.** The ticket says "at most two levels" for the owl; a rig with a two-piece arm is three. The cap of 4 holds.

Not built: a transition end or an anchor *inside* a nest naming across the boundary is exercised only by the anchor and slide tests above; the contact sheet reads the painter and was not tested separately.
