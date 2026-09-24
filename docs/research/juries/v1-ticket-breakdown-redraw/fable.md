# Ballot — Fable — the v1 ticket breakdown, redrawn

**Verdict: REFUTED as a complete absorption of the first court; the graph is materially better and still carries four defects of the class the court refuted it for, plus five misdescribed ADRs that are new.**

Verified against the working tree at `73c09f1b` (+ ADR-0068 on `main`): `docs/adr/README.md` (68 rows counted), ADR-0008/0015/0018/0021/0024/0027/0028/0033/0034/0041/0043/0044/0054/0057/0058/0059/0060/0062/0067/0068, `CONTEXT.md`, the fixture (`handle-logo` now carries `"effects":[{"name":"mask","shape":"circle"}]` — migrated in `9f2431c7`), `gh issue view 36 / 168 / 185 / 186`, and the three prior ballots plus their README. #168 does carry stories 84–113; #185 and #186 exist and say what the brief says they say.

---

## Part A — the first court's findings, one by one

Source is the README's list plus each ballot's concrete items. "Absorbed" means the redraw states it as a deliverable or an edge; "partial" means the correction is present but the reasoning or the placement is wrong; "missed" means it is not there.

| # | Finding (who) | Status | Where / why |
| --- | --- | --- | --- |
| A1 | `R-EASE-INERT` misfiled into the caption ticket (all three) | **absorbed** | T21 |
| A2 | `probe` does not depend on the document model; must land early (all three) | **absorbed** | T3 ← 1 |
| A3 | FFmpeg resolution inverted four tickets downstream of `probe` (Fable, Opus) | **absorbed** | T3 owns it |
| A4 | `preview` needs the encoder, not `frame` (Fable, Opus) | **absorbed** | T28 ← 27 |
| A5 | Cut list / `--where --census` do not need the resolver (Fable) | **absorbed** | T8 ← 4a |
| A6 | `compare` does not need the resolver (Fable, Opus) | **absorbed** | T31 ← 4a, 9 |
| A7 | Caption checks do not need the text engine — *Fable's reading governs*, per the README (Fable, Sonnet) | **partial** | T11 is free of T15 — but the same ruling covers `R-BOX-SLACK`, and **T17 is blocked on T15 again**. See B1. |
| A8 | Slack primitive owned by nobody (Fable, Opus) | **absorbed** | T9 owns it; story 113 |
| A9 | Anchor/layer *resolution function* owned by nobody (Fable) | **partial** | T10 owns it — and then bundles it with a check (ADR-0060's tie) that needs the resolver T10 is upstream of. See B2. |
| A10 | `fmt --check` and `LAYOUT` are one predicate in two tickets with no edge; prefactor it into the shape ticket (Fable) | **missed** | T5 cites ADR-0041, T10 cites ADR-0041, neither blocks the other, 4a does not name the order predicate. ADR-0041 §4 says *"there is exactly one place the rule lives, not two"*. The redraw reproduces the two places. |
| A11 | `fit` arithmetic is not text; move story 52 out of `measure` (Fable, Opus) | **partial** | T14 is correctly a `fit` ticket — but it still ships *"`measure`'s fitted-extent output"* while the `measure` verb is T15's, with no edge either way. Two tickets now build one verb. See B3. |
| A12 | Story 53 "renderer opens nothing outside the font chain" is a renderer property, verifiable at the rasterizer (Fable, Opus) | **missed** | Still in T15 (`measure`): *"the renderer opening nothing outside the declared font chain"*. |
| A13 | Story 54 / font files in the probe cache belongs with `validate`, not `measure` (Fable) | **absorbed** | T16 |
| A14 | `fonts list`/`fonts vendor` + attestation checks have no ticket (Fable, Opus) | **absorbed** | T18 |
| A15 | ADR-0007's five text checks have no ticket (Fable) | **absorbed** | T16 |
| A16 | Audio mixing has no owner (Fable, Opus, Sonnet) | **absorbed, with a new error** | T26 — but it claims *"the `loop` wrap"* as render work. ADR-0062: `loop` is *"purely `validate`-facing … `render` and every other tool are unaffected."* See B6. |
| A17 | The whole-video comparison is not redundant with frames; it is the only test of audio (Fable) | **absorbed** | T29 |
| A18 | Check registry / refuse-class declaration / completeness test ownerless (Opus) | **absorbed** | T1 |
| A19 | Ticket 1's demo needs the negative arm — `E-PARSE` exit 2, bad invocation exit 3 (Opus) | **absorbed** | T1 |
| A20 | `Finding` designed against one finding that uses none of its hard fields; construct census / refuse-class / citation fixtures in ticket 1 (Opus) | **absorbed** | T1 note |
| A21 | ADR-0030 + ADR-0042 force `Option<T>` and a permissive parse path; make the lossless unknown-key round-trip an acceptance criterion of the *model* ticket (Opus) | **absorbed, and now double-owned** | 4a's acceptance criterion requires the permissive path; T5 says *"Includes the permissive, order-and-presence-preserving parse path"*. See B4. |
| A22 | Refuse-class acceptance (38–39) belongs to the first refuse-class instance, `gravity` (Fable, Opus); the *non-bypassability* half belongs to `render` (Fable) | **partial** | 4b has the `gravity` instance. T25 does not cite ADR-0043 or 4b, and "no future flag lifts it" is a `render` property no ticket tests. |
| A23 | The canary is two jobs; acceptance is the key `jpegd-jpege-pdf`; feature set must not change in the raster tickets (Opus, Fable) | **absorbed** | T2, T24 note |
| A24 | ADR-0010's two-arm `rust-rasterizer` harness is a golden-frame guard *"not optional"*, and #36's "Related, same shape" asks it be run on every rasterizer bump — assign it to 2 or 15 (Opus) | **missed** | T2 claims #36 and does not mention the harness; T23 does not either. |
| A25 | Golden frames we commit are self-confirming; only `reference/frame-*.png` falsify; the text arm compares two typefaces and must be region-masked (Opus, Fable) | **absorbed** | T23 |
| A26 | A ticket to re-derive the 22 text sizes under Open Runde (Opus, Fable) | **absorbed** | #186 |
| A27 | Ticket 3 / 15 / 17 cannot fit a context window (all three); Sonnet split 15 into six | **partial** | 15 → 22/23/24 (three); 17 → 25/26/27; 3 → 4a/4b. 4a is now the biggest ticket in the set. See B8. |
| A28 | Ticket 1 is over-scoped: split 1a/1b (Opus) | **missed, and moved the other way** | T1 gained the check registry and its completeness test. Fable held 1 was fine as drawn; the author took that side but added Opus's registry on top without re-sizing. |
| A29 | Story 70 / the write-tool-returns-findings invariant must be re-asserted at every write tool; `shift` returns a knowingly partial set (Opus) | **absorbed** | T5 ("every later write tool reuses"), T30 note |
| A30 | Story 81 "no unsolicited network calls" needs a test from the first network code onward (Opus) | **absorbed** | T3 note |
| A31 | Stories 49 and 60 (budgets) have no CI home (Opus) | **missed** | T22 and T27 state the budgets; no ticket says where they run. And T27's budget cites an ADR that retired it — B7. |
| A32 | The fixture migration for the bare `mask` (Fable) | **resolved upstream** | ADR-0068 landed the bytes. 4a's demo is now true. |
| A33 | `R-VISUAL-GAP` belongs with the document checks, not the motion checks (Fable) | **absorbed, but misdescribed** | T12 — see B5: it is described as a check about *rects* and blocked on *layers*, and ADR-0018 contains neither. |

Tally: 19 absorbed, 8 partial, 5 missed, 1 resolved upstream. The five missed (A10, A12, A24, A28, A31) are each small on their own. A10 is not: it is the prior court's own "double-owned" row, unchanged.

---

## Part B — the redraw on its own terms

### B1. T17 re-introduces the edge the court refuted, and its #186 rationale is wrong on the ADR

T17 (`R-BOX-SLACK`) is *blocked by: 15, #186*. ADR-0058, Consequences: computed *"from `size`, `line_height`, `runs` and `height` with no I/O"*; the height is ADR-0028's `(size × n × line_count + 9) // 10` where `line_count` is the mandatory-break count (ADR-0008). **There is no font metric in it.** Open Runde versus SF Pro Rounded cannot move the number. So:

- The edge 17 → 15 is the exact edge the README recorded as *"Fable's reading governs"*, put back.
- The #186 block is reasoned from #186's own text (*"The computed height depends on the font"*), which is wrong on ADR-0058, and the redraw copied the issue rather than the ADR.
- The premise "this check may fire on the regression guard" is not a risk, it is a *fact already recorded in the ADR*: ADR-0058's table shows **7 of the 22 text elements fire today** (`sentence-05` at 177 %, `handle-text` at 121 %, …). That is the ADR "saying otherwise" under #168's rule. The implementer needs to be told that, not sent to wait on #186.

T17 → 4a only. #186 stays where it belongs, on T23's text arm.

### B2. T10 cannot finish with its declared blockers done — the tie check needs the resolver T10 is upstream of

T10 owns the anchor → integer-layer function **and** ADR-0060's *"geometry-overlapping layer tie (error)"*. ADR-0060: *"The check samples across the elements' shared time range, not just their rest extents … Evaluate both boxes at each keyframe boundary plus an interval between them"* — and a static-extent check is explicitly rejected. That is the keyframe resolver, which is T19, which is *blocked by: 4a, 10*. Circular. Either the resolution function is its own prefactor (the court's A9, applied literally) and the tie check moves to T21 with the other resolved-geometry checks, or T19 loses its dependency on T10 (it cannot — painter's order needs integer layers). The former.

### B3. T14 and T15 both build the `measure` verb, with no edge

ADR-0024: *"`measure` gains the fitted-extent output — no new verb."* T14 ships that output (blocked by 3, 4a); T15 ships `measure` (blocked by 4a). Neither blocks the other and neither names who lands the verb's adapter surface, its result shape, or the element-kind dispatch ADR-0024's Consequences describe. This is the prior court's A10 shape again — one surface, two tickets, no edge — on a new verb. Fix: T14 owns the *arithmetic* (a function `validate` calls for the fit-deviation error, story 52's `E-*`), T15 owns the verb, and T15 gains ← 14 for the raster branch. T20 already lists both, which is the tell.

