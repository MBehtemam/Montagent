# Ballot — the redrawn v1 ticket breakdown (32 tickets, drawn from `docs/adr/README.md`)

Juror: Sonnet 5. Ruling alone, blind to any co-sitting juror. Verified against the working
tree at `main` (`73c09f1b`), `docs/adr/README.md` and the individual ADR files, the prior
court's four files under `docs/research/juries/v1-ticket-breakdown/`, and `gh issue view`
for #168, #185, #186. Default to refuted; I depart from that only where the text of the
cited ADR itself makes the redraw's claim checkable and true.

---

## Part 1 — Walking the first court's findings against the redraw

**Absorbed, and cleanly:**

- **Story 29 / `R-EASE-INERT` misfiled out of captions** — ticket 21 now carries it with an
  explicit note crediting all three jurors. Absorbed.
- **`probe` does not depend on the document model** — ticket 3 is `blocked by: 1` only.
  Absorbed.
- **The check registry / refuse-class mechanism was ownerless** (Opus) — ticket 1 now names
  it explicitly ("the first draft left this ownerless and two tickets would each have
  invented it") and carries ADR-0043's declaration. Absorbed.
- **Ticket 1's `Finding` type needs harder fixtures than `E-PARSE` alone** (Opus's
  counter-proposal) — ticket 1's note requires "a census-carrying, a refuse-class and a
  citation-carrying finding as test fixtures," near-verbatim. Absorbed.
- **The unknown-key/retired-spelling round-trip belongs at the document-model ticket, not
  `fmt`** (Opus) — ticket 4a's note states this explicitly, giving the ADR-0030/0042
  reasoning. Absorbed.
- **FFmpeg resolution belongs at the first subprocess-spawning tool (`probe`), not the
  packaging ticket four layers downstream** (Fable's headline finding, Opus's parallel
  finding) — ticket 3 now owns it explicitly. Absorbed, and it is the redraw's own stated
  headline correction.
- **`preview` needs the encoder, not `frame`** (Fable) — ticket 28 is `blocked by: 27`
  with an explicit note. Absorbed.
