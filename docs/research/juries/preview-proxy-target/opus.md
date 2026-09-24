VERDICT REPORT — issue #158 (juror: Opus 5)

**Q1. Proxy target and ladder — ACCEPTED WITH MODIFICATION.**

720p as the single fixed target survives attack on the load-bearing backend: skia-safe clears with real margin at both sizes (2.68 s / 3.78 s, 24 % under budget at 8K), and 1080p's 8K miss (5.79 s, 16 % over) is decisive against the "obvious" number. No cheaper single target is needed, and a scale factor was already ruled out by ADR-0021.

But the draft's parenthetical "(thinly) on tiny-skia" launders a caveat FINDINGS states flatly: at 8K/720p tiny-skia is 4.91 s, "too thin to certify as safe without more samples," on a **single run, one machine, no repeats**. The draft cites the 1.8 % margin as support; FINDINGS cites it as a refusal. Required modification: the ADR must adopt 720p **on skia-safe's numbers, per ADR-0009's load-bearing-backend rule**, and state explicitly that 8K/720p on tiny-skia is *not* certified within budget — runtime degradation, triggered by measured miss, is the mechanism that covers it. Do not write the tiny-skia margin as a supporting datum.

Ladder shape (720p → 540p → hard fail) is otherwise sound: 720p→540p→360p combined buy 1.06 s at 8K and &lt;1 s at 4K, so tiers past the first are not load-bearing.

**Q2. Legibility pass — ACCEPTED WITH MODIFICATION (the draft's arithmetic is wrong).**

The draft claims 540p text is "roughly double 360p's, since 540p is 1.5x the linear scale." Those cannot both be true. 540/360 = 1.5 exactly (4K fractions 0.250000 vs 0.166667). FINDINGS' ~19–27 px at 360p becomes **~29–40 px at 540p, not ~38–54 px**. The conclusion (540p is outside the flagged-risky configuration) still holds, but on half the margin the draft asserts. Fix the sentence or it becomes the next ADR-0005.

Deeper defect, and the one I'd insist on: FINDINGS' px figures are properties of *its fixture's* overlay (57–80 px at 1080p-equivalent), not of Montagent projects generally. A project whose captions are half that relative size reaches 19–27 px at **540p**, i.e. the flagged-risky regime, inside the shipped ladder. So "no legibility pass needed for 720p/540p" is only defensible as *fixture-conditional*. Record it that way, and record the 540p floor as **provisional pending a legibility pass**, not settled. Asserting 360p is illegible is as unmeasured as asserting it is legible; the honest framing is conservative default, not established visual floor. With that framing the draft's "legibility pass required before any 360p tier" is right and should stay.

**Q3. Heavy composite stack — REFUTED as framed; accept the outcome, reject the rationale.**

The GPU analogy is invalid and should be struck. GPU rasterization is an unmeasured **upside**: not measuring it can only mean the system is faster than promised. The heavy composite stack is an unmeasured **downside** against an *enforced* number, and ADR-0021 names it explicitly as a degradation trigger — FINDINGS says multiple clips/effects raise the raster side further. Recording a downside risk to an enforced budget by citing the precedent set for an upside lever misrepresents the precedent.

It does not block the ladder — runtime degradation is measured, so a heavy stack degrades rather than lies. But "open for a future ticket if ever shown to blow 540p" is wrong: the failure mode it guards is the **hard-fail floor firing on real projects**, which is user-visible. File the follow-up ticket now, unconditionally, and make it a prerequisite for calling the ladder general.
