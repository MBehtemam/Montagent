# G2 verdict — round 2, Montagent issue #48, the `fit` vocabulary

Juror G2 (Fable). Sources read: issue #48, ADR-0003, ADR-0005, ADR-0006, ADR-0012 (clip/aperture
section), ADR-0013 (in full), ADR-0014 (gravity clause), CONTEXT.md, the 8 image elements of
`fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json`, and a live run of
`docs/research/sample-project-migration/fit_rounding_scan.py` (all assertions passed, exit 0).
All arithmetic below was computed in exact integer arithmetic, not guessed.

## R2-Q1

**Decision:** `declared`.

**Why:** The round-1 premise is that `fit` is a **derivation claim** — it records HOW the author
computed the extents, and its consumer is `validate`. So the escape member must name the
**cause** (no derivation rule; the provenance of these integers is the author), not the effect
(anisotropic resampling), for two reasons. First, the effect is not reliably present: an author
who computes isotropic extents by hand and writes them under the escape has stretched nothing,
so an effect-name like `stretch` would be false on its face there. Second, an effect-name reads
as a render instruction, which the settled premise says `fit` is not — under
declared-rect-authoritative *every* value resamples to the declared rect; the escape differs only
in what `validate` may check.

Eliminations of the other candidates:

- `none` — the direct CSS `object-fit: none` collision: in CSS `none` means "do not resize; draw
  at natural size and crop." An agent arriving from CSS would read `fit:"none"` as
  natural-size rendering, which is exactly the natural-size default ADR-0012 forbade
  ("a size is required, not defaulted"). Worst possible false friend.
- `stretch` — effect-naming (wrong per above), false when the declared rect is isotropic, and it
  is CSS `object-fit: fill`'s semantics under a name CSS uses elsewhere (`align-self: stretch`),
  so it teaches an agent a render behaviour the field does not have.
- `exact` — says nothing about provenance; under ADR-0013 *every* fit value's extents are exact
  integers, so "exact" fails to discriminate the escape from `cover`.
- `fill` (the CSS-natural spelling of the effect) is **spent twice** in this format: shape paint
  (CONTEXT.md) and ADR-0005's `fill: hold/loop`. Unavailable regardless of merit.

`declared` is spent nowhere in the schema, collides with no CSS `object-fit` member, and rhymes
with the format's own load-bearing phrase "the declared rect is authoritative": `fit:"declared"`
reads as "these extents are declared, full stop — no rule stands behind them, check nothing
against a rule." That is precisely the semantics: the fit-deviation check does not apply, and the
element's disk-agreement question on extents is answered "the author owns it."

**Strongest counter:** `declared` is arguably redundant — *every* rect in this format is declared
and authoritative, so `fit:"declared"` risks teaching an agent by contrast that `fit:"cover"`
rects are somehow *not* authoritative (they are; `cover` only adds a checkable provenance claim).
`none` avoids that by naming the absence of a rule directly ("no fit rule"), and the CSS
collision could be handled by a schema description. I hold with `declared` because the CSS
collision of `none` is a silent misreading (natural-size), while the redundancy of `declared` is
at worst a benign tautology — and this format has consistently priced silent misreadings above
verbosity (`center-center`, `#RRGGBBFF`).

## R2-Q2

**Decision:** `contain` ships in v1. Its definition: under `fit:"contain"` with source `sw x sh`
into box `bw x bh`, **both derived extents must be `<=` the box dimension** (the inequality that
defines it); the driving axis is chosen by integer cross-multiplication — width drives when
`bw*sh <= bh*sw` — takes the box dimension verbatim; the slack axis is
`(s_slack * b_driving) // s_driving`, **floor**, in exact integer arithmetic. Here floor is not a
tiebreak: it is the only rounding that preserves `derived <= box` (ADR-0013 said this itself —
"if `contain` is ever defined with the obvious semantics, only floor is safe there"), and it
keeps ADR-0013's tiebreak (2) promise of one rounding operation across the whole vocabulary.
Verified on the fixture geometry: 1536x2720 contained in 1080x1300 gives (734, 1300), both
within the box.

The aperture-coverage collision is resolved by **parameterising the error's predicate by the
element's `fit` value** — the error is not "rect contains clip", it is "the geometric claim the
element's `fit` makes about `clip` holds":

- `fit:"cover"` → the declared rect must **contain** `clip` (the existing ADR-0013 error,
  unchanged; still passes 8 of 8 committed elements).
