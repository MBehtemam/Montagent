# Ballot — Opus 5 (1M) — the v1 ticket breakdown, redrawn

**Verdict: REFUTED.**

The redraw absorbed most of the first court's *structural* corrections and its method change is
real — the graph is genuinely layered now, and `probe`, `preview`, the cut list and `compare`
all sit where the data says they should. But the author re-derived the new edges from the first
draft's prose rather than from the ADRs, and the result is a second draft that is wrong in a
new place each time it reasons about a check's *inputs*. Three of the four edges it argues for
explicitly are spurious, two load-bearing edges are missing, and two tickets misdescribe the
ADR they name.

Verified against the working tree at `73c09f1b` plus `main`: `docs/adr/README.md` (68 rows,
0001–0068), ADR-0007, 0010, 0015, 0018, 0020, 0024, 0027, 0028, 0032, 0034, 0041, 0054, 0058,
0065, 0067, 0068; the committed fixture; `gh issue view 168` (now 113 stories — the brief's
claim holds), `gh issue view 185`, `gh issue view 186`; the three prior ballots and their README.

---

## Part A — the first court's findings, one by one

### Unanimous

| finding | status |
| --- | --- |
| Per-check tickets are genuinely vertical (the invited attack failed) | upheld, not a correction |
| Story 29 / `R-EASE-INERT` is misfiled | **absorbed** — ticket 21, with the misfiling named |
| `probe` does not depend on the document model | **absorbed** — ticket 3 ← 1, and FFmpeg resolution moved in with it |
| Three tickets cannot fit one context window: document model, text engine, rasterizer | **two absorbed, one missed** |

On the third: the text engine became four tickets (15, 16, 17, 18) and the rasterizer three
(22, 23, 24). The document model **grew**. Ticket 3 in the first draft was sized against two
stories; ticket 4a now names nineteen ADRs and is still one ticket. That is not a split, it is
the same finding with a longer justification attached. See Part C, question 4 — and note that
4a's list is itself an undercount (Part B, N7).

### Where the court split

**The caption-to-text-engine edge.** The README records Fable's reading as governing: ADR-0058
computes `R-BOX-SLACK` *"with no I/O"* and where the code lives is a packaging choice. The
caption checks (ticket 11) were correctly freed. But `R-BOX-SLACK` was pulled into its own
ticket 17 and re-blocked on the text engine — the same edge the court killed, restored under a
new number. **Missed, and inverted.** (N2.)

Fable's second half of that finding — *"13 → 8 is also unneeded: `R-CAPTION-NO-AUDIO` is a
time-range intersection, not the overlap check"* — is **missed**: ticket 11 is still blocked
by the time-checks ticket. (N8.)

**Whether the whole-video comparison is a ticket.** Fable's refutation of my own merge proposal
held, and the redraw keeps it as ticket 29 with the right reasoning. **Absorbed** — though the
ticket as drawn cannot run (N5).

**The wide-refactor exception on the document model.** Correctly not invoked; no expand–contract
sequencing appears. Both jurors' mitigations — Fable's "freeze field order because ADR-0041
makes it format bytes", my "specify the type against its hardest downstream consumers" — appear
as the two Notes on 4a and on ticket 1. **Absorbed.**

### The three headline findings

**Opus — sized against the story list, not the ADR series. Absorbed in method, overreached in
claim.** Every ticket now names ADRs; #168 was amended to 113 stories; the three previously
unowned bodies of work (audio, `fonts`, slack) have owners. But the redraw states *"every row
of the index has a ticket"* and that is false for four rows (N9), and true-at-row-level while
false-at-obligation-level for four more (N7).

**Fable — the graph was drawn from the verb table.** Absorbed as a principle and delivered for
the spine. The residue is that the *leaves* were never re-derived: 11 ← 9, 12 ← 10, 17 ← 15 and
17 ← #186 are all first-draft reasoning carried forward, and 25 and 29 are missing edges of
exactly the kind Fable named.

**Sonnet — a numeric audit passes clean because the story is claimed by the wrong ticket; every
other ticket deserves the same second pass.** **Missed as a generalisation.** The redraw does
the second pass for `R-EASE-INERT` and stops. `R-VISUAL-GAP` (N1) and `R-BOX-SLACK` (N2) are
the identical failure: a check claimed by a ticket whose author had not read the ADR's own
description of what the check consumes.

### The two escalated defects, and the gap

| | status |
| --- | --- |
| ADR-0040's self-contradiction on `mask` | **absorbed** — ADR-0068 is on `main`, the fixture migrated in `9f2431c7`, 4a's round-trip demo is now true |
| The falsification compares two typefaces | **absorbed** at ticket 23 (region-masked, non-gating text arm, substitution named) — and **over-applied** at ticket 17 (N2) |
| Nothing owns audio mixing | **absorbed** — ticket 26, with #168 stories 93–99 |