### B4. The permissive parser is double-owned by 4a and 5

4a's acceptance criterion — *"a lossless round-trip of a file containing an unknown key and a retired spelling"* — cannot be met by the `deny_unknown_fields` model; it needs the order-and-presence-preserving path. T5 then says it *"Includes the permissive, order-and-presence-preserving parse path ADR-0042 forces."* Two tickets each believe they are writing the second parser. Decide: it is 4a's (the acceptance criterion is there), and 5 consumes it. This also answers the sizing question — see B8.

### B5. T12 is described against an ADR-0018 that does not exist; the edge to T10 is ceremonial

ADR-0018 pairs, *per `group`*, the **time-union** of the group's audio members against the time-union of its visual members, symmetrically. No rect is read; no layer is resolved; no geometry appears anywhere in the ADR — area-awareness is the thing it *rejected* (*"needs painter's-order resolution and per-element opacity … a much larger check"*). The redraw says *"a check about `group` and declared rects"* and blocks it on layer resolution. Both wrong. T12 ← 4a only, and the ticket text must say "time-union of audio vs visual within a group", or the implementer builds the rejected check. (Answers the author's Q3 second example: no, and it does not need rects either.)

### B6. T26 puts `loop` in `render`; ADR-0062 says the opposite

T26 lists *"the `loop` wrap"* under the audio mix and claims ADR-0062. ADR-0062's index row: *"purely `validate`-facing; the wrap seam reuses `R-SOURCE-CUT-POP` unchanged"*; its body: *"`render` and every other tool are unaffected"* and *"Montagent writes no container-level loop metadata."* A renderer that wraps audio at the seam is a format violation. `loop` is 4a (the field) and T21 (the check) and nowhere else. Similarly, T9 cites ADR-0062 for the structural time checks — nothing of 0062 is there.

