---
status: accepted
amends: 0012 (discharges its "whether `clip` is keyframable" deferral)
---

# `clip` stays static

[ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md) established `clip` —
the frame-space aperture a source is drawn through — as static, but left open whether it
should ever be keyframed: *"a wipe or reveal is exactly a keyframed aperture. If it is, it
joins the properties SPLIT must handle and the 0.000 px result must be re-run over it."*
[ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md) then made `clip`'s
width/height the box a `fit` rule (`cover`/`contain`) derives an element's declared
`width`/`height` against, checked at strict integer equality — which only has a well-defined
predicate if `clip` has one value, not one per instant. [#55](https://github.com/MBehtemam/Montaget/issues/55)
asked whether that tension should be resolved by making `clip` keyframable (and picking an
instant for the fit check) or by keeping it static.

## Decision

**`clip` stays a permanently static rectangle. Wipes and reveals belong to the effect model
(#22), not to an animated `clip`.**

`clip` is currently doing two jobs at once: it is the aperture a source is drawn through, and
it is the fixed denominator ADR-0015's fit-deviation check compares a declared extent
against. Keyframing it would corrupt the second job with no principled repair — every
candidate instant for the fit check (the element's `start`, the first keyframe, forbidding
`cover`/`contain` under a keyframed `clip`) is arbitrary, and would leak that arbitrariness
into `validate`'s semantics permanently. ADR-0015's strict-equality guarantee is only
meaningful because the box it checks against is a single value; that matters more for an
agent-first format than a human-driven one, since an agent reasons about the invariant "the
fit box is always one rectangle," not about a rendered preview.

Ken Burns — the one motion idiom already shipped — is the existing proof that this format's
model of motion is *a static aperture with content moving behind it*: `scale`/`x`/`y` do the
moving, `clip` does not need to. A wipe or reveal in the CapCut/Premiere reference class
(ADR-0003) is almost never authored as an animated source crop; it is a transition or a mask
applied *over* an already-fitted element — a **visibility-over-time** concept, not a
**source-selection** one. Montaget's `clip` is the latter. Forcing a reveal through it would
buy only the weakest version of the feature (a hard-edged rectangular wipe, no direction, no
softness) at the cost of reopening SPLIT — verified to 0.000 px for its current property
set — for a property zero of the eight elements in the only real project file use.

Putting reveals in #22 as "how much of the already-fitted element is visible" is a
post-fit operation and sidesteps the instant problem entirely, because the fit box underneath
it never changes.

## Constraint placed on #22

Whatever mechanism #22 settles on for a reveal/wipe-style effect, **its time-varying
parameters must animate through the existing keyframe representation (`{t, v, ease}`) and be
subject to the existing SPLIT algorithm** — not invent a second, parallel animation system.
The risk every juror in this decision's court named independently is that an effect model
built without this constraint duplicates interpolation machinery the format already has and
has already verified.

## Evidence

Three-model court (Claude Opus, Claude Haiku 4.5, Claude Fable 5.1), each blind to the
others' ballots. **Unanimous 3/3**, converging independently on the same structural
argument — `clip`'s role as a fit-check invariant is incompatible with animating it — while
each also independently naming and accepting the same cost (v1 ships without wipes/reveals
until #22 lands) and the same mitigation (constrain #22 to reuse existing keyframe/SPLIT
machinery).

## Consequences

- ADR-0012's "whether `clip` is keyframable" is settled: no. No schema change; `clip` remains
  exactly as specified there.
- SPLIT's verified property set and its 0.000 px result are unaffected — nothing new joins it.
- ADR-0015's fit-deviation check keeps a single, permanently well-defined box with no
  exceptions.
- #22 ("Define the effect model and its closed vocabulary") inherits a stated design
  constraint: any effect parameter that varies over time must use the element-level keyframe
  mechanism, not a second one.
- Reversibility is asymmetric, and this decision takes the cheap side of it: static-now can
  become keyframable-later behind a schema version bump; shipping a keyframable `clip` with an
  arbitrary fit-instant rule now would be much harder to walk back once real projects depend
  on it.

## Not settled here

- The reveal/wipe effect itself — its parameters, direction, edge behaviour — is #22's to
  design, not this ADR's.
