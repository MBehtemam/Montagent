# Juror 3 — Fable 5.1

Recorded verbatim, unedited.

---

🗳️ **Juror 3** (Claude Fable 5.1) — **VOTE: Q4 (a), class `review`; Q5 (a); Q6 (a), plus a constructed test document before the code ships**

**Reasoning:**

*Q4.* A code is "the identity a future `compare` diffs on" and a finding "says something about *this document*". A run the frame grid never paints is one fact about one document, and which verb happens to notice it must not change its identity — otherwise the day `validate` learns to detect it, the same fact has two codes and `compare` sees a phantom change. The condition is also, literally, quantization: two millisecond boundaries snapping onto one frame. ADR-0006's escalation test for `N-QUANTIZATION` is "the rendered frames do not show what the document declares", and a declared visual state that is never on screen is exactly that — it is "rounds out of existence" applied to a state instead of an element. So reuse, at `review`, for consistency with the sibling escalation; the `detail` must name the two boundaries and their elements so the instance is distinguishable from an element-level one. The validate gap is real and should be filed, but it is a gap in validate's detector, not a reason to mint a verb-scoped code. (b) would create a code whose meaning is "frame saw it", which is provenance, not a document fact. "Disclosure only" (c) is wrong per ADR-0094's own line — `skipped` describes the document, and the mandated `skipped` list should carry the code so suppression and `compare` work.

*Q5.* `blind_to` describes the rule. It is printed unconditionally, it never varies with the document, and it judges nothing — that is the NOT CHECKED block's shape exactly, so it gets the NOT CHECKED block's form. `U-` findings (b) would be findings, which ADR-0094 explicitly refuses, and "unchecked" means "could not establish for this document", not "structurally cannot ever". Free prose (c) fails ADR-0097's need for a full plain-text rendering *and* the JSON form to stay in step — fixed tokens each bound to one fixed sentence is the only form that satisfies both without letting them drift. No letter prefix so nobody mistakes them for codes. Dropped audio-only boundaries, untiled keyframe change points, served tile width and budget rung are all *disclosure*: they are correct consequences of the rule operating on this document, not defects in it — nothing to fix, nothing to suppress, nothing for `compare` to flag. Yes, the sheet should point at `frame --crop --at <instant>`: once, in the `below-tile-width` sentence, and the per-tile provenance should print each tile's sampled instant so the pointer is directly actionable. That also answers the ADR-0103 juror without duplicating text.

*Q6.* The ADR's §4 example is false after the re-merge §4 itself mandates; leaving it stands a ratified decision on a computation that does not exist. The sibling ticket set the exact precedent — amend the ADR, ship a re-executable check — and it was accepted. Zero instances on the fixture does not change Q4: the rule reasons about all documents, and the fact is real when it occurs. But a code that never fires on any committed document is untested; the ticket should add a constructed project (two visual boundaries on different tracks under one frame period apart) so the `skipped` entry, the `N-QUANTIZATION` instance at `review`, and the prose rendering each have a fixture.

**Trade-offs:**

*Q4 (a)* costs a transient inconsistency — `frame` emitting a code `validate` does not yet, and `N-QUANTIZATION`'s template broadening from "boundaries" to states — plus a filed follow-up. (b) buys a clean per-verb story at the price of splitting one fact into two identities and forcing a later rename or alias. (c)/disclosure-only loses suppression and `compare` diffs for a genuine defect and contradicts ADR-0094.

*Q5 (a)* costs a second fixed block on every answer (length) and a small vocabulary to keep stable. (b) buys reuse of the finding machinery but misuses `U-` semantics and produces six "findings" on every clean sheet. (c) is cheapest to write and impossible to keep JSON and prose in step. Classing the provenance items as disclosure means they are not suppressible or diffable — acceptable, because none of them is a defect.

*Q6 (a)* costs an ADR amendment and a small script to maintain, and the constructed fixture is extra work. (b) is free and leaves a ratified ADR citing a run that does not exist, which is the failure mode the precedent was created to stop.
