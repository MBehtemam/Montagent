# Ballots

Four jurors, four models, each blind to the others, each pinned to commit `aef1cdcf` and
blocked from the implementing agent's branch, PR and issue comments. Reproduced verbatim
as delivered. See [`README.md`](README.md) for the synthesis and for which claims were
verified by script afterwards.

**The six questions, identical for all four** (the adversarial juror additionally had to
predict the consensus and argue against it before giving its own verdict):

1. A file carries `"box": [x, y, w, h]` on an image element. Under ADR-0043 **as
   written**, is the finding refuse-class or advise-class? If advise, state the exact
   repair value the check should emit.
2. Same for `"box": "card-05"` — ADR-0007's containment reference.
3. Same for `"align": "top"` on an image element.
4. Same for `"bold": true` on a text run. If advise, state the exact repair value.
5. ADR-0043 offers exactly two classes. Do your answers to 1–4 sort cleanly into that
   binary, or does the binary have a gap? If it has a gap, state the smallest amendment to
   ADR-0043 that closes it, and what that amendment costs.
6. ADR-0068 contains the clause "`box` was migrated by arithmetic script". Is that a
   classification of `box` under ADR-0043, or a historical remark about the repo's
   migration script? Which, and how do you know?

---

## Juror A — Opus, briefed adversarial ("default to refuted")

**Contamination note:** I read only ADRs 0007, 0012, 0014, 0016, 0043, 0068 and
`migrate.py`/the fixture at `aef1cdcf`. I did not open `crates/montaget-core/src/checks/`,
#192, #228, #229, or the working tree. I saw no other agent's argument.

**Premise defects found first (these outrank my answers):**

1. **ADR-0043 is internally inconsistent on its own founding example.** Its advise test
   admits three inputs — *"fully determined by the document, the media on disk, and
   published rendering semantics"* — but its Why section justifies refusing `gravity` on
   the ground that *"the fact that separates the 6 safe instances from the 2 load-bearing
   ones — the old field's source-pixel-dimension inputs — is **not in the document**"*.
   Source pixel dimensions **are** on the media on disk (`migrate.py` reads them:
   `SRC_DIMS = { # ffprobe…`). Applied literally, the test classifies `gravity` as
   *advise*. ADR-0068 then repeats the bad reason verbatim: *"`gravity`… is refuse-class
   because its repair needed source pixel dimensions the document does not carry."* The
   conclusion may survive on a different ground (the fork is *which* of two inconsistent
   statements the author meant, not a missing measurement), but the stated ground is
   refuted. Nobody else will attack this, and every answer below depends on which clause
   governs.
2. **"ADR-0012 retired `box` in both its meanings" is right; "a retirement whose repair is
   arithmetic ships a script" never happened.** ADR-0016: *"Note this has never actually
   been done: `migrate.py` is a cumulative regenerator from a fixed origin… not an N→N+1
   chain."*

**Q1 — `"box": [x,y,w,h]` on an image.** Predicted consensus: advise. Case against:
granularity is *per check*, and `box`'s other spelling (Q2) is indeterminate, so one `box`
check must refuse everything — *"If any instance a check can match is capable of being
load-bearing, the check emits `repair: "none"` for **every** instance."* Why it fails: the
two spellings are different value types on different element types and are naturally two
checks. **Verdict: advise.** Repair (image): `{"x": x, "y": y, "origin": "top-left",
"width": dw, "height": dh, "clip": [x, y, w, h]}`, `fit` retained, where `(dw,dh) =
cover(sw,sh,w,h)` by ADR-0013 integer cross-multiplication. For `box:[0,0,1080,1300]` on a
1536×2720 source: `width 1080, height 1912, clip [0,0,1080,1300]`. **Omitting `clip` is
not a safe repair** — ADR-0012 measures 612.5 px of spill. Confidence 0.7 (0.55 that it's
one check with Q2).