### Findings from individual ballots that did not make the README

| finding | status |
| --- | --- |
| Fable: `fmt --check` and `LAYOUT` are **one predicate in two tickets**; prefactor it into the model ticket | **missed** — ticket 5 owns `fmt`'s canonical convention, ticket 10 owns "`LAYOUT` key order checked unconditionally", neither blocks the other and 4a claims only "field order frozen" |
| Fable: story 53 (*"the renderer opens nothing outside the declared font chain"*) is a **renderer** property, observable only at the rasterizer | **missed** — still inside ticket 15, the `measure` ticket, where it cannot be demonstrated |
| Opus: ADR-0010's **two-arm `rust-rasterizer` oracle** (*"not optional"*, and #36 asks for it on every bump) has no owner | **missed** — named in neither ticket 2 nor 23 |
| Opus: stories 49/60 (`frame` < 500 ms, 60 s render < 2 min) have **no CI home** | **missed** — the budgets are stated as criteria on 22 and 27; ticket 2 still owns only the matrix and the canary |
| Opus: story 42 (*every tool fails identically on a malformed file*) is an unstated inherited criterion on every later verb ticket | **missed** |
| Opus: the schema and format-docs MCP resources are static the moment the schema is generated and need not wait on `fmt` | **partially absorbed** — ticket 6 says so in prose and keeps the 6 ← 5 edge anyway |
| Opus: ticket 1 is itself over-scoped, split 1a/1b | **not taken** (non-unanimous; but note the redraw *added* the check registry to it) |
| Fable: merge `create_project` into `fmt` | **not taken** (non-unanimous, defensible) |

---

## Part B — what is newly wrong

### N1. Ticket 12 misdescribes ADR-0018, and the blocker follows from the misdescription

The redraw: *"`R-VISUAL-GAP` … a check about `group` and **declared rects**, with no motion in
it"*, blocked by 4a and **10** (layer and anchor resolution).

ADR-0018 has no rects in it. The Decision is a **temporal** predicate:

> For every `group` whose members include both an audio element and a visual element,
> `validate` reports where either side's time-union is not covered by the other's — symmetric.

and area-awareness was considered and **rejected** on the fixture, for four stated reasons —
one of which is precisely *"real occlusion exists … so a correct area answer needs painter's-
order resolution and per-element opacity"*. The ADR rejected the check that would have needed
ticket 10. The redraw has restored the rejected reading as the ticket's description and then
drawn the dependency that reading implies.

Two consequences: **12 ← 10 is spurious**, and an implementer opening ticket 12 will build a
geometric coverage check the ADR explicitly refused. The real inputs are declared `start`/`end`
and `group` — the check is a sibling of ticket 9's material, not ticket 10's.

*(For the record: my own first ballot's "14 → 6, because `R-VISUAL-GAP`'s coverage needs the
fitted extent" was wrong for the same reason. The redraw did not inherit that error; it
arrived at an equivalent one independently.)*

### N2. Ticket 17's two blockers are both wrong, and the second rests on a false premise

`17. R-BOX-SLACK — blocked by: 15, #186.`

ADR-0058, Consequences, verbatim: *"`validate` gains `R-BOX-SLACK` … computed from `size`,
`line_height`, `runs` and `height` **with no I/O**."* ADR-0028's formula is
`height = (size × line_height×10 × line_count + 9) // 10`. There is no font term in it, and
there cannot be: ADR-0007 forbids auto-wrap and ADR-0008 makes `\n` the only break mechanism,
so `line_count` is author-written too.

- **17 ← 15 is spurious.** This is the exact edge the first court struck, restored.
- **17 ← #186 is spurious, and #186's stated reason is false.** #186 asserts *"The computed
  height depends on the font."* It does not. ADR-0058 re-derived the whole table against the
  **committed** fixture, enumerated which seven elements fire (`sentence-05/06/07/08/quiz`,
  `handle-text`, `chip-text`, slack 26–108) and which fifteen read exactly 0, and was accepted
  with them firing. The check firing on the fixture is already sanctioned by an accepted ADR;
  there is nothing for #186 to unblock. The font swap moves `measure`'s *width*, which
  ADR-0058 leaves parked as `UNCHECKED` and this ticket does not touch.

Ticket 17 can open the day 4a lands. As drawn it is stalled behind the text engine and an open
prerequisite for a dependency that does not exist.

### N3. Ticket 24's #185 blocker is over-applied

