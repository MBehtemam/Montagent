---
status: accepted
amends: 0006 (states explicitly, as a named test plus a fenced exception, the
  boundary this ADR left implicit — "may not state anything that requires
  knowing what the video is for" — between an internal-consistency fact and a
  judgment call), 0034 (`R-CAPTION-PACE` is named as the first, and to date
  only, member of the fenced category; its existing citation and
  raw-measurement reporting already satisfy the rule below without change)
---

# `validate`'s fact/judgment boundary: threshold provenance decides admission, not severity; a fenced exception for a cited external number

> **Amended by [ADR-0071](0071-caption-check-evidence-corrected-and-the-fenced-category-has-two-members.md).**
> **The fenced category has two members, not one.** `R-CAPTION-MIN-DURATION`'s 834 ms
> (ADR-0054) fails the same fact-only test and is admitted on the same three conditions —
> which is this ADR's rule working, not being bent. What is retired is counting the
> members in prose: `crate::registry`'s `ThresholdProvenance` is the live answer, and the
> test below already holds every `External` check to the fence rather than a named one.

[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) says `validate`
answers one question — is this file internally legal, and does it agree with the
media on disk — and never *"does this file say what you meant it to say."* Two
tickets landed since and were read, on the surface, as pressure on that line:
[#64](https://github.com/MBehtemam/Montaget/issues/64), which made certain
timeline intervals ("slack") invariant, and
[#66](https://github.com/MBehtemam/Montaget/issues/66), which added
`R-CAPTION-PACE` and `R-CAPTION-REPEAT-DURATION` to `validate` via
[ADR-0034](./0034-caption-pace-and-repeat-duration-checks.md). This ADR draws
the boundary the two tickets left implicit.

## #64 is a contrast case, not evidence of drift

#64's enforcement lives entirely in `shift` (which refuses an edit that would
change an existing slack's size) and `compare` (the backstop for slack drift
under a raw file edit) — [ADR-0032](./0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md).
It added no check to `validate` at all. The reason is structural: whether a gap
is load-bearing is a property of *intent*, undecidable by reading one document,
so the project already routed it to the tools that mutate and diff rather than
the tool that reports. #64 is useful here only as a worked example of the
escape hatch this ADR's fenced exception does *not* need to reach for —
not as a precedent that `validate`'s scope grew.

## The test: threshold provenance, not severity

The candidate test that fails is *"severity already does the job — `error` is
reserved for guaranteed-wrong facts, and `review`/`note` may carry externally
calibrated judgment, since neither blocks the render."* Severity answers *how
bad is it if true*; it says nothing about *how do you know it's true*, and
collapsing the two licenses any human-factors heuristic whatsoever — "this cut
is too fast," "this font is too small" — the moment it is filed under `review`.
That is exactly the territory ADR-0006 fenced off, wearing a severity label
instead of a `validate` label.

The test that holds: **a check is fact-only if every number that decides
whether its finding fires is derivable from the document itself, or from the
format's own fixed rendering semantics — fps, a declared box, a declared
duration, codec-level quantization.** A number borrowed from outside both —
published human-factors guidance, a style convention, anything not present in
the document and not entailed by the format's own semantics — is external.
What matters is the *deciding* number, not the check's subject matter or a
tolerance used only to make a comparison well-defined: `R-CAPTION-REPEAT-DURATION`'s
one-frame tolerance comes from the project's own declared `fps`, so the check
passes even though "captions shouldn't visibly shrink on repeat" is itself a
human intuition. `R-CAPTION-PACE` fails: strip its 20 cps and the check has no
predicate left, and that number comes from published Netflix/BBC timed-text
guidance — nowhere in the document, not entailed by any rendering rule.

Applied to every check `validate` currently runs, `R-CAPTION-PACE` is the only
one that fails.

## The fenced exception

`R-CAPTION-PACE` is kept, not moved or grandfathered. Moving it costs a working,
already-useful check (it fires on the fixture's `hook-loop` at 25.8 cps) to
protect a purity ADR-0006 never actually promised, and there is no existing
tool whose job is "read one document, report one measurement" for it to move
to — inventing one duplicates `validate`'s traversal for a single finding.
Grandfathering it with no rule leaves the boundary exactly as implicit as the
state that produced this ticket, and the next borrowed threshold arrives with
no citation, no raw measurement, and no severity discipline, citing
`R-CAPTION-PACE` as precedent for all three.

Instead: a check may compare a document-derived fact against an
externally-sourced numeric threshold if, and only if:

1. **Severity is `review` or `note`, never `error`.** `error` means the render
   is refused or guaranteed wrong; no externally-calibrated human-factors
   number can guarantee that a legal render is wrong.
2. **The finding states the raw measured fact as its substance** — the actual
   cps, not a pass/fail — so a reader who disagrees with the threshold still
   has the number.
3. **The threshold's source is cited inline in the finding, and documented in
   the check's own ADR** — not `validate`'s general documentation, the specific
   ADR that introduced the check.

`R-CAPTION-PACE` already satisfies all three as written in ADR-0034: it reports
measured cps, cites Netflix/BBC guidance, and runs at `review`. No change to
ADR-0034 or its implementation is required — this ADR ratifies existing
practice and makes it a checkable rule rather than an unstated precedent.

Citation is **binding policy for future checks of this shape, not
best-effort.** A citation requirement that is optional degrades within a
handful of checks to "read it somewhere," and once that happens the test above
can no longer be applied to the check — there is no provenance left to
inspect. The enforcement cost is one paragraph in the introducing ADR and one
cited field in the finding; the payoff is that "which `validate` checks carry a
borrowed judgment?" stays answerable by inspection instead of by re-deriving
every check from scratch, which is the exercise this ticket had to be opened
to perform once already.

## What this does not change

`validate`'s governing sentence from ADR-0006 stands: a finding may state
anything derivable from the project, the media on disk, and the published
rendering semantics, and may not state anything that requires knowing what the
video is for. This ADR adds a name for the one place that sentence was being
stretched, and a bound on how far it may stretch. `R-CAPTION-REPEAT-DURATION`
and every other existing check were already compliant and are unaffected.
`validate` now has two compartments — internal-consistency facts, and facts
compared against a cited external number at non-blocking severity — and a
reader should be able to tell which compartment a finding is in from the
finding itself.

## Resolution process

Resolved via a two-round court, three jurors per round (Opus, Haiku, Fable),
each blind to the others and to the other round, given the same question in
four parts. **Unanimous across all six ballots, both rounds:** threshold
provenance (not severity) is the test, and citation is binding rather than
best-effort. **5 of 6** for keeping `R-CAPTION-PACE` fenced in `validate`
rather than moving it to a new mechanism or leaving it unfenced; the one
dissent (round 1) was from a model that reversed itself to the majority
position on independent resampling in round 2. All six ballots converged,
independently, on treating #64 as a contrast case rather than as evidence of
boundary drift.
