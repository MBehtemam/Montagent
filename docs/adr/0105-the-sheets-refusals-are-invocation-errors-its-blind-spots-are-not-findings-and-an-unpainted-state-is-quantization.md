---
status: accepted
amends: 0006 (`N-QUANTIZATION` gains a third `review` escalation condition — a visual state the grid never paints — beside an element rounding out of existence and a rounding that manufactures an overlap or gap), 0094 (section 4's worked example is corrected — the fixture has no unpainted visual state; section 6's `skipped` is specified as a disclosure field of which only `no-grid-frame` entries raise a finding, and `blind_to` as a fixed token list in the NOT CHECKED block's shape), 0095 (the refusal it left uncoded is `E-SHEET-OVERFLOW`), 0097 (exit 3 for that refusal is ratified rather than inherited; section 7's not-yet-legal code, closed by 0103, is `E-INVOCATION`), 0103 (the refusal's "stable code" is `E-INVOCATION`; the sheet's own disclosure points at `frame --crop --at`, settling the juror request that ADR left open)
---

> **Amended by [ADR-0106](0106-the-sheets-opt-ins-are-keyframes-and-infill-ceiling-and-a-keyframe-tile-is-sampled-where-its-change-first-paints.md).** `E-SHEET-OVERFLOW` gains two finding fields —
> `fits_without_keyframes` and `keyframe_tiles_admitted` — rendered only when `--keyframes` was
> passed, because a refusal the flag caused has a cheaper remedy than a narrower range and the
> sub-ranges alone cannot say so. The `between-keyframes` sentence gains where a keyframe tile is
> sampled: the first painted frame at or after the change. `untiled` keyframe change points carry
> reason `no-grid-frame` and, like `infill-evicted`, raise no finding.

> **Amended by [ADR-0114](0114-the-sheet-carries-a-reader-check-a-handshake-on-its-first-label-that-names-no-reader.md).** Section 6's `blind_to` stays about the **rule**. What
> the reader can see is not a blind spot and gets no token. The READER CHECK is a sibling of the
> NOT CHECKED block, printed on every range answer and never a finding.

> **Amended by [ADR-0118](0118-an-unpainted-visual-state-is-one-finding-per-state-from-one-selection-in-every-verb.md).** [#437](https://github.com/MBehtemam/Montagent/issues/437) closed the gap this
> ADR filed: `validate` raises `N-QUANTIZATION` for every unpainted visual state, **one finding
> per state**, over the one selection `frame`'s range mode is to share. ADR-0118 names the
> finding's fields, and records that a vanished element or a sub-frame gap now raises two
> findings, the element's or gap's and the state's. Where this ADR says `validate` is silent, that
> was true when it was accepted; the ninth claim of `check_unpainted_runs.py` now checks the
> closing instead.

# The sheet's refusals are invocation errors, its blind spots are not findings, and an unpainted state is quantization

[#412](https://github.com/MBehtemam/Montagent/issues/412), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Grilled as six questions. Three
(Q4–Q6) were put to a jury of three models — Opus 5.5, Sonnet 5.5 and Fable 5.1 — split 2–1 on
one and unanimous on two; ballots verbatim in
[`docs/research/juries/contact-sheet-finding-codes/`](../research/juries/contact-sheet-finding-codes/README.md).
The other three (Q1–Q3) were decided by the human on argument from committed ADR text, **not by
a panel**, and are marked as such below.

The one measured claim is re-derived by
[`docs/research/contact-sheet-finding-codes/check_unpainted_runs.py`](../research/contact-sheet-finding-codes/check_unpainted_runs.py)
— **nine claims**, all passing as committed.

## The question

The sheet emits five things that look like findings and are not obviously the same kind of
object: ADR-0095's overflow refusal, the flag combinations ADR-0097 and ADR-0103 refuse, a
`skipped` run, ADR-0094's `blind_to` enumeration, and the rest of the disclosure (dropped
audio-only boundaries, untiled keyframe count, served tile width and rung). ADR-0094 settled that
a blind spot gets no code and a skipped run gets one; ADR-0095 left its refusal uncoded;
ADR-0097 §5 added that everything must render as prose, not only as an enum.

The answer turns on one test ADR-0094 §6 already states and this ADR applies to every member:
**a finding says something about this document.** What says something about the invocation is
an invocation error under ADR-0073. What says something about the rule is disclosure.

## Decision

1. **The overflow refusal is `E-SHEET-OVERFLOW`**, declared `NotAboutDocument` (ADR-0073), exit 3,
   reached through `Report::refused_invocation` — the `E-PROJECT-EXISTS` pattern (ADR-0080). It
   carries no `repair` field. The sub-ranges that would fit, the visual-state count, the tile
   count the floor admits and **which limit bound** are **finding fields**, rendered by its
   template as prose. *(Q1 — human, not the panel.)*
2. **ADR-0098's type-floor refusal and a `--keyframes` call pushed past the floor (#407) are the
   same code.** All three are one condition — this range holds more visual states than the sheet
   can show legibly — with one next move, a narrower range. The `limit` field says whether
   ADR-0095's 140 px tile width or ADR-0098's 8 px served type bound.
3. **The flag-combination refusals are bare `E-INVOCATION`**: `--at`, `--full` and `--crop` each
   composed with `--from`/`--to`. The message content ADR-0097 §4 and ADR-0103 require (the tier
   fact; the two-step loop, "whole frames by design") is the `reason` text. **ADR-0103's "stable
   code" is `E-INVOCATION`**, which also retires ADR-0097 §7's "not-yet-legal code" as a question.
   *(Q2 — human, not the panel.)*
4. **`skipped[]` is a disclosure field, not the finding channel.** It lists every run the sheet
   drew no tile for, with a reason: `no-grid-frame` or `infill-evicted`. **Only `no-grid-frame`
   entries also raise a finding**, because only that reason is about the document; an evicted
   infill tile is the instrument's own choice under budget, like the rung. *(Q3 — human, not the
   panel.)*
