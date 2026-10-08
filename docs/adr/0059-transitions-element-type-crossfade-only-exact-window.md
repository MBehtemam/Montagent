---
status: accepted
amends: 0040 (confirms the "own shape — likely id-targeting" prediction; the effect vocabulary's element-locality is not stretched to cover transitions), 0004 (confirms the track non-overlap rule needs no exception — a transition's two bridged elements still live on separate tracks/layers, exactly as any other simultaneous-visibility case already requires)
---

# Transitions: their own element type, crossfade only in v1, exact-window, non-overlap rule untouched

> **Amended by [ADR-0176](./0176-a-transition-carries-the-audio-across-its-cut-in-one-field-and-an-audio-only-crossfade-is-a-transition-kind.md)**: a transition may now carry sound as well as picture. Visual kinds gain an
> optional `audio` field (`cut`, `constant_power` or `constant_gain`) on the window this ADR already pins, and a
> fifth kind, `audio_crossfade`, bridges `audio` or `video` elements and paints nothing. The rationale below
> that a transition "inherently reads two elements' pixels" is widened, not retired.

> **Amended by [ADR-0150](./0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)**: the deferral is discharged. `wipe`, `slide` and `push` join
> `crossfade`, with a required `direction` (the way the motion travels) and an optional
> `ease`, and `validate` now checks the `from`/`to` references.

CapCut and Premiere both make transitions first-class; nothing in the settled model
(ADR-0012's transform/keyframes, ADR-0040's effect model) could express one. This
settles representation, scope, timing, and interaction with the track model —
resolving [#133](https://github.com/MBehtemam/Montagent/issues/133).

## Decisions

**Transitions are their own element type**, not a property on either bridged element
and not an effect-vocabulary entry. `type: "transition"`, its own `id`, its own
`start`/`end`, a closed-vocabulary `kind` field, and two id references naming the
elements it bridges — the same id-targeting namespace
[ADR-0019](./0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md)'s anchors
already use. An effect is explicitly element-local
([ADR-0040](./0040-effect-model-attachment-and-v1-vocabulary.md)) and a transition
inherently reads two elements' pixels together, so stretching effects to cover it
would either break element-locality for every other effect reader or bolt on a
second-id reference nothing else there has. A property on one of the two elements
creates an unprincipled ownership question (outgoing or incoming? both, now synced?)
and gives the transition no time range of its own for `montagent timeline`/`query` to
surface.

**Scope includes plain crossfade**, not only directional transitions (wipe, slide,
push). A pair of opposite opacity ramps on two tracks is arguably already expressible
today, but it fails this project's standing discoverability test: nothing states the
two ramps are one authorial unit, so an agent recovers "this is a crossfade" only by
cross-referencing tracks and noticing a matching, oppositely-signed ramp — exactly the
inference-from-scattered-state the format exists to eliminate. Crossfade is also the
one member of the family every reference-class editor treats as first-class.

**v1 `kind` vocabulary is `crossfade` only.** Wipe, slide and push are deferred, not
rejected — reference-class ubiquity is evidence they're eventually needed, but it is
not evidence for how they parameterize (direction vocabulary, push-vs-overlay, default
duration), and freezing a closed-vocabulary member on a guess is the trap
[ADR-0040](./0040-effect-model-attachment-and-v1-vocabulary.md) avoided with colour
filter. Crossfade alone commits to nothing beyond the time range the element type
already carries. Graduates to a future ticket once a forcing case exists to shape
those parameters.

**A transition's `start`/`end` must exactly equal the intersection of the two elements
it bridges.** Outside that window only one of the two elements exists, so a wider
declared range is a field the renderer cannot honour — worse than no field
([ADR-0007](./0007-text-runs-literal-size-declared-fonts.md)). A "hold before the
crossfade begins" is not a gap: it is already expressible as keyframes on the bridged
elements' own transform properties inside an exact-overlap transition, so looser
containment would just give that shape a second, redundant home. `validate` can
therefore check transition bounds as a closed-form function of the two referenced
elements' own ranges — drift on either side surfaces immediately as an error.

**The per-track non-overlap rule ([ADR-0004](./0004-tracks-as-constrained-lanes.md))
is untouched — no exception for transitions.** The two bridged elements must already
live on separate tracks (or stack via layer/anchor), exactly as anything else that
needs simultaneous visibility already requires. A same-track exception would leave
z-order undefined at the one moment it matters most — the renderer would have to
invent an ordering from array position or id order, implicit semantics the format
forbids — and would turn the overlap checker from a local, order-independent predicate
into one that must cross-reference every transition element in the file. The
transition element's job stays purely descriptive: name the pair, own the exact
window, let `validate` check it.

## Consequences

Authoring a transition costs one extra track declaration per bridged pair — a
"checkerboard" of tracks A/B for a sequence of cross-fading clips — and the
transition's range is derived, redundant data that must track the two elements it
bridges: trimming either invalidates the file until the transition is updated. Both
costs are mechanical and loud rather than silent, the trade this project has taken
every other time (ADR-0004's anchors, ADR-0019's dangling-target checks).

This does not preclude wipe/slide/push arriving later as additional `kind` values —
the element type, timing rule and non-overlap answer all extend to them unchanged;
only their own parameter shape is future work.