### B7. T27 cites ADR-0021 for a budget ADR-0021 retired

T27: *"the 60 s-under-two-minutes budget"*, ADR-0021. ADR-0021 opens *"The original budget — a 60 s render under 2 minutes, a 10 s preview under 5 seconds — was written for 1080×1920/30 and never re-derived"*, and replaces it with two numbers: `<5 s` for proxy `preview` (enforced) and full-resolution *"observational only (unenforced)"*. `render` gets no number at all. Story 60 in #168 still says it; #168 says the ADR wins. The ticket either states story 60 as a story-only budget with no ADR behind it, or drops it.

### B8. Sizing — the field-order argument is half right, and the half that is wrong is the half that matters

ADR-0041 freezes *insertion*, not *delivery*: appending a new field at the end of a landed struct changes no existing file's canonical order (ADR-0068 appended `effects` exactly this way; ADR-0041 says the introducing ADR fixes each field's position). So the shape *could* be split by type family as long as every struct lands with its full field list. But I would not split it there, because 4a's real bulk is not nineteen ADRs of struct fields — those are a few hundred lines of `serde` declarations, one type per ADR row. **The bulk is the second parser** (B4): a lossless, unknown-key-tolerant, order-preserving representation and its round-trip against the typed one, which is a design problem in its own right and is currently claimed by two tickets. Split that out as 4c ← 4a, make 4b and 5 depend on it, and 4a becomes the typed shape + schema + drift test + the byte-for-byte fixture round-trip — which fits. The stated argument ("field order frozen ⇒ one piece") is a rationalisation for leaving the wrong thing whole.

