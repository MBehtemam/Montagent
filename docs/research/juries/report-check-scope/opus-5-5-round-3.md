# Juror 1 — Opus 5.5 (round 3, BRIEF-3.md)

Recorded verbatim, unedited.

---

🗳️ **Juror 1** (Claude Opus 5.5) — **Q10 VOTE: C, using a partial marker rather than a split. A stopped run records a separate token such as `validate:partial`, never plain `validate`, and `NOT CHECKED` names the half that did not run and why.**

**Reasoning:** The rule you ratified is that the field records what actually executed. In this case, what executed is "part of validate." A and B both break that rule, in opposite directions. A claims a full run that never happened. B denies a run that did happen and produced findings. B is the worse of the two: it prints "no checks run" above real document-half errors, so the zero-suppression rule hides zeros the document half did earn. That teaches agents that the header and the findings can contradict each other.

A partial marker keeps the field truthful. Making it a *distinct token* rather than `validate` plus a side flag gives a fail-safe property. A consumer that tests `"validate" in check_sets` gets false for a stopped run. A naive reader then under-trusts the report instead of over-trusting it, which is the direction ADR-0006 asks for. The incident was exactly a reader over-trusting a report's shape, so the naive path has to fail closed.

The other pieces follow from what is already decided:
- **Prose header:** prints zeros only for classes the document half could have raised, and says `validate stopped before media checks (ffprobe missing)`.
- **Scope sentence:** the "did not run the full validate set" trigger already covers this case, and names source-media and fit checks as unchecked.
- **Exit code:** exit 70 and `E-TOOL-MISSING` stay as they are, so the partial state is corroborated in three places.

**Trade-offs:**
- **Against A:** it is the simplest option, but it recreates the incident's failure mode. An agent that truncates to the header sees `validate` with zero media findings and reads it as a clean run. Exit 70 helps only if the exit code survives truncation.
- **Against B:** it is literally consistent with "record on completion", but that consistency is false: it states "no checks run" while showing check-raised findings.
- **Against splitting into `document`/`disk`:** it reopens the ratified glossary, which has three check sets with `validate` meaning validate's full set. It also makes every consumer learn that `validate` = `document` + `disk`. And it blurs the no-media case: a complete run that never needed the disk half would record `document` alone and look exactly like a stopped one, unless you also record a vacuous `disk`.
- **What my choice costs:**
  - A fourth token in the field's vocabulary, which the glossary has to define as "a check set that began but did not finish."
  - The recording code has to mark the set on entry and upgrade the mark on completion, a small two-step state instead of a single mark.
  - The same marker has to apply to `render`, `preview` and `shift`, which run the set the same way.
