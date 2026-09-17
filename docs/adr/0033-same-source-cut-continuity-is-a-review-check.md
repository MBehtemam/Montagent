---
status: accepted
---

# Same-source cut continuity is a `review` check, keyed on source and track — not `group`

> **Amended by [ADR-0062](./0062-loop-declares-a-boolean-wrap-r-source-cut-pop-extends-mechanically.md)**,
> which discharges this ADR's deferred loop-seam fog entry: a project-level
> boolean `loop` field, and confirmation that the wrap extension is exactly
> the mechanical one predicted below — no new finding code, no new mechanism.

[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) named this check
inside a list of two the exercise found unnamed, gave it a timestamp that does not
match the fixture, and left it blocked on [#21](https://github.com/MBehtemam/Montaget/issues/21).
#21 is closed. This ADR gives the check an owner and a full specification.

**Correction to ADR-0006:** the second same-source cut is at **64016**, not 64816.
The cut is real — `photo-05-quiz` ends and `photo-05-loop` begins, both drawing
`images/05.png` — only the digit was transposed. Nothing about the finding changes;
only the timestamp does.

## The defect

Two adjacent elements on one track draw the same source with no gap between them —
the image, video frame, or audio does not change — and an animated property
resolves to different values on either side of the cut. The viewer sees one
continuous shot; the frame pops.

Measured on the fixture's `photo` track, all six cuts, `scale`:

| cut | t (ms) | same source | out | in | pop |
| --- | --- | --- | --- | --- | --- |
| photo-05-intro → photo-05 | 3018 | yes | 1.01610 | 1.00000 | +1.61% |
| photo-05 → photo-06 | 17472 | no | 1.07709 | 1.0 | +7.71% |
| photo-06 → photo-07 | 30603 | no | 1.07003 | 1.0 | +7.00% |
| photo-07 → photo-08 | 42763 | no | 1.06485 | 1.0 | +6.49% |
| photo-08 → photo-05-quiz | 53856 | no | 1.05916 | 1.0 | +5.92% |
| photo-05-quiz → photo-05-loop | 64016 | yes | 1.05419 | 1.00000 | +5.42% |

The four different-source cuts resetting `scale` to `1.0` are correct — a new
image starting its own Ken Burns move. The two same-source cuts are the same
shot, cut with no visual reason to reset anything, and the discrimination is
exactly one field wide: `source`.

## Why `group` is the wrong key

[ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)'s
group-keyframe-time check answers a different question — *do elements sharing a
`group` move together?* — and the fixture shows it cannot stand in for this one:
`photo-05-intro`/`photo-05` share `group: item-05`, so ADR-0012's check happens
to catch the 3018 pop by coincidence. `photo-05-quiz` (`group: quiz`) and
`photo-05-loop` (`group: loop-tail`) do not share a group, so the same mechanism
misses the **larger** pop at 64016 entirely. Coverage that depends on whether an
author happened to assign a shared `group` — an authoring convenience with no
obligation to track source identity — is not coverage. The two checks stay
separate, each named for what it actually answers.

## The check

**Trigger.** For two elements `A`, `B` on the **same track**, where `A.end ==
B.start` (a hard cut — a gap suppresses the check, since a gap is content, not a
seam) and `A` and `B` resolve to the **same source**, comparing:

- **Source identity is the canonicalized path**, not a raw string match.
  `images/05.png` and `./images/05.png` name one file; comparing the resolved
  path is a fact derivable from the document exactly as reading the file from
  disk already is — it is not the perceptual or fuzzy matching this project's
  checks otherwise refuse to do.
- **For a time-based source (video, audio), same path is necessary but not
  sufficient.** Two clips of one file whose `source_start`/`source_end` are not
  contiguous across the cut (`B.source_start` is not `A.source_end`) are a
  deliberate cut *within* the media, not a continuation of one shot, and a reset
  there is exactly as correct as a different-source cut. The check additionally
  requires source-time continuity for such sources. An image source has no time
  axis, so this term is vacuously satisfied — which is why the fixture, being
  all stills, never exercises it, and why it must be specified now rather than
  discovered on the first project with video in it.

**What is compared.** Every animatable transform property — `x`, `y`, `scale`
(`sx`, `sy` independently), `rotation`, `opacity` — using each element's
clamped-resolve value ([ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)):
`A`'s value as resolved at `A.end` (equal to its last keyframe under clamping,
whether or not `A` is trimmed short of its own last keyframe) against `B`'s
value resolved at `B.start`. `A` never actually renders at `t = A.end` under the
half-open interval — the comparison is between the limit approaching the cut and
the value entering it, both well-defined by the existing clamp rule.

