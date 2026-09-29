# The reader check, shown cold to the readers it targets

Prototype for [#481](https://github.com/MBehtemam/Montagent/issues/481), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). It tests the prediction
[ADR-0114](../../adr/0114-the-sheet-carries-a-reader-check-a-handshake-on-its-first-label-that-names-no-reader.md)
§5 records:

> A reader that cannot read the gutter labels fails the handshake **visibly**: it quotes a
> different string or says it cannot find one. It does not confirm the quoted label.

**The short answer.** **The prediction is falsified.** On a sheet whose tile 1 label was
deliberately drawn differently from the string the reader check quotes, 3 of 6 Haiku 4.5 readers
confirmed the quoted string: *"all 18 tile labels match the provenance list exactly"*. A fourth
handed the check back to the user. Every capable reader, 4 of 4, caught the mismatch. So the
check works as a handshake for readers that can see, and the weak reader passes it by copying the
text. That is the confident-liar case ADR-0114's *Costs* named, and it turned out to be the
typical case, not a rare one. The check also costs a capable reader nothing.

## Method

- **The answer.** `build.py` renders a mock plain-text range answer for the doctored fixture's
  whole-document call (`answers/with-check.txt`), from ADR-0094, 0097, 0098, 0105, 0106, 0112 and
  0114: header with `no checks run`, READER CHECK, 18 provenance lines each carrying its exact
  `label`, `skipped`, 28 dropped audio-only boundaries, `0 tiled, 0 untiled` keyframes, the six
  `blind_to` sentences, and the prose restatement. `answers/without-check.txt` is identical except
  that the READER CHECK paragraph is removed.
- **The sheet.** #422's 18-tile 6×3 sheet at 184 px, rebuilt from the same frames with the **full
  ADR-0098 label** (`1 0ms +0 +intro-title`, 10.2 px served, which reproduces ADR-0098's 10.17 px).
  #422's picture sheets drew only the numeric core. A label without its id would break ADR-0114's
  byte-identity rule, so the sheet was rebuilt. The pictures are unchanged.
- **Four conditions.** (A) full answer + doctored sheet; (B) answer with no reader check +
  doctored sheet, the control; (C) full answer + clean sheet; (D) full answer + a sheet whose tile 1
  label is drawn as **`1 40ms +40 +intro-title`**. **D was added to the ticket's three**, because
  it is the only condition that separates a reader that compares from one that copies. In A and C
  the quoted string is true, so "it matches" is right whether or not the reader looked.
- **Readers.** Fresh subagents, one file pair each, in randomly named directories outside the
  repo, told to read those two files and nothing else. The prompt is #422's (*"the video's author
  says something about it looks wrong"*). Each reading is framed as the reader's own `frame` call,
  with next calls written out, not made. 3 readings per model per cell for A–C, 2 for D, and 4 more
  Haiku readings on D once the first two split. **37 readings**; `plan.json` maps each directory to
  its cell.
- **Follow-up.** After their report, seven Haiku readers were asked to transcribe the strip under
  tiles 1, 7 and 10 and to say whether they had compared text against image (`transcripts/probes/`).
  **This proved non-discriminating**: the same strings are on the provenance lines in the text, so
  exact answers cannot be told apart from copies. It is kept for its admissions, not its
  transcriptions.

## Handshake

| condition | Sonnet | Opus | Haiku 4.5 |
| --- | --- | --- | --- |
| A, true label | 3/3 quote it exactly, pass | 3/3 quote it exactly, pass | 2 claim "all 18 match", 1 silent |
| C, true label, clean | 3/3 pass | 3/3 pass | 2 claim "match exactly", 1 silent |
| **D, label drawn differently** | **2/2 fail it, quoting `1 40ms +40 +intro-title`** | **2/2 fail it, quoting `1 40ms +40 +intro-title`** | **3 confirm the quoted string**, 2 fail it, 1 defers to the user |

- **Capable readers compare.** All four capable D readers quoted the drawn string exactly and
  said so first. Three of them then demoted what followed: Opus calls its findings *"leads, not
  verdicts"*, a Sonnet reader *"treat[s] the labels as unverified"*. That is the behaviour the
  check asks for.
- **The weak reader copies.** Three Haiku D readers asserted the string that is not on the sheet
  (*"tile 1: `1 0ms +0 +intro-title` … match the provenance list exactly"*). None of the six quoted
  what is drawn. Of the two that failed it, one misread the string as `1 0ms +40` and the other
  stated a mismatch without saying what it read. **So the handshake does not fail visibly for this
  reader. It passes falsely, in the check's own vocabulary.** A supervising reader who trusts
  "reader check passed" is misled exactly as a trusting reader of the trial's sampler was.
- **The follow-up confirms the mechanism.** Asked afterwards, four of seven Haiku readers said
  outright that they had not compared: *"I … stated they 'matched perfectly' without actually
  reading the dark strips"* (C); *"My initial report claimed verification I did not actually
  perform"* (C); *"I did not verify that the dark strip labels … matched"* (B).
- **Where the prediction partly holds**: 2 of 6 weak D readers did report a failure, and one
  deferred. It is not a coin that always lands wrong. It lands wrong about half the time, and
  confidently.

## Behaviour after

- **Nobody made the named call as a reaction to failing.** The two Haiku readers that failed the
  check named `frame --at 0`, which is the check's own `frame --at <instant>` pointed at tile 1.
  So they re-checked the label, not the pictures. One capable D reader named `frame --at 0` to
  *"redo the reader check at full scale"*. Every capable reader, in every condition, named
  `frame --at` or `--crop --at` on the tiles it suspected. The check did not change that.
- **The check cost capable readers nothing measurable.** A against B: the empty card (tile 10)
  was found 6/6 in A and 6/6 in B. The wrong photo (tiles 7–8) was found 4/6 in A (3 Opus, 1
  Sonnet at 60%) and 4/6 in B (3 Opus, 1 Sonnet). There was no false modesty and nothing dropped.
  A check they pass adds one sentence to their report.
- **The weak reader's disclosure became its alibi.** Haiku readers that passed the check,
  falsely or not, found nothing, and **five of them explained the author's complaint with the
  `blind_to` list** (*"it likely involves motion, transitions, or timing within states — not
  visible in still frames"*). The disclosure ADR-0094 §6 wrote to stop silence reading as coverage
  was read, and was used to turn a miss into "outside what the sheet shows". That is the opposite
  of its intent.
- **Without the check, Haiku invented defects** (B: 3/3, on tiles 17 and 18, where the labels
  and pictures are correct). With it, Haiku mostly reported nothing wrong. **The check traded
  false positives for false negatives.** Neither is a reading.

## Text rescue

No. With no reader check (B), **0 of 3 Haiku readers found `sentence-08`**, which the provenance
list lists as present and the picture does not show. They used the provenance list, but to
invent mismatches (a *"cobweb"* on tile 17 that is supposed to be there, per its own presence
set). Capable readers used the same fact as corroboration: *"The provenance list says
`sentence-08` is present, so the element exists but its text isn't visible"* (Opus, A). The text
lets a reader that can already see name what it sees. It does not let a reader that cannot see
find what it missed.

## Is a disclosure read when no doc string contradicts it?

**Yes, by every reader, and it was acted on.** This settles the fog's untested half. All 22
capable readings cited a disclosure: `no checks run` (often followed by *"so I'd run `validate`"*),
the 184 px width, `inside-run`, the 28 audio-only boundaries, and `motion`. Weak readers read it
too, and used it as above. **ADR-0094's bet that the reader meets the disclosure holds.** The
weak-reader failure is not that the disclosure goes unread. It is that the weak reader reads it
and then misuses it.

## What this means for ADR-0114

§5 states that a falsified prediction changes the **wording** under an amendment, and the
obligation to disclose does not change. The measurement says something more specific: **no
wording change can make a copy-passable handshake fail visibly**, because the weak reader copies
the quoted string out of the text whatever the text around it says. The failure is in quoting the
exact string in the text at all. Options, for the human to choose:

1. **Stop quoting the string.** The check says where the label is and asks the reader to *report*
   what tile 1's strip reads. The text keeps each tile's `label` in the JSON (a supervisor compares
   mechanically), but the plain text does not print it in the paragraph. The provenance line
   prints it today, so a copier has it one line lower. Removing the label from the plain-text
   provenance lines is a change to ADR-0114's Consequences.
2. **Quote something the text does not carry.** For example, ask for a label other than tile 1's,
   named only by index, while the provenance lines keep their labels. A copier still finds it in
   the list, so this only moves the copy.
3. **Keep the wording and change the claim.** Record that the check detects a reader that is
   honest and weak, not one that confabulates. It stays because capable readers pass it for free
   and act on a failure correctly (D), and because the JSON `label` makes any report checkable
   after the fact. The stated guarantee in ADR-0114's Costs, *"such a lie is checkable"*, becomes
   the check's primary job and not its fallback.

## Limits

- n is small: 6 weak readings on the discriminating condition. The split (3 confirm / 2 fail /
  1 defer) is a direction, not a rate.
- One weak model. "Weak reader" here means Haiku 4.5 on a 184 px sheet with 10.2 px labels. #422
  found it never located the strip even at 10 px. Two D readers here did report a mismatch, one
  with a partially correct reading (`+40`), so this reader may see the strip sometimes when the
  text points at it.
- The mismatch is on the instant (`0ms` vs `40ms`), with the id identical. A mismatch on the id
  was not tested.
- Readers were told to read the files and nothing else, and they could not make the next call. A
  real agent can make it, and D's capable readers said they would.
- Prose was judged by hand. The verdicts per reading are in the transcripts, one file each, named
  `<cond><model initial><rep>-<dir>.txt`.

## Reproducing

```sh
# frames: see ../contact-sheet-legibility/make_sheets.py for the doctored-project recipe;
# render tiles/ (doctored) and clean/ (undoctored) with `frame --at <t> --full --png` at the 18
# run-start instants listed in build.py. They are not committed (74 MB).
python3 build.py   # writes sheets/, answers/, manifest.json; asserts the 18 runs reproduce
```
