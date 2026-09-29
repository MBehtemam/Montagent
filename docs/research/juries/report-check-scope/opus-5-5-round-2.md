# Juror 1 — Opus 5.5 (round 2, BRIEF-2.md)

Recorded verbatim, unedited.

---

🗳️ **Juror 1** (Opus 5.5) — **Q5 VOTE: B. Name the check sets that ran.**

**Reasoning:** The field should record a fact about the run: which checks executed. Which classes each set could have raised follows from a small static table (validate → error/review/note/unchecked/layout, layout → layout, drift → drift), and the header's per-class checked/not-checked marking can be derived from that table. So B gives you everything A gives, and more. A also mixes up two different things. Fact 3 shows a class can hold findings even when no check looked for it: `frame`'s E-NOT-A-PROJECT is error-class with `[]` checks run. Under A, `frame` would show a non-zero error count for a class the field says nobody looked for. That is confusing even if it is technically consistent. Under B it reads cleanly: `summary` says what was raised, and the field says which checks produced the check-derived part.
**Trade-offs:** A is easier for a consumer that only wants "is this zero meaningful?", because there is no lookup table. The cost of B is that the set-to-class table must be written down, preferably in the ADR and exposed as a constant, so consumers don't each re-derive it. That is a small price for not baking a derived view into the wire format.

🗳️ **Juror 1** (Opus 5.5) — **Q6 VOTE: B. Record the field as checks execute, starting from `[]`.**

**Reasoning:** Fact 2 decides it. `validate` on a transcript export runs nothing. A declared `["validate"]` would repeat the incident in `validate`'s own output: a header saying validate's checks ran, over zeros that nobody computed. Recording is the only option that is truthful by construction. It also means a future early return, or a verb that gains or loses a check path, can't silently leave the field stale. The same reasoning is behind ADR-0006's point that a noisy validator does more damage than an unrun one.
**Trade-offs:** Declared is simpler, grep-able, and trivially testable per verb. Recorded needs discipline about where the mark goes. I'd put it at the entry of the check-set runner, so it is set if and only if that runner was reached. It also needs a test per verb that the normal path records the expected set, and that the refusal path records `[]`. One open point: if validate's set can stop partway (for example, later checks skipped after a schema failure), "validate ran" becomes an overstatement. That should be decided explicitly and not left to chance.

🗳️ **Juror 1** (Opus 5.5) — **Q7 VOTE: A. Three names: `validate`, `layout`, `drift`.**

**Reasoning:** Fact 4 says the only subset in use is fmt's layout check. Three names describe every run that actually happens. Twenty-five names would put implementation detail into a wire format that ADR-0011 wants to be the single stable thing to parse. Every refactor that splits or merges a check would then become a breaking change. The prose sentence ("validate's checks were not run; run validate for them") also needs a name a reader can act on, and "validate" is that name.
**Trade-offs:** This loses precision if a verb later runs some other partial subset, or if validate stops partway (see Q6). At that point you add a name or move to finer granularity; a list of strings can be extended without breaking anything. Going coarse now and finer later is cheap. Going fine now and trying to take it back is not.

🗳️ **Juror 1** (Opus 5.5) — **Q8 VOTE: A. Adopt the rule, including dropping `0 drift` from validate's line.**

**Reasoning:** Fact 1 makes `0 drift` on validate the same kind of lie that caused the incident: a zero for a check validate never runs. Option B would write that lie into the verb people trust most, and turn the rule into "zeros are honest except on the main verb". The precedents in ADR-0006, 0013 and 0041 all push toward counts that mean something. Here that means removing a meaningless one, where those ADRs added missing ones. JSON `summary` keeps all six keys per Q2, so machine consumers see no change in shape. Only the prose line changes, and the prose is generated.
**Trade-offs:** Snapshot tests for validate, render, preview, shift and create-project all need updating, and anyone who pattern-matches the prose line with six fixed fields breaks. That breakage is acceptable, because the canonical interface is JSON. A related refinement: when validate itself records `[]` (fact 2), its header should read "no checks run" like timeline's does. The rule as stated already produces that, provided Q6 is B.

🗳️ **Juror 1** (Opus 5.5) — **Q9 VOTE: (a) Yes, its own ADR. (b) Agree: "Check set", avoiding *engine* and *scoreboard*.**

**Reasoning:** (a) The change meets all three criteria. It is hard to reverse, because it changes every verb's wire format and the prose line that snapshots pin. It is surprising without context: validate no longer prints `0 drift`, and NOT CHECKED stops being fixed text. And it is the result of real trade-offs (sets vs classes, declared vs recorded, coarse vs fine). The ADR should supersede or amend the relevant parts of 0006, 0011, 0013 and 0041, and it should contain the set-to-class table. (b) "Check set" is literal, matches the field's contents, and fits the existing "check" vocabulary. I'd also add *scope* to the terms to avoid, or reserve it for the NOT CHECKED block, because ADR-0006 already uses "scope" for that block and the two ideas are close enough to blur.
**Trade-offs:** Folding this into amendments of four existing ADRs would spread the reasoning across documents, and no single place would say why the zeros changed. The cost of a separate ADR is one more document to cross-link. As for the term, "check set" is slightly awkward when it's empty ("an empty check set"), but that describes the no-checks case accurately.
