---
status: accepted
amends: 0114 (section 5's prediction is recorded as falsified and the check ships under a narrower claim — it orients a reader that can see and catches a weak reader that reports honestly, and its pass is not evidence that the sheet was read; the Costs entry "such a lie is checkable" is corrected; the wording and the JSON are unchanged)
---

# The reader check orients a reader that can see, and its pass is not evidence of reading

[#481](https://github.com/MBehtemam/Montagent/issues/481), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). The measurement ADR-0114 §5 made a
shipping condition for the reader check. The evidence is in
[`docs/research/reader-check/`](../research/reader-check/FINDINGS.md). The human chose among three
options on that evidence.

## What was found

A mock plain-text range answer for the doctored fixture's whole-document call was shown cold, with
#422's 18-tile sheet at 184 px, to Haiku 4.5, Sonnet and Opus: 37 readings in four conditions.
The ticket's three were the full answer, the answer without the reader check, and the clean
control. A fourth was added: the full answer on a sheet whose tile 1 label is **drawn differently
from the string the check quotes**. It is the only condition where "it matches" is false, so it is
the only one that tells a reader that compares from one that copies.

- **ADR-0114's prediction is falsified.** On the mismatched sheet, 3 of 6 Haiku 4.5 readers
  confirmed the quoted string (*"all 18 tile labels match the provenance list exactly"*), 2
  reported a failure, and 1 handed the check back to the user. None quoted what is drawn. Asked
  afterwards, several said outright that they had never compared. **The weak reader does not fail
  the handshake visibly. It passes it in the handshake's own words.**
- **Capable readers compare.** All four Sonnet and Opus readers on the mismatched sheet quoted the
  drawn string exactly, called the check failed, and demoted their findings to leads. On true
  labels they passed 12 of 12 without friction.
- **The check costs a capable reader nothing.** With and without it they found the same defects,
  in the same proportions.
- **A disclosure is read when no tool description contradicts it.** All 22 capable readings cited
  one and acted on it. The weak reader read it too, and five of its readings used the `blind_to`
  list to explain the author's complaint away.

## Decision

### 1. The wording and the JSON are unchanged, and the check ships

ADR-0114 §3's paragraph, its four required parts, the `reader_check` JSON field and each provenance
entry's `label` stand as written. #481 was the shipping condition, and it has run.

No wording repairs what was measured. The weak reader copied the one exact string the text put in
front of it, whatever the sentences around that string said. Two rewordings were weighed and not
taken. **Stop quoting the string and ask the reader to report the strip**: the provenance lines
print every label, so a copier finds it one line lower, and a change that also stripped the labels
from the plain text is untested. Nothing shows this reader would transcribe the strip rather than
invent one. **Quote a different tile's label**: this moves the copy, it does not prevent it.

### 2. The claim narrows to what the check does

The reader check does two things, and ADR-0114's other claims are withdrawn:

- **It orients a reader that can see.** It says where the labels are and which channel is the
  record. That reader acts correctly on a failure: it re-reads, demotes its findings and names the
  next call.
- **It catches a weak reader that reports honestly.** A reader that says it cannot find or match
  the strip has been told the sheet is below what it can see, and where to go instead.

**Its pass is not evidence that the sheet was read.** In real use the drawn label always equals
the quoted one, so a reader that copies the string reports a true match. Neither the JSON nor a
second reader can tell that report from a real reading. A supervising agent or a human must not
treat *"reader check passed"* as a reason to trust a reader's findings, nor its absence of findings.

### 3. ADR-0114's Costs entry is corrected

ADR-0114 wrote that the check *"does not stop a confident liar"* but makes *"such a lie
checkable"*, because the string, the image and the report are all on the record. **That holds for
a false claim about a picture, not about the handshake.** A reader's findings about specific tiles
can be checked by a second reader against the image. A reader's claim to have matched the label
cannot, because the match is true. The confident liar was named as the rare case. It is about half
of the weak readings measured.

## Consequences

- ADR-0114's gate lifts: the READER CHECK paragraph ships with the range mode's answer.
- ADR-0114 and `CONTEXT.md`'s **Reader check** entry gain the narrowed claim.
- `frame`'s tool description, already owed ADR-0114's fact, must not describe a passed check as
  confirming that a sheet was read. It promises nothing about the reader, as ADR-0114 §4 already
  requires.
- No test changes. ADR-0114's tests (the paragraph and field on a perfect answer; byte identity of
  the quoted string, the drawn string and `provenance[0].label`; no model name) are unaffected.

## Costs, stated

- **A disclosure whose pass means less than it sounds.** "Reader check passed" reads like a
  certificate. It is kept anyway, because the reader it helps pays nothing for it and the reader it
  cannot help is no worse off than without it.
- **The weak reader is still unhandled.** Without the check, Haiku 4.5 invented defects on correct
  tiles (3 of 3). With it, Haiku mostly reported nothing wrong. It traded false positives for false
  negatives, and neither is a reading. Montagent still cannot make this reader see, and now cannot
  claim to detect it either.
- **The disclosure can serve as an alibi.** The `blind_to` sentences ADR-0094 §6 wrote so that
  silence would not read as coverage were used by the weak reader to explain a miss away. That is
  recorded as open fog on map #395, not decided here.
- **Six discriminating readings.** The 3 / 2 / 1 split is a direction, not a rate. One weak model
  was tested, with one kind of mismatch: the instant, not the id.

## Considered and rejected

- **Withdraw the reader check.** It helps a capable reader for free and tells an honest weak one
  where to go. Withdrawing it leaves ADR-0114 §1's obligation undischarged.
- **Strip the labels from the plain-text provenance lines** so that the check's string is the only
  one in the text. This is untested, and it costs every reader the per-tile attribution ADR-0097 §7
  put there.
- **Reword the check to warn against copying.** The measurement shows the copier does not act on
  the sentences around the string. This would also be the imperative ADR-0114 §3 declined.
