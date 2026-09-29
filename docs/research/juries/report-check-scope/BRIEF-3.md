You are a juror. Answer the Question below and nothing else. Do not use any tools, do not edit anything, do not make recommendations to whoever sent this — just your ballot, in exactly this format, naming the model that backs you:

🗳️ **Juror <n>** (<the model backing you>) — **Q10 VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <why not the other options, and what your choice costs>

Rejecting the framing is a valid ballot.

---

# The Question

## Context

Montagent is a CLI + MCP server whose verbs read, check and render a declarative video project file. Every verb answers with one **Report**: its findings, a count per class (error, review, note, unchecked, layout, drift), an exit code, and a `NOT CHECKED` block that states the report's own scope. JSON is canonical; the prose form is generated from it. ADR-0006: "a noisy validator manufactures false confidence faster than an unrun one does"; `0 errors, 47 notes` reads as a pass.

**The incident that started this.** An agent ran `timeline` — a verb that runs no checks — and read its header `0 errors, 0 reviews, 0 notes, 0 unchecked, 0 layout, 0 drift`, identical in shape to a clean `validate`, as confirmation that validate had found nothing. It had not; validate had found the defect, and the agent had truncated that output.

## Already decided (two panels, ratified by the human)

- Every report gains a JSON field naming the **check sets** that ran: `validate` (validate's full set of ~25 checks; also run by render, preview, shift, create-project), `layout` (the one check fmt runs), `drift` (compare's own checks). `[]` means no checks ran. The glossary term is **Check set**.
- `summary` keeps its six keys and still means "findings raised, per class". A refusal is a finding without being a check, so a verb running no checks can still count an error.
- The field is **recorded as checks execute**, not declared per verb: the code that runs a check set marks the report, and a new report starts as `[]`. (Chosen because `validate` pointed at a non-project returns `E-NOT-A-PROJECT` before any check runs, and a declared `["validate"]` would then claim a run that never happened.)
- Prose header rule: a zero is printed only for a class some check set that ran could have raised; a non-zero count always prints. E.g. `timeline: no checks run (validate runs them) — <path>`; `frame: no checks run (validate runs them); 1 error — <path>`.
- `NOT CHECKED` gains a generated scope sentence for any report that did not run the full `validate` set.

## The case this question is about

`validate`'s set has two halves. The **document half** (schema, anchors, tracks, captions, motion, ink, … — pure reads of the file plus fonts) always runs first. The **disk half** (the source-media and fit checks, which need `ffprobe`) runs only if the project references any media. If `ffprobe` is missing, the run **keeps every finding the document half already made**, adds `E-TOOL-MISSING` (a finding whose subject is Montagent's environment, not the document — ADR-0083/0091 class it `NotAboutDocument`), and exits **70** ("Montagent could not run"), not 0 or 1. The code's own comment: "a run that found a retired spelling and *then* discovered there is no `ffprobe` has learned two things, and an agent told only the second would fix its `PATH`, re-run, and only then hear about the key it could have fixed in the same turn." `render`, `preview` and `shift` run the same `validate` set the same way.

(A project that references no media completes the set without the disk half ever being needed; that is a complete run, not this case.)

## Q10 — What does the check-set field record for a `validate` set that stopped partway?

- **A. Record the set when its runner is entered.** The field says `validate` although the disk half never ran.
- **B. Record it only on completion.** The field says `[]`; the header reads "no checks run" above findings that document-half checks did raise.
- **C. Record on completion, and let a stopped run say how far it got** — e.g. `validate` with a partial marker, or by splitting the set's name into two (`document`, `disk`) so a stopped run records `document` alone.
- **D. Something else** (say what).

You may reject the framing — saying the options are all wrong, or the question is wrong, is a valid answer.
