You are a juror. Answer the Question below and nothing else. Do not use any tools, do not edit anything, do not make recommendations to whoever sent this — just your ballot. Answer each of Q2, Q3, Q4 with its own ballot block, in exactly this format, naming the model that backs you:

🗳️ **Juror <n>** (<the model backing you>) — **Q<n> VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <why not the other option(s), and what your choice costs>

Rejecting the framing is a valid ballot.

---

# The Question (three linked multiple-choice parts)

## Context

Montagent is a CLI + MCP server whose verbs read, check and render a declarative video project file. Every verb answers with one **Report**: its findings, a count per class, an exit code, and a `NOT CHECKED` block. JSON is canonical; the prose form is generated from the JSON ("nothing is assembled twice"), and `--json` prints JSON instead of the text. ADR-0011 wants "exactly one thing to parse across the surface".

The prose form's first line is a summary of the six finding classes, e.g. from `validate`:

    0 errors, 12 reviews, 55 notes, 0 unchecked, 0 layout, 0 drift — <path>

Every verb's JSON carries the same `summary: {error, review, note, unchecked, layout, drift}` object, and the prose line is generated from it.

Only some verbs run the check engine (`validate`'s set of checks):

| Checks run | Verbs | What the counts actually count |
|---|---|---|
| all | validate, render, preview, shift, create-project | the document's findings |
| a subset | fmt (only the LAYOUT check), compare (only its drift checks) | one class is real; the other five read zero |
| none | timeline, query, frame, measure, probe, fonts | only the verb's own refusals (E-NOT-A-PROJECT, E-PARSE, frame's declined-to-paint findings) |

On the repo's fixture, `validate` prints `0 errors, 12 reviews, 55 notes, …`, while all eight lower-row verbs print `0 errors, 0 reviews, 0 notes, 0 unchecked, 0 layout, 0 drift` — identical in shape to a clean validate.

**The incident.** An agent hunting a planted defect piped `validate --verbose` through `tail`, accidentally cutting off the whole `review` section, which held the one finding naming the defect. It then ran `timeline`, read its all-zeros header as confirmation that validate had found nothing, and lost fifteen minutes. Its own words: "a verb that doesn't run the check engine shouldn't print a check-engine scoreboard." The `tail` was its own mistake; the header turned a recoverable mistake into a confirmed false belief.

**Precedent.** ADR-0006: `0 errors, 47 notes` reads as a pass, and "a noisy validator manufactures false confidence faster than an unrun one does". ADR-0013 put an `unchecked` count in the summary line because `0 errors` over a file where nothing could be probed is exactly that false-confidence failure. ADR-0006 also says the report "ends with its own scope, unconditionally" — the NOT CHECKED block — which today is one fixed text on every verb: "This file was not compared against any prior version or instruction. validate verifies that the file is internally legal; it cannot tell you whether it says what you meant it to say." So timeline's closing scope statement is word-for-word validate's.

A separate, still-open question (Q1, not yours to answer, but you may assume any answer and say which) decides what the header on a no-checks verb becomes: dropped entirely; kept but stating whose checks ran (e.g. `checks not run (validate runs them) — 0 findings from timeline — <path>`); or those verbs run the engine too.

## Q2 — Does the fix reach the JSON, or only the text?
- **A.** Text only: the prose renderer derives "checks not run" from the report's `tool` name.
- **B.** The report gains a field stating which checks ran, and the text is derived from it. `summary` keeps its shape on every verb.

## Q3 — Two tiers or three: does partial checking (fmt, compare) get its own answer?
- **A.** Two tiers: "checked" / "not checked"; fmt and compare count as checked.
- **B.** Name what ran: all checks / a named subset (`layout`, `drift`) / none.

## Q4 — Does the NOT CHECKED block also say validate's checks were not run?
- **A.** Header only; NOT CHECKED stays one fixed text.
- **B.** Header, plus a scope sentence in NOT CHECKED for any report that did not run the full check set (e.g. "validate's checks were not run; run validate for them").

You may reject any framing — saying the options are all wrong, or the question is wrong, is a valid answer.
