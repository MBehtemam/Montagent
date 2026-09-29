# Juror 1 — Opus 5.5

Recorded verbatim, unedited.

---

🗳️ **Juror 1** (Opus 5.5) — **VOTE: Q1 (a); Q2 `--keyframes`, bare boolean; Q3 (b) `--infill-ceiling <MS>`; Q4 coin "infill ceiling", drop "gap ceiling"; Q5 (a); Q6 (c); Q7 (a); Q8 adopt the definition, and always report both `tiled` and `untiled`; Q9 (a); Q10 agree (a), (b), (c)**

**Reasoning:**
- **Q1:** The measurement replaces the "absence of evidence". On the one real project, keyframes contribute no tiles. On a heavily animated one, turning them on by default turns working calls into `E-SHEET-OVERFLOW` (the 13th tile on 18 states). So default-on costs refusals and buys nothing on real data. The width-conditional default (b) fails ADR-0103: whether keyframe instants are seen would depend on the state count and range. That is a blindness that varies per call, which that ADR says cannot safely be disclosed. Retire the caveat and cite #407 in its place.
- **Q2:** `--keyframes` is a bare word, matching `--full`, `--all` and `--census`, and it matches the provenance class token `keyframe`. It names a tile class that gets added, not a selection rule: the run-start rule still runs unchanged. `--with-*` and `--include-*` have no precedent in this CLI. There is nothing to put in a value, so it is a boolean.
- **Q3:** `--infill <MS>` can read as "500 infill tiles", which is the `--n` mistake again. `--infill-every` reads as a fixed sampling period, and infill is not that: it only fills spans longer than the ceiling. `--max-gap` collides with the glossary's **Gap**, which means a stretch of a track with no element. `--infill-ceiling` says it is a bound, and `value_name = "MS"` carries the unit, as the conventions require.
- **Q4:** "Gap" is already taken by the glossary, and the two meanings would get confused (a track gap versus the spacing between tiles). Glossary entry: "Infill ceiling: the longest span between consecutive tiles, in painted time, that the sheet may leave unsampled when infill is requested." Keeping one term for the flag and the concept avoids a translation step.
- **Q5:** ADR-0094 §5 says infill is strictly additive and never displaces a document-derived tile. Pushing the rung down shrinks every document tile, which is displacement by resolution. So the document tiles fix the rung, and infill only fills the slots left over at that rung. Requesting infill can then never degrade or refuse a call.
- **Q6:** A uniform achieved ceiling keeps the guarantee constant and learnable across the whole sheet. A subset (b) holds in some places and not others: a patchwork blindness that varies per call. Dropping all infill (a) throws away value, and refusing (d) contradicts "first class evicted". Disclose "requested X ms, achieved Y ms", or "achieved: none" if no slot is left. `skipped[]` records the difference as `infill-evicted`, grouped into runs so the list does not blow up.
- **Q7:** These tiles have no boundary change, so no §8 field exists for them. Making one up (`element.property`) lets a single long id strip the identifying field from every document tile through sheet-wide elision, so the least important tile degrades the most important ones. The class mark and sigil identify the tile. The provenance line names every keyframed `element.property` at that instant. The label says where to look; the provenance list has the full account.
- **Q8:** The declared-list count prints 14 where the honest count is 0. That is noise that reads as a gap, and it teaches readers to ignore the number. Count keyframes interior to a run, on an element visible at that instant, and not landing on the run's first painted frame. With the flag, report both `tiled` and `untiled`, as zero-asserted fields like `0 keyframe, 0 infill`. `untiled` can still be non-zero with the flag, when several keyframes land on one painted frame or between grid frames. Give those a reason.
- **Q9:** The existing sub-ranges point the caller at a narrower range. The real fix is "drop `--keyframes`", or know that N of M keyframe tiles fit. Say so as a finding field with prose. Degrading silently (c) is the thinning ADR-0095 forbids. Keyframe tiles were explicitly requested and derive from the document; they are not infill.
- **Q10:** (a) The two surfaces must mean the same thing; MCP names are `keyframes` and `infill_ceiling`. (b) Both flags only mean something over a range. With `--at` or alone they are flag-combination errors, which are already bare `E-INVOCATION`. (c) A ceiling ≤ 0 is meaningless. A ceiling below one frame period can never be met: the grid cannot paint more densely than that. Clamping would silently rewrite the caller's promise, so refuse and name the frame period. A ceiling ≥ the range length is a legal no-op.

**Trade-offs:**
- **Q1:** Off by default means careless callers never see mid-run overshoots, like the 1.85× crop or the text drifting off canvas. The unconditional honest count and the `between-keyframes` blind spot are what make that acceptable.
- **Q2:** `--keyframes` is slightly less self-describing than `--include-keyframes`, but it stays consistent with the house style.
- **Q3:** `--infill-ceiling` is longer to type.
- **Q4:** Renaming means amending the wording of ADR-0094 §5.
- **Q5:** A dense document can leave infill no room at all at the 180 px target, even when the 140 px floor would have fit it. That is accepted.
- **Q6:** The search for the achievable ceiling adds implementation work, and the achieved value may be much coarser than requested. The disclosure makes that visible.
- **Q7:** A keyframe or infill tile read on the sheet alone does not say which property it is showing without the list. That is acceptable because the list is required anyway.
- **Q8:** The refined definition needs a visibility and interior computation per keyframe. That is cheap compared with a count nobody can trust.
- **Q9:** One more field on the refusal payload.
- **Q10:** Strict refusals cost a little convenience in exchange for promises that hold exactly as stated.