`24. Effects, colour filters, transitions and highlight — blocked by: 22, 23, #185.` Reason
given: *"the `mask` member's explicit parameters do not exist yet; ADR-0068 fixed only the
param-less form."*

ADR-0068 Consequences: *"The `mask` member's full parameter set is **graduated to its own
ticket**, not decided here. **Until it lands, the param-less form is the only spelling.**"*
#185 is out of v1's scope by its own framing (*"not this spec's to decide"*, #168 story 90a).
The fixture's instance is param-less — `{"name":"mask","shape":"circle"}`, where `shape` is
ADR-0040's union discriminator, not a geometry parameter.

So v1 needs nothing from #185, and the redraw's own ticket 4a already ships the `mask` member
in the schema. Blocking `blur`, `shadow`, the four colour scalars, `crossfade` and `highlight`
on a deferred geometry vocabulary is a self-inflicted stall on the largest unstoried body of
work in the format. This is the clearest instance of the failure the brief predicted: a
correction applied too literally.

### N4. Ticket 25's blocker set is a first-draft leftover, and `render` does not run the engine it claims

`25. render, video only — blocked by: 22, 9, 10, 13. … the identical check engine validate
runs, refusing on any error — this is the enforcement, and it is why **all three check tickets**
block it.`

"All three check tickets" is the first draft's world (tickets 7, 8, 9). The redraw has roughly
ten, and at least four of them own `error`-class findings `render` must refuse on:

- **4b** — the nine retired spellings, each a schema error
- **16** — glyph coverage across the font chain, `error` (ADR-0007, story 101)
- **18** — the font-attestation `error` on a hash mismatch (ADR-0057, story 111)
- **24** — a transition whose derived range no longer matches the elements it bridges
  (ADR-0059, story 92)

None of them blocks 25. As drawn, `render` refuses on a subset of the errors `validate` emits,
which is the one property ADR-0006 makes load-bearing.

### N5. Ticket 29 — the only test capable of falsifying the format — cannot render the fixture

`29 ← 26 ← 25 ← 22 ← {19, 3}`. **Ticket 23 (text drawing) and ticket 24 (effects) are nowhere
in that chain.** The committed fixture is 22 text elements out of 60 and carries the one `mask`
effect in the repository. Ticket 29 as drawn renders the fixture with every caption, card,
badge and title blank, no mask, and compares that to `reference/en-halloween-decorating.mp4`.

The same gap makes ticket 25's own demo ambiguous: "video only" reads as "no audio", but the
graph also delivers "no text".

**29 ← 26, 23, 24** is the honest edge set, and `render`'s first demo should name a fixture it
can actually draw.

### N6. Ticket 22 misdescribes ADR-0015

`22. … Image resample through clip/fit …`

