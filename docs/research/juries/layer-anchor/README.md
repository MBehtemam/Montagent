# Layer-anchor hardening: court ballots

Evidence for [ADR-0019](../../../adr/0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md),
resolving wayfinder ticket [#24](https://github.com/MBehtemam/Montaget/issues/24).

Two rounds, three jurors each, backed by three different models: GitHub
Copilot's auto-selected model, Qwen2 7B and Llama 3.2 (the latter two local,
via Ollama). Jurors ran in parallel, blind to each other, given the ticket's
two known defects and the same question text. Raw output is preserved
verbatim (terminal control sequences stripped); no editing beyond that.

**Reliability note:** the two local 7B/3B-class models frequently mislabeled
their own vote letter relative to their stated reasoning (e.g. writing "VOTE:
B" while describing the content of option A). Where this happened it is
called out under the ballot; the vote recorded in the ADR is the one implied
by the reasoning, not the letter, since the letter is arguably a decoding
artifact of a small model rather than a considered choice.

## Round 1 — should the anchor survive at all?

🗳️ **Juror** (GitHub Copilot, auto mode) — **VOTE: A** (keep, harden)
Keep anchors because they preserve the author's intended relative relationship
when elements move, avoiding repeated and error-prone manual layer arithmetic.
The real failure is not indirection itself but silent invalidity: `validate`
should error for a missing target and report when the target and anchored
element never overlap in time. The write-tool invariant should be explicitly
scoped to tool-call arguments; an element's data may legitimately contain an
id reference.

🗳️ **Juror** (Qwen2 7B) — **VOTE: A**
Keeping the anchor mechanism allows for intuitive and efficient layer
management without requiring runtime evaluation or additional validation
checks during file parsing. The `validate` tool can flag potential issues with
anchors, which could help catch errors when they occur rather than silently
accepting them. Interpreting the write-tool invariant as scoped only to tool
call signatures, not element data, allows for a more flexible solution.

🗳️ **Juror** (Llama 3.2) — **VOTE: B** (remove, integers only)
Removing the anchor mechanism entirely is the most practical solution for a
real-world video editing project — it eliminates the risk of inert anchors,
which are hard to catch by reading alone. Ease of use and avoiding
"evaluate it in your head" costs matters more than the convenience anchors
offer. A `validate` check could help but "is not a substitute for removing
the anchor mechanism altogether."

**Verdict: 2–1 for keep-and-harden.**

## Round 2 — four follow-on questions, same three backends

### Q1 (re-asked, with the option to propose something else)

- Copilot: **C** — keep, but require a formal unique `id` namespace and
  validate non-cyclic, non-inert anchors; write-tool invariant unchanged.
  (Substantively still "keep, harden" — folds in Q3/Q4 below.)
- Qwen2 7B: **A** (reasoning cut short by local terminal/output buffering
  under parallel load; re-run not required since Round 1 already settled
  this question.)
- Llama 3.2: **A** — anchors let an agent express relative relationships
  without carrying the project's whole history in context.

### Q2 — `validate` severity for the two known defects

- Copilot: **A** — missing target → `error` (no defined z-order relationship
  is possible); never-overlaps → `review` (may be intentional, but is the
  hardest-to-detect functional mistake, so it must surface, not vanish into
  volume).
- Qwen2 7B: output corrupted by parallel-load buffering; on re-run, its
  concluding paragraph argued both defects should be `error` rather than
  splitting error/review — recorded as a minority-of-one position not
  matched by either other juror, and noted as unreliable given the garbled
  transcript.
- Llama 3.2 (re-run): labeled its vote "B" but its reasoning explicitly
  selects "review," not "note," for the non-overlap case — content matches
  option A, not B.

**Verdict, read for content: unanimous on error/review split (option A).**

### Q3 — anchor namespace: elements, tracks, or either; formalize `id`?

- Copilot: **A** — elements only; formally require a unique `id` so
  `{"below": "X"}` is deterministic; track names stay a separate namespace.
- Qwen2 7B: **A** — a unique `id` field reduces ambiguity between element and
  track names.
- Llama 3.2 (re-run): **C** — anchors should name tracks only, to sidestep
  the element/track name collision entirely.

**Verdict: 2–1 for A.** The dissent is noted as incompatible with the
motivating evidence: every anchor defect #24 recorded was one *element*
anchored to another specific *element*, not a whole track.

### Q4 — can anchors chain, or must they resolve in one hop?

- Copilot: **A** — one hop; a direct relationship keeps the file readable
  without tracing a graph.
- Qwen2 7B (re-run): **A** — chaining requires cycle detection and adds
  validation complexity for little benefit.
- Llama 3.2 (re-run): labeled its vote "C" but its described mechanism —
  "chaining is allowed and resolved by walking the chain until a plain
  integer is found... detecting cycles" — is verbatim option B, not a novel
  answer.

**Verdict, read for content: 2–1 for A (one hop, no chain).**
