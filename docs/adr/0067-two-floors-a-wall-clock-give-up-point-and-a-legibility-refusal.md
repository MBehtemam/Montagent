---
status: accepted
amends: 0050 (states where its floor binds, given 0065's ladder), 0065 (retires its "legibility is unmeasured" clause and its deferral of a sub-540p pass), 0046 (confirms the long-edge cap survives 0065)
---

# `preview` has two floors, not one: a wall-clock give-up point at 540p and a legibility refusal at 360p

> **Amended by [ADR-0095](./0095-the-sheets-budget-is-served-tile-width-and-overflow-refuses.md)**,
> which **narrows the 360p legibility floor below to a standalone proxy frame.** This ADR
> keeps the floor as *"a guard on the ladder's future, and on any caller-specified proxy
> resolution"* — and `frame`'s contact-sheet range mode is the first thing that clause
> reaches for by name. It does **not** govern a sheet tile: an 18-tile sheet's tile is
> 184×328, so no tile count clears a 640 px long edge, and if the floor applied the feature
> could not exist. The reason it does not is that the two numbers measure different things —
> ADR-0050 judged a frame *standing alone in a viewport*, while the sheet's own measurement
> judged a tile *in a grid, beside its neighbours, under a label*, which is why absence-of-text
> survived to 92 px tiles. The sheet carries **its own two limits in its own currency**
> (served tile width: a 180 px target and a 140 px floor), so *floor* now names **three**
> things across the two ADRs and all three are spelled out in `CONTEXT.md`. Everything about
> the `preview` ladder here is untouched, and 360p remains unreachable by degradation.

[ADR-0050](0050-preview-hard-refuses-below-360p.md) and
[ADR-0065](0065-preview-proxy-target-720p-540p-floor-disclosed-not-certified.md) both
landed on `main` as `status: accepted`, both using the word *floor*, with different
numbers and neither amending the other — recorded as
[#178](https://github.com/MBehtemam/Montagent/issues/178). This ADR resolves it.

**Neither is overturned.** They are answering two different questions, and the word
*floor* was doing double duty.

## The two questions

| | ADR-0065's floor | ADR-0050's floor |
| --- | --- | --- |
| question | at what point does the **ladder give up** rather than degrade again? | below what resolution is a frame **not worth looking at**? |
| number | 540p | 360p |
| refusal is triggered by | a *time* miss that one degrade step could not rescue | a *resolution* that loses the picture |
| evidence | [#87](https://github.com/MBehtemam/Montagent/issues/87)'s savings curve: `720p → 540p → 360p` buys under 1s combined at 4K, ~1.06s at 8K | [#117](https://github.com/MBehtemam/Montagent/issues/117)'s rendered legibility pass on the real fixture, unanimous 3-juror court |
| kind of argument | wall-clock | perceptual |

ADR-0065's is a **give-up point**: when a project's 8K render misses budget by more than
the ~0.71s that `720p → 540p` buys, nothing in the ladder can rescue it and `preview`
fails. That is a statement about time, and it is untouched by anything in ADR-0050.

ADR-0050's is a **refusal threshold**: below 360p the fixture's smallest on-screen text
(the 34–35px handle/chip badge and smallest caption line) stops being readable as
anything but a smear an agent can only "read" by already knowing the string. That is a
statement about pixels, and it is untouched by anything in ADR-0065.

## What governs

1. **The ladder is `720p` → `540p` → hard fail, unchanged.** ADR-0065's wall-clock
   argument against a third tier is not a legibility argument and survives ADR-0050
   completely: a `360p` degrade tier would buy under a second and is still not worth
   having. **No `360p` tier is added.**
2. **The long-edge cap stands.** ADR-0046 fixed the 720p target as *long edge capped at
   1280px, aspect preserved, rounded to even*. ADR-0065 re-derived the 720p target
   without citing ADR-0046 — it was written against a `main` that did not contain it —
   but states nothing inconsistent with the cap. ADR-0046's spelling is the operative
   one.
3. **360p is recorded as the legibility floor, and today it does not fire.** Under the
   ladder above, `preview` never renders below 540p, so ADR-0050's threshold is never
   reached by degradation. It is kept, not deleted, because it is a real measurement and
   it is the number that governs two live cases: any future extension of the ladder, and
   any caller-specified proxy resolution, should one ever be admitted. **Stating plainly
   that a recorded constant does not currently fire is the honest form** — deleting the
   measurement to make the document tidier would discard evidence that was expensive to
   produce and cheap to keep.

## ADR-0065's "unmeasured" clause is retired

ADR-0065 ships 540p on a **disclosed-not-certified** framing, resting on this:

> *540p is a wall-clock floor. Its legibility is explicitly unmeasured, and this ADR does
> not assert it is fine to read.*

**That was true when it was written and is false now.** ADR-0050 ran the pass. On the
real committed fixture, rendered at 720p/540p/360p/240p at four timestamps chosen to
stress its smallest declared text, three jurors independently located the break point
between **360p and 240p** — so 540p is two tiers clear of the measured failure, and 720p
three.

ADR-0065 did not merely fail to know this. It **named the condition and the condition was
already satisfied**:

> *Any tier below 540p (e.g. a future `360p`) requires an actual human-legibility pass
> before being added.*

ADR-0050 is that pass, accepted, with its prototype now committed. So ADR-0065's gate is
walked through rather than broken: the pass exists, and its result is that 540p is safe
and 360p is the real edge. What retires is the *hedge*, not the *floor* — 540p remains
where the ladder gives up, for the wall-clock reason, now without the caveat that its
legibility is unknown.

ADR-0021's mandatory disclosure of which tier was used is unaffected and still required
on every degraded preview. Disclosure was never only a hedge against uncertainty; it is
how a caller knows what it is looking at.

## Two limits, stated rather than smoothed over

**This certifies 540p for this corpus, not universally.** ADR-0050 recorded
content-dependence as a *finding* and deliberately did not adopt it as the mechanism:
legibility tracks the smallest on-screen text, not resolution alone. A project whose
captions are proportionally smaller than the fixture's reaches the risky range at a
higher tier — ADR-0065 makes exactly this point about #87's synthetic overlay sizing, and
it is correct. Nothing here promises 540p is legible for every project ever authored;
what is retired is the claim that **nothing had been measured at all**.

**The two measurements are not directly comparable, and are not being compared.**
ADR-0065's ~19–27px-at-360p figures are pixel heights inside #87's *synthetic* proxy
frame. ADR-0050 downscaled the *real* fixture and scaled it back to native for
equal-size viewing — the way a preview player shows a proxy in a fixed viewport — so its
judgment is about information loss rather than angular size. ADR-0050's setup is the one
that matches how a preview is actually read, which is why it is the pass being admitted.
Neither number is used to correct the other.

## Why no court

Every comparable decision in this series was settled by an independent multi-model court,
and this one was not. The reason is that **the two ADRs never produce different
behaviour**: under ADR-0065's ladder `preview` never renders below 540p, so ADR-0050's
threshold is unreachable, and there is no project, tier or invocation on which the two
disagree about what the tool does. What was in conflict was one sentence of ADR-0065's
*justification*, which ADR-0050's evidence settles directly.

Decided by the author on that analysis. **The risk this accepts is recorded rather than
hidden:** if the reading above is wrong — if some case exists where the two floors really
do select different behaviour — this ADR would be resolving a live conflict by assertion,
which is the failure mode ADR-0005 and ADR-0021 are both named for. A court remains the
correct remedy if such a case is found, and finding one falsifies this ADR rather than
merely refining it.

## How this happened, and what it costs

ADR-0065 was decided on 2026-09-18 against a `main` that did not contain ADR-0046 or
ADR-0050, both accepted and both sitting unmerged on `domain/*` branches. Nine ADRs were
in that state. `docs/agents/domain.md` predicted the cost in advance:

> *any agent that missed one read `main` and reasoned from a domain model several
> decisions stale*

This is the first case where it produced a contradiction on `main` rather than a merely
stale read, and the cost is concrete: a three-juror court spent a full round re-deriving
a 720p target ADR-0046 had already fixed, and reasoned carefully about an absence of
legibility evidence that was not absent. **The rule was already written; what was missing
was anything that enforced it.** All nine ADRs are now on `main`, ADR-0050's prototype is
committed alongside it, and `docs/adr/README.md` carries the supersession view that makes
this shape visible before an ADR is read rather than after.

## Consequences

- The ladder is `720p` (long edge capped at 1280px, per ADR-0046) → `540p` → hard fail.
  Unchanged from ADR-0065 in behaviour.
- `preview` additionally refuses any request to render below **360p**, naming the floor
  and the reason. Unreachable today; it is a guard on the ladder's future, not a live
  branch.
- ADR-0065's "legibility is explicitly unmeasured" clause and its deferral of a sub-540p
  pass are both retired. Its wall-clock reasoning, its rejection of a third tier, and
  ADR-0021's mandatory disclosure all stand.
- [#168](https://github.com/MBehtemam/Montagent/issues/168)'s user stories 61–63 are
  unblocked and should be read against this ADR.
- Nothing about `render` changes. Proxy degradation has never applied to it.
