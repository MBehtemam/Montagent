# Jury Verdict — Juror Report (Sonnet 5)

## Q1: Proxy target(s) and degradation ladder

**VERDICT: Accepted with modification.**

720p as the single fixed target is well-supported: it's the smallest target clearing &lt;5s at both sizes on skia-safe (2.68s/3.78s), while 1080p misses at 8K (5.79s, 16% over) — the numbers check out exactly as claimed. The "one degrade step, then hard floor" shape is also defensible given the savings curve: 4K's 720p→540p→360p combined buys under 1s, 8K's buys ~1.06s, versus native→first-clearing-target buying 78.6s at 8K. Diminishing returns genuinely support stopping the ladder early.

But the proposal is silent on a real risk FINDINGS flags explicitly: tiny-skia's 8K/720p margin is **1.8%** (4.91s vs 5s budget), and FINDINGS says outright this is "too thin to certify as safe without more samples" (single-run, no-repeats methodology, per its own limitations section). Declaring 720p settled without addressing this is premature. Modification: accept 720p/540p/floor shape, but require either (a) repeated-sample confirmation of the tiny-skia 8K/720p margin, or (b) an explicit fallback rule if tiny-skia is ever load-bearing in production (currently it isn't — skia-safe is per ADR-0009 — but the ADR should say so rather than let the thin margin go unmentioned).

## Q2: Is a legibility pass required before committing to a floor?

**VERDICT: Refuted.**

The proposal draws a line at 360p ("~19-27px, the *specific* configuration FINDINGS calls out") and declares 540p (~28-40px, "roughly double" 360p's) clear to ship as the hard floor without any legibility check. This is the weakest link in the draft. FINDINGS' own language is not scoped to 360p specifically — it states generally that "the *time* floor and the *visual* floor are different questions; only the time floor is answered here," and that "the visual floor is very likely the binding one in practice, but that needs a human-legibility pass this ticket didn't run." Nothing in FINDINGS establishes that 28-40px text is legible; the absence of an explicit callout for 540p is silence, not evidence. Treating "not called out as risky" as equivalent to "verified safe" is precisely the unmeasured-claim-as-fact move ADR-0021 was written to prevent (its own cited precedent: ADR-0005's frame-alignment mistake), and this ticket's Q2 asks this exact question almost verbatim.

Modification: 720p (default) can ship without a dedicated pass — its scaled text (~38-53px) is far from the flagged risk zone and the wall-clock case for it is unambiguous. But **540p as the hard floor requires a legibility check before being committed as such**, not deferred to "if 540p ever proves insufficient in practice." Until that check runs, either (a) ship 720p only, with no auto-degrade tier, and hard-fail below it, or (b) run a lightweight legibility spot-check on 540p now, before writing it into the ADR as the floor.

## Q3: Does the composite-stack scope gap need its own follow-up?

**VERDICT: Refuted.**

The proposal's analogy to ADR-0021's GPU-rasterization treatment doesn't hold. GPU was recorded as an *optional, never-promised* future lever, outside the degradation trigger's definition. "A heavy composite stack" is not optional scope — ADR-0021 explicitly names it, alongside 8K sources, as one of the two triggering conditions for degradation ("When even the proxy-resolution target can't meet the &lt;5s scrub-preview budget (an 8K source, **a heavy composite stack**)…"). #87 measured one clip/one still/one overlay and explicitly says multi-clip/multi-effect stacks "would raise the raster (not decode) side of the budget further and were out of scope." That means half of ADR-0021's own named trigger scenario is unverified by the ladder being adopted here. Recommend a targeted follow-up ticket (not merely a footnote) before calling the ladder general for the composite-stack case specifically.
