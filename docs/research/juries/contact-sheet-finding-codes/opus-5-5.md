# Juror 1 — Opus 5.5

Recorded verbatim, unedited.

---

🗳️ **Juror 1** (Opus 5.5) — **VOTE: Q4 (a) N-QUANTIZATION at review; Q5 (a); Q6 (a), plus a constructed test document before the code ships**

**Reasoning:**
- **Q4:** A `no-grid-frame` run means the rendered frames never show a combination the document declares. ADR-0006 already uses that exact reason to escalate N-QUANTIZATION to review ("the rendered frames do not show what the document declares"). This is the same fact. The only difference is that it comes from two elements together rather than one element or one track. The code is the handle for suppressing a class and the identity `compare` diffs on. If `frame` got its own code, one fact about the document would end up with two identities once `validate` learns to detect it, and suppressing one would leave the other showing. So broaden N-QUANTIZATION from elements to visual states, and emit this at review because you have to look to judge it. The template needs a detail variant that names the run's interval and the two boundaries (element, track, ms) that land on the same frame. File the `validate` gap as separate work. Until it lands, `frame` reporting the fact first is fine, because the fact belongs to the document, not to the verb that finds it.
- **Q5:** `blind_to` describes the rule, not this document. ADR-0094 says so explicitly, which rules out (b): U- findings on every answer would present a fixed property of the tool as a per-document fact, and would make every answer look noisy. (c) fails ADR-0097 and gives JSON consumers nothing stable to match on. So use (a): fixed kebab tokens with no letter prefix (so they can't be mistaken for finding codes), each rendered by one fixed sentence, printed unconditionally. It is the sheet's version of NOT CHECKED. The other items should all be disclosure, not findings:
  - dropped audio-only boundaries
  - untiled keyframe change points
  - served tile width and the budget rung that produced it

  Dropped audio boundaries and untiled change points follow from the rule working as designed, not from anything wrong in the document. Tile width and budget rung describe this answer, not the document. Only `skipped` runs earn a finding. The sheet should point at `frame --crop --at <instant>` in the `below-tile-width` sentence, which is where an agent hits the limit. The `inside-run` sentence could also point at `frame --at`. This settles ADR-0103's deferred juror request.
- **Q6:** (a). The example was computed before the re-merge that the same ADR requires, so the ADR text is wrong. Leaving the error in the ADR, the durable record, while fixing it only in a ticket comment invites someone to re-derive it later. The §3 keyframe precedent already set the pattern: record the correction in the new ADR and commit a re-executable check script. Zero instances on the fixture should not change Q4: the condition is still possible, and the finding is about correctness, not frequency. But it does mean the code has no real example today. So before the code ships, it needs a constructed test document with an instance: two visual boundaries on different elements and tracks less than 40 ms apart at 25 fps. The test should assert the skipped entry, the finding, and its review class.

**Trade-offs:**
- **Q4 (a):** Broadening an existing code's scope changes a published contract, and there's a period where `frame` emits a code that `validate` doesn't yet. That's acceptable; it's better than a second code that later has to be deprecated or aliased. (b) is simpler to ship but splits one fact in two. (c) "disclosure only" would contradict ADR-0094's correct distinction that a skipped run is a document fact.
- **Q5 (a):** It commits to a closed vocabulary of tokens that is expensive to change, and each new blind spot needs a token and a sentence. That cost is exactly what makes them stable.
- **Q6 (a):** It adds a script to maintain and an amendment to track. (b) is cheaper but repeats the kind of unverified derivation that caused the error.