**`origin` is not itself compared, but an unequal `origin` across the cut is its
own trigger for the same finding, unconditionally.** `origin` is not a timeline
value — it is the reference frame `x`/`y`/`scale`/`rotation` are read against —
so comparing raw numbers across two elements with different origins can both
miss a real pop (compensating numbers) and manufacture a false one (equal
numbers, different meaning). Rather than resolve into a common frame, the
cheaper and equally honest fact is: two elements presenting one continuous shot
whose declared reference frames disagree is itself the finding, independent of
whether the raw property values happen to agree.

**Tolerance is per property, not one number.** A single epsilon does not
generalize once the check covers more than `scale` — 0.0005 is reasonable
headroom against float noise for `scale` and `opacity`, meaningless for `x`/`y`
in pixels or `rotation` in degrees:

| property | tolerance | purpose |
| --- | --- | --- |
| `scale` (per axis) | 0.0005 | float/round-trip noise only |
| `opacity` | 0.005 | float/round-trip noise only |
| `rotation` | 0.01° | float/round-trip noise only |
| `x`, `y` | 0.5px | sub-pixel noise only |

None of these ever carries the discrimination — every fixture pop clears its
column by two or more orders of magnitude. Their only job is refusing to fire on
noise; a real pop is never close to the line.

**Severity: `review`.** The cut renders; it is legal; a same-source pop is
sometimes a deliberate device (a jump-cut zoom-punch is real editorial
vocabulary), and nothing in the document says which this is — exactly the
"legal, renders, you must look at a frame to know if it was meant" case ADR-0006
reserves `review` for. Not `error`: that would be a verdict about intent, which
findings may not state. Not `note`: a visible pop in a published video is not
something the reader will skim past.

**The finding states the facts and stops.** Per property that fails its
tolerance: the resolved out-value, the resolved in-value, and the delta —
enough to triage from the report without re-opening the file, per ADR-0006's
"every number inline" rule. Finding code: `R-SOURCE-CUT-POP`.

> *Example: `R-SOURCE-CUT-POP` at 64016 — `photo-05-quiz` → `photo-05-loop`, both
> `images/05.png`: `scale` 1.05419 → 1.00000 (Δ −0.05419, one axis shown, both
> equal).*

## What this check does not cover

**The loop seam is out of scope for this check and this ticket.** Measured
across the fixture's own wrap — end of file back to `t=0` — `photo-05-loop`'s
resolved `scale` (~1.0064) disagrees with `photo-05-intro`'s start value
(1.0000) by ~0.64%. That comparison requires treating the project as looping,
and nothing in the schema declares that; asserting it would require knowing
what the video is *for*, which ADR-0006 forbids a finding from doing. This is
recorded as a map fog entry, not folded into `R-SOURCE-CUT-POP`: if a field
ever declares loop playback, this check's own logic extends to the wrap
mechanically (the last element and the first element become an ordinary
same-source-adjacent pair), and no new mechanism is needed then either.

**Cross-track adjacency does not trigger this check.** Two elements on
different tracks that happen to share a boundary instant are not a cut — both
may be visible and composited simultaneously. The trigger is scoped to one
track by construction.

## Consequences

- **`validate` gains `R-SOURCE-CUT-POP`**, keyed on same track + canonicalized
  source path + (for time-based sources) source-time continuity + no gap,
  comparing clamped-resolved `x`/`y`/`scale`/`rotation`/`opacity` per component
  against a per-property tolerance table, plus an unconditional trigger on
  unequal `origin` across such a cut.
- **ADR-0006's fixture timestamp is corrected**: 64816 → 64016.
- **ADR-0012's group-keyframe-time check is unchanged** and is not this check;
  a reader landing on either should be pointed at the other; both are cross-
  referenced here.
- **The loop-seam finding is deferred to the map's fog**, not this check,
  pending a field that declares loop playback.