### B9. Coverage against the index is claimed complete and is not reported honestly

Walking all 68 rows: **0003, 0022, 0027, 0037** are named by no ticket. All four are non-implementing (a scope rule, a relabelling, an exclusion, a "no tool ships"), and that is a fine answer — but the redraw says *"the check is that every row of the index has a ticket"* and does not record the four exclusions. #168's own rule for open questions is *record, don't decide*; a coverage claim that silently skips rows is the story-list audit failure in a new costume. ADR-0027 in particular is a testable exclusion (an `.svg` source is a decode failure, not a fit case) and deserves one line in T13.

### B10. A `validate` check parked behind the rasterizer and an open ADR ticket

T24 carries *"the check that a transition's derived range still matches the two elements it bridges"* (ADR-0059, story 92). That is a document-only check on three `start`/`end` pairs. T24 is *blocked by: 22, 23, #185* — the rasterizer, text drawing, and the `mask` parameter ADR. Move it to T9 (structural time) where it can ship the week 4a lands.

### B11. Ceremonial edges, in one place

| edge | why it is ceremonial |
| --- | --- |
| 11 → 9 | ADR-0054: `R-CAPTION-NO-AUDIO` is *"a project-wide interval query"* on declared times; ADR-0034: all four are *"no I/O"*. Nothing in T9 (overlap/gap/speed/quantization/slack) is consumed. Carried over from the first draft's 13 → 8, which Fable's ballot already called unneeded. **11 → 4a only.** (Answers the author's Q3 first example.) |
| 12 → 10 | B5. |
| 17 → 15 | B1. |
| 21 → 14 | ADR-0044: `R-OFF-CANVAS` uses the *declared* rect resolved across keyframes; ADR-0024 keeps `width`/`height` author-written. `R-SOURCE-CUT-POP`'s time-based-source term reads `source_start`/`source_end` off the element. Nothing in 21 reads a fitted extent. **21 → 19 only.** |
| 18 → 15 | ADR-0057's licence heuristic reads `name` table IDs 13/14 — a font *parser* (`read-fonts`/`skrifa`), not the layout engine or `measure`. A crate dependency, not a ticket dependency. |
| 16 → 15, for three of five | grapheme-cluster, invisible-character and mixed-normalization checks are Unicode, not layout — the redraw's own T11 principle, not applied. Glyph coverage and the font census do need the chain opened. Not worth a split; worth saying in the ticket. |

Two edges the redraw got right that the first draft did not and that survive attack: 22 ← 3 (the decode path, with the fixture-has-no-video note), and 25 ← 13 (`render` verifies sources independently, ADR-0056).

### B12. Smaller misstatements

- T1: *"the four non-severity categories (`NOT CHECKED`, `UNCHECKED`, `LAYOUT`)"* lists three. ADR-0041 says `LAYOUT` is a *fourth report category alongside `error`/`review`/`note`/`UNCHECKED`*.
- T4b: *"Nine spellings."* `CONTEXT.md` and the ADRs name at least eleven schema-error-naming-replacement spellings (`anchor`-string, `box`, `align`-on-image, `line`/`polygon`/`path`, `gravity`, `center-center`, `#RRGGBBFF`, `fit: none|fill`, `bold`/`weight`, `opacity`-on-audio → `volume`, bare `mask`). The number was carried from the first draft; ADR-0055 and ADR-0068 have added to the list since.
- T22: *"image resample through `clip`/`fit`"*. ADR-0015: *"`fit` never executes. No renderer reads it."* The renderer resamples to the declared rect and clips to `clip`. Small wording, but it is the mis-reading ADR-0015 predicts (*"two of eight agents read `fit` as a render instruction"*), in the ticket that writes the renderer.
- T25: *"all three check tickets block it."* There are now nine tickets that emit findings (4b, 9, 10, 11, 12, 13, 16, 17, 18, 21), three of which emit `error` and are not blockers (4b, 16, 18). If the registry (T1) is real, that is fine and the sentence is stale; if it is not, `render` ships without refusing on a retired spelling. Say which.
- The frontier omits 13 and 14, which open the moment 3 and 4a have both landed.

---

## Verdicts, per the brief's four prompts

1. **Edges — REFUTE.** Two tickets cannot finish with declared blockers done (T10 via ADR-0060; the `measure` verb across T14/T15). Five blockers are ceremonial (11 → 9, 12 → 10, 17 → 15, 21 → 14, 18 → 15). One prior double-ownership survives unchanged (A10) and two new ones were introduced (B3, B4).
2. **Coverage — ACCEPT WITH MODIFICATION.** 64 of 68 rows are claimed; the four unclaimed are legitimately non-implementing but must be recorded as such. Five ADRs are claimed and misdescribed: 0015, 0018, 0021, 0058, 0062.
3. **Reasoning behind absorbed corrections — REFUTE on both examples the author named.** 11 → 9 is carried over, not re-examined. 12 → 10 rests on a description of ADR-0018 that inverts it.
4. **Sizing — REFUTE the argument, not the ticket.** Field order constrains insertion, not delivery. The piece to remove from 4a is the permissive parser, which is double-owned with T5 regardless.

## The single most important thing still wrong

**The redraw corrected the graph from the court's conclusions rather than from the ADRs the court cited, and where the two differ it followed the wrong one.** Every new defect above has that shape: T17 blocks on the text engine because #186 says the height "depends on the font" (ADR-0058 says no I/O, and prints seven fixture elements already firing); T12 blocks on layers because coverage sounds spatial (ADR-0018 is a time-union per group); T26 loops audio because `loop` sounds like playback (ADR-0062 says `render` is unaffected); T27 keeps a budget because story 60 states it (ADR-0021 retired it); T10 bundles a tie check with the resolver it needs because both are "layer" things (ADR-0060 samples keyframes). The first court's headline was that the graph was drawn from verbs, not data. This one was drawn from the court's summaries, not the ADRs — the same error one level up, and the sharpest instance is the one the README singled out as *"Fable's reading governs"* being reversed for `R-BOX-SLACK` without anyone noticing it had been ruled on.
