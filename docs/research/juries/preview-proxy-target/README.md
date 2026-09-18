# Jury: proxy target resolution, degradation ladder and floor

For [#158](https://github.com/MBehtemam/Montaget/issues/158), graduated from
[#87](https://github.com/MBehtemam/Montaget/issues/87), amending
[ADR-0021](../../../adr/0021-preview-budget-and-graceful-degradation.md), part of the
map [#2](https://github.com/MBehtemam/Montaget/issues/2).

Three independent jurors (Opus, Sonnet, Fable), each given the same brief (the
author's draft answer plus #87's `FINDINGS.md`), blind to each other, instructed to
default to "refuted" and attack the draft rather than assume it. Full ballots:
[`opus.md`](opus.md), [`sonnet.md`](sonnet.md), [`fable.md`](fable.md).

## Verdicts

**Q1 (target and ladder): unanimous accept-with-modification.** 720p as the single
fixed target, one degrade tier to 540p, hard fail below, all confirmed against
`FINDINGS.md`'s numbers. All three independently caught the same defect in the
draft: it cited tiny-skia's 8K/720p margin (4.91s, 1.8% under budget, single run) as
supporting evidence, when `FINDINGS.md` itself calls that margin "too thin to
certify as safe without more samples." **The `<5s` guarantee at 720p is a
skia-safe guarantee** (the load-bearing backend per ADR-0009/#34); tiny-skia is not
certified at 8K/720p. Fable adds a sharper framing worth keeping: 720p→540p only
buys 0.71s at 8K (decode-bound, not resample-bound), so a 720p miss larger than
that at 8K cannot be rescued by the ladder at all — it lands on the hard-fail floor
by design, not as an oversight.

**Q2 (legibility pass): 2-1 accept-with-modification, all three independently found
the same arithmetic error.** The draft's "540p text is roughly double 360p's, since
540p is 1.5x the linear scale" is internally contradictory — 1.5x is 1.5x, not 2x.
Corrected: `FINDINGS.md`'s ~19-27px at 360p becomes **~28-40px at 540p**, not
~38-54px. Opus and Fable accept shipping 540p as the floor without a dedicated
legibility pass, but only framed as a **wall-clock floor with the visual floor
explicitly unmeasured** — not an assertion that 540p is legible — leaning on
ADR-0021's existing mandatory-disclosure mechanism (every degraded tier is
disclosed to the caller, who can request full resolution). Sonnet refutes shipping
540p as a *certified* floor on the current evidence, holding the line at 720p only
until a legibility check runs. Resolution below adopts the 2-1 framing precisely
because it does not certify legibility — it discloses degradation and lets the
caller escape it, which is the mechanism ADR-0021 already built for exactly this
uncertainty.

**Q3 (composite-stack scope gap): 2-1 refuted as originally framed.** Opus and
Sonnet both reject the analogy to ADR-0021's GPU-rasterization treatment: GPU is an
unmeasured *upside* lever nobody promised; the composite-stack gap is unmeasured
risk against an *enforced* budget, and ADR-0021 already names "a heavy composite
stack" as one of exactly two triggering conditions for degradation. Half of that
named trigger is unverified by this ticket's harness (one clip, one still, one
overlay). Both want a real follow-up ticket filed now, not deferred until a stack
is shown to fail in the field. Fable accepts the footnote treatment. Majority
governs: a follow-up ticket is filed as part of landing this decision.

## Method note

The brief itself carried two errors a diligent reader could have caught blind
(the 1.5x/double contradiction, and citing tiny-skia's unsafe margin as support) —
all three jurors caught both independently, which is the working exercise, not a
coincidence: the same discipline this project's other juries have applied to
author-drafted "obvious" answers.
