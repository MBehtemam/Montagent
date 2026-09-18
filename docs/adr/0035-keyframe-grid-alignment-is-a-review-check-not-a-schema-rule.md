---
status: accepted
amends: 0005 (publishes the sampling rule the frame-alignment paragraph promised), 0006 (adds one check to the list), 0011 (adds a `measure` output)
---

# Off-grid keyframe times stay legal; `validate` gains an unreached-target check, and `measure` gains the grid arithmetic

[Ticket #67](https://github.com/MBehtemam/Montaget/issues/67), graduated from the map's
*"Frame alignment and the rounding rule"* fog entry by [#12](https://github.com/MBehtemam/Montaget/issues/12).
Two measured facts started it: fading `opacity` 1 → 0 over `[end-300, end]`, with `end`
off the project's 25fps/40ms grid, leaves **0.010–0.171 residual opacity** on the last
sampled frame instead of reaching exactly 0 — the element visibly pops off rather than
fading — and two agents independently wrote that literal, naive spelling and caught the
defect only in self-audit. Separately, the grid itself is non-obvious per `fps`: at 30fps
only multiples of 100ms are frame-exact, not the more intuitive 33.33ms, a fact one agent
found only because it had written its own checker.

Resolved by a jury of three independent models (Opus, Sonnet, Haiku) put to four
sub-questions, cross-examined against ADR-0005, ADR-0006 and ADR-0012.

## A keyframe's `t` is not constrained to the frame grid

Unanimous on the *mechanism*, split 1–2 on the *consequence*. Two jurors voted to make
off-grid `t` a schema error, on the argument that a keyframe names a specific intended
instant in a way an element boundary does not. That is rejected: [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)
already establishes that interpolation is a continuous function of a keyframe's `t`, so
an off-grid `t` is ordinary, meaningful data — a fade landing on a beat, or a keyframe
imported from an off-grid source, are legitimate authoring, not malformed input. A schema
constraint would also bake `fps`-dependent arithmetic into the schema layer, which
[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) has already kept out of
validation once, for exactly this reason: **alignment is never itself the check, only its
structural consequences are.** That is the same principle ADR-0006 applied to element
boundaries (109/120 off-grid on the real fixture, harmless), and it is applied here
consistently rather than narrowed to elements.

A bare `note` on every off-grid keyframe `t` is also rejected — it would fire on the
overwhelming majority of real keyframes (the fixture's element boundaries already show
the base rate), is non-actionable, and buys discoverability at exactly the cost
[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) names as a genuine
safety problem: alarm fatigue. The ticket's own evidence makes the case against it directly
— two agents had this exact fact available to read and missed it; a low-value note would
not have changed that.

## `validate` gains `R-KEYFRAME-UNREACHED`

**What it checks.** For every animated property, whether the property's stated final (or
first) keyframe value is ever actually produced by a sampled frame within the element's
own `[start, end)` range. This is computable from the grid (`fps`), the element's `start`/
`end`, and the keyframe list, with no I/O — the same budget every other `validate` check
in [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) holds to.

**Severity: `review`.** Unanimous 9/9 across all three questions this jury reached
consensus on — this is not illegal (`error` would refuse projects that are deliberately
mid-fade at a cut, which is legitimate), and it is not inert (`note` implies no action,
but this finding's entire content is "the rendered picture does not match what the file
declares," which a human must look at and either accept or fix by retargeting the
keyframe). It is the same shape as ADR-0006's uncovered-gap check — a quantization
artifact with a perceptible visual consequence — which is also `review`.

**The finding states the numbers**, per ADR-0006's anti-vagueness requirement: the
declared target value, the actually-sampled value, the sampled instant, and the
declared `t`. **Scope generalizes past opacity and past 0/1**: any animated property
whose declared endpoint is never sampled qualifies, not a special case pinned to fades.

**Why this stays narrow enough not to reopen the alarm-fatigue floor.** It fires only
when a sampled value provably diverges from a stated target — never on off-grid `t` by
itself. On a project where every keyframe's target is reached by some sampled frame
(which grid-aware authoring achieves trivially), this check is silent.

## The renderer needs no keyframe-specific rounding rule

Unanimous 3/3. The residual-opacity number in the ticket is not a rounding defect — it
is the *correct* value of a continuous function (ADR-0012), evaluated at the frame's own
exact sample timestamp, which happens to fall before the declared `t`. Manufacturing a
different value — by snapping the declared `t` to the grid before interpolating, or by
rounding the evaluated result — would mean a clip's opacity and a clip's visibility could
be computed on two different effective clocks, which is strictly worse than the cosmetic
defect: it breaks the one-absolute-clock premise the whole format is built on, and makes
the renderer non-portable (a second implementation snapping differently produces
different pixels from the same file).

This closes half of [ADR-0005](./0005-absolute-integer-milliseconds.md)'s stated debt —
"the renderer must publish its rounding rule" — by stating there is no rounding rule
*specific to keyframes*: evaluation is the general continuous-property rule, unchanged.
The other half of that debt is real and separate: the frame→timestamp sampling mapping
itself (which exact instant frame *N* samples, and that the element range is half-open
so `end` is never a sampled instant) still needs to be published normatively for the
format to be independently implementable. That is out of this ticket's scope and is
carried forward to a future edit of [ADR-0005](./0005-absolute-integer-milliseconds.md)
rather than re-litigated here.

## The grid formula is published and put in `measure`

Unanimous 3/3, and both halves are load-bearing rather than redundant. `measure` gains
an output — the nearest sampled instant at-or-before a given time, for the project's own
`fps` — so an author targeting an exact rendered value never derives the grid arithmetic
by hand. The same formula is also published normatively in this ADR, because
[ADR-0005](./0005-absolute-integer-milliseconds.md)'s debt was a **textual** promise: a
reader holding only the file and the spec — reviewing a diff, or implementing a second
renderer, with no live MCP session — must still be able to tell whether a stated fade
reaches zero. Neither form substitutes for the other; a tool call cannot discharge a
spec-readability promise, and prose alone repeats the exact failure the ticket recorded
twice (agents deriving fps-dependent gcd arithmetic in their heads and getting it wrong).

**The formula.** For a project at `fps`, the sampled frame grid step is `1000/fps` ms,
not necessarily integral. The nearest sampled instant at-or-before time `t` is:

```
floor(t * fps / 1000) * 1000 / fps
```

evaluated in exact rational arithmetic (integer numerator/denominator, never float), so
that e.g. at `fps=30` the grid step is `100/3` ms and the formula correctly resolves to
multiples of 100ms being the only frame-exact instants — the non-obvious fact one agent
found only by building a private checker. `measure`'s new output returns this value
directly; an author who wants `opacity` to read exactly 0 at the frame nearest an
element's `end` targets that returned instant instead of the literal `end`.

## Consequences

- `R-KEYFRAME-UNREACHED` joins `validate`'s check list from [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md),
  severity `review`, no I/O beyond what the grid and the document already provide.
- `measure` gains a nearest-sampled-instant output, keyed on the project's `fps`.
- No schema change — keyframe `t` remains an unconstrained integer millisecond.
- No renderer change — keyframe evaluation is unchanged; this ADR only names the rule
  that was already implicit.
- Carried forward, out of scope for this ticket: publishing the frame→timestamp sampling
  mapping itself, the remaining half of ADR-0005's debt.
