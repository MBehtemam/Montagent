---
status: accepted
amends: 0033 (discharges the deferred "loop seam is out of scope for this check
  and this ticket" fog entry by naming the field `R-SOURCE-CUT-POP` was written
  to wait for, and confirms the extension it predicted is mechanical)
---

# `loop` is a boolean project field; the wrap seam reuses `R-SOURCE-CUT-POP` unchanged

[ADR-0033](./0033-same-source-cut-continuity-is-a-review-check.md) specified
`R-SOURCE-CUT-POP` — same track, same canonicalized source, no gap, resolved
transform properties compared at the cut — and explicitly declined to make it
cover the fixture's own wrap-around: `photo-05-loop`'s resolved `scale`
(~1.0064) disagrees with `photo-05-intro`'s start value (1.0000) by ~0.64%,
the same defect class, but "that comparison requires treating the project as
looping, which nothing in the schema asserts." It predicted the fix would be
mechanical once a field existed: "the last element and the first element
become an ordinary same-source-adjacent pair, and no new mechanism is needed
then either." This ADR supplies that field and confirms the prediction.

## The field

**`loop`, a project-level boolean**, default `false`/absent. Declaring
`loop: true` asserts that the project is meant to be played wrapped —
`duration` connects back to `0`.

**No loop-start point beyond `0`.** Nothing in the one real fixture, or in any
prior ADR, names an intro-then-loop pattern (a sub-range from a declared
`loopFrom` repeating after a one-shot lead-in). Inventing an object-shaped
field (`{"from": <ms>}`) now would mean specifying, with no fixture to
arbitrate it, what happens when `loopFrom` falls mid-element rather than on a
track boundary, whether elements straddling it are legal, and how `duration`
and `loopFrom` interact — a pile of undesigned questions bought for a pattern
nobody has asked for. A boolean is the shape fully determined by fields that
already exist (`duration`, `0`), and it widens without breaking any file
already written (`loop: true` ≡ a future `loop: {"from": 0}`) if that need
ever arrives with real evidence behind it.

**`loop` affects nothing but this `validate` check.** It does not touch
`render`, and Montagent writes no container-level loop metadata. Loop support
at the container level is not a stable cross-format fact to begin with — MP4
has no standard loop atom, GIF/APNG do, HTML `<video loop>` is a player
attribute, not a file property — so "write loop metadata" is really "pick a
per-container heuristic Montagent cannot actually guarantee," exactly the kind
of judgment call ADR-0006 keeps out of the tools. Whether and how a hosting
platform loops the rendered file is outside this boundary; `loop` states
authorial intent for `validate` to act on, nothing more.

## The check

**The wrap-adjacency condition is the direct generalization of ADR-0033's
`A.end == B.start`.** For each track, when `loop: true`: let `first` be the
track's element with the smallest `start` and `last` the element with the
largest `end`. If `last.end == duration` and `first.start == 0`, `last` and
`first` are treated as an ordinary adjacent pair under `R-SOURCE-CUT-POP` —
same canonicalized-source test, same source-time-continuity term for
time-based sources, same per-property tolerance table, same unconditional
`origin`-mismatch trigger, same `review` severity, same per-property
out-value/in-value/delta report.

**A gap at either edge suppresses the check on that track, exactly as an
ordinary gap does today.** If `last.end < duration` or `first.start > 0`, the
wrap-adjacency condition simply fails and `R-SOURCE-CUT-POP` does not fire for
that track's wrap — no new gap rule is needed, because this is the same
condition (`A.end == B.start`) ADR-0033 already requires, evaluated at the
wrap instead of at an interior cut.

**Location cites both boundary instants.** The seam is not one interior
timestamp; it is two boundary instants on two different elements. Per
ADR-0061's "state the raw measured fact," the finding names both:

> *Example: `R-SOURCE-CUT-POP` at the loop seam (duration=65216 → 0) —
> `photo-05-loop` → `photo-05-intro`, both `images/05.png`: `scale` 1.00640 →
> 1.00000 (Δ −0.00640, one axis shown, both equal).*

**No new finding code.** ADR-0033 already reasoned this through — "no new
mechanism is needed then either" — and the comparison is byte-identical to an
interior cut. `R-SOURCE-CUT-POP` fires on the wrap pair when `loop: true`; the
dual-instant location clause is what tells a reader it's the wrap rather than
a mid-timeline cut, so a second code would encode information the location
already carries.

## Resolution process

Two rounds. Round 1 (grilling, 4 questions: field scope, what `loop` affects,
wrap-location citation, finding-code reuse) answered directly with the author.
Round 2: a three-juror court (Opus, Sonnet, Fable), each independently given
the same four-part question, blind to each other. **Unanimous, 3/3, on all
four parts** — boolean field, `validate`-only scope, dual-instant location
citation, reuse of `R-SOURCE-CUT-POP`. All three converged on the same
underlying reasoning: don't build schema surface or cross-tool coupling the
fixture hasn't earned, and don't split one mechanically-identical check into
two names when ADR-0033 already named it as one.

## Consequences

- **New project field `loop` (boolean, default `false`).**
- **`R-SOURCE-CUT-POP` extends to the wrap** when `loop: true`, per-track,
  under the generalized adjacency condition above — no new finding code, no
  new tolerance table, no new severity.
- **The map's "loop seam has no owner" fog entry is discharged.**
- **`render` and every other tool are unaffected.**
