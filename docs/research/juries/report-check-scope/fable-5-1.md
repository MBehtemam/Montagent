# Juror 3 — Fable 5.1

Recorded verbatim, unedited.

---

🗳️ **Juror 3** (Claude Fable 5.1) — **Q2 VOTE: B — the report gains a field stating which checks ran; text is derived from it; `summary` keeps its shape.**

**Reasoning:** The project's own rule is "nothing is assembled twice" and "exactly one thing to parse across the surface" (ADR-0011). Option A puts a fact — "the check engine did not run" — into the renderer only, keyed off `tool`. That fact would then exist in prose but not in the canonical JSON, which is the one place the project says truth lives. It is also the same false-confidence failure ADR-0013 already fixed once for `unchecked`: an MCP client reading `summary: {error: 0, review: 0, …}` from `timeline` is exactly as misled as the agent that read the text header. The incident's agent was reading text, but the next one will be reading JSON, and A leaves that path unfixed. A tool-name lookup table in the renderer is also a second, silent registry of "which verbs run checks" that will drift from the real dispatch. Whatever Q1 decides (I assume "kept, but stating whose checks ran"), the renderer needs a fact to render, and that fact belongs in the report.

**Trade-offs:** B is a schema addition, so every verb's JSON grows one field and fixture snapshots move. That is cheap and one-time. Keeping `summary`'s shape is the right half of B: consumers that index `summary.error` on every verb keep working, and the new field says how to read those zeros rather than making them disappear. What B costs is a small chance the field is ignored by a reader that only looks at `summary` — but that reader is no worse off than today, whereas under A the JSON reader is worse off than the text reader.

---

🗳️ **Juror 3** (Claude Fable 5.1) — **Q3 VOTE: B — name what ran: all / a named subset / none.**

**Reasoning:** The incident's mechanism was "zeros that look like a clean pass". Under two tiers, `fmt` and `compare` are labelled "checked" and print `0 errors, 0 reviews, 0 notes, 0 unchecked, …` — five of those six zeros are exactly the unrun-check zeros the fix exists to stop. Calling `fmt` "checked" reproduces the bug one row down the table. Naming the subset is also what the JSON field from Q2 naturally is: a list (or enum) of the check classes that ran. `["layout"]`, `["drift"]`, all, or empty — one representation, no special "partial" tier to invent. The prose then has an honest thing to say: `layout checked (validate runs the rest) — 0 layout — <path>`.

**Trade-offs:** Three tiers is more text variants to render and test, and two extra verbs to get right. It also invites the question "should `render` list all six by name?" — the answer is a single word `all`, so the cost is small. Two tiers is simpler to explain in a sentence, but the sentence would be false for `fmt`, and a simpler lie is still what ADR-0006 warns against. If Q1 ends with "those verbs run the engine too", the subset tier still matters for `fmt`/`compare`, so this vote does not depend on Q1.

---

🗳️ **Juror 3** (Claude Fable 5.1) — **Q4 VOTE: B — header plus a scope sentence in NOT CHECKED for any report that did not run the full set.**

**Reasoning:** ADR-0006 says the report "ends with its own scope, unconditionally". Today `timeline`'s scope block says "validate verifies that the file is internally legal" — it is not timeline's scope, it is validate's, pasted in. That is already a defect against the ADR; the incident just made it visible. The scope block is the part of the report that survives `head`/`tail`-style truncation from the top and is the last thing an agent reads before concluding; the agent that lost fifteen minutes would have been stopped by one sentence there: "validate's checks were not run; run validate for them." Fixing only the header repairs the first line and leaves the last line contradicting it. And because the sentence derives from the same Q2 field, it is one fact rendered in two places, not two facts — which is what "nothing is assembled twice" permits.

**Trade-offs:** The NOT CHECKED text stops being a single constant string, so the renderer branches on the field and snapshots multiply. There is also a redundancy argument: "header already says it". But the header is precisely what the reader skipped; redundancy at the two edges of the report is the design ADR-0006 asked for. The cost of A is lower diff and one fewer sentence; the cost of B is a branch in a renderer that already has to branch for Q1's header anyway.