5. **A `no-grid-frame` run raises `N-QUANTIZATION` at `review`.** `N-QUANTIZATION`'s scope
   broadens from elements to visual states: *a visual state the grid never paints* joins
   ADR-0006's two escalation conditions, on ADR-0006's own reason for them — *"the rendered frames
   do not show what the document declares."* The `detail` names the state's interval and the two
   boundaries (element, track, ms) that land on one frame. `validate` does not detect it yet;
   that gap is [#437](https://github.com/MBehtemam/Montagent/issues/437), outside the map.
   *(Q4 — panel, 2–1.)*
6. **`blind_to` is the sheet's NOT CHECKED block**: a fixed list of stable lowercase tokens with no
   letter prefix, each bound to one fixed sentence, printed on every answer including perfect
   ones, never a finding. The tokens and sentences are below. **Every other disclosure item —
   dropped audio-only boundaries, untiled keyframe change points, served tile width, rung — is
   disclosure, not a finding.** *(Q5 — panel, unanimous.)*
7. **The sheet points at the next call.** The `below-tile-width` sentence names
   `frame --crop --at <instant>`; the `inside-run` sentence names `frame --at <instant>`. Both
   depend on ADR-0097 §8's provenance line printing each tile's sampled instant, so the instant to
   type is on the line. This settles the request ADR-0103 recorded and left here. *(Q5.)*
8. **ADR-0094 §4's example is corrected**, and `N-QUANTIZATION`'s sheet path **does not ship
   without a constructed document containing an instance** — the fixture has none. *(Q6 — panel,
   unanimous; the shipping condition was added by all three jurors independently.)*

**ADR-0043 is not amended.** Its refuse/advise binary is about findings on a document; ADR-0073
already carved out findings about something else, and every member here lands on one side of that
existing line. **No fourth category** — "about the instrument" — is needed. The overflow refusal
looked like it might need one, being neither a document fact nor a malformed command, but its
*subject* is the range the caller typed and its remedy is a different range, which is exactly
ADR-0073's *"the subject is the invocation."*

## Why

### 1. The overflow refusal is about the range, so it is an invocation error with fields

ADR-0095 §3: *"The caller asked a question the instrument cannot answer at the fidelity that
would make the answer mean anything, and the remedy — ask twice over two halves — is one the tool
can compute exactly."* Every part of that is about the question. The document is legal and
unchanged; nobody should edit a video to make it fit a contact sheet. So ADR-0043's
advise/refuse question — *is the fix determined by the document?* — is a category error here,
and ADR-0073 is the ADR that says so.

The fully-determined remedy looked advise-class, but ADR-0043's advise shape is a change to a
project file, `{"value": …}`, applyable. The sub-ranges are a change to a command. ADR-0073 puts
exactly that in message text; ADR-0080's `E-PROJECT-EXISTS` shows a `NotAboutDocument` code
carrying **fields** its template reads, which is the shape that lets an agent loop over the
sub-ranges without parsing prose while the prose still says them in full (ADR-0097 §5).

**`preview`'s budget refusal is the nearer precedent and is deliberately not followed.** It is bare
`E-INVOCATION` with everything in `reason`, via `Report::rejected`. That is right for `preview`,
whose remedy is a sentence ("preview a shorter range"). This refusal's remedy is a list of
ranges, which ADR-0095 ruled load-bearing: *"the refusal is the common case for anything longer
than the fixture... A refusal that does not make narrowing obvious will read as the feature being
broken."* A list of ranges in a prose string is not obvious to a machine. Whether `preview` should
follow is out of this map's scope.

**Exit 3 is ratified, not inherited.** ADR-0097 flagged it as an inheritance from ADR-0078's
reading 8. Under ADR-0011's table, whose distinguishing question is *what the caller does next*:
the document is legal (not 1), the file parsed (not 2), Montagent did not break (not 70), and the
next move is to fix the command. `refused_invocation` asserts `NotAboutDocument` at exit 3, so the
declaration and the exit code cannot drift.

**Why one code for three limits (decision 2).** ADR-0093 records that ADR-0043 *"forces one code
per reason"* because repair form is fixed per code. A `NotAboutDocument` code has no repair form
to fix, so that argument does not reach it. What remains is ADR-0011's test, and all three limits
share their next move. The `limit` field keeps them distinguishable without asking the agent to
learn three codes for one action.

### 2. The flag combinations carry nothing a code would add

`--at`, `--full` and `--crop` with a range are malformed commands in the plainest sense; the
usage text is the whole answer. A code of their own would carry no fields, and nobody would ever
gate on it — the move is the same whichever flag was wrong. ADR-0103 required a *stable* code,
and `E-INVOCATION` is one. What those ADRs actually cared about — the tier fact, the two-step
loop, "by design, not pending" — is message content, and ADR-0068's amendment already
distinguishes a message-text property from a classification.

### 3. `skipped` mixes a document fact with an instrument fact, so it cannot be the finding channel

ADR-0094's own test splits it. `no-grid-frame` says *this document declares a visual state its
video never shows* — true whoever asks and at any tile budget. `infill-evicted` says *this answer
spent its budget elsewhere* — true only of this call. If `skipped` were the finding channel,
eviction would become a finding about the document, and a caller passing a gap ceiling would see
the document's finding count move with a flag. So `skipped` is the census the reader reads, and
the finding is raised beside it for the one reason that is about the document. With crop out of
scope (ADR-0103), there are two reasons, not three.

### 4. An unpainted visual state is quantization, and one fact gets one identity

ADR-0006 escalates `N-QUANTIZATION` to `review` for an element rounding out of existence and for
a rounding that manufactures an overlap or gap, because *"both mean the rendered frames do not
show what the document declares."* A visual state between two boundaries on different elements
that both land on one frame is that fact, arrived at through two elements rather than one. It is
also literally quantization: two millisecond boundaries snapping onto the same frame.

The majority's argument is the code's contract. ADR-0006 makes a code *"the identity a future
`compare` diffs on"*. A `frame`-only code gives this fact a second identity the day `validate`
learns to see it, and `compare` then reports a change that did not happen. Juror 3: a verb-scoped
code *"would create a code whose meaning is 'frame saw it', which is provenance, not a document
fact."*

**Class `review`, not `note`**, on ADR-0006's own escalation and on the definition *"legal,
renders, and you must look at a frame"*. All three jurors, including the dissent, chose it.

### 5. `blind_to` has the NOT CHECKED block's shape, so it takes its form

It is printed unconditionally, never varies with the document, and judges nothing — exactly what
ADR-0006's NOT CHECKED block is, and for the same reason: *"without it a clean run is read as 'the
file is right'."* ADR-0094 §6 made the same argument for the sheet: *"a caveat that appears only
when something went wrong teaches the reader that its absence is an all-clear."*

`U-` findings were rejected unanimously. `U-` means *could not establish for this document*; a
blind spot is *structurally cannot, ever*, and six `U-` findings on every clean sheet would make
a finding mean nothing. Free prose was rejected because ADR-0097 needs the JSON and the prose in
step, and a fixed token bound to a fixed sentence is the only form that cannot drift. No letter
prefix, so a token is never mistaken for a code.

| token | sentence |
| --- | --- |
| `inside-run` | Each tile shows the first painted frame of its visual state; change inside a state — a source clip's own cut, motion within a still-looking element — is not on this sheet. `frame --at <instant>` looks at any other instant. |
| `between-keyframes` | Keyframed values are shown only where a tile falls, and keyframe instants are untiled unless asked for; a wrong easing curve shows only if its endpoints are wrong. |
| `below-tile-width` | Tiles are served at the width stated above; detail finer than that is not visible here. `frame --crop --at <instant>` looks closely at one region at true scale. |
| `across-sheets` | Only tiles on this one sheet can be compared with each other; a relation with a state outside this range is not visible. |
| `audio` | Nothing audible is on this sheet. |
| `motion` | A sheet is stills; whether motion looks right is `preview`'s question. |

The sentences are fixed text, specified here so the prose and the JSON cannot diverge; wording may
be tightened at implementation, but a token never changes meaning, and a new blind spot is a new
token.

**Everything else is disclosure.** Dropped audio-only boundaries and untiled keyframe points are
the rule working as designed on a legal document; width and rung describe this answer. None is a
defect, so none is suppressible or diffable, and that is correct.

### 6. The §4 example was a pre-merge interval

ADR-0094 §4: *"The fixture's 4 ms run at 56112–56116 contains no painted frame at 25 fps."* The
interval is real in `query`'s cut list, and taken alone it paints no frame. But the boundary at
56112 is `vo-quiz` — **audio** — ending, and ADR-0094 §1 drops audio and re-merges equal
neighbours. After that, 56112–56116 lies inside the visual run **53856–56116**, which paints from
53880 ms. **The fixture has 46 intervals, 18 visual runs, and zero unpainted runs.** The example
was computed before the re-merge the same ADR mandates — the shape #407 found for §3's keyframe
population.

The rule §4 decided is unaffected; only its example is. The condition is reachable, and the check
builds one: a 25 fps document with `a` ending at 1010 and `b` starting at 1030 on different
tracks has the visual state `1010..1030`, frames paint at 1000 and 1040, and **`validate` exits 0
with no `N-QUANTIZATION` at any class** — which is why decision 5 files a gap rather than reusing
a detector.

All three jurors independently made the zero count a shipping condition, not only a correction: a
code that never fires on any committed document has an untested emission path, prose rendering
and `skipped` shape. The constructed document is where those tests start.

### 7. Nothing here gives `frame` a verdict

ADR-0006 gives `validate` facts and no verdicts, and the map rules a verdict layer out of scope.
`N-QUANTIZATION` on a skipped run is a fact about the document, the same fact `validate` will
state under #437. The refusals are about the call. `blind_to` is about the rule. No member says a
tile looks wrong.

## Costs, recorded honestly

- **Suppression granularity — Juror 2's dissent, not answered by the majority.** *"Suppressing
  N-QUANTIZATION should not also hide 'a declared combination is never shown.'"* Broadening a
  code widens what one suppression silences. Accepted because a split identity is costlier to
  undo than a broad one, but it is a real cost, and the one to reopen if suppression by code
  becomes a thing agents actually do.
- **`frame` states a fact `validate` does not, until #437 lands.** One document, two verbs, one of
  them silent. Bounded by a filed issue, not designed away.
- **Q1–Q3 rest on argument, not a panel.** The human decided them on the judge's recommendation.
  The strongest is Q1, which follows from ADR-0073's text; the weakest is decision 2's folding of
  three limits into one code, which was not put to anyone as a separate question.
- **A second token vocabulary beside finding codes.** Small, closed, and the price of `blind_to`
  being keyable at all.
- **The sheet path of `N-QUANTIZATION` has no real instance.** Everything above about its
  rendering is specified against a constructed document.

## What this ADR does not decide

- **The sub-range algorithm** — how ADR-0095's refusal chooses the ranges it names. ADR-0095 owns
  that; this ADR only fixes that they are finding fields.
- **`preview`'s budget refusal**, which stays bare `E-INVOCATION`. Recorded as the precedent this
  ADR departs from, not amended.
- **Whether a disclosure is read at all** — the map's fog patch from ADR-0101. This ADR makes the
  disclosure structured and fixed; it does not show that an agent reads it.
- **The flag names** for keyframes and infill —
  [#418](https://github.com/MBehtemam/Montagent/issues/418).

## Consequences

- The registry gains `E-SHEET-OVERFLOW` (`Declared`, `NotAboutDocument`). `N-QUANTIZATION`'s
  declaration records its third escalation condition and that `frame`'s range mode emits it.
- `frame`'s range mode refuses `--at`, `--full` and `--crop` with a range as `E-INVOCATION`, and
  every over-long range — by tile width, type floor or keyframe load — as `E-SHEET-OVERFLOW`, exit 3.
- Every range answer carries `skipped[]` with reasons, `blind_to` as six fixed tokens and their
  sentences in both forms, and an `N-QUANTIZATION` `review` finding per `no-grid-frame` entry.
- ADR-0094's banner records the §4 correction; `CONTEXT.md` gains **Blind spot** and **Unpainted
  visual state**.
- [#437](https://github.com/MBehtemam/Montagent/issues/437) owns teaching `validate` the same fact.
