---
status: accepted
amends: 0112 (a fifth check set, `deliverable`, recorded when `verify`'s measurement completes; the summary line of a verb that runs no check set and still raises a finding is specified; the classes each set can raise are the registry's, which puts no `note` under `disk`), 0117 (its Scope's open question is decided: `verify` has a check set of its own)
---

# `verify`'s measurement is a fifth check set, and a checkless verb's finding prints after its scope

[#403](https://github.com/MBehtemam/Montagent/issues/403), a ticket under spec
[#486](https://github.com/MBehtemam/Montagent/issues/486), builds ADR-0112. Building it settled
two questions that two ADRs left to the implementer, and found one table that does not match the
registry it describes. Under ADR-0031 each is a spec gap until an ADR ratifies it. The two
decisions were put to the human with the real output beside each option, and both were ratified.

## Decision

### 1. `verify` records a check set of its own, `deliverable`

ADR-0117's Scope left this open: *"ADR-0112's `check_sets` … will need a `verify` entry, or will
print this verb's summary as the check-free case. The implementer of ADR-0112 should decide
which."* It gets an entry. `verify` records **`deliverable`** when its measurement of the file
completes, as the other four sets are recorded (ADR-0112 §3).

The check-free case was rejected because it gives up a true zero. A clean `verify` has measured
the file's frame size, timing, extent and audio against the document. As the check-free case,
its whole header would read `no checks run (validate runs them)`, so the one answer the verb
exists to give, *nothing wrong with this file*, would not print. That is ADR-0112's defect in
the other direction: that ADR stops printing zeros that were never earned, and this one keeps a
zero that was.

```
clean verify:        deliverable checks only (validate's checks not run); 0 errors, 0 reviews, 0 notes — p.json
nothing at output:   no checks run (validate runs them); 1 error — p.json
```

What the set covers:

- **The codes it can raise** are those a completed measurement can carry: the `E-`, `R-` and
  `N-VERIFY-` codes, except `E-VERIFY-NO-OUTPUT` and `E-VERIFY-STALE`. Those two return before
  anything is measured (ADR-0117's gate), so they are refusal-shaped. Like `E-OUTPUT-FOREIGN`,
  they are raised by a run that records `[]`.
- **Its classes** are derived like the others: `error`, `review` and `note`.
- **What it does not do.** `deliverable` is not one of validate's halves, so a `verify` run's
  NOT CHECKED still says that validate's checks were not run.

This amends ADR-0112 §2's *"There are four"*. `validate` is still only a verb, and every set
name is still a flat string on the wire.

### 2. A verb that runs no check set and raises a finding prints the finding after its scope

This is #486 Further Note 12. ADR-0112 illustrates a refusal on a checkless verb,
`no checks run (validate runs them); 1 error`, but not a checkless verb raising a finding about
the document. `frame`'s range mode will do that: it records `[]` and raises `N-QUANTIZATION` at
`review`. It gets the same shape:

```
no checks run (validate runs them); 1 review — p.json
```

One rule covers both. A count above zero always prints (ADR-0112 §5), after the scope clause, and
the clause says which sets ran. ADR-0112 §2's reason is why the line does not contradict itself:
**a finding is not a check.** The rejected alternative named the raising verb
(`frame raised 1 review`). It gives a refusal and a document finding two line shapes when the
reader's question is the same for both: *what ran, and what did it find*. It also makes the
renderer name a verb that the JSON does not attribute findings to.

### 3. Each set's classes are the registry's, and `disk` raises no `note`

Every registry entry now declares the check sets that can raise it. Each set's classes are
derived from those declarations and never kept as a second table (ADR-0112 §2: *"The
implementation derives it and does not keep a copy"*). The derivation disagrees with ADR-0112's
table in one cell. The disk half's codes are `E-SOURCE-MISSING`, `E-SOURCE-OVERRUN`,
`E-FIT-DEVIATION`, `U-SOURCE-EXISTENCE-ONLY`, `U-SOURCE-UNPROBEABLE` and
`R-CHROMA-ON-ALPHA-SOURCE`. None of them is a `note`, so `disk` raises `error`, `review` and
`unchecked`. It changes nothing a reader sees: a `disk` run is always preceded by a `document`
run, and `document` raises `note`.

| Set | Classes, as derived |
|---|---|
| `document` | error, review, note, unchecked, layout |
| `disk` | error, review, unchecked |
| `layout` | layout |
| `drift` | drift |
| `deliverable` | error, review, note |

## The wording as built

ADR-0112 §5 and §6 ratified the rule, not the words. The words are generated from the recorded
sets, and no verb keeps its own:

| Recorded | Summary line opens | NOT CHECKED adds |
|---|---|---|
| `document`, `disk` | (nothing) | (nothing) |
| `document` alone | `disk checks not run; ` | *validate's disk checks were not run: the run stopped before them.* |
| `[]` | `no checks run (validate runs them)` | *validate's checks were not run; run validate for them.* |
| any other sets | `<sets> only (validate's checks not run); ` | the same as `[]` |

A set is written as `layout check` when it is the layout set alone, and as `<name> checks`
otherwise. The only way to hold one of validate's halves without the other is a run that
stopped between them, so the NOT CHECKED sentence for that case says so. The finding above it,
`E-TOOL-MISSING` or `E-INTERNAL`, says why.

## Consequences

- `verify`'s JSON carries `check_sets: ["deliverable"]` on a completed measurement, and `[]` on
  every refusal.
- Every registry entry gains `sets`. A new check declares the sets that raise it, and the summary
  line follows that declaration.
- #486's `frame` range mode needs nothing new here. It records `[]`, and its `N-QUANTIZATION`
  prints as §2 says.