**Q2 — `"box": "card-05"`.** Predicted consensus: advise ("resolve the id, copy
width/height"). **Verdict: refuse, `repair: "none"`.** *"15 of the fixture's 22 text
elements have no rect element behind them at all, so under a required id they would name
something that does not exist."* Even when it resolves, the text box ≠ the rect:
`migrate.py` needed a hand-measured table (`"chip-text": (420 - 182, 84)`) and a
`DESIGN_WIDTH = 984` constant that is author design knowledge, not document content.
Confidence 0.85.

**Q3 — `"align": "top"` on an image.** Predicted consensus: refuse. Best case against:
ADR-0015 says the declared rect plus `clip` already determine the crop, so the repair is
"delete" — a pure spelling retirement. It fails because that is exactly the delete-only
repair ADR-0043 measured as wrong on 2 of 8 elements. `align`-on-image *"becomes
**`gravity`**"*, and `gravity` is the one spelling ADR-0043 classifies. **Verdict: refuse,
`repair: "none"`.** Confidence 0.9 — my strongest.

**Q4 — `bold: true`.** Framing slip: ADR-0007 puts this on the text *element* (*"The real
project file carries `"weight":"bold"` on all 22 text elements"*), not on a run; runs are
`{"text": …}`. Case for advise: the fixture declares exactly one chain, `"brand":
[{"file": "fonts/OpenRunde-Bold.otf"}]`, already bold — repair = delete. It fails on
uniformity: another file declares a regular face, and then the repair is to vendor a font
file — *"authoring an asset is something an agent can do and a program categorically
cannot."* **Verdict: refuse, `repair: "none"`.** Confidence 0.8.

**Q5 — is there a gap?** Predicted consensus: no gap. There is one, and Q4 exhibits it.
ADR-0043's two arms are *determined* vs *depends on knowing what the author meant*. `bold:
true` with no bold face declared is **determined but not writable**: the fix ("a different
weight is a different file") is fully nameable without knowing intent, yet it is not a
document value, so `"none"` is the only legal encoding — and `"none"`'s instruction is
*"stop, do not attempt a repair… surface the finding verbatim to whoever is operating
it"*, which is wrong advice for a fix the agent can perform. Smallest amendment: one
sentence permitting a structured `repair` whose content is a named out-of-file action,
still non-bypassable. Cost: an action string is prose about a retired field's meaning —
the surface the glossary experiment measured at **0/3 vs 2/3** — and it weakens `repair ==
"none"` as a machine gate. Confidence 0.6 that it's a real gap rather than acceptable
over-refusal.

**Q6 — "`box` was migrated by arithmetic script".** **Historical remark, not a
classification.** Four reasons: it sits under *"A note on precedent, because it is easy to
overstate,"* whose opening sentence is *"ADR-0043 classifies exactly one retired spelling
— `gravity`"*, and ADR-0043's banner agrees (*"This ADR classifies `gravity` and nothing
else"*); it is past tense about a completed event; ADR-0068's Consequences classify only
`mask`; and ADR-0012 predates ADR-0043, so no class existed to assign. It is also **partly
false as description**: `migrate.py`'s text branch uses a measured `TEXT_BOX` table and
`DESIGN_WIDTH`, not arithmetic, and ADR-0016 says the ship-a-script rule *"has never
actually been done."* Confidence 0.9.

---

## Juror B — Haiku

**Q1.** `"box": [x, y, w, h]` on an image element is **advise-class**. The fix is fully
determined by the document: split the array into `x: array[0]`, `y: array[1]`, `width:
array[2]`, `height: array[3]`, with `origin: "top-left"` as the default for an
array-specified rect. Sentence: "The rect is now `x`,`y`,`origin`,`width`,`height` plus
`clip`" (CONTEXT.md, Box section). Confidence: high.

**Q2.** `"box": "card-05"` (string reference) is **refuse-class**. The fix requires knowing
what containment the author intended by the reference. The document does not carry this
semantic; CONTEXT.md states the reference "is now literal `width`/`height`, because fifteen
of the fixture's twenty-two text elements have no element behind them to name." The repair
field should be `"none"`. Confidence: high.

**Q3.** `"align": "top"` on an image element is **refuse-class**. While CONTEXT.md gives a
replacement path — `"x`/`y`/`origin` against the aperture's own `clip`" — some instances
are inert (already achieving the alignment), others require geometric repositioning that
silently changes the render if done incorrectly. This parallels the `gravity` retirement,
documented in CONTEXT.md's Gravity section: "`gravity:"bottom"` element needs `y`
recomputed... deleting it unmoved silently swaps which 1300px band of the source is on
screen." Per ADR-0043's granularity rule: "If any instance a check can match is capable of
being load-bearing, the check emits `repair: "none"` for **every** instance it matches,
including ones that look safe." Refuse-class uniformly. Confidence: high.

**Q4.** `"bold": true` on a text run is **refuse-class**. CONTEXT.md (Weight / bold
section): "`bold: true` could only mean synthetic emboldening, which is renderer-specific
and machine-dependent." The fix requires the author to declare the bold font file and
select it — author intent, not document-determined. Repair: `"none"`. Confidence: high.

**Q5.** The four answers sort **cleanly into the binary with no gap**. Q1 is advise (rect
array, mechanical); Q2–Q4 are refuse (all depend on author intent: reference semantics,
geometric loads, font selection). ADR-0043 defines the binary as: (a) repairs fully
determined by document + media + semantics = advise, (b) repairs requiring author intent =
refuse. Every question falls into one of these two categories. No case falls between them.
The binary is complete. Confidence: high.

**Q6.** "Box was migrated by arithmetic script" is a **historical remark, not a
classification under ADR-0043**. ADR-0068's "note on precedent" states: "ADR-0043
classifies exactly one retired spelling — `gravity`. `center-center` is a pure spelling
substitution (ADR-0013), and `box` was migrated by arithmetic script." This lists three
different retirement mechanisms: (1) ADR-0043's refuse-class (`gravity`), (2) ADR-0043's
advise-class (`mask`, added by amendment), (3) ADR-0016's spelling substitution (not
ADR-0043), (4) ADR-0016's arithmetic script (`box`). The statement refers to ADR-0016's
framework: "A retirement whose repair is arithmetic ships as a script beside the ADR."
Thus `box` is classified under ADR-0016's mechanism, not ADR-0043's binary. The clause is
a **mechanical reference** distinguishing which migration pathway `box` took, not a
refusal/advise classification. Confidence: high.

**Summary:** Q1 advise, Q2–Q4 refuse, Q5 binary holds, Q6 is historical reference. All
answers grounded in text read as of commit aef1cdcf.

---

## Juror C — Fable

**Contamination note:** I read only `git show aef1cdcf:` blobs (ADRs
0007/0012/0014/0015/0016/0043/0068, CONTEXT.md,
`docs/research/sample-project-migration/migrate.py`). I saw no argument for any particular
answer.

**Background checks:** all claims hold. ADR-0043's banner: *"This ADR classifies `gravity`
and nothing else"*; Not-settled: *"None are reclassified by this ADR."* ADR-0016: *"A
retirement whose repair is arithmetic ships as a script beside the ADR that caused it. A
retirement whose repair requires knowing what the author meant is refused, not guessed."*
ADR-0012: *"`box` is retired in both of its meanings"*; *"`align` splits."* ADR-0007: *"No
`weight`, no `bold`. A different weight is a different file."* ADR-0014 Consequences:
*"The schema rejects … `weight`/`bold` naming the font chain."* CONTEXT.md Rejected terms
covers Box, Align-for-images, Gravity, Weight/bold (lines 487–495, 580–584).

**1. `"box": [x,y,w,h]` on image — advise-class.** ADR-0043 Consequences: *"an arithmetic
retirement (its repair script, per ADR-0016's existing rule) is advise-class and states the
fix"*, and the advise test admits *"the media on disk"*. The repair is what `migrate.py`
lines 98–108 compute: `{"x": x, "y": y, "origin": "top-left", "width": dw, "height": dh,
"fit": <existing>, "clip": [x,y,w,h]}` with `(dw,dh) = cover(source_dims, w, h)` from the
probed source. Confidence: medium. Caveat: the script only implements `cover`; for
`contain`, where the old rect sat inside the box was never published, and the per-check
rule (*"If any instance a check can match is capable of being load-bearing, the check emits
`repair: "none"` for every instance"*) would pull the whole check to refuse. ADR-0043's two
sentences disagree on this edge.

**2. `"box": "card-05"` — advise-class.** ADR-0012: *"becomes literal `width`/`height`.
This is a correction to that ADR's spelling, not its decision."* The referenced id's rect
*is* the box; repair: `{"width": <card-05.width>, "height": <card-05.height>}`, i.e.
`984`/`169` for the fixture. Fully document-determined; the lost anti-drift binding is a
semantic loss, not an undetermined value. Confidence: medium — a missing id is a separate
finding, and `migrate.py` hand-narrowed two of seven boxes (`chip-text`, `handle-text`),
which shows an author might want an inset, but that inset was never the declared
semantics.

**3. `"align": "top"` on image — refuse-class**, `repair: "none"`, sibling census.
ADR-0012: *"On images it meant which part of the source survives the crop … and becomes
`gravity`."* It is `gravity` one rename earlier, and the gravity fork applies verbatim: an
`align:"bottom"` beside a literal `y` needs `y = clip.y + clip.h − height` or not, and
*"the fact that separates the 6 safe instances from the 2 load-bearing ones … is not in the
document."* No ADR classifies it explicitly (0043 classifies gravity only), so this is by
analogy. Confidence: medium-high.

**4. `"bold": true` — refuse-class as written**, `repair: "none"`. The fix (declare a
second font file, point the run at it) is a known *shape*, but its *value* — which face,
which path — is *"not fully determined by the document, the media on disk, and published
rendering semantics"*: ADR-0007 records *"no font file exists anywhere in this repo"* and
*"4/4 invented font paths."* Confidence: low. This one is a bad fit for refuse, see Q5.

**5. The binary has a gap.** Q1–3 sort; Q4 does not. Refuse-class is defined by *"knowing
what the author meant"* and carries a non-bypassable *"stop, do not attempt a repair via
ordinary file edit"* posture. `bold` is missing an *asset*, not an *intent*, and ADR-0016
already named that third thing: *"authoring an asset is something an agent can do and a
program categorically cannot."* Refusing `bold` blocks the one actor who should act;
advising it requires inventing a path. Smallest amendment: let an advise-class `repair`
state an edit *shape* with an explicitly unfilled slot (e.g. `{"font": {"declare": "<bold
face file>"}}`) rather than only *"a value"*. Cost: `repair != "none"` stops meaning
"apply verbatim", weakening ADR-0043's *"directly machine-checkable"* gate, and it
reintroduces tool-authored guidance of the kind the glossary experiment (0/3 vs 2/3)
measured as harmful. A second, unrelated gap: the Q1 `contain` edge, where the Consequences
bullet and the per-check uniform rule contradict.

**6. Historical remark, not a classification.** The clause sits in a paragraph opening *"A
note on precedent, because it is easy to overstate. ADR-0043 classifies exactly one retired
spelling — `gravity`"* and closing *"This ADR classifies the second one"* — a count that
excludes `box`. The verb is past tense, describing `migrate.py` line 98 (`x, y, w, h =
e.pop("box")`). ADR-0068's front-matter amends 0043 as *"classifies a second retired
spelling"* — one, `mask`. `box`'s class follows only from ADR-0043's general
arithmetic-retirement rule, never from an assignment.

---

## Juror D — Sonnet

**1. `"box":[x,y,w,h]` on an image — ADVISE-class.** ADR-0012: *"The literal `[x,y,w,h]`
rect becomes `x`, `y`, `origin`, `width`, `height`... Pure gain: the pivot that the 4-array
left implicit becomes explicit."* This is a pure arithmetic unpacking — fully determined by
the document. Repair: `{"x": box[0], "y": box[1], "width": box[2], "height": box[3],
"origin": "top-left"}` (drop `box`). Confidence: medium — the ADR never spells the literal
origin keyword for this migration; "top-left" is my inference from "the pivot... left
implicit becomes explicit," not a quoted value.

**2. `"box":"card-05"` — REFUSE-class.** ADR-0012: *"fifteen of the fixture's twenty-two
text elements have no rect element behind them at all, so under a required id they would
name something that does not exist."* ADR-0043's granularity rule is per-check: *"If any
instance a check can match is capable of being load-bearing... the check emits `repair:
"none"` for every instance."* Since this check's population provably contains unresolvable
instances, it must refuse uniformly — even for instances (like `card-05`, which does exist)
where the id resolves. Confidence: medium — this is my application of ADR-0043's general
test to a check no ADR ever explicitly classified; ADR-0068 states plainly that "`box` was
migrated by arithmetic script" is *not itself* a classification (see Q6), so nothing
settles this textually either way.

**3. `"align":"top"` on an image — REFUSE-class.** CONTEXT.md: *"`align` on a non-text
element is a schema error naming `x`/`y`/`origin` and `clip` as the replacement."* This is
the same underlying quantity ADR-0012 renamed to `gravity`, which ADR-0043 is built around
as its canonical refuse-class case (the entire gravity-fork evidence section). Confidence:
medium-high.

**4. `"bold":true` on a text run — REFUSE-class, `repair: "none"`.** ADR-0007: *"No
`weight`, no `bold`... the schema must **reject** it naming the replacement, never ignore
it."* CONTEXT.md: *"the schema rejects it and names the replacement."* No document field
names which physical font file is the "bold" variant — that's an authorial/vendoring
decision, not derivable from document, media, or rendering semantics. Confidence:
medium-high.

**5. Gap in the binary — yes, two of them.**
(a) `box` bundles two structurally different repairs under one key spelling — the array
form is cleanly advise-class, the string form is refuse-class — yet ADR-0043 defines
granularity "per check," treating a retired *spelling* as one unit. Amendment: classify per
retired-value-*shape*, not per key name, so array-`box` and string-`box` are separate
checks. Cost: check taxonomy fragments (one retired key can spawn several checks), and "per
check, never per instance" needs a footnote defining what counts as one check.
(b) Transitive retirement: `align`→`gravity`→`x/y/origin/clip` — ADR-0016 says the error
"names its replacement," but never says what happens when the named replacement is itself
later retired. CONTEXT.md resolves it by naming the *final* live replacement, but no ADR
states that chase-to-the-end rule. Amendment: ADR-0016 gains a sentence requiring the
message to name the current live replacement, not the intermediate one. Cost: near zero,
but it's a real gap — nothing enforces it today.

**6. ADR-0068's "`box` was migrated by arithmetic script" — a historical remark, not a
classification.** Directly quoted from ADR-0068: *"ADR-0043 classifies exactly one retired
spelling — `gravity`. `center-center` is a pure spelling substitution (ADR-0013), and `box`
was migrated by arithmetic script. ... it is not a repair class, and the two should not be
conflated."* High confidence — this is the ADR explicitly disclaiming that the clause is a
classification.

**Contamination note:** I read only the specified ADRs and CONTEXT.md at `aef1cdcf`, via
`git show`; I did not touch `crates/montaget-core/src/checks/`, issues #192/#228, PR #229,
or the working tree, and encountered no argument for a particular answer from any
contaminated source.
