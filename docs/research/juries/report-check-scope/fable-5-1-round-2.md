# Juror 3 — Fable 5.1 (round 2, BRIEF-2.md)

Recorded verbatim, unedited.

---

🗳️ **Juror 3** (Claude Fable 5.1) — **Q5 VOTE: B — the field names check sets that ran; `summary` stays "findings raised, per class"**

**Reasoning:** The two axes are different things and Fact 3 proves it: `frame` runs no checks yet raises a real error-class finding. Under A, `frame`'s field would have to say either "error was checked" (a lie — nothing was checked) or "error was not checked" while `summary.error` reads 1 (a contradiction the reader has to resolve). Under B there is no tension: check set `[]`, one finding raised. Fact 1 is the same shape from the other side — `validate` raising nothing in drift is not because it looked and found none; it is because the drift set belongs to `compare`. A check set names the cause; a class names the effect. The header's per-class rendering (Q8) is then a lookup from set to "classes this set can raise", which is a small static table, not a second source of truth.

**Trade-offs:** A is closer to what the prose header finally shows, so it saves one mapping step. But it bakes the rendering into the wire format, and the day a check set starts raising a new class every emitter has to be revisited instead of one table. B costs that table and the discipline of keeping it beside the check registry.

---

🗳️ **Juror 3** (Claude Fable 5.1) — **Q6 VOTE: B — recorded as the check set executes; a new report starts as `[]`**

**Reasoning:** Fact 2 kills A on its own. `validate` pointed at a transcript export pushes `E-NOT-A-PROJECT` and returns before any check runs; a declared `["validate"]` would then say the full set ran over a file that was never probed — precisely ADR-0013's false-confidence failure, now stamped into the field meant to cure it. Recording at the point where the set actually runs means the field cannot claim more than happened, whatever early return or internal failure (missing `ffprobe`) intervenes. It is also the cheaper option to keep honest: there are three run sites (validate's runner, fmt's layout call, compare's drift checks), versus twelve verbs each declaring a value someone must remember to update.

**Trade-offs:** A is easier to read in the source ("this verb's set is X") and easier to unit-test in isolation. B's cost is that a report constructed without ever touching a runner is silently `[]`, which is the right default here but means a forgotten runner call shows up as "no checks run" rather than a compile error. That is the failure mode I prefer, because it is visible in the header rather than hidden behind a confident claim.

---

🗳️ **Juror 3** (Claude Fable 5.1) — **Q7 VOTE: A — three set names: `validate`, `layout`, `drift`**

**Reasoning:** The question the field answers is the incident's question: "did validate's checks run over this file, yes or no?" Three names answer it; twenty-five names answer a question nobody has asked. Fact 4 says no verb runs any subset other than fmt's layout check, so per-check granularity would enumerate the same ~25 entries on every full run, expand the header or force a summarisation rule back to three names anyway, and create a public naming contract for internal check identities that will churn. The list-of-names shape keeps the door open: if a fourth set ever exists, it is one more name, not a format change.

**Trade-offs:** B would let a partially completed validate run say exactly which checks got through — but Fact 2 says validate either runs nothing or runs its set, and the ffprobe case exits as internal failure, so there is no partial state to describe today. A's cost is that `validate` is both a verb name and a set name, which readers will occasionally conflate (render "ran validate's set" without being validate). The Q1 header wording already leans on that reading, so it is consistent rather than new.

---

🗳️ **Juror 3** (Claude Fable 5.1) — **Q8 VOTE: A — adopt the rule, including dropping `0 drift` from validate's line**

**Reasoning:** `0 drift` on a validate run is a zero for a check validate never runs. That is not a cosmetic detail; it is the exact shape of the incident, sitting on the most-read verb, and it is what ADR-0006 named: "a noisy validator manufactures false confidence faster than an unrun one does". ADR-0013 and ADR-0041 added `unchecked` and `layout` to the line so a zero would not hide something that was not probed; dropping `0 drift` is the same principle pointed the other way — do not print a zero that claims a probe which did not happen. Option B keeps that one false zero because five verbs share a snapshot, which is letting the pinned test define the behaviour instead of the behaviour define the test. The rule also satisfies ADR-0011's "nothing assembled twice": one derivation from the recorded set, no verb-specific carve-out. The `frame` example is the proof the rule is right — "no checks run (validate runs them); 1 error" reads as exactly what happened.

**Trade-offs:** A retrains eyes: anyone who has learnt the six-count line will notice drift is missing and wonder. That is mitigated by the Q4 scope sentence in NOT CHECKED and by the fact that a non-zero drift count still prints wherever it occurs. B's cost is a permanent exception in the renderer ("full set prints six, everything else prints the rule"), and a validate line that can never be trusted as a statement of what ran. I would rather pay the one-time snapshot update.

---

🗳️ **Juror 3** (Claude Fable 5.1) — **Q9 VOTE: (a) yes, its own ADR; (b) agree with "Check set"**

**Reasoning:** (a) It meets all three tests. Hard to reverse: every verb's wire format changes, and the MCP clients parse it. Surprising without context: a future reader will ask why validate's header is silent about drift and why three ADRs' summary line no longer holds; the incident and Facts 1–3 are the context, and they live nowhere else. A real trade-off: Q8 chose accuracy over the six-count stability that ADR-0013/0041 had just built. Write it as one ADR that amends 0006, 0011, 0013 and 0041 by reference rather than editing those in place, so the reasoning chain stays readable. (b) "Check set" is precise: it is the set of checks, it is the thing recorded, and it does not overload existing words. Add "scope" and "coverage" to the avoid list alongside *engine* and *scoreboard* — ADR-0006 already uses "scope" for the NOT CHECKED block (what in this document could not be probed), which is a different thing from which checks ran, and "coverage" reads as test coverage to every engineer.

**Trade-offs:** Folding this into an amendment note on ADR-0006 would be less paperwork but would scatter the rationale across four amended documents with no single place that says why. "Check set" costs a little: it is two words and it will sit next to "check" and "checks not run" in the header, so the glossary entry must draw the line between a check (one probe) and a check set (the unit the report records) explicitly.
