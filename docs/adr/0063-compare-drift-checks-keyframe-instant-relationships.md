---
status: accepted
amends: 0032 (states precisely where slack-drift's coverage of coincidence ends), 0036 (designs the destroyed-coincidence mechanics it deferred), 0039 (designs the coupled-motion-drift mechanics it deferred)
---

# `compare` gets one drift predicate over keyframe-involving instant pairs; boundary-vs-boundary residue is named, not solved here

[Ticket #151](https://github.com/MBehtemam/Montaget/issues/151), graduated from the
map's fog. [ADR-0036](./0036-shift-preambles-coincident-instants-validate-and-compare-stay-out.md)
parked the **destroyed-coincidence** hazard on `compare` without designing it;
[ADR-0039](./0039-group-keyframe-time-check-retracted-from-validate-reassigned-to-compare.md)
did the same for **coupled-motion-drift**, fixing only that it must be keyed
per-property, not per-`group`. Neither designed mechanics. Resolved by a jury of
three independent models (Opus, Sonnet, Haiku, Fable rotating across two rounds)
against the real, currently-committed fixture
(`fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json`).

## The predicate

One detector, one rule: **two instants that were numerically equal in the caller's
supplied ref are candidates; if they are no longer equal in the current file,
report it.** Exact equality only — not a preserved offset. A fixed-offset
relationship ("keyframe B is always 300ms after keyframe A even though they were
never equal") is a hypothesis about intent inferred from arithmetic, and admitting
it reopens exactly the failure [ADR-0039](./0039-group-keyframe-time-check-retracted-from-validate-reassigned-to-compare.md)
already paid for: on a 60-element file every pair of instants that both happened
not to move trivially "preserves its offset," so the candidate population explodes
and the signal drowns. Exact coincidence is the one relationship the file asserts
without the tool guessing — a reader can verify it by finding two identical
numbers. **This is deliberately narrower than the hazard the map named** (a
lower-third's bar and text drifting apart need not have ever shared an instant);
closing the fixed-offset case is left as an explicit fog entry below, not silently
dropped.

## Scope: keyframe-involving pairs, not boundary-vs-boundary

The predicate runs over three candidate populations, all requiring at least one
keyframe `t`:

- **(a) An element's own keyframe against its own boundary** — e.g. a Ken Burns
  ramp's first record landing exactly on its element's `start`.
- **(b) An element's keyframe against a different element's boundary.**
- **(c) Two different elements' same-*property* keyframe times** — the coupled-motion
  case, keyed per-property per [ADR-0039](./0039-group-keyframe-time-check-retracted-from-validate-reassigned-to-compare.md),
  never per-`group`.

**Boundary-vs-boundary coincidence — no keyframe on either side — is not designed
here.** [ADR-0032](./0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md)
already ships a drift check for it: **slack** (the distance from an element's
boundary to its nearest neighboring boundary in any track, or to `duration`) is
invariant, `shift` refuses to change it silently, and `compare` already reports
when a raw edit changes a slack's size between two versions. A coincidence is
slack of size zero, so slack-drift already catches the common case of a
boundary-vs-boundary coincidence being destroyed.

**It does not catch all of it, and the ADR record must say so rather than imply
otherwise.** Slack is a **minimum over the nearest partner**; coincidence is
**pairwise**. On the real fixture, 92 of 120 boundary endpoints (76.7%) coincide
with at least one other boundary, and many of those coincidences are
multi-partner: **14 of the 19 coincident instants have three or more boundaries
on them**, confirmed by
[`count_coincidences.py`](../research/juries/compare-drift-checks/count_coincidences.py).
If one partner in such a cluster moves
off the shared instant while another stays, the moved boundary's *nearest*
neighbor may still be at distance zero — slack reports nothing — yet a specific
pairwise coincidence was destroyed. **This residue is named as fog, not
mechanized here**: it is a refinement of an already-shipped check (ADR-0032), not
new content, and this ticket's design budget belongs to the keyframe-involving
cases no other check reaches. See Consequences.

## Case (a): self-keyframe-to-own-boundary is in scope, and is the only measured case

[ADR-0036](./0036-shift-preambles-coincident-instants-validate-and-compare-stay-out.md)
excluded "self-keyframe-on-own-boundary" from a proposed `validate` **census**,
reasoning it is "the most structurally benign category" — a keyframe sitting on
its own element's boundary is definitionally correct authoring, so reporting its
mere presence adds nothing. That reasoning is about presence and does not
transfer here. `compare` never reports presence; it reports **destruction** — a
relationship that held in the ref version and doesn't now. A self-keyframe
drifting off its own element's boundary is not a restatement of benign structure;
it is a report that something changed. Concretely: [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)
made a keyframe's `t` absolute geometry the element carries, not a timeline time —
so a raw hand-edit that moves an element's `start` without moving its keyframes
(bypassing `shift`, which carries them together) silently begins the Ken Burns
ramp before the photo appears, or mid-shot. That is exactly the silent-breakage
shape `compare` exists to catch, and it carries none of ADR-0039's cross-element
inference risk — one element, one property, no intent asserted about another
author's element.

This is also the **only one of the three candidate populations with a live
population in the real fixture today**: all 7 keyframed elements (the Ken Burns
photos) have their first `scale` keyframe exactly on their own `start`. Excluding
it would ship a check with no real-data coverage at all.

## Case (c): coupled-motion-drift ships, unmeasured

The real fixture has **zero** cross-element same-property keyframe-time matches
today: only 7 elements carry any keyframed transform property, each occupying a
disjoint time segment (the photos never overlap), so no two elements' `scale`
keyframes ever land on the same instant to begin with. The predicate for (c) is
identical to (a) and (b) — per-property, cross-element, delta-zero-in-ref,
nonzero-in-current — so designing it costs no additional mechanism, only the
honesty of saying what is and isn't known.

**Ship it, stated as unmeasured.** This project has repeatedly found theory-only
designs wrong on contact with real data — frame-alignment rules, the `sequence`
label, and the direct ancestor of this check, which false-positived on 5/5 real
correct `group`s when tried in `validate`. Claiming (c) is validated when the only
real file measured has a population of zero would repeat that mistake in the
other direction: confidence the project never earned. The trigger for revisiting
this — tightening the predicate, adding a tolerance, or scoping it further — is
the first real project with genuine coupled motion, or the first reported false
positive, whichever comes first.

## Output shape

Reuses `compare`'s existing grouped-deltas-as-facts convention
([ADR-0011](./0011-tool-surface-reads-checks-renders.md)), the same shape as
slack-drift ([ADR-0032](./0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md)):
a population, a common outcome, and the exceptions named. One addition — a
grouping-key naming which of the three populations produced the line, since the
existing template only ever named element populations:

```
sentence-06's opacity keyframe and photo-05's own start both sat at 3018 in <ref>;
in <current> they no longer coincide (3018 vs 3200).
```

```
2 of 3 elements in the lower-third group had scale keyframes at 6000; lt-text's
scale keyframe moved to 6300.
```

**No severity.** `compare` stays purely descriptive, unchanged from its existing
shape. Severity in this project is `validate`'s vocabulary
([ADR-0006](./0006-validate-reports-facts-and-render-enforces.md),
[ADR-0061](./0061-validate-judgment-boundary-threshold-provenance-and-a-fenced-exception.md)),
load-bearing because `render` enforces on `error` — `compare` has no such
enforcement relationship. A destroyed coincidence is frequently the entire point
of an edit (the author deliberately split a synchronized cut); assigning severity
would require judging intent `compare` cannot see, reproducing ADR-0039's 5/5
false-positive failure in a new tool.

## Scope boundary confirmed, not reopened

The first-authoring blind spot — a coupling that was never correct from the
moment of authoring, with no prior correct version to diff against — stays
explicitly out of scope, as ADR-0039 already ruled. `compare` is a differ; there
is nothing to diff against when the relationship never held. Closing it requires
a schema field declaring intended coupling, which nobody is proposing here or in
this ticket.

## Consequences

- **`compare` gains one new predicate** — exact-equality drift — run over three
  candidate populations, all requiring at least one keyframe `t`: self-keyframe-
  to-own-boundary, keyframe-to-other-element's-boundary, and cross-element
  same-property keyframe-to-keyframe.
- **Boundary-vs-boundary coincidence is not redesigned.** ADR-0032's slack-drift
  covers the single-nearest-partner case. The map's **Not yet specified** gains a
  fog entry: multi-partner coincidence clusters, where one pairing can be
  destroyed while slack's minimum stays at zero via a different partner — real on
  this fixture (14 of 19 coincident instants have 3+ boundaries), unmechanized.
- **The map's Not yet specified also gains**: fixed-offset / staggered-entrance
  relationship drift (e.g. a fade always completing exactly 300ms before a cut,
  never coincident), explicitly deferred rather than silently excluded from "held."
- **Case (c) ships with an explicit unmeasured-noise caveat** in its own
  documentation, with a named revisit trigger (first real coupled-motion project,
  or first false positive).
- **No severity, no schema change, no renderer change.**
