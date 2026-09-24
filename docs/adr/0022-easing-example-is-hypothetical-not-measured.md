---
status: accepted
amends: 0012 (corrects the worked example's provenance; no decision below changes)
---

# ADR-0012's easing worked example is hypothetical, not measured

[ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md) argues that **"the
raw form is forced, not chosen"**: splitting a named ease other than `linear`/`step` at an
arbitrary cut point yields a raw bezier that matches no published name. It illustrated this
with `photo-06`, published there as `"ease":"ease-in-out"`, computing a 10.3–10.9 px framing
cost if the split halves were snapped back to the nearest name, and stating *"four jurors
computed this independently and three agree to the digit."*

[#42](https://github.com/MBehtemam/Montagent/issues/42) later measured `photo-06`'s actual
motion by SSIM against the real reference video (`reference/kenburns/06.mp4`, 7 of 7 points)
and found it is **`linear`**, not `ease-in-out`. The migrated project file on `main` records
`photo-06`'s ease as the measured value. Grepping the committed fixture confirms it further:
**all 7 keyframes in the one real project file use `linear`; zero use any other named ease.**
`linear` is closed under subdivision, so `photo-06` was never at risk of the defect the
example was built to demonstrate.

## Decision

**The general claim stands; the worked example does not, and is corrected here rather than
in a new decision.** Closure under subdivision is a property of a finite name set under a
continuous family of curves (de Casteljau subdivision varies continuously in the cut
parameter; the named eases are five fixed control-point quadruples, and `linear`'s alone are
collinear with its endpoints, which subdivision preserves). That holds for any element
carrying a non-`linear`/`step` named ease, independent of whether `photo-06` — or anything in
the current fixture — happens to be one. Nothing in [#45](https://github.com/MBehtemam/Montagent/issues/45)
reopens ADR-0012's decisions: the property set, the keyframe shape, `entering`-semantics for
`ease`, the closed name set, or the SPLIT algorithm.

What is corrected: **ADR-0012's specific numbers describe no element that exists.** Read
`photo-06`'s example there as **hypothetical** — "an element with `photo-06`'s transform and
cut point, *if* it carried `ease-in-out`" — not as a measurement. The sentence *"four jurors
computed this independently and three agree to the digit"* is struck from this reading: four
agents agreeing on arithmetic from a shared, now-false premise is not independent
corroboration of anything, and reads as one when left standing next to a real element's name.
No new "realistic" example is substituted — the fixture has no non-`linear`/`step` eased
element to draw one from, and manufacturing one to look real would repeat the defect this ADR
exists to name, in a harder-to-spot form.

**The `fmt`/`validate` consequence — "a shifted file grows two bezier arrays at the cut
segment" — is a deduction from already-accepted rules (the closed name set, the SPLIT
algorithm, and which eases are closed under subdivision), checkable by direct computation on
any two control points. It ships as a documented fact, unconditionally, with one honest
caveat: it is **logically entailed and not yet observed in any real project file**, because
the only committed project has no non-`linear`/`step` eased keyframe to trigger it. That
absence is a scope fact about current data, not evidence against the deduction, and it is not
a reason to hold `fmt`/`validate` design for a fixture that would have to be authored
specifically to exercise it.

## Consequences

- Read `photo-06` in ADR-0012 as **hypothetical for the easing example only**; every other use
  of that element in ADR-0012, ADR-0013 and ADR-0015 is unaffected, since none of them depend
  on its ease.
- The committed fixture (`fixtures/en-halloween-decorating/`) currently exercises **zero**
  non-`linear`/`step` eases, so the SPLIT rule's bezier-subdivision path is untested by the
  fixture suite. This is a coverage gap to track, not a defect to fix by inventing data.
- No field, tool or check changes. This ADR corrects one paragraph's evidentiary status; it
  makes no new decision.
