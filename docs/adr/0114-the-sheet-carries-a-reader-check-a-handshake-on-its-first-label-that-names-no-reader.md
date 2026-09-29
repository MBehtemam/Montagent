---
status: accepted
amends: 0094 (section 6's unconditional disclosure gains a member that is about the reader rather than the rule — the reader check — and the per-tile provenance gains each tile's exact rendered label string), 0097 (the plain-text form carries the reader check ahead of the provenance list, and `frame`'s tool description states the same fact in the same words, promises no reader that the sheet is legible, and names no model), 0105 (section 6's `blind_to` stays about the rule — the reader is not a blind spot and gets no token; the reader check is a sibling of the NOT CHECKED block, never a finding)
---

# The sheet carries a reader check: a handshake on its first label that names no reader

[#476](https://github.com/MBehtemam/Montagent/issues/476), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Three questions were put to a jury of
three models, Fable 5.1, Opus 5.5 and Sonnet 5.5, and the human took the judge's read of the
ballots. The ballots are verbatim in
[`docs/research/juries/contact-sheet-weak-reader/`](../research/juries/contact-sheet-weak-reader/README.md).
The panel was unanimous on Part 1 and agreed in substance on Part 2. It split 2–1 on Part 3,
which was settled by an option the judge added after the ballots.

## What was found

[#422](https://github.com/MBehtemam/Montagent/issues/422) re-measured ADR-0095's 140 px width
floor and ADR-0098's 8 px type floor with cold observers. It made 42 readings, each showing one
image with no text and no defect list
([`docs/research/cold-observer-floors/`](https://github.com/MBehtemam/Montagent/pull/475)). Both
floors hold for a capable reader: Sonnet found all three planted defects down to 120 px and Opus
down to 92 px, and both transcribed 8 px labels exactly. **Haiku 4.5 found none of them at any
width, including the 184 px target.** On the clean control it invented defects: a duplicate "5",
an umbrella scene, a dreamcatcher. On every label sheet, even at 10 px, it transcribed the
captions inside the video frames and never found the label strip.

That is this map's founding failure moved into the reader. The trial's blind sampler reported
silence as coverage, and ADR-0094 §6's defence was to disclose what the **rule** cannot see.
Nothing in the answer disclosed what the **reader** might not see, and Montagent cannot know
which model is calling.

## Decision

### 1. The weak reader is this map's problem, and the answer owes it a disclosure

The destination asks for an artifact honest enough that *"I checked the whole thing"* is an
artifact rather than an argument. A weak reader's confident report is an argument in the
artifact's clothes, and nobody who sees only the transcript can tell it from a real reading.
Montagent cannot make that reader see. It can make the reader's failure to see **detectable**:
by the reader if it is honest, and by whoever reads its report if it is not. *(Part 1 —
unanimous.)*

Two positions were rejected. **Out of scope** leaves the founding failure in place one step
further along. The caller who chose the model does not know that the model cannot see the sheet.
**Spec fact only** records the truth where no caller will meet it. The ADR-0101 incident showed
that a reader acts on what is in the answer and in the tool description, and an ADR note is in
neither. The spec facts still stand: PR #475 adds notes to ADR-0095 and ADR-0098 that both floors
are reader-conditional. They are necessary, but they are not enough.

### 2. The disclosure is a reader check, and it is not a `blind_to` token

A **blind spot** is a property of the rule. It is constant and learnable, and it never varies
with the document (ADR-0105 §6). What the reader can see is none of those things, so it gets no
token. Putting it in `blind_to` would make that block a place where any caveat can go. The reader
check is a **sibling** of the NOT CHECKED block: printed unconditionally on every range answer,
including perfect ones, never a finding, never suppressible, and judging nothing about the
video. *(Part 1 — unanimous.)*

### 3. It is a handshake: it compares, never asks the reader to grade itself

The first draft, the judge's, asked the reader to grade itself: *"if you cannot read it, this
sheet is below what you can see."* All three jurors found the same hole independently. **A reader
that fails confidently grades itself confidently.** Haiku read captions as if they were labels,
and would likely "confirm" a label it never found. The check therefore combines three repairs,
one from each juror:

- **It names where the label is**: the strip beneath each tile, outside the video frame. The
  text inside a tile belongs to the video. This targets the one way the weak reader was seen to
  fail. *(Juror 3.)*
- **It quotes tile 1's label exactly**, so the check is a comparison against a string and not an
  affirmation. **It says the text is the complete record**: the provenance list is the whole
  range and the sheet is a picture of it, because text is the one channel every reader can read.
  *(Juror 2.)*
- **Every tile's exact rendered label string is in the JSON**, as a `label` field on its
  provenance entry. A supervising agent or a human can then check the reader mechanically,
  whether or not the reader was honest. *(Juror 1.)*

The plain-text form prints it **directly after the header and before the provenance list**,
where a reader that stops early still meets it. It reads, with tile 1's label in the one slot:

> **READER CHECK.** Each tile's label is the line in the strip beneath it, outside the video
> frame; text inside a tile is the video's own. Tile 1's label reads exactly `<tile 1's label>`.
> The provenance list below is the complete record of this range, and this sheet is a picture of
> it. If the strip beneath tile 1 does not read exactly that, this sheet is below what you can
> see, and `frame --at <instant>` shows any listed instant at full scale. Reading the labels is
> necessary for seeing the pictures, not sufficient.

The JSON carries `reader_check: { "tile": 1, "label": "<exact string>", "sentence": "<the
rendered sentence>" }`. Like the `blind_to` sentences, the wording may be tightened when it is
built, but its four parts may not be dropped: where the label is, the exact string, which channel
is the record, and the next call.

The last sentence is required. It is Juror 3's warning that the handshake tests **label**
legibility, and a reader that passes it has not been shown to see the pictures. Sonnet's picture
floor and its label floor are different measurements. The check must not claim more than it
tests.

**Juror 2's imperative is not adopted.** Its proposed wording said *"do not report visual
findings from this sheet."* Until now Montagent's output has stated facts and named next calls.
It has never told a reader what not to do, and one sentence is not the place to start. The same
point is made as a fact: *"this sheet is below what you can see."*

### 4. Nothing `frame` prints names a model

This covers the answer text and the tool description alike. The only earlier binding to the host,
ADR-0095's *"the tier the API serves"*, named a documented and stable host property. A model's
ability to read a sheet is neither documented nor stable, and a model name goes out of date with
every release. A weak reader also cannot tell which class it belongs to, so a name is a worse
self-test than the handshake. The tool description is the worst place for a name, because it is
where a stale promise outranked a live fact in ADR-0101. The readers #422 measured are named
**only as dated evidence**: in this ADR, in the ADR-0095 and ADR-0098 notes, and in
`docs/research/`. *(Part 2 — two votes of No; the third, "only in the tool description", still
allowed no names there.)*

`frame`'s tool description states the reader check's fact once, in the same words. It does not
promise that a sheet is legible to its reader, and it does not describe the range mode as "one
call sees the whole span" without that qualification.

### 5. A measurement gates the reader check, not the range mode

The panel split 2–1. Juror 1 (Fable) argued that the measurement can only change the wording
and never the decision to disclose, so it should not gate anything. Jurors 2 and 3 (Opus,
Sonnet) argued that a defence line nobody has tried on the reader it targets is itself an
argument and not an artifact, so it should gate shipping. Both points stand. The judge proposed
an option none of the jurors was offered, and the human took it: **the measurement is a shipping
condition for the reader check alone.** The range mode ships on its own schedule. The reader
check ships once
[#481](https://github.com/MBehtemam/Montagent/issues/481) has shown it to cold readers.

This ADR records the prediction #481 tests, as Juror 1 asked:

> **A reader that cannot read the gutter labels fails the handshake visibly**: it quotes a
> different string or says it cannot find one. It does not confirm the quoted label.

If the prediction is falsified, the **wording** changes under an amendment to this ADR. The
obligation in §1 does not change.

## Consequences

- Every range answer gains `reader_check` in the JSON and a READER CHECK paragraph in the plain
  text, directly after the header. The paragraph is gated on #481.
- Every provenance entry gains `label`, the exact string drawn in that tile's gutter strip, after
  ADR-0098's sheet-wide elision. It is not gated: it is a plain fact about the answer, and a
  supervising reader needs it whether or not the reader check ships.
- `frame`'s tool description gains the reader check's fact, and must not promise legibility.
  This is in the same code site [#405](https://github.com/MBehtemam/Montagent/issues/405)
  already owns for the doc string's token figures.
- Tests: the paragraph and the JSON field appear on a perfect answer; tile 1's quoted string is
  byte-identical to the string drawn and to `provenance[0].label`; the tool description carries
  no model name.

## Costs, stated

- **One more fixed paragraph on every answer.** It is the price of a disclosure that reaches the
  reader where it acts. It also weakens slightly the austerity of an answer whose disclosures
  never vary, since this one has one slot that does.
- **It does not stop a confident liar.** A reader can copy the quoted string out of the text
  and claim a match. What the check guarantees is that such a lie is **checkable**: the exact
  string, the image and the reader's report are all on the record, and a second reader can
  compare them. That is as much as disclosure can do, and it is what this map promised.
- **Until #481 resolves, a range answer carries no reader check.** That is the state ADR-0094
  ratified, now with the floors' reader-conditionality recorded in the spec. If the range mode
  ships first, it ships with a known gap, and this ADR names the gap.
- **A new kind of disclosure, about the reader and not the rule.** It is kept to one paragraph
  and one JSON field, so it cannot grow into a second `blind_to`.
- **The ballots rest on a brief with one error.** It listed five `blind_to` tokens where ADR-0105
  §6 has six, omitting `motion`. No ballot depends on the count.

## Considered and rejected

- **A `reader` token in `blind_to`**: rejected unanimously, as above.
- **Refusing the call, or gating on the reader's identity**: Montagent cannot know the model, and
  it would be a verdict about the reader, the same kind of verdict this map rules out.
- **Naming the reader class that the floors were measured on** in the answer or the tool
  description: rejected by §4.
- **Deciding only after measuring**: this would block the ADR on a prototype whose design needs
  the ADR's wording. No juror chose it.
