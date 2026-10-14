---
status: accepted
amends: 0003 (After Effects joins the reference class as precedent for rendered results, while CapCut and Premiere stay first for mechanism and shape), 0145 (After Effects is precedent that a rendered result is wanted, so an After Effects-documented capability needs no entry-test prototype to prove demand; CapCut and Premiere remain first for mechanism and schema shape, and an After Effects-only mechanism is reshaped to the four invariants, a skill script or pre-render; the 26 rulings in the AE-only census are disposed of), 0149 (§1's "no precedent in the reference class" bar on sweep and two_point_conical gradients is lifted; both are documented in After Effects and enter the After Effects map's triage, still subject to §8's Skia byte-identity prototype gate before a build), 0171 (§5's reference-class reopen trigger for native SVG or Lottie import continues to mean CapCut or Premiere; After Effects precedent alone does not reopen native import while the skia_pin textlayout/skottie ban, byte-identity, clock and literal-value grounds hold)
---

# After Effects is precedent for rendered results, and CapCut and Premiere stay first for shape

> **Amended by [ADR-0217](0217-motion-bakes-and-keyframe-assistants-enter-in-one-keyframes-script-auto-colour-is-refused-and-a-skill-script-with-its-spec-is-a-template.md)**: disposes of the motion-bake (`wiggle`, `bounce`, `loopIn`/`loopOut`, `valueAtTime`, *Sequence Layers*, time-stretch/reverse), keyframe-assistant (*Convert Audio to Keyframes*, *Smoother*, *Exponential Scale*), measure-then-write auto-colour (*Auto Levels* / *Auto Contrast* / *Auto Color*), and template/MOGRT/Essential-Properties/Expression-Controls rows from map [#896](https://github.com/MBehtemam/Montagent/issues/896)'s censuses ([#1059](https://github.com/MBehtemam/Montagent/issues/1059)).
>
> **Amended by [ADR-0218](0218-a-visual-element-moves-along-its-own-motion-path-in-place-of-x-and-y-with-arc-length-offset-and-orient.md)**: disposes of the curved spatial motion path, roving constant-speed keyframe, and *Orient Along Path* rows from map [#896](https://github.com/MBehtemam/Montagent/issues/896)'s transform and animation census ([#898](https://github.com/MBehtemam/Montagent/issues/898)) by admitting an optional `motion_path` (`closed`, `points`, `offset`, `orient`) in place of `x` and `y` on visual elements ([#1060](https://github.com/MBehtemam/Montagent/issues/1060)).

[#897](https://github.com/MBehtemam/Montagent/issues/897), on the map
[#896](https://github.com/MBehtemam/Montagent/issues/896).
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
replaced [ADR-0003](0003-general-video-editor-not-channel-tooling.md)'s *"After Effects is out
of scope for now"* with the four invariants and the entry test, while keeping CapCut and
Premiere as the sole source of precedent for whether a capability enters without an
owner-accepted prototype and permitting After Effects to be cited *"for the shape of a
capability, never as the reason it enters."* Map
[#896](https://github.com/MBehtemam/Montagent/issues/896) now surveys everything Adobe After
Effects documents across ten area censuses
([#898](https://github.com/MBehtemam/Montagent/issues/898)–[#907](https://github.com/MBehtemam/Montagent/issues/907))
and asks which of those capabilities enter Montagent and how earlier After Effects-only
rulings—audited in
[`docs/research/after-effects/ae-only-rulings.md`](../research/after-effects/ae-only-rulings.md)
([#910](https://github.com/MBehtemam/Montagent/issues/910))—are read.

## The decision

### 1. Precedent for rendered results; CapCut and Premiere first for mechanism and shape

1. **Demand (why a capability enters):** **CapCut, Premiere, and After Effects** all count as
   precedent that a **rendered result** (a visual or audio capability in the finished output)
   is wanted. Being After Effects-only is never a ground to refuse a capability.
2. **Mechanism and shape (how it is written):** **CapCut and Premiere remain the first place a
   design question looks for mechanism and schema shape.** When either CapCut or Premiere
   documents the capability, the ADR starts from that editor's mechanism and names each
   departure ([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)).
   When only After Effects documents the capability, After Effects' **rendered result** is
   precedent, never its **mechanism** (such as precompositions with local clocks, adjustment
   layers, track-matte layers, text-animator range selectors, or expressions). Its ticket and
   ADR weigh reaching that result through:
   - existing format primitives;
   - a skill script that writes ordinary literals into the project file
     ([ADR-0177](0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md));
   - pre-rendered footage or stills via `montagent-prerender`
     ([ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md)
     §7,
     [ADR-0171](0171-svg-and-lottie-do-not-enter-as-sources-and-each-is-pre-rendered-through-the-skill.md));
     or
   - a format addition reshaped to pass all four invariants, naming each departure from After
     Effects' mechanism.
3. **The four invariants remain unwaivable:** literal values, closed vocabulary, checkable by
   `validate`, and editable by exact-string replace bind every capability regardless of which
   tool provides its precedent
   ([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
   §2).

### 2. The entry test and when prototypes still run

The **entry test**
([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
§3 — the four invariants plus an owner-accepted rendered prototype to establish that the
capability is wanted) applies to a capability with no precedent in **any** of CapCut, Premiere,
or After Effects.

A capability documented in After Effects needs no entry-test prototype merely to prove demand.
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)'s
floor rule stands: a ticket, map, or ADR still requires a prototype whenever visual behavior,
Skia feasibility, cross-platform byte-identical painting
([ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)
§8,
[ADR-0152](0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md),
[ADR-0155](0155-motion-blur-is-a-per-element-field-that-accumulates-the-element-over-a-centred-shutter.md)),
measured audio tolerance
([ADR-0173](0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md)),
or a map's A/B evidence bar ([#591](https://github.com/MBehtemam/Montagent/issues/591)) asks
for one.

### 3. Disposition of the 26 rulings in the After Effects census

[`docs/research/after-effects/ae-only-rulings.md`](../research/after-effects/ae-only-rulings.md)
([#910](https://github.com/MBehtemam/Montagent/issues/910)) audited every ADR and decision
comment through `ADR-0191` and found 26 rulings that refused or scoped out a capability on an
After Effects or reference-class ground. Under §1 and §2 they are disposed of as follows:

- **Ten `INV`-only rulings (rows 2, 9, 12, 14, 15, 16, 17, 18, 20, 25) — stand unchanged.**
  Expressions ([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)),
  an adjustment-layer shape that transforms elements below it
  ([#632](https://github.com/MBehtemam/Montagent/issues/632),
  [ADR-0201](0201-nested-sequences-are-bounded-containers-rendering-to-an-isolated-surface-on-an-absolute-timeline.md)),
  shape/element repeaters
  ([ADR-0148](0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md)),
  trimming a shape's fill, `trim_offset` on an open path, and dashes that travel with a trim
  window
  ([ADR-0160](0160-a-stroke-draws-a-window-of-its-outline-measured-in-fractions-of-its-length.md)),
  multi-line text on offset curves and borrowing another element's path by id
  ([ADR-0161](0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md)),
  audio varispeed under `source_time`
  ([ADR-0157](0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md)),
  and procedural shake / wiggle
  ([ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md))
  each rest on an explicit invariant failure and are untouched by widening precedent.
- **Nine `RC + INV` rulings (rows 1, 3, 4, 10, 11, 13, 22, 23, 24) — stand on their invariant
  grounds.** Transform-only keyframes (row 1,
  [ADR-0003](0003-general-video-editor-not-channel-tooling.md)) was already discharged by
  [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md).
  Nesting's reference-class premise (row 3) was corrected by
  [ADR-0166](0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md),
  and [ADR-0201](0201-nested-sequences-are-bounded-containers-rendering-to-an-isolated-surface-on-an-absolute-timeline.md)
  admits the bounded `nest` container on an absolute timeline while
  [ADR-0001](0001-flat-element-list.md)'s refusal of a local clock stands. An open effect stack
  or plugin door (row 4,
  [ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md),
  [ADR-0017](0017-closed-schema-no-escape-hatch.md)), a track matte from another element (row 10,
  [ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)),
  spring / physics easing (row 11,
  [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)
  §8), a render-time resampling morph rule (row 13,
  [ADR-0162](0162-a-morph-between-unlike-shapes-is-written-as-matching-vertex-lists-and-no-rule-resamples-them.md)),
  and an SVG `d` path string (row 24,
  [ADR-0014](0014-stroke-is-paint-the-text-box-is-required.md),
  [ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md))
  stand on their invariant clauses. For native SVG and Lottie sources (rows 22 and 23,
  [ADR-0027](0027-vector-sources-out-of-scope.md),
  [ADR-0171](0171-svg-and-lottie-do-not-enter-as-sources-and-each-is-pre-rendered-through-the-skill.md)),
  [ADR-0171](0171-svg-and-lottie-do-not-enter-as-sources-and-each-is-pre-rendered-through-the-skill.md)
  §5's *"if a reference-class tool gains native import of it"* trigger continues to mean CapCut
  or Premiere: After Effects precedent alone does not reopen native import while the
  `skia_pin.rs` `textlayout`/`skottie` ban
  ([ADR-0010](0010-skia-safe-rasterizer-text-beside-it.md)), `resvg`'s second-backend and float
  determinism grounds, Lottie's player, clock ([ADR-0005](0005-absolute-integer-milliseconds.md))
  and literal-value grounds, and the sanctioned `montagent-prerender` routes hold.
- **Two `other`-only rulings (rows 19, 21) — stand on their recorded grounds.** `path_reverse`
  on text-on-a-path
  ([ADR-0161](0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md) §4) waits on
  its own trigger (evidence that reversing `points` by hand fails agents), and project-wide
  motion-blur shutter
  ([ADR-0155](0155-motion-blur-is-a-per-element-field-that-accumulates-the-element-over-a-centred-shutter.md)
  §2) stays per-element so cost and look remain visible where the agent edits.
- **Four of the five `RC`-only rulings (rows 5, 6, 7, 8) — re-read and kept closed on
  mechanism, model, and literal-value grounds:**
  1. **Sibling `parent` link (row 7, [#594](https://github.com/MBehtemam/Montagent/issues/594)
     on [#591](https://github.com/MBehtemam/Montagent/issues/591)):** A sibling
     `parent: "<id>"` field is After Effects' *mechanism* (cross-element transform inheritance
     across flat siblings). Under §1, CapCut and Premiere's container mechanism (`nest`,
     [ADR-0166](0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md),
     [ADR-0201](0201-nested-sequences-are-bounded-containers-rendering-to-an-isolated-surface-on-an-absolute-timeline.md))
     and the baker (`bake_rig.py` on
     [#591](https://github.com/MBehtemam/Montagent/issues/591)) come first. A sibling `parent`
     link stays closed and reopens on [#591](https://github.com/MBehtemam/Montagent/issues/591)
     only if a concrete case requires shared motion across elements separated by an ungrouped
     track in stacking order—the one rendered result a container cannot express without
     splitting.
  2. **Inverse kinematics and rig controllers (row 8,
     [#591](https://github.com/MBehtemam/Montagent/issues/591) *Out of scope*):** Vanilla After
     Effects ships no built-in bone IK or rig controller (in After Effects they are expressions
     or third-party scripts such as Duik;
     [#898](https://github.com/MBehtemam/Montagent/issues/898),
     [#905](https://github.com/MBehtemam/Montagent/issues/905)), and an in-file IK solver
     derives joint angles at render time from coupled element positions, failing the
     literal-values invariant. Stays refused on literal-values grounds; any future IK helper
     belongs to a skill script that writes literal keyframes
     ([ADR-0177](0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md)).
  3. **3D camera and depth sorting (row 6,
     [ADR-0167](0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md),
     [#591](https://github.com/MBehtemam/Montagent/issues/591)):** An After Effects Camera
     layer is a non-painting track entity that transforms other elements at render time, and
     `z`-placement / plane-intersection depth sorting overrides track stacking order
     ([ADR-0001](0001-flat-element-list.md),
     [ADR-0004](0004-tracks-as-constrained-lanes.md)). Per-element 3D tilt is covered by
     Projection (`swivel`, `tilt`, `perspective` in
     [ADR-0167](0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md)),
     whole-scene pans and zooms belong to `nest`
     ([ADR-0201](0201-nested-sequences-are-bounded-containers-rendering-to-an-isolated-surface-on-an-absolute-timeline.md),
     [#591](https://github.com/MBehtemam/Montagent/issues/591)), and true multi-plane 3D scenes
     enter through `montagent-prerender`
     ([ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md)
     §7).
  4. **Skew and corner pin (row 5,
     [ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md),
     [ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md),
     [ADR-0167](0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md)):**
     After Effects has no skew on a layer's base Transform either (only inside shape-group and
     text-animator sub-transforms or the `Transform` effect;
     [#898](https://github.com/MBehtemam/Montagent/issues/898),
     [#901](https://github.com/MBehtemam/Montagent/issues/901)), and
     [ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md) refuses skew as an effect
     because it is a spatial transform that changes bounds (`R-OFF-CANVAS`, `query --at`),
     while `path`
     ([ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md))
     draws any sheared shape directly. For `Corner Pin`,
     [ADR-0167](0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md)'s
     Projection (`swivel`, `tilt`, `perspective`) turns any visual element into a perspective
     quadrilateral, and arbitrary four-corner screen inserts can be pre-rendered or solved into
     transform/projection keys unless a forcing case appears. Both stay refused on their model
     grounds.
- **One `RC`-only ruling (row 26,
  [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)
  §1) — "no precedent in the reference class" bar lifted:**
  [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)
  §1 excluded `sweep` and `two_point_conical` gradients solely because *"Sweep and two-point
  conical have no precedent in the reference class."* The After Effects censuses confirm both
  rendered results in After Effects: shape-layer `Gradient Fill` and `Gradient Stroke` carry
  **Highlight Length** and **Highlight Angle**, which place the focal point inside a radial
  gradient (`two_point_conical`,
  [#901](https://github.com/MBehtemam/Montagent/issues/901)), and Layer Styles'
  `Gradient Overlay` carries an **Angle** style (`sweep`,
  [#898](https://github.com/MBehtemam/Montagent/issues/898)), and both shaders are already
  present in the pinned `skia-safe` build
  ([ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)).
  The "no precedent in the reference class" bar on `sweep` and `two_point_conical` is lifted,
  and both are handed to [#908](https://github.com/MBehtemam/Montagent/issues/908) on map
  [#896](https://github.com/MBehtemam/Montagent/issues/896) to triage alongside the other shape
  and paint gaps. Any build remains gated by
  [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)
  §8's Skia byte-identity prototype rule.

## Why

**After Effects' results are in scope; its layer-graph mechanisms are not.** An agent asked for
a motion-graphics piece needs the visual and audio results After Effects produces, not the file
mechanisms an interactive compositor uses to wire layers together. Treating After Effects as
precedent for **rendered results** removes a pointless demand-prototype hurdle for capabilities
an established motion tool already ships, while keeping **CapCut and Premiere first for shape**
and the **four invariants** unwaivable prevents After Effects' layer-reference and expression
mechanisms from entering by analogy.

## Considered and refused

- **Keep ADR-0145 word-for-word (After Effects is never precedent; every After Effects-only
  capability needs an entry-test prototype to prove demand).** Refused: map
  [#896](https://github.com/MBehtemam/Montagent/issues/896)'s premise is that Montagent edits
  video and generates animation, and requiring an owner-accepted prototype merely to establish
  that an After Effects-documented capability is wanted confuses proving demand with testing
  schema shape or Skia byte-identity.
- **Give After Effects equal weight with CapCut and Premiere for mechanism and shape.**
  Refused: After Effects regularly reaches a look through non-painting layers (cameras, nulls,
  adjustment layers, track mattes), range selectors, or expressions—mechanisms that fail the
  four invariants when copied into a flat, literal JSON document. CapCut and Premiere's flat,
  self-contained element mechanisms fit Montagent's model first.
- **Reopen all five `RC`-only rulings automatically.** Refused: four of the five (`parent`
  link, IK/rig controllers, 3D camera & depth sorting, and skew & corner pin) are either After
  Effects *mechanisms* whose results `nest`, Projection, `path`, or a skill script already
  cover, or capabilities not in vanilla After Effects at all (built-in bone IK). Only `sweep`
  and `two_point_conical` gradients were held out solely for lacking precedent in the reference
  class.

## Consequences

- `CONTEXT.md`'s **Reference class** and **Entry test** entries are updated to state the
  two-tier rule (CapCut, Premiere, and After Effects are precedent for rendered results; CapCut
  and Premiere are first for mechanism and shape; the entry test applies when none of the three
  tools has the capability).
- [ADR-0003](0003-general-video-editor-not-channel-tooling.md),
  [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md),
  [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md),
  and
  [ADR-0171](0171-svg-and-lottie-do-not-enter-as-sources-and-each-is-pre-rendered-through-the-skill.md)
  gain amendment banners pointing here.
- Map [#896](https://github.com/MBehtemam/Montagent/issues/896) drops *"Whether parenting comes
  back"* from `## Not yet specified`, adds `sweep` and focal-point radial
  (`two_point_conical`) gradients to its shapes/paint fog, and unblocks
  [#908](https://github.com/MBehtemam/Montagent/issues/908) to triage the ten After Effects
  censuses under this rule.
