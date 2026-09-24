# Chroma key: two courts

[ADR-0088](../../adr/0088-chroma-is-a-matte-operation-and-color-stays-literal.md)
characterises both of these courts, and decides against the first one's unanimous verdict.
They are committed verbatim so a reader can check that characterisation rather than take
it on trust — `docs/agents/domain.md` requires the artifact itself for a qualitative claim,
since there is no formula to re-run.

Every juror was a separate model call, given the same question text cold, blocked from the
other ballots and from the author's recommendation. Two jurors (Haiku, in both courts)
departed from the requested ballot format and returned prose; their text is reproduced as
received rather than reformatted.

A third court sat earlier, on whether to charter the effort at all and where the tickets
should live. It is not reproduced here because ADR-0088 makes no claim about it; its
ballots are in the history of
[#340](https://github.com/MBehtemam/Montagent/issues/340).

---

# Court 1 — the mechanism question

**Asked:** *What is the best mechanism for Montagent to support compositing a subject shot
on a green background?* Options: **(a)** an in-format `chroma` effect; **(b)** pre-keyed
alpha source only; **(c)** hybrid, sequenced; **(d)** something else.

Jurors were briefed on the ADR constraints, the shipped seven-member vocabulary, and the
measured decode facts. **They were not briefed on `CONTEXT.md`'s document-authority
premise** — the omission that ADR-0088 turns on, and the reason it decides against them.
The forcing case did not exist when they voted.

**Verdict: 3/3 against the in-format keyer.**

## Juror 1 (Claude Opus 5) — VOTE: (c) hybrid, sequenced

> **Reasoning:** The decisive fact in the brief is that the pipeline already decodes to `-pix_fmt rgba` and the renderer already composites unpremultiplied alpha correctly. That means transparency-carrying sources are plausibly *already* supported — this is not a feature to build but a capability to test, specify and make legible. ADR-0049's stopping rule bites hardest here: if a subject on transparency composites today by composing already-admitted machinery (a video source with alpha, plus `x/y/scale/opacity`), an eighth vocabulary member is refused by the format's own mechanical test. So the work of (b) is small and entirely in-format discipline: an empirical test that VP9-alpha and ProRes 4444 round-trip; a spec clause saying source alpha is honoured; a `validate` finding for a source whose container/codec cannot carry alpha where the document's composite implies one; a `measure` reading for alpha presence and opaque-region extents so `frame` isn't the only evidence. I vote (c) rather than bare (b) only because ADR-0003's asymmetry forbids treating the fixture's silence as proof keying is unneeded — (c) keeps the door explicitly unlocked and names the condition for opening it, which is honest where a flat (b) would pretend the question is settled.
>
> The static-parameter constraint is the strongest argument against (a) and it is not a detail — it is the whole shape of the problem. A keyer's tolerance is exactly the parameter that must drift: a cloud passes, the subject steps toward the cyc, the lighting shifts across a ten-second take. Under ADR-0012 the agent's only remedy is to cut the clip into N elements each with its own frozen tolerance, and then hand-tune N values by looking at `frame` outputs — and a visible seam appears at every cut where the tolerance steps. That is a worse artifact than either alternative and it *degrades* as the clip gets longer, which is precisely backwards. A pre-keyed asset has no such failure mode: the external tool that produced it had per-frame control, and the matte it baked is as good as its author could make it. Admitting a keyer whose central parameter is structurally un-animatable is admitting a tool that works well only on the easy cases — and the easy cases are exactly the ones a pre-keyed asset also handles.
>
> On (3), since my vote does not admit an in-format keyer, I owe only the boundary: if a future case forces (a), the minimal defensible surface is `{name, tolerance}` with the key colour *derived* — not authored — from the source (dominant background chroma at the frame edges, reported by `measure`), keeping every authored parameter a bounded scalar and leaving ADR-0049's closed exception genuinely closed. I would refuse `spill` in v1 (it is a second knob with no readable meaning and no way to verify independently) and refuse an authored hex `color` outright. An `{name, tolerance}`-only keyer that cites no colour parameter is the only version of (a) that does not reopen a door the ADRs deliberately nailed shut; a `chroma{color, tolerance, spill}` as written in the option is a straightforward ADR-0049 violation dressed as an amendment.
>
> On (4): ADR-0040's stated reason does **not** survive `frame` and `preview`, and the ADR should be amended to say so honestly rather than left standing on a premise the tool surface has falsified. An agent can now author, look, and revise — "not readable from the schema alone" is no longer true of anything, and if it were a valid exclusion it would also exclude `blur`, `shadow` and `saturation`, whose results equally depend on source pixels. But killing the stated reason does not resurrect the conclusion. The *surviving* reason to refuse chroma key is the one ADR-0040 did not articulate: a keyer's essential parameter cannot be animated, so the format cannot express a correct key for any shot whose background is not uniform in time. That is a real, structural, format-level objection, and it is the sentence ADR-0040 should carry going forward.
>
> **Trade-offs:** (b)/(c) pushes real work onto the agent's asset-acquisition step — the agent must obtain or produce a pre-keyed file, which the MCP surface does not do and which may be impossible if the user hands over a raw green-screen MP4 and nothing else. That is a genuine capability gap and I am not pretending otherwise; the honest answer in that case is `validate` telling the agent plainly that this source carries no alpha and must be keyed before use, which is at least a legible failure instead of a silent bad matte. (a) buys the raw-MP4 case at the cost of a second non-scalar parameter that ADR-0049 pre-emptively refused, an eighth vocabulary member that the stopping rule arguably excludes, and a tolerance that freezes for the clip's whole duration — three violations for a capability that works only on well-lit uniform screens. (d)'s general matte/alpha-source concept is the most architecturally interesting option and I would expect it to win eventually, but it needs the asset-reference-for-effect-inputs model that ADR-0040 already deferred as nonexistent; voting for it now is voting for a design that has not been done, and (b) is the correct first increment toward it regardless — an honoured source alpha channel is the narrow case of exactly that concept. The cost of (c) over flat (b) is process overhead: one more ADR clause stating the reopening condition. That is cheap, and it is the clause that keeps ADR-0003's asymmetry from being quietly violated.

## Juror 2 (Claude Haiku 4.5) — VOTE: (b) pre-keyed alpha source only

*Returned as prose rather than the requested ballot block.*

> REASONING:
> Option (b) is the only mechanism that honors Montagent's core design constraints without precedent damage. The renderer already composites alpha correctly; ffmpeg decodes to RGBA; some codecs can carry alpha natively. This means green-screen footage can enter with transparency already resolved, requiring only a spec clause, a validate finding for alpha presence, and a measure reading. No vocabulary extension, no parameter rule violation, no closed exception to reopen.
>
> GREEN-SCREEN KEYING IS GENUINELY DIFFICULT IN A DETERMINISTIC, DOCUMENT-CENTRIC FORMAT because it is pixel-sensitive and lighting-dependent. Moving it to asset preparation transforms this from an unsolvable constraint into a solvable workflow one.
>
> TRADE-OFFS:
>
> (a) violates ADR-0049's parameter rule—it requires color as a second non-scalar parameter, reopening an exception explicitly marked closed. Worse: it surfaces the static-parameter limitation painfully. Lighting drifts mid-shot? You must split the element and re-enter tolerance twice, but the parameter itself still won't keyframe. You've added friction without gaining expressiveness.
>
> (c) is temporizing. Once (a) ships with its parameter-rule violation, the precedent is set; (b) alone is clean.
>
> (d) is speculative. No greenscreen content exists in the fixture (ADR-0003: absence is not evidence of lack, but presence is evidence of need). A general matte-source or image-source mask requires an asset-reference model the brief already notes doesn't exist.
>
> ON THE FOUR REQUIRED POINTS:
>
> 1. Static parameters: (b) sidesteps the issue entirely—keying happened before import, so no tolerance exists in the document to keyframe. Lighting changes require clip splits, but that's already true for any shot boundary.
>
> 2. Parameter surface: Not applicable; no new parameters enter the effects vocabulary.
>
> 3. ADR-0049 reconciliation: Not required. No color parameter admitted.
>
> 4. ADR-0040 survival: Yes, completely. "Not readable from the schema alone" remains: you must know the source carries alpha. But this is verifiable (check presence) not predictive (compute which pixels to discard). The frame and preview verbs are verification tools, not schema generators. They let an agent confirm authorship, not predict pixel results from parameters alone. The document must remain self-contained and reproducible.

## Juror 3 (Claude Fable 5.1) — VOTE: (c) hybrid, sequenced; a hue/tolerance pair if (a) is ever admitted

> **Reasoning:**
>
> 1. **Mechanism.** (b) is the only candidate that is *already almost true*. Decoding requests `-pix_fmt rgba`, the canvas reasons about per-source alpha presence, and compositing is premultiplication-correct. A pre-keyed VP9/ProRes 4444 source plausibly composites today with zero format change. That is not a design proposal; it is an untested fact waiting for a fixture. The correct first act is therefore empirical, not legislative: record an alpha-carrying clip, run `frame`, and either confirm (b) or find the bug. Everything the ADR discipline demands for (b) is small and honest: a spec clause naming alpha-bearing containers as supported, a `validate` finding when a source is declared/used as a cutout but carries no alpha (mirroring the existing JPEG reasoning), and a `measure` reading for alpha coverage/extents so an agent can *read* where the subject is. None of this touches ADR-0040; the keying decision moves to the tool that has the frames, and the document carries only the result. That is exactly the boundary Montagent already drew for LUTs.
>
>    I pick (c) over pure (b) because ADR-0003 forbids me from concluding "unneeded" from the fixture's silence, and because the pipeline's untested claim may fail (ffmpeg's alpha handling for VP9 requires `-c:v libvpx-vp9` on decode, and ProRes 4444 alpha can be dropped by default pixel format negotiation). (c) keeps the door open on evidence rather than on absence.
>
> 2. **The static-parameter constraint is decisive against (a) as the primary path.** A one-value-per-element `tolerance` is the wrong shape for chroma keying, because keying is the *one* colour operation whose correct parameter drifts with source lighting on every real shoot. Under ADR-0012 the only remedy is cutting the clip into elements at drift boundaries, each with its own `tolerance` — and the agent can only discover those boundaries by iterating `frame` across the timeline, then splitting, then re-checking. That is a manual keyframe curve smuggled through the element list, and it produces a hard step in matte quality at every cut. `blur{radius}` does not suffer this because blur is source-independent; chroma is the counterexample that shows why "effects are static" and "chroma is an effect" cannot both hold cleanly. (b) has no such problem: the upstream keyer had the frames and could animate whatever it liked; the document sees only alpha.
>
> 3. **If (a) is ever admitted, the minimal defensible surface** is `{"name":"chroma","hue":120,"tolerance":0.35,"spill":0.2}` — three bounded scalars: `hue` in degrees [0,360), `tolerance` in [0,1], `spill` in [0,1] — and nothing else (no `softness`, no `edge`, no `despill_color`). Deliberately *not* `color`. A screen colour is a hue with a nominal saturation/luma that the source never matches anyway; encoding it as `#00FF00` implies a precision the keyer does not honour. This reconciles ADR-0049 without reopening the `tint` exception: no non-scalar parameter is requested, so the closed exception is not cited. If the drafters insist a hue integer is a colour in disguise, the honest answer is that ADR-0049's rule is about *bounded scalars*, and a degree on a circle is one; `tint.color` is grandfathered precisely because RGB is three coupled unbounded-in-meaning channels, which `hue` is not.
>
> 4. **ADR-0040's stated reason does not survive on its own terms, but its conclusion does on better ones.** "Not readable from the schema alone" is false as a distinguishing test the moment `frame` and `preview` exist: `blur{radius: 12}` is also not predictable from the schema — no agent knows what a 12px blur does to *this* frame without looking, and the tools exist so it can look. If unpredictability-from-schema excluded chroma, it should have excluded blur, shadow, and every saturation adjustment. So the ADR-0040 sentence should be rewritten, not merely upheld. The reason that *does* survive is the one in point 2 plus the ADR-0049 composition rule in a different guise: chroma is the only proposed effect whose output quality is a function of a source-dependent, time-varying parameter that the format's static-parameter rule cannot express, and whose upstream alternative (a pre-keyed source) is a standard deliverable already handled by the decoder. That is a real, mechanical reason to defer — and it is falsifiable by a single failing case, which is why (c) not (b).
>
> **Trade-offs:**
>
> - (c) costs the agent the ability to key inside Montagent. An agent handed a raw greenscreen MP4 must invoke an external keyer first; the project's "one JSON, one tool surface" story gets a footnote. That is a real loss and I would state it in the ADR rather than pretend pre-keyed sources are universal.
> - (b) alone is refused because it lets the fixture's silence do work ADR-0003 says it may not do; (c) is (b) plus an explicit re-entry condition.
> - (a) alone is refused because it admits the format's first source-dependent, drift-sensitive parameter under a rule that forbids animating it, and because as drafted (`color`) it needs the exact exception ADR-0049 closed. The hue-scalar version is defensible but only *after* someone shows a pre-keyed workflow failing — otherwise it is vocabulary admitted on speculation, which ADR-0049's stopping rule exists to prevent.
> - (d) (a general `matte`/image-mask source concept) is the most principled long-term answer, but ADR-0040 already correctly names its blocker: no asset-reference model for effect inputs exists. Building that model to serve greenscreen would be the tail wagging the dog; it should arrive when a second consumer (image mask, luma matte) needs it, and (b) does not need it at all.
> - Risk in my vote: the "plausibly composites today" claim is untested. If the ffmpeg path silently drops alpha, (b)'s cost rises from "spec clause + two findings" to "decoder work", and the sequencing argument weakens but does not invert — fixing the decoder is still cheaper and more general than admitting a new effect.

---

# Court 2 — review of ADR-0088 in draft

**Asked:** *Is ADR-0088 right or wrong?* Jurors were given read-only repository access and
asked to verify the draft's claims rather than accept them, across six axes: the ADR-0049
scoping argument, `color` vs `hue`, the static-parameter boundary, the finding codes and
readings, factual errors, and evidence discipline.

**Verdict: 3 REVISE, 1 ACCEPT — unanimous that the decision is right, majority that the
document was not.** Every defect named was repaired before this file was committed; the
repairs are listed in ADR-0088's own court section.

The two most consequential findings: Jurors 1 and 2 **independently** caught that `spill`
contradicted the draft's central "RGB is untouched" claim, and Juror 4 caught three factual
errors — a false claim that `frame`/`preview` postdated ADR-0040, a `spill` value that was
actually ffmpeg's `blend` parameter (losing edge softness from the vocabulary), and a hue
measurement run against a saturated literal rather than a hue keyer.

## Juror 1 (Claude Opus 5) — VOTE: REVISE

> **the framing is right and the conclusion is right, but the central legal claim is stated in a form its own parameter set falsifies: `spill` changes pixel colour.**
>
> **Reasoning:**
>
> **1. The legal argument — sound in structure, overstated in text.** I verified ADR-0049 verbatim. The stopping rule does open *"A **colour operation** is admissible in v1 only if…"* and the exception does close *"a future **colour-op** proposal does not get the same allowance."* Both clauses are scoped in their own words. The `shadow.color` corroboration is real and stronger than the draft claims: `effects.rs:46-52` carries `Shadow { dx, dy, radius, color: Colour, opacity }`, it predates ADR-0049 (shipped in ADR-0040), and ADR-0049 says `tint.color` is *"the **sole** grandfathered exception"* while never mentioning `shadow` — which is only coherent if the rule was scoped. The code's own doc-comment on `Tint` independently glosses the exception as closed against *"a future **colour operation**."* And CONTEXT.md:255 defines "Colour filter" as *"The four scalar `effects` members that change pixel colour rather than geometry"* — the glossary already treats the colour family as a named sub-family, not as "effects." That is a free, uncited corroboration the draft should pick up. This is not a loophole; it is the text.
>
> **But the draft breaks its own rule.** It asserts: *"A chroma keyer does not change pixel colour… the RGB it leaves behind is the RGB it was given."* Its own third parameter is `spill` — *"suppression of the screen colour reflected onto the subject"* — which by definition rewrites the RGB of **retained, fully opaque** pixels. The member as specified is a matte operation **and** a colour operation. The argument survives the repair easily (`spill` is a bounded scalar with documented identity `0`, so it satisfies clauses (a)–(d) on its own; at `spill: 0` the RGB is untouched and the member is pure matte), but the ADR must say that instead of the sentence it currently says. As written, the one paragraph the whole admission rests on is refuted by reading the JSON three lines above it. **This is the blocking revision.**
>
> Related: "matte" is load-bearing here and appears nowhere in CONTEXT.md. Per docs/agents/domain.md, an argument turning on a distinction the glossary does not carry is a signal; the matte/colour-op boundary should land in CONTEXT.md, not only in this ADR.
>
> **2. `color` vs `hue` — conclusion right, reasoning one step short.** The numbers hang together. `(0,205,0)` is `#00CD00` and is hue 120; its BT.601 chroma-plane distance from `#00FF00` normalised to 255 is ≈0.105, which brackets exactly where the table flips (0.10 keys 0.10%, 0.15 keys 89.37%). The measurement is internally consistent and the silent-nothing failure mode is the right thing to decide on. The gap is the premise: *"a hue angle… must be the fully-saturated colour at that angle"* is a design choice, not a necessity — a hue-only keyer that ignores saturation entirely is the other coherent spelling and was not measured. It would lose too (hue is undefined at low saturation, so it keys dark green subject shadows and near-grey pixels), but the ADR should say that in a sentence rather than leave the strongest form of the rejected alternative untested. Conclusion stands; the argument is narrower than it reads.
>
> **3. Static-parameter boundary — honest, not waved away.** ADR-0040 already carries *"Animated effect parameters — v1 effects are static"* as a stated boundary marker, so this is the format's existing idiom, not an invention. The ADR names the mechanism, the remedy, the cost of the remedy, credits the prior court for it, and files "Real shot footage" under *What is still not measured*. That is the opposite of waving. I would not call it fatal: a keyer that serves uniform screens is a capability, and refusing it entirely serves nobody. **One thing is missing** — the author who hits drift has no instrument, and the `measure` coverage derivation is the obvious one, which brings me to (4).
>
> **4. Finding codes — right surface, ADR-0006 respected, `measure` under-specified.** Both findings pass ADR-0061's threshold-provenance test (the real fence): `R-CHROMA-AFTER-COLOUR` is purely structural, `R-CHROMA-ON-ALPHA-SOURCE` derives from `Probe::alpha` (verified: `media/probe.rs:91`), neither borrows an external number, both are `review` and neither proposes a repair. No ADR-0006 violation — the ADR is explicitly right that a check which had to key frames would be the *"does it say what you meant"* verb. The `#339` caveat is accurate: `pix_fmt_has_alpha("yuv420p")` is `false` for a VP9-alpha WebM, per `docs/research/alpha-decode/FINDINGS.md` finding 3.
>
> **The defect is `measure`.** "opaque fraction, transparent fraction, and the fraction in between" is specified over **no stated temporal sample**. One frame? Which one? Every frame? The draft's own 89.25%/89.44% figures come from some unstated sampling. This matters twice over: it is the reading the ADR says *"is the mechanism that makes it readable,"* and a per-frame coverage series is precisely the drift detector the static-parameter boundary in (3) leaves the author without. Specify it, and (3) gets cheaper at the same time.
>
> **5. Factual errors — none found besides the `spill` contradiction.** I checked and confirmed verbatim: ADR-0040's chroma sentence and its out-of-scope list; ADR-0049's stopping-rule and exception wording; ADR-0006's *"internally legal, and does it agree with the media on disk"* (line 144); ADR-0014's `#RRGGBB`/`#RRGGBBAA`-only rule and its uppercase-hex clause; ADR-0024's no-verdict shape; ADR-0084's element-local/rides-the-transform rule; seven current members in `effects.rs`; `Shadow.color` present and pre-0049; CONTEXT.md's opening line quoted exactly; `#338`/`#339`/`#340`/`#342` open with the titles described; `docs/research/alpha-decode/` present with `alpha_decode_scan.sh`. The `blur{radius: 12}` parity argument against ADR-0040's "not readable from the schema alone" is correct and, in my view, decisive.
>
> **6. Evidence discipline — `status: proposed` is the right call and the list is nearly complete.** domain.md is explicit that a numeric claim needs a re-executable script, that *"a transcript of a number is not enough,"* and that the check happens *before* `status: accepted`. Every number here is a transcript, so `proposed` is correct and the ADR says so itself. Item 2 is right (`docs/research/juries/` has 26 precedents; arguing against a court without committing it is the one thing this repo's practice forbids). Item 3 is enforced by `check_amendment_banners.py`, which I read — it checks the `amends:` front matter, the banner region, and README.md's *Amended by* column, so all three surfaces are genuinely owed. **Two additions:** the CONTEXT.md glossary rows (the vocabulary list at line 239-244 and the "Colour filter" definition at 255) are not in the list, and the matte-vs-colour-operation distinction belongs there since the admission now rests on it.
>
> **What I would not change:** the decision to overturn ADR-0040 and admit the keyer, the literal `#RRGGBB`, the two finding codes, the `proposed` status, and the section recording the opposing court. Unanimous opposition from independent models is evidence, and this ADR does the one thing that makes overriding it legitimate — it adopts their strongest argument as a boundary rather than burying it, and it measured the alternative one of them proposed instead of dismissing it.
>
> **Trade-offs:**
>
> What my REVISE costs: three paragraphs of rewriting on an ADR whose conclusion I agree with, which delays #342. If the author judges the `spill` sentence to be loose phrasing rather than a load-bearing claim, ACCEPT-with-edits is defensible and I would not fight it hard — but I do not think it is loose phrasing, because that sentence *is* the jurisdictional argument. If a later reader checks whether chroma was legitimately admitted, that sentence is what they will check, and as written it fails against the JSON example on the same page.
>
> What I am least sure of: **(a)** whether the matte/colour distinction is a real ontological line or a serviceable one. I believe it holds on the text and CONTEXT.md's glossary backs it, but I am aware I am a juror being asked to overturn my own prior vote, and the distinction is exactly the kind of thing that reads as principled when it yields the answer you wanted. The honest defence is that `shadow.color` predates the argument by forty ADRs and nobody invented it for this — but I hold this at maybe 80%, not 95%. **(b)** The hue comparison. I verified the numbers are arithmetically consistent, but I have not re-run anything — nothing here is re-executable yet, which is the ADR's own item 1, and my confidence in the `#00FF00` column is confidence in a transcript. **(c)** I have not seen the prior court's ballots (they are not committed), so my assessment of *"two of their claims were over-read"* rests entirely on this ADR's characterisation of them. That is precisely the gap item 2 closes, and it is why I would not accept this ADR before those ballots land regardless of the `spill` question.

## Juror 2 (Claude Sonnet 5) — VOTE: REVISE

> **the matte/colour distinction is sound in the abstract but the ADR's own `spill` parameter contradicts its central premise, and evidence discipline is otherwise correct**
>
> **Reasoning:**
> 1. *Central legal argument*: Verified against ADR-0049's actual text — it does open "A **colour operation** is admissible..." and close "...a future **colour-op** proposal does not get the same allowance" (both scoped exactly as ADR-0088 quotes). ADR-0040 (accepted, unamended on this point) shipped `shadow{dx,dy,radius,color,opacity}` before ADR-0049 existed, confirmed by `model/effects.rs`'s `Shadow { color: Colour, ... }`. So the "shadow predates and sits outside the colour-filter family" claim is factually correct, not a fabricated loophole — it's a real textual scoping the prior court's brief evidently didn't surface. This part is sound.
>    However: the ADR's own decisive sentence — *"the RGB it leaves behind is the RGB it was given"* — is contradicted by the very member it defines two sections later: `spill` "suppression of the screen colour reflected onto the subject." Despill is by definition a colour correction on subject pixels, not an alpha computation. The ADR never reconciles this; it states the pure-matte premise, then ships a parameter that falsifies it for any nonzero `spill`. This is a real internal inconsistency in the load-bearing argument, not a nitpick — if `chroma` includes despill, it partially **is** a colour operation, and the "it never needed ADR-0049's exception" claim needs qualifying (or `spill` needs its own justification under ADR-0049 rather than riding on the matte argument).
> 2. *`color` vs `hue`*: Sound and honestly measured. The table shows a real, non-obvious failure mode (a tolerance correct for the true colour silently keys nothing under the fully-saturated hue spelling) rather than a preference dressed as measurement. Choosing literal colour over the ticket's own proposed `hue` alternative, on evidence, is good practice.
> 3. *Static-parameter boundary*: Stating it as a boundary rather than a blocking defect is honest disclosure, not evasion — it's written where a reader will find it, with the CGI forcing case's own limits stated (uniform-by-construction, zero real-footage evidence). But this is a judgment call, not a factual matter: the prior court's real objection was that most real-world green-screen footage isn't uniform, so the "boundary" may cover a minority of real use, and the ADR doesn't quantify that gap — it only says footage will drift and a cut is the remedy. Not wrong, but the ADR asserts survivability of the trade more confidently than its (CGI-only) evidence supports.
> 4. *Findings/measure surface*: Correctly scoped under ADR-0006. Both findings are structural/cheap facts, not verdicts; `R-CHROMA-ON-ALPHA-SOURCE` correctly reuses the existing `Probe::alpha` field (confirmed present in code) and correctly flags its own dependency on #339 (VP9 alpha detection, confirmed as an open, correctly-described issue). The `measure` derivation matches ADR-0024's bare-derivation-no-verdict shape. No boundary violation found.
> 5. *Factual errors*: None found beyond the spill/RGB self-contradiction above. Quotes from ADR-0049, ADR-0040, and issue #339/#340 all check out verbatim or in substance against the source files.
> 6. *Evidence discipline*: Correct per `docs/agents/domain.md`, which does require a re-executable script for numeric claims before acceptance — `status: proposed` is the right call, not a formality, and the three owed items (re-executable check, committed ballots at `docs/research/juries/`, amendment banners enforced by `check_amendment_banners.py`) match real repo precedent. Nothing obviously missing from that list, though I'd add a fourth: reconciling the spill/RGB contradiction above before acceptance, since it's a hole in the argument itself rather than in its evidentiary backing.
>
> **Trade-offs:** My verdict costs the ADR a clean ACCEPT-track path even though the harder legal move (the ADR-0049 scoping argument) holds up under direct text comparison — I'm not rejecting the framing, only withholding acceptance until the internal contradiction is fixed (either drop/rename `spill` as out of scope for this ADR, or admit `chroma` is partially a colour operation and address ADR-0049 on that basis too). I'm least sure about the real-world materiality of the static-boundary trade-off (point 3) — I can't independently test how much of realistic green-screen footage is "uniform enough," so I'm deferring to the ADR's own honesty about the gap rather than calling it fatal.

## Juror 3 (Claude Haiku 4.5) — VOTE: ACCEPT

*Returned as prose rather than the requested ballot block. Note that this ballot restates
the draft's `spill`-contradicting sentence as its own reasoning — "leaving RGB untouched" —
without testing it against the parameter list, and reports no factual errors where Juror 4
found three. It is reproduced in full regardless; a court records what its jurors said.*

> **Reasoning:**
>
> 1. **The central legal argument is RIGHT.** ADR-0049's stopping rule and `tint.color` exception are explicitly scoped *per their own text* to "colour operation(s)" only. A chroma keyer computes alpha matte (transparency), not pixel colour—leaving RGB untouched makes it categorically a matte operation, not a colour operation. The decisive proof is correct: `shadow{dx, dy, radius, color, opacity}` predates ADR-0049 (in v1 from ADR-0040) and carries a non-scalar `color` outside the colour-filter family. This establishes that colour-typed parameters existed before the stopping rule; the rule never governed them.
>
> 2. **The `color` vs `hue` reasoning is RIGHT.** The measurements are sound: literal `#00CD00` works from tolerance 0.05, but fully-saturated keying at that hue requires tolerance ≥0.15. At tolerance 0.10—a reasonable middle ground—the hue spelling produces 0.10% keying vs. 89.44% with literal, which is the critical silent-failure problem. Using literal color preserves saturation information already present in the source.
>
> 3. **The static-parameter boundary is HONESTLY STATED.** The ADR frames "serves screens uniform in time" as a stated boundary, not a hidden defect or a dismissed objection. Explicitly: *"On the forcing case this costs nothing…Footage shot on a real cyc under real lights will drift, and there the boundary bites."* This is transparent scoping. The forcing case (CGI greenscreen, uniform by construction) does not test drift, but the ADR acknowledges this as a real limitation rather than claiming it doesn't exist.
>
> 4. **The finding codes and readings are SOUND.** Both `R-CHROMA-AFTER-COLOUR` (structural, free) and `R-CHROMA-ON-ALPHA-SOURCE` (one ffprobe call) are review-class facts, fitting ADR-0006's boundary. Neither proposes repair. The `measure` keyed-alpha derivation is a bare derivation with no verdict—exactly ADR-0024's shape. The note about VP9 probe bug (#339) as prerequisite is appropriate boundary-setting.
>
> 5. **No factual errors found.** ADR-0040's stated reason ("result depends on source pixels, not readable from schema alone") is correctly identified as falsified—blur equally depends on source pixels and wouldn't be readable without `frame`/`preview`. The interpretation of ADR-0049's scope is accurate. The measurements are clearly marked as transcripts, not re-executable checks (status: proposed).
>
> 6. **Evidence discipline is CORRECT.** Three owed items before `accepted`: (a) re-executable numeric check—required by domain.md for measured claims, (b) court ballots at BALLOTS.md—required by precedent (colour-filter holds them), (c) amendment banners—required by check_amendment_banners.py. All appropriate.
>
> **Trade-offs:** The prior unanimous court (Opus, Haiku, Fable) carries weight and was right about the drift mechanism, but they lacked: (1) the forcing case with measurement, (2) the document-authority premise from CONTEXT.md, (3) re-reading of ADR-0049's explicit scope. The forcing case does not exercise the stated static-parameter limitation; real-world cyc footage under real lights remains unmeasured. The decision trades usability (literal color's 0.05 tolerance vs. hue's 0.15) against keeping ADR-0012 settled. That is a trade, not an error.

## Juror 4 (Claude Fable 5.1) — VOTE: REVISE

> **the admission and the ADR-0049 scoping hold; the `hue` rejection rests on a strawman measurement that is not on #340, and two historical claims are false.**
>
> **Reasoning:**
>
> 1. *Central legal argument — sound, but over-sold.* ADR-0049's text really does open *"A **colour operation** is admissible in v1 only if…"* and close its exception with *"a future **colour-op** proposal"*; and `shadow{…, color, opacity}` really is in ADR-0040's Consequences, before 0049, with a `Colour` field in `crates/montagent-core/src/model/effects.rs`. So 0049 never spoke to every colour-typed parameter, and the draft's scoping is a fair reading, not a loophole. The render code corroborates the category: `crates/montagent-render/src/canvas.rs` keeps the alpha row identity in every colour matrix (a chroma keyer is the inverse — alpha changes, RGB stays). Two caveats the draft glosses. `shadow.color` is a *paint* for authored pixels, not a *key* — it proves 0049 wasn't global, not that a colour key is like a shadow. And 0049's last clause (*"a proposal wanting an arbitrary-colour parameter needs its own stated justification"*) says *"a proposal"*, unscoped; the draft concedes this and points to the measurement for the justification — which brings us to point 2.
>
> 2. *`color` vs `hue` — the reasoning is unsound; the conclusion may still be right.* The "hue" column was produced by running `chromakey=0x00FF00` — a Euclidean-distance key on a saturated literal, not a hue-angle key. ffmpeg's `hsvkey` (a true angular-hue keyer, installed on this machine) would key `(0,205,0)` at hue 120 regardless of saturation; the premise *"the key point it denotes must be the fully-saturated colour"* is an implementation choice presented as a property of hue. Worse, **the `#00FF00` table is not on #340**: the ticket has two comments, neither contains `00FF00`, `0.15`, or a saturated-vs-literal comparison. The draft's *"recorded on #340"* is true only of the base plateau table. The real argument for `color` — a hue key discards saturation and so keys every green including greens on the subject, while the literal discriminates — is one sentence in the draft and is what should carry the section.
>
> 3. *Static-parameter boundary — honest.* ADR-0040 already listed "Animated effect parameters" as a boundary marker for *every* effect, so chroma is not asking for an exception. The draft keeps the pre-keyed path un-deprecated as the route for drifting footage and admits *"the drift boundary is reasoned, not observed"*. That answers the jurors' "easy cases only" point squarely: the easy case is where document authority is bought cheaply. Stating it is right. The one soft spot: *"on the forcing case this costs nothing"* is tautological on CGI with no drift.
>
> 4. *Findings and readings — right surface, two defects, one gap.* Both `R-` findings are facts derivable from document plus probe and never render, so ADR-0006's boundary holds. But `R-CHROMA-ON-ALPHA-SOURCE`'s dependency on #339 is inverted: `Probe::alpha` (`crates/montagent-core/src/media/probe.rs:536`) from `pix_fmt` gives *false negatives* for VP9, so the check is trustworthy when it fires and merely incomplete when it doesn't — #339 is a coverage prerequisite, not a trust one. The `measure` derivation never says *at which instant* the alpha coverage is computed for a video element; that is the load-bearing reading and it is under-specified. Missing: chroma on a `text`/`rect`/`ellipse` element (almost always a mistake, structural, free), and `tolerance: 0` as an inert declaration in the `R-EASE-INERT` (ADR-0052) mould.
>
> 5. *Factual errors.* (a) *"That verb [`frame`], and `preview`, did not exist when ADR-0040 was written"* is false: ADR-0011's verb table lists `frame` ("what it looks like") and ADR-0021 names both `frame` and `preview` — both precede 0040. ADR-0040's author refused chroma *with* `frame` in hand, which weakens the "tool surface has since disproved it" narrative (the blur-symmetry argument survives on its own). (b) The example `"spill": 0.08` takes the third positional of `chromakey=0x00CD00:0.10:0.08`, which is ffmpeg's `blend` (edge softness), not despill; the measured despill was `mix=0.5`, and #340's own example says `spill: 0.5`. The parameter surface was not derived from the command that produced the numbers, and edge softness has silently dropped out of the vocabulary. (c) The `#00FF00` provenance, per point 2.
>
> 6. *Evidence discipline.* `proposed` is the right call under `docs/agents/domain.md`. The owed list is incomplete: the hue comparison must be re-measured with a real hue keyer or the argument rewritten; the fixture question (a 1920x1080 h264 clip, or a synthetic equivalent) needs sizing; and the draft's own qualitative claims about the prior court (their three points, and which juror proposed `hue`) cannot be checked until the ballots are committed — item 2 is listed but is doing more work than the draft admits.
>
> **Trade-offs:** REVISE rather than ACCEPT costs a delay on a decision I think is correct in outcome — a keyer in the document beats a key in shell history, and the jurors before me were not given that premise. REVISE rather than REJECT is where I am least sure: an ADR that argues against a unanimous court, cites a measurement that is not where it says it is, and mis-dates the tool surface, has spent credibility it needed for exactly this fight. I keep it at REVISE because every defect is repairable without changing the decision, and because the `proposed` status is doing its job.

---

## What the courts changed

Court 1 supplied the **static-parameter boundary** that ADR-0088 adopts as the replacement
for ADR-0040's falsified reason, and the **`hue` proposal** that ADR-0088 measured and
rejected. Its unanimous verdict was overridden on a premise it was never given.

Court 2 caught seven defects in the draft, every one of which was repaired: the `spill`/RGB
contradiction, the false `frame`/`preview` history, the strawman hue measurement, `softness`
lost from the parameter surface, `measure`'s unstated temporal sample, the inverted #339
dependency, and two missing findings. The decision did not change; the document did.
