# Jury: what does the sheet owe a reader too weak to use it?

One panel of three jurors, three different models, put to
[#476](https://github.com/MBehtemam/Montagent/issues/476) (*Decide what the sheet owes a reader
too weak to use it*), on the map [#395](https://github.com/MBehtemam/Montagent/issues/395).
Resolved as
[ADR-0114](../../../adr/0114-the-sheet-carries-a-reader-check-a-handshake-on-its-first-label-that-names-no-reader.md).

Every juror received [BRIEF.md](BRIEF.md) **verbatim and identical**, apart from their juror
number. They got no assigned stance, no persona, no sight of each other's ballots, and none of
the judge's recommendations. Jurors were dispatched in parallel. Ballots are recorded here
unedited.

| | Ballot | Part 1: is it this map's problem? | Part 2: name a model? | Part 3: measure first? |
| --- | --- | --- | --- | --- |
| Juror 1 | [Fable 5.1](fable-5-1.md) | (c), outside `blind_to`; every tile's label string in JSON | No | (i) decide now, measure later |
| Juror 2 | [Opus 5.5](opus-5-5.md) | (c), outside `blind_to`; the text is the authority | No | (iii) measurement gates shipping |
| Juror 3 | [Sonnet 5.5](sonnet-5-5.md) | (c), outside `blind_to`; a handshake that names the gutter | Only in the tool description, capability wording, no names | (iii) measurement gates shipping |

The human took the judge's read, which was:

- **Part 1**: all three repairs compose. The judge's own draft self-check (*"if you cannot read
  it…"*) asked the reader to grade itself, and all three jurors found independently that a
  reader that fails confidently grades itself confidently. So the line names **where** the label
  is (Juror 3), quotes it **exactly** so the check is a comparison, not an affirmation (all
  three), says the text is the **complete record** and the sheet a picture of it (Juror 2), and
  carries **every tile's label string in the JSON** so a second reader can check the first
  (Juror 1). Juror 2's imperative — *"do not report visual findings from this sheet"* — is **not
  adopted**: Montagent's output states facts and names next calls, and has never told a reader
  what not to do.
- **Part 2**: no model name anywhere `frame` prints. Juror 3's "only in the tool description"
  still allowed no names there, so the panel agreed on substance.
- **Part 3**: the real 2–1 split, settled by an option none of the jurors was offered: the
  measurement is a shipping condition for **the reader check alone**, not for the range mode.
  Juror 1's point that a measurement can only reword the line stands, and so does Jurors 2 and
  3's point that an untested defence line is an argument rather than an artifact.

## A correction to the brief

The brief says `blind_to` has **five** tokens. ADR-0105 §6 lists **six**: the brief omitted
`motion` (*"A sheet is stills; whether motion looks right is `preview`'s question."*). No ballot
depends on the count or on `motion`, so the panel was not re-run.