- **`compare` has no dependency on the keyframe resolver** (Opus's "20 → 11 is false";
  Fable's parallel finding) — ticket 31's note states this with the same ADR-0063/0066
  reasoning. Absorbed.
- **A `slack` primitive (ADR-0032) is consumed by two tickets and owned by none** (Opus) —
  ticket 9 now owns it explicitly, with the same "two tickets would implement it twice"
  language. Absorbed.
- **A layer/anchor resolution function is needed by the resolver and the rasterizer and
  owned by neither** (Fable's "11 needs 9, or 9's resolution must be prefactored out") —
  ticket 10 now owns it explicitly as "anchor → integer layer in one hop... because the
  keyframe resolver and the rasterizer both need painter's order and neither should own
  it." Absorbed.
- **`shift` returns a knowingly partial finding set at its point in the sequence** (Opus,
  Fable) — ticket 30's note states this near-verbatim. Absorbed.
- **Audio mixing has no owner** (unanimous escalated finding) — ticket 26 is new and
  explicitly credited ("had no story at all in #168's original 83"). Absorbed.
- **`fonts vendor`/`fonts list` has no story** (Opus's headline finding) — ticket 18 is new
  and explicitly credited. Absorbed.
- **The falsification test compares two typefaces and must be region-masked, not sold as
  golden** (Opus and Fable, escalated to #186) — ticket 23 is explicitly blocked by #186,
  region-masks to the non-text elements, states the self-confirming-vs-falsifying
  distinction Opus demanded, and names the font substitution. Absorbed, and well.
- **`R-BOX-SLACK` may fire on the fixture under Open Runde** (escalated finding, #186) —
  ticket 17 is explicitly blocked by #186 with the reasoning restated. Absorbed.
- **The `mask` self-contradiction in ADR-0040** (Fable, escalated to a separate ADR) — this
  was resolved outside ticketing by ADR-0068 (now on `main`, confirmed by reading it), and
  the redraw threads the fixture migration into ticket 4b as an advise-class retired
  spelling. Absorbed, correctly, and the redraw does not re-litigate what the ADR settled.
- **Whole-video comparison is not redundant with the frame test** (Fable's ruling, which the
  README records as governing) — ticket 29 survives as its own ticket with the same
  reasoning ("frames cannot falsify duration... or `speed: 0.645`"). Absorbed.

**Missed, or only partially absorbed:**

- **Ticket 13→10 (now 11→15, if you trace captions to the text engine): Fable's specific
  refutation was not absorbed at the ticket that replaced it, and a new, undefended edge
  was substituted for it.** The redraw correctly drops the caption→text-engine edge (good —
  see the credited note at ticket 11). But it then blocks ticket 11 on **ticket 9**
  (structural time checks and slack) instead. I read ADR-0034 and ADR-0054 in full: all
  four caption checks are stated by their own ADRs to be **"no I/O"** document reads —
  `R-CAPTION-PACE`/`-REPEAT-DURATION` over `runs`/`start`/`end`/`fps`, and ADR-0054 is
  explicit that `R-CAPTION-NO-AUDIO` is "a pure document-level check" needing only declared
  `start`/`end` values project-wide, and `R-CAPTION-MIN-DURATION` is a fixed 834 ms
  constant against `end - start`. None of the four needs `9`'s overlap-within-track logic,
  the exact-arithmetic `speed` invariant, `N-QUANTIZATION`, or the `slack` primitive ticket
  9 now owns. Fable's original ballot explicitly refuted the analogous edge ("13 → 8 is
  also unneeded: `R-CAPTION-NO-AUDIO` is a time-range intersection, not the overlap
  check"). The redraw's own prose gives no justification for `11 ← 9` beyond naming ADRs
  that ticket 9 *implements* (not that ticket 11 *needs*). **This reads as carried over
  without re-examination**, exactly the risk the brief names, and the correct edge is
  `11 ← 4a` only.

- **Ticket 12 (`R-VISUAL-GAP`) blocked by ticket 10 (layer/anchor resolution) is spurious,
  and it is worse than a re-examination failure — it contradicts what the first court's own
  juror said about this exact check, and the redraw's own description of the ADR is wrong.**
  I read ADR-0018 in full. It is a **purely temporal** check: for every `group` containing
  both an audio and a visual element, `validate` reports where either side's **time-union**
  is not covered by the other's. Nowhere does ADR-0018 mention a rect, a coordinate, a
  painter's-order tie, or anything spatial — it is `group` membership plus `start`/`end`,
  both already in the document at 4a. It needs no resolved layer. Yet the redraw's own
  ticket-12 text calls it "a check about `group` and declared rects, with no motion in it"
  — **"declared rects" is not in ADR-0018 at all**; the redraw (echoing language from
  Fable's original ballot, which used the same phrase for the *old* ticket 14 bundle that
  mixed `R-VISUAL-GAP` with genuinely spatial motion checks) has carried a stale
  characterization into the one ticket that no longer has any spatial checks bundled with
  it. Worse: Fable's original ballot explicitly recommended "31 belongs with 9" (the
  structural-checks ticket) — the redraw's ticket 12 does neither what Fable recommended
  nor what ADR-0018 actually requires; it invents a third, unsupported dependency on
  resolved layers. The correct edge is `12 ← 4a` (or fold it into ticket 9 as Fable
  suggested), not `12 ← 10`. **This is a newly-wrong edge, not merely an unabsorbed one** —
  see Part 3, item 3, and the closing finding.

- **Three tickets could not fit a context window (doc model, text engine, rasterizer) —
  absorbed for the rasterizer (split three ways: 22/23/24) and effectively absorbed for the
  text engine (its fit-extent half moved to ticket 14, its font-swap-census half to ticket
  16, leaving ticket 15 close to Fable's proposed "10a"), but not absorbed for the document
  model.** Ticket 4a is not split at all; see Part 3, item 4, below — the redraw's
  rationale addresses only *why 4b must come after 4a*, not the separate, unaddressed
  complaint (Opus and Sonnet both) that the *shape* portion alone (minus retired spellings)
  needs internal decomposition.

- **`fmt --check`/`LAYOUT` as one predicate implemented twice** (Fable's missing-edge #6:
  "4 and 9 implement one predicate twice... the order predicate lands once, in 3"). Ticket
  5 (`fmt`) and ticket 10 (layer/anchor + `LAYOUT` key-order check) are still two separate,
  mutually unblocked tickets in the redraw, exactly as Fable flagged. Not absorbed — no
  ticket states the order predicate is a single shared implementation, and 5 and 10 remain
  parallel/unlinked.

- **`fmt --check` and `validate`'s `LAYOUT` category, plus the render-gating question** —
  related to the above: Opus's finding that story 18's second half ("`LAYOUT` never gates
  `render`") is only verifiable at the render ticket is not restated anywhere in the
  redraw; ticket 25 doesn't mention `LAYOUT` at all. Not absorbed, low stakes.

---

## Part 2 — New coverage gaps against `docs/adr/README.md` (68 rows)

I walked every row. Nearly everything is claimed (see the ADR lists printed against each
ticket in the brief) — this redraw's coverage is meaningfully better than the first
draft's, and the explicit new tickets (18, 26) are real corrections. Two defects survive:

1. **ADR-0051's two `validate` checks are unclaimed by any ticket.** ADR-0051 mandates
   three deliverables: a `compare` fact (drift between text and an unchanged `highlight`),
   and **two new `error`-level `validate` checks** — highlight-window **containment**
   within the parent element's range, and **non-overlap** between sibling runs' highlight
   windows. The redraw's ticket 31 (`compare`) claims ADR-0051 and correctly delivers the
   `compare` half ("a run whose text changed while its `highlight` timing did not"). But no
   ticket claims the `validate` half. Ticket 24 mentions `highlight` only as a rendering
   feature (windows on runs, for the effects/paint ticket), never as a check. This is a
   genuine, checkable gap the redraw's own "the index is the authority" standard should
   have caught.

2. **Four ADRs are pure scope/meta decisions with no ticket and, on inspection, probably
   need none, but the redraw's stated coverage standard ("every row of the index has a
   ticket") does not actually hold**: ADR-0003 (scope statement — general editor, not
   channel tooling), ADR-0022 (relabels an existing example as hypothetical — no new
   behavior), ADR-0027 (vector sources out of scope — a negative decision), ADR-0037 (no
   authoring tool ships for derived-time signatures — also a negative decision). None of
   these are defects in the *plan* — there is no code to write for "we decided not to build
   X" — but the redraw asserts a stronger coverage claim than it delivers, and a literal
   walk of "every row" turns up these four unclaimed rows. Low severity; flagging because
   the brief asks for the walk to be literal.

Everything else — including the format's previously-unstoried vocabulary (effects, colour
filters, transitions, `volume`, `loop`, `rect`/`ellipse`, `highlight`, required unique
`id`s) that was the first court's headline finding — is now named against a ticket, mostly
inside 4a and 24.

---

## Part 3 — Attacking the redraw on its own terms

### 1. Did splitting tickets introduce edges that are now missing?

Yes, one concrete instance beyond the caption/visual-gap edges already covered in Part 1:

**Ticket 25 (`render`) is blocked by `22, 9, 10, 13` — but ADR-0006 states `render` runs
"the identical check engine" `validate` does, refusing on any `error`.** By the time ticket
25 is drafted, `error`-producing checks also live in ticket 4b (retired spellings —
`gravity` is refuse-class `error`), ticket 11 (caption checks, though all `review`), ticket
12 (`R-VISUAL-GAP`, `note`/`review` depending on read), ticket 16 (glyph-coverage `error`),
ticket 17 (`R-BOX-SLACK`, `note`), and ticket 21 (contains no `error`-class check per the
ADR list, all `review`). Of these, **ticket 4b and ticket 16 both produce `error`-severity
findings that ADR-0006 requires `render`'s check gate to refuse on**, and neither is in
ticket 25's blocker list. Either `render` ships with a check gate narrower than `validate`'s
(a real defect ADR-0006 forbids), or the blocker list is incomplete. The ticket's own
description ("the identical check engine `validate` runs") claims the stronger, correct
property but the graph doesn't enforce it.

### 2. Is coverage actually complete against `docs/adr/README.md`?

Mostly, with the exceptions in Part 2: ADR-0051's `validate`-side checks are dropped, and
four scope/meta ADRs are technically unclaimed (almost certainly requiring no ticket, but
the redraw's own stated standard doesn't literally hold).

### 3. Ticket 11 and ticket 12 — the two edges the author flagged as unsure

Both are answered in Part 1 above, and both resolve the same way: **the redraw's fix
address the symptom the first court named (which ticket the caption checks/the visual-gap
check are bundled with) but not the underlying question (what data each check actually
reads), and in ticket 12's case the redraw's own prose misdescribes the governing ADR as
spatial when it is temporal.** Ticket 11 should be `blocked by: 4a` only. Ticket 12 should
be `blocked by: 4a` only (or folded into ticket 9, as Fable originally suggested) — not
`10`.

### 4. Sizing — is the field-order-freeze argument for leaving ticket 4a whole correct?

**The argument is valid for what it actually proves, and invalid for what the ticket uses
it to justify.** ADR-0041 (Rust struct field order is canonical key order) is a real reason
the *shape* of every type must be frozen before dependents start reading fields from it —
that is why 4b (retired spellings, a *consumer* of the frozen shape) correctly comes after
4a. But that is an argument for **sequencing**, not for **ticket boundaries within the
sequence**. Nothing about field-order freezing prevents landing the nineteen ADRs' worth of
shape across two or three PRs merged back-to-back before 4b or any other ticket opens — the
field order is frozen the moment the last of those PRs lands, exactly as it would be frozen
after one PR. Opus and Sonnet's original splits (header+track+common fields; visual
vocabulary; text/fonts/highlighting) are splits *along this exact axis* and would not
reopen the freeze once landed. The redraw's own reasoning proves 4a must be *sequenced*
before 4b — it does not prove 4a must be *one ticket*. As written, it is a rationalization:
the real reason it stayed whole looks like unwillingness to re-derive Opus/Sonnet's split
against the new nineteen-ADR inventory, not a consequence of ADR-0041.

---

## The single most important thing still wrong

**Ticket 12 (`R-VISUAL-GAP`) is blocked on resolved layers (ticket 10) for a check that
ADR-0018 defines as purely temporal — group membership and declared `start`/`end` values,
nothing spatial, nothing resolved.** This is the sharpest instance of the redraw's own
stated failure mode (a correction applied too literally) turned inside-out: the redraw
correctly diagnosed that the *first* draft's graph was drawn from the verb table rather
than the data, fixed nearly every other instance of that error, and then reproduced it here
by inheriting stale "declared rects" language from a juror's ballot about a *different*,
now-dissolved ticket bundle, rather than re-reading ADR-0018 itself. The first court's own
juror (Fable) had already named the correct home for this check ("31 belongs with 9"); the
redraw neither took that recommendation nor re-derived the dependency from the ADR text —
it invented a third, spurious edge to a ticket about geometric painter's order that the
check never reads. It is a small edge in isolation, but it is the one place in this redraw
where re-checking against the primary source (the ADR, not a prior ballot's shorthand)
would have caught what re-checking against the prior court's verdict alone did not.
