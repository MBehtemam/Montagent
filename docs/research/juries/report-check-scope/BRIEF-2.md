You are a juror. Answer the Question below and nothing else. Do not use any tools, do not edit anything, do not make recommendations to whoever sent this — just your ballot. Answer each of Q5, Q6, Q7, Q8, Q9 with its own ballot block, in exactly this format, naming the model that backs you:

🗳️ **Juror <n>** (<the model backing you>) — **Q<n> VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <why not the other option(s), and what your choice costs>

Rejecting the framing is a valid ballot.

---

# The Question (five linked multiple-choice parts)

## Context

Montagent is a CLI + MCP server whose verbs read, check and render a declarative video project file. Every verb answers with one **Report**: its findings, a count per class, an exit code, and a `NOT CHECKED` block. JSON is canonical; the prose form is generated from the JSON ("nothing is assembled twice"), and `--json` prints JSON instead of the text. ADR-0011 wants "exactly one thing to parse across the surface". Findings have six classes: error, review, note, unchecked, layout, drift.

The prose form's first line is a summary of the six classes, e.g. from `validate`:

    0 errors, 12 reviews, 55 notes, 0 unchecked, 0 layout, 0 drift — <path>

Every verb's JSON carries the same `summary: {error, review, note, unchecked, layout, drift}` object, and the prose line is generated from it.

Only some verbs run checks:

| Checks run | Verbs | What the counts actually count |
|---|---|---|
| validate's full set (~25 checks) | validate, render, preview, shift, create-project | the document's findings |
| a subset | fmt (only the LAYOUT check, which is one of validate's), compare (its own drift checks, which validate does not run) | one class is real; the rest read zero |
| none | timeline, query, frame, measure, probe, fonts | only the verb's own refusals (E-NOT-A-PROJECT, E-PARSE, frame's declined-to-paint findings) |

**The incident.** An agent piped `validate --verbose` through `tail`, cutting off the `review` section that held the finding naming the defect it was hunting. It then ran `timeline`, read its all-zeros header (`0 errors, 0 reviews, 0 notes, 0 unchecked, 0 layout, 0 drift`, identical in shape to a clean validate) as confirmation that validate had found nothing, and lost fifteen minutes.

**Precedent.** ADR-0006: `0 errors, 47 notes` reads as a pass; "a noisy validator manufactures false confidence faster than an unrun one does". ADR-0013 put an `unchecked` count in the summary line because `0 errors` over a file where nothing could be probed is that false-confidence failure; ADR-0041 did the same for `layout`. ADR-0006: the report "ends with its own scope, unconditionally" (the NOT CHECKED block).

## Already decided (by a previous panel, ratified by the human)

- **Q1.** A verb that runs no checks keeps a header, but the header states whose checks ran (e.g. `checks not run (validate runs them) — <path>`); it is not dropped, and those verbs do not start running checks.
- **Q2.** The report gains a JSON field stating which checks ran; the text is derived from it; `summary` keeps its shape on every verb.
- **Q3.** The field names what ran — all / a named subset / none — not a two-tier checked/not-checked boolean.
- **Q4.** NOT CHECKED also carries a scope sentence, generated from the same field, for any report that did not run the full set (e.g. "validate's checks were not run; run validate for them").

## Facts found since

1. **Every drift-class code is raised only by `compare`.** So `validate`'s own `0 drift` is itself a zero for a check validate never runs.
2. **`validate` can run nothing.** Pointed at a file that is not a project (e.g. a transcript export), it pushes `E-NOT-A-PROJECT` and returns before any check runs. A missing `ffprobe` ends the run in an internal-failure exit of its own.
3. **Verbs that run no checks still raise findings.** `frame` can raise `E-NOT-A-PROJECT`, which is error-class; a refusal is a finding (ADR-0011) without being a check.
4. No verb runs any subset of validate's checks other than fmt's layout check.

## Q5 — Does the new field name check sets or finding classes?
- **A.** The finding classes that were looked for (e.g. `["layout"]`, all six, or none), so the header can mark each class as checked or not checked.
- **B.** The check sets that ran (e.g. `["validate"]`, `["layout"]`, `["drift"]`, `[]`); `summary` stays "findings raised, per class", and the field says which checks produced them.

## Q6 — Is the field declared per verb, or recorded as checks execute?
- **A.** Declared: each verb states its check set once, where its report is constructed.
- **B.** Recorded: the code that runs a check set marks the report when it runs; a new report starts as `[]` (no checks run).

## Q7 — How finely are check sets named?
- **A.** Three names: `validate` (the full set), `layout` (fmt's subset), `drift` (compare's own).
- **B.** One name per individual check (~25: `schema`, `anchor`, `tie`, …).

## Q8 — Which zeros does the prose header print?
Proposed rule: *a zero is printed only for a class that some check which actually ran could have raised; a non-zero count always prints.* Illustration:

    validate:  0 errors, 12 reviews, 55 notes, 0 unchecked, 0 layout — <path>
    fmt:       0 layout (layout check only; validate runs the rest) — <path>
    timeline:  no checks run (validate runs them) — <path>
    frame:     no checks run (validate runs them); 1 error — <path>

This drops `0 drift` from validate's line (and from render/preview/shift/create-project, which share it); snapshot tests pin that line.
- **A.** Adopt the rule, including dropping `0 drift` from validate's line.
- **B.** Keep all six zeros on any run whose check set is validate's full set; apply the rule only to other runs.
- **C.** Something else (say what).

## Q9 — Record-keeping.
The change alters every verb's wire format, amends ADR-0006 (NOT CHECKED stops being one fixed text), ADR-0011 (the report's shape) and the summary line ADR-0013/0041 shaped. The project writes an ADR when a decision is hard to reverse, surprising without context, and the result of a real trade-off.
- **(a)** Does this warrant its own ADR? yes / no.
- **(b)** The glossary will need a term for "the set of checks a report's run executed". Proposed: **Check set**, with *engine* and *scoreboard* as terms to avoid. Agree, or propose another term.

You may reject any framing — saying the options are all wrong, or the question is wrong, is a valid answer.