ADR-0015: *"Follow that through and `fit` **never executes**. No renderer reads it … its only
consumer is `validate` (and `render`'s copy of those checks)."* ADR-0024 keeps `width`/`height`
*"required and author-written"*. The rasterizer resamples to the declared extent under `clip`;
`fit` is a claim `validate` checks, not a layout mode the renderer applies. This is the sentence
that would make an implementer build the layout mode the format was designed to refuse.

### N7. Ticket 4a's nineteen ADRs are an undercount, and "field order frozen" is false as drawn

4a does not name ADR-0007, ADR-0020, ADR-0028 or ADR-0015/0026 — all of which carry **schema**
obligations on the types 4a freezes:

- **ADR-0007** *is* the `text` element's schema: the `runs` array (always an array), literal
  `size`, `line_height`, `align`, `font`-by-name, and *"element-level `text` is a schema error
  with a message naming `runs`"*. The redraw assigns 0007 to tickets 15, 16 and 23 — all
  blocked by 4a.
- **ADR-0028** restricts `line_height` to tenths. Assigned to ticket 15.
- **ADR-0020** defines `speed` and `overrun` and makes `speed: 0` and negative `speed` schema
  errors. Assigned to tickets 9 and 26.
- **ADR-0015/0026** fix `fit`'s value set and its required-on-raster-source rule. Assigned to 14.

Under the redraw's own argument — ADR-0041 makes struct field order the canonical key order, so
a later ticket adding a field is a **format change visible in `fmt` and `LAYOUT` on every
file** — those four tickets would each have to reopen a frozen struct. The argument that 4a
must land whole is being used to justify a ticket that does not in fact contain the whole shape.

### N8. Ticket 11 ← 9 is ceremonial

ADR-0054 on `R-CAPTION-NO-AUDIO`: *"whether any audio element overlaps a text element's time
range is answerable from declared `start`/`end`"* and it is *"a project-wide interval query, no
track-name restriction"*. `R-CAPTION-MIN-DURATION` is `end − start < 834`. `R-CAPTION-PACE` and
`-REPEAT-DURATION` are per-element (ADR-0034). None consumes ticket 9's overlap rule, gap
reporting, exact `speed` invariant, `N-QUANTIZATION`, or `slack`. **11 ← 4a.** This is the
half of Fable's finding the redraw dropped.

### N9. The coverage claim overreaches

The redraw sets itself the standard *"every row of the index has a ticket"*. Walking all 68
rows: **0003, 0022, 0027, 0037 are named by no ticket.** Three are genuine no-ops
(0022 *"No decision changes"*; 0037 *"No authoring tool ships"*; 0003 is the scope frame, cited
only in a note). ADR-0027 has one live consequence — *"`probe` need not define a 'dimensionless
source' report category; every source Montagent accepts has decodable pixel dimensions"* — which
belongs to ticket 3 and is unnamed there. Small in itself; it matters because the redraw claims
the walk came out clean and it did not.

Stories 84–113 fare better: all are owned except **story 96** (`opacity` on an audio element is
a schema error naming `volume`), which 4b's "nine spellings" does not obviously include and
ticket 26 does not claim.

### N10. Two small misstatements in ticket 1 and ticket 22

- Ticket 1: *"the four non-severity categories (`NOT CHECKED`, `UNCHECKED`, `LAYOUT`)"* — names
  three, and one of them is a report block, not a category. ADR-0041: `validate` gains *"a
  fourth report category, `LAYOUT`, alongside `error`/`review`/`note`/`UNCHECKED`"*.
- Ticket 22 cites **ADR-0009** (the Rust host, the stdio MCP binding, one subprocess per
  session). That ADR belongs to ticket 1, which builds both adapters and cites it nowhere.

---

## Part C — the four questions the brief asks directly

**1. Did splitting introduce missing edges, and are any blockers ceremonial?**
Missing: **29 ← 23, 24** (N5) and **25 ← 4b, 16, 18, 24** (N4). Ceremonial: **11 ← 9** (N8),
**12 ← 10** (N1), **17 ← 15** and **17 ← #186** (N2), **24 ← #185** (N3). The claim that the
graph is drawn from data dependencies holds for the spine and fails at every leaf the author
argued about in prose.

**2. Is coverage complete against the index?** No — N9 at row level, N7 at obligation level.
The row-level miss is trivial; the obligation-level one is not, because it lands on the ticket
the redraw says must not be split.

**3. Are ticket 11's and ticket 12's blockers real?** Ticket 11 ← 9: **no**, carried over
unexamined (N8). Ticket 12 ← 10: **no**, and worse than carried over — the redraw invented a
new justification for it by misreading ADR-0018 as a rect check (N1). The author was right to
be unsure about both.

**4. Is ticket 4a's "leave it whole" argument right?** The argument is sound and the conclusion
does not follow. ADR-0041 does make a later field addition a format change, so the *declaration
order* genuinely cannot be landed incrementally. But that constrains only the **declaration**:
the types, their field order, `deny_unknown_fields`, schema generation and the drift test. It
says nothing about per-type validation semantics, the `Option<T>` presence discipline, the
retired-spelling table, or the round-trip harness, none of which reorders a field. The split
that respects ADR-0041 is *declaration vs. semantics*, not *type A vs. type B*. And as N7
shows, 4a as written does not contain the whole declaration anyway — four ADRs' worth of fields
are parked in tickets that would have to reopen it. So: right about why the naive split is
forbidden, wrong that the ticket therefore fits, and not currently doing the thing the argument
requires.

---

## The single most important thing still wrong

**The encode arm is drawn behind a `render` that draws almost nothing and checks almost
nothing.** Ticket 29 is, by #168's own words, the only test capable of falsifying the format,
and the only test of ticket 26; it is blocked on 26 → 25 → 22, a chain that contains no text
drawing and no effects, so it would render the fixture's 22 text elements and its one `mask` as
absent and compare that to the published MP4. The same missing-edge class leaves ticket 25
claiming to run *"the identical check engine"* while blocked on three of the ten tickets that
produce `error`-class findings — so the format's central enforcement property is asserted in
prose and contradicted by the graph.

Both are the first court's headline finding reappearing at the far end of the timeline: the
author redrew the spine from data dependencies and then let the last five tickets keep the
first draft's edges. The fix is small — **29 ← 26, 23, 24** and **25 ← 22, 23, 4b, 9, 10, 13,
16, 18, 24** — but until it is made, the redraw's deepest claim, that the graph is sequenced by
what each ticket consumes, is false exactly where it matters most.