- `fit:"contain"` → the declared rect must be **contained by** `clip` (the dual: a contain
  element spilling past its aperture is silently cropped, which contradicts contain's
  whole-source-visible claim — the document contradicting itself, ADR-0006's charter).
- `fit:"declared"` → **no aperture-relation error**; the author made no claim relating rect to
  clip. (The frame-bounds and other checks are untouched.)

This is not a weakening of the existing error: it is the recognition that the hard-coded cover
direction was always cover's inequality wearing a general name. Both directions remain computable
from `x`, `y`, `origin`, `width`, `height`, `clip` — all in the document, no probe — so the
structural property "an unprobeable source can never suppress an error" survives intact.

Why ship rather than defer: (1) ADR-0003's asymmetry rule bars the only argument for deferral —
the fixture's 8-of-8 `cover` is one geometry counted seven times (the scan says so) and proves
nothing about `contain` being unneeded; letterboxing a source into a panel without cropping is
bread-and-butter CapCut/Premiere capability, squarely inside the reference class. (2) The escape
alone does not substitute: an author *can* hand-compute contain extents and write
`fit:"declared"`, but that element then loses the stale-source detection which is the check's
entire justification ("swap the file and the declared rect silently means a different crop
forever"). `contain` keeps letterboxed elements inside the promoted error's protection.
(3) Issue #48 demands the value set "published closed"; a closed set whose most obvious second
member is missing reopens on first contact with a real letterbox, and ADR-0013 already did the
hard part by naming exactly what a new value must bring — its inequality — which `contain`
brings above.

**Strongest counter:** ADR-0014's honesty clause — none of this has been rendered, and `contain`
would ship with **zero** committed elements exercising it, so its error direction, its floor
rule, and the parameterised predicate are all untested prose of exactly the kind that produced
the ADR-0011 "two agents shipped two different videos" failure. Deferring to a first real
letterboxed element would let evidence lead. I hold because the fit-deviation error this round
promoted **cannot ship coherently** with a vocabulary that has no checkable second rule — and
because the parameterisation cost is paid now or paid as a breaking rewrite of a shipped
hard-coded error later.

## R2-Q3

**Decision:** A derivation-claiming `fit` (`cover` or `contain`) on an element with no `clip` is
a **schema error naming both repairs**: add the `clip` that was the box, or write
`fit:"declared"`. `fit:"declared"` without `clip` is legal and ordinary.

**Why:** The fit rule is a two-operand computation — source and box — and every candidate
fallback leaves the second operand implicit, which is the #44 shape reborn: two authors assuming
different boxes (self-rect vs project frame) write different files for the same picture and
nothing in either file says which is right.

Each fallback fails on its own terms, and I computed rather than assumed:

- **Self-rect fallback.** The fixed-point property makes it undiscriminable by the fixture
  (cover of 1536x2720 into box 1080x1912 returns exactly (1080, 1912) — verified), and it
  silently degrades the check's meaning from "these extents came from that box" to "the declared
  aspect roughly matches the source aspect." It is not *vacuous* — against the stale 1536x2200
  source the self-box rule gives (1334, 1912) and the deviation still fires — but the reader of
  the file can no longer tell which of two different claims the element is making, and a check
  whose meaning quietly changes based on a field's absence is ADR-0006's silently-weakening
  check in a new costume.
- **Project-frame fallback.** Falsified as the box for the committed elements (gives 1084x1920,
  not 1912 — re-verified), so the fallback box and the explicit box would be *different
  quantities under one field value*. An agent that learned "the box is `clip`" from the fixture
  would compute wrong extents on every clip-less element.
- **UNCHECKED.** ADR-0013 built UNCHECKED for questions that are *unanswerable*, not questions
  the document declined to ask; letting an author disable the highest-value class of check
  (declared-vs-disk) by *omitting* a field is precisely "an opt-in check they might not tag" —
  the ADR-0004 argument ADR-0006 spent its opening section on. Silence must not mean "don't
  check me"; `fit:"declared"` exists so that declining the check is a visible, spelled decision.

The error is cheap and in the schema-error-naming-the-replacement pattern (ADR-0012, twice):
8 of 8 committed image elements carry `clip`, so it breaks 0. The common full-bleed case pays
one boilerplate line (`clip:[0,0,frameW,frameH]`) and buys locality — every fitted element
carries both operands of its own claim.

**Strongest counter:** The full-bleed background image — the most common image element in
general video work — now requires a `clip` that merely restates the project frame, and ADR-0012
established `clip` as needed by "7 of 7 photos plus handle-logo; zero text and zero rect
elements", i.e. as an *aperture* feature, not a *fit prerequisite*; conscripting it inflates
every simple file. A frame-fallback would be right for exactly that case and the deviation check
would stay fully armed. I hold because the frame-fallback was *falsified as the box on every
committed element*, so shipping it would enshrine as the default the one candidate the evidence
rejects, and because a two-operand rule with a sometimes-implicit operand is the class of
ambiguity this format exists to kill.

## R2-Q4

**Decision:** Normatively: **the source's dimensions are the counts of pixels in the sampling
grid the renderer draws from, after applying the file's own orientation metadata, ignoring pixel
aspect ratio.** Concretely: (a) start from the coded/storage pixel dimensions; (b) EXIF
orientations 5–8 on an image, and a video stream display-matrix/rotation of an odd multiple of
90 degrees, **transpose** width and height (orientations 2–4 and 180-degree rotations do not);
(c) PAR/SAR is **never** applied — dimensions are integer pixel counts, not display aspect;
anamorphic footage whose author wants display-corrected extents computes them and writes
`fit:"declared"`. And **yes, `validate` must print the dimensions it used** — ADR-0013's note
format already carries them ("cover from images/06.png (1536x2200)"); this becomes normative,
extended to flag the transform applied, e.g. `(3024x4032, EXIF-transposed)`.

**Why:** Each clause is forced.

*Orientation applied:* ADR-0013's UNCHECKED asymmetry says "render must decode the source to
draw it" — and every modern decode path draws phone JPEGs orientation-applied. If `validate`
read storage dimensions instead, then on every portrait phone photo (an enormous share of real
inputs) the rule value would be computed from transposed operands and the promoted fit-deviation
**error** would fire on a perfectly correct element — a validator manufacturing false alarms on
the most common case, while the actually-drawn geometry matched the file. The definition must
match what render samples, or validate and render disagree about the same element, which is the
one split this system cannot tolerate.

*PAR ignored:* applying PAR yields non-integer "dimensions" (720x480 at 10:11 has no integer
display width), and the entire lesson of ADR-0013's 31-million-combination sweep is that the
rule's operands and operations must be integers end to end. PAR is display shaping, and under
declared-rect-authoritative the renderer resamples the pixel grid to the declared rect
regardless — the grid is the fact; the aspect is an interpretation.

*Printing is mandatory:* this is ADR-0005's `speed` divergence applied exactly. `3368/0.645`
sent four agents to 5222 and one to 5220 and nothing in any output exposed *which operand
reading* diverged; the fix Montagent already institutionalised (ADR-0005: "the error states the
duration it found") is to print the measured operand next to the verdict. With two known
transpose/PAR ambiguity sources, a deviation finding that does not show the dimensions used is
undebuggable across implementations; one that does turns any implementation split into a
one-line diff.

Residual case, named so it is not discovered later: a video stream whose coded dimensions
change mid-stream within the element's source range has no single (sw, sh); its fit check is
**UNCHECKED** with the reason stated (the question is unanswerable with one number — ADR-0013's
own criterion for the category).

**Strongest counter:** "Apply EXIF" imports a dependency on metadata handling that varies by
library (raw libjpeg exposes storage dims only; EXIF can be stripped, duplicated, or
contradicted by embedded thumbnails), so the storage-dimensions definition is the more
mechanically reproducible one — and reproducibility across implementers is this rule's stated
soul. I hold because storage dims buy reproducibility of a number that is *wrong relative to
what is drawn*, guaranteeing correct-element false errors at the promoted severity; the
divergence risk of EXIF handling is exactly what the mandatory printed-operands clause exists to
surface, whereas a wrong-but-consistent operand is surfaced by nothing.

## R2-Q5

**Decision:** Write the rules **type-generically now**: every rule in this ADR applies to any
element carrying a raster source — `image` and `video` alike. `fit` is required on both;
omission is a schema error on both; the box is `clip` on both; the escape, the inequalities,
and the rounding are identical. The video-specific wrinkles are named in the ADR rather than
deferred: (1) orientation/rotation metadata and PAR, already settled normatively in Q4 —
dimensions are rotation-applied pixel counts, PAR ignored, `declared` is the anamorphic
author's escape; (2) mid-stream dimension changes → UNCHECKED with reason (Q4); (3) probing
cost — the check needs stream headers only, never a frame decode, the same weight as ADR-0005's
duration probe, so UNCHECKED semantics for unprobeable sources carry over unchanged; (4) what
`fit` does **not** touch on video: the temporal axis. `fit` is a claim about the spatial extent
derivation only; source range, `speed`, `fill: hold/loop` (ADR-0005) are orthogonal and no
member of this vocabulary may ever grow a temporal meaning — recorded so nobody reaches for
`fit` to spell temporal stretching.

**Why:** ADR-0003 is dispositive twice over. Montagent's reference class "edits video" in its
first sentence, and the guard's asymmetry rule makes "the fixture has zero video elements" (I
checked — it has none) inadmissible as an argument for scoping to `image`. Scoping to `image`
and "extending later" would ship a format in which `fit` on a video element is *undefined* —
which is the exact hole issue #48 was opened about for images ("a field that reads as settled
until someone writes an element"), recreated knowingly for the neighbouring type. The rule's
operands (source pixel grid, clip box) are type-independent; the only genuinely type-specific
content is the dimension-reading question, and Q4 settles that for both types in one
definition. A type-generic rule also keeps the schema honest under ADR-0012's "the schema
varies by type" discipline: the variation is *which fields exist per type*, and here the same
field exists on both with one meaning — forking its semantics by type would be two spellings of
one concept.

**Strongest counter:** Zero video elements exist in any fixture, so the video half ships with
no regression guard, no `verify.py` recomputation, and no rendered evidence — and ADR-0013's
own credo is "a fixture that passes proves the fix is safe; only the scan proves it is
necessary," neither of which exists for video. First contact with real footage (variable frame
sizes, rotation flags ffmpeg and a browser read differently) could falsify a detail the way
the frame-box guess was falsified. I hold because the falsifiable details are quarantined into
Q4's dimension definition — one clause, amendable without touching the vocabulary — while
scoping the *vocabulary* to `image` would be a structural hole; and the missing evidence is an
argument for committing a video fixture element, not for leaving `fit` undefined on half the
reference class.

## Anything these five missed

Three holes, in descending weight:

1. **`gravity`'s retirement message and migration are unowned.** Round 1 retired `gravity` as
   "a schema error naming its replacement" — but on 8 of 8 committed image elements `gravity`
   is *present* (`"top"` seven times, `"center"` once), so the committed fixture becomes
   schema-invalid the day this ships. The ADR must (a) state the error message's named
   replacement — the honest one is not a field but a fact: "which part of the source survives
   is determined by the declared rect and `clip`; delete `gravity`" — and (b) order
   `migrate.py`/`verify.py` updates with the byte-diff discipline ADR-0013 used ("verified by
   regenerating and byte-diffing"). ADR-0012's pattern demands the replacement be *right*, and
   here there is no field to name; the message shape needs a decision, not an assumption.

2. **The promoted error's interaction with UNCHECKED needs one explicit sentence.** The
   fit-deviation check is now an `error`, but it needs the source's dimensions, so on an
   unprobeable source it is UNCHECKED — meaning a *missing file* silently downgrades an
   error-class check. ADR-0013's structural boast ("an unprobeable source can never suppress an
   error") was true when the only error was probe-free; it is false for this new error. That is
   acceptable — render still enforces, since render must decode — but the ADR must say it
   aloud, and the summary-line unchecked count is what carries the residual risk. Left
   unstated, the boast and the new error contradict each other in the same document set.

3. **`fit:"declared"` on `handle-logo`-class exact matches invites drift.** The one committed
   escape-adjacent case (800x800 into 68x68) is an exact aspect match where `cover`, `contain`
   and hand-declaration all yield the same integers. Authors and migration tools will face a
   genuine spelling choice with zero observable difference — two legal spellings of one
   geometry, the situation this format hates. Worth one normative sentence: when a derivation
   rule *does* describe the extents, write the rule (`cover`/`contain`), because it keeps the
   stale-source check armed; `declared` is for extents no rule produces. Not an error — the
   ambiguity is only resolvable by intent — but the canonical-preference sentence prevents
   `fmt`-adjacent tools from ever "normalising" one to the other, which Q1's semantics forbid
   but nothing yet states.
