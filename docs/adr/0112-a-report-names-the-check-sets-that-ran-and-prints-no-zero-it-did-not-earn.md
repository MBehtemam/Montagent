---
status: accepted
amends: 0006 (the NOT CHECKED block stays unconditional but stops being one fixed text — a report that did not run both halves of validate's checks gains a generated sentence naming what did not run), 0011 (every report gains a `check_sets` field; the summary line is generated from it, not from `summary` alone), 0013 (the `unchecked` zero prints only when a check set that can raise it ran), 0041 (the `layout` zero, likewise)
---

# A report names the check sets that ran, and prints no zero it did not earn

[#403](https://github.com/MBehtemam/Montagent/issues/403), found while evaluating
[#395](https://github.com/MBehtemam/Montagent/issues/395). Decided by three juries of three
models each, ratified by the human; ballots verbatim in
`docs/research/juries/report-check-scope/`.

## What was there

Every verb's report opened with the same six-class summary line, generated from the same
`summary` object on the wire. On `fixtures/en-halloween-decorating`:

```
$ montagent validate <project>
0 errors, 12 reviews, 55 notes, 0 unchecked, 0 layout, 0 drift — <path>

$ montagent timeline <project>
0 errors, 0 reviews, 0 notes, 0 unchecked, 0 layout, 0 drift — <path>
```

`timeline` runs no checks. Its zeros meant *nothing was looked for*, in exactly the shape that
means *nothing was found*. The sweep found the same on **eight** verbs, not one:

| Checks run | Verbs | What the zeros meant |
|---|---|---|
| validate's full set | `validate`, `render`, `preview`, `shift`, `create-project` | the document's findings — **except `0 drift`** |
| one subset | `fmt` (the layout check), `compare` (its own drift checks) | one class real, five vacuous |
| none | `timeline`, `query`, `frame`, `measure`, `probe`, `fonts` | only the verb's own refusals |

Two further facts came out of the sweep. **Every `D-` code is raised by `compare` alone**, so
`validate`'s own `0 drift` was a zero for a check `validate` never runs. The bug was on the
verb people trust most, only smaller. And the NOT CHECKED block ended every report with one fixed
sentence — *"validate verifies that the file is internally legal"* — so `timeline`'s statement of
its own scope was word for word `validate`'s.

**It misled a careful agent.** During the #395 trial an agent truncated `validate --verbose`
with `tail`, cutting off the `review` section holding `R-SOURCE-CUT-POP`, the finding that named
the defect it was hunting. It then ran `timeline`, read the all-zeros header as confirmation
that validate had found nothing, and lost fifteen minutes. The `tail` was its own mistake; the
header turned a recoverable mistake into a confirmed false belief. This is ADR-0006's warning
arriving on a verb that runs no checks at all: *"`0 errors, 47 notes` reads as a pass."*

## Decision

### 1. Every report states which check sets ran, as data

The canonical JSON gains **`check_sets`**, a list of names. `summary` keeps all six keys on
every verb and keeps its meaning — *findings raised, per class* — so nothing that indexes it
breaks (ADR-0011's *"exactly one thing to parse"*). The prose is generated from the pair.

The rejected alternative was a text-only fix, with the renderer inferring "checks not run"
from `tool`. It fixes the channel the incident happened on and leaves the one agents mostly
read: an MCP client reading `summary.error == 0` from `timeline` is misled exactly as the text
reader was. It also makes the renderer a second, silent registry of which verbs check, which
is what *"nothing is assembled twice"* forbids.

### 2. The field names check sets, not finding classes

A **check set** is a named group of checks a run executes. There are four:

| Set | What it is | Who runs it | Classes it can raise |
|---|---|---|---|
| `document` | validate's checks that read only the file and its fonts | validate's set | error, review, note, unchecked, layout |
| `disk` | validate's checks that need `ffprobe` — source, fit, chroma-on-disk | validate's set | error, review, note, unchecked |
| `layout` | the key-order/line-break check, alone | `fmt` | layout |
| `drift` | `compare`'s own checks | `compare` | drift |

`validate`, `render`, `preview`, `shift` and `create-project` record `["document", "disk"]` on
a complete run. The class column is today's registry, not a separate promise. The
implementation derives it and does not keep a copy.

Classes were the rejected alternative, and fact 3 of the second brief is why: **a refusal is
a finding without being a check.** `frame` runs no checks but can raise `E-NOT-A-PROJECT`, an
`error`. A class-valued field would have to say either *"error was checked"*, which is false,
or *"error was not checked"* beside `summary.error == 1`, which contradicts it. The check-set
field has neither problem: `[]`, one error.

### 3. The field is recorded as checks execute, never declared

The code that runs a check set marks the report **when the set completes**, and a new report
starts at `[]`. Declaring the set per verb was rejected because it is false on real runs:
`validate` pointed at a transcript export returns `E-NOT-A-PROJECT` before any check runs, so a
declared `["document", "disk"]` would put the incident into `validate`'s own output. The
default is the safe direction. A verb that forgets to record reads *"no checks run"* in its
header rather than claiming a run it never made.

### 4. The two halves are named because validate already stops between them

The first jury wrote `validate` as one set. The third jury split it, and the reason is a run
the first two panels were never shown. With `ffprobe` missing, the document half completes, its
findings **stay in the report** (the code's own reason: *"an agent told only the second would fix
its `PATH`, re-run, and only then hear about the key it could have fixed in the same turn"*),
`E-TOOL-MISSING` is added, and the run exits 70. That run records `["document"]`.

Three other recordings were weighed:

- **Record the set on entry.** The field says the disk half ran when it did not. This is the
  incident again, on the one field built to prevent it.
- **Record only a complete set, so `[]`.** This was the judge's own recommendation, and all
  three jurors rejected it. It prints *"no checks run"* above findings the document half really
  raised, which breaks §3's own rule (the document half *did* execute) and teaches the reader
  that header and findings can contradict each other.
- **A partial marker, `validate:partial`** (Juror 1). It keeps one name but makes the field
  names-plus-state, and every consumer must learn what *partial* means for each set. The split
  keeps a flat list, and it names a boundary the code already has, one it did not invent.
  Both are fail-closed for a naive `"validate" in check_sets`, because under the split no name
  `validate` exists to match.

**A project that references no media records `disk` as run.** The disk runner is entered,
finds nothing to probe, and completes. So a no-media run is complete on the wire, as it
is in fact, and only a stopped run reads as partial. Juror 1 raised this ambiguity against the
split, and Jurors 2 and 3 each closed it independently. It needs its own test.

Juror 2 also proposed listing `validate` alongside the two halves whenever both ran. That was
dropped, because it puts two spellings of one fact on the wire.

### 5. The summary line prints a zero only where a check set that ran could have raised it

A count above zero always prints. A zero prints only for a class some recorded check set can
raise (§2's table). When the recorded sets are not both halves of validate's, the line says so
first. Illustrative wording, not pinned by this ADR:

```
validate (complete):   0 errors, 12 reviews, 55 notes, 0 unchecked, 0 layout — <path>
validate (no ffprobe): disk checks not run; 0 errors, 1 review, 3 notes, 0 unchecked, 0 layout — <path>
fmt --check:           layout check only (validate runs the rest); 0 layout — <path>
compare:               drift checks only (validate's checks not run); 2 drift — <path>
timeline:              no checks run (validate runs them) — <path>
frame (refused):       no checks run (validate runs them); 1 error — <path>
```

**`0 drift` leaves `validate`'s line**, and so leaves the line `render`, `preview`, `shift` and
`create-project` share. Keeping all six zeros on a full validate run was rejected unanimously.
It would keep one false zero on the most-read verb and make the rule *"zeros are honest, except
on validate"*. ADR-0013 and ADR-0041 added `unchecked` and `layout` to this line so that a zero
could not hide something unprobed. Dropping `0 drift` is the same principle in the other
direction: no zero may claim a probe that did not happen. The JSON keeps `drift: 0`, so only
the prose, which is generated, changes shape.

### 6. NOT CHECKED names what did not run

The block stays unconditional (ADR-0006). A report whose `check_sets` is not
`["document", "disk"]` gains one generated sentence naming the sets that did not run and the
verb that runs them, e.g. *"validate's checks were not run; run validate for them"*, or, on a
stopped run, *"the disk checks were not run: ffprobe is missing"*. The header prevents the
misreading at the top. The block keeps the report's own closing statement of scope true, which
is the job ADR-0006 gave it, and which the fixed sentence was failing on every verb that does not run validate's checks.

## Consequences

- **Wire format.** Every verb's JSON gains `check_sets`, so fixtures, snapshot tests and the MCP
  contract move. `summary` is unchanged.
- **Prose.** The first line changes on every verb, including `validate` (`0 drift` goes).
  Anything pattern-matching six fixed fields breaks. The canonical interface is JSON, and this
  was accepted.
- **Recording discipline.** There are four run sites (the document half, the disk half, `fmt`'s
  layout call, `compare`'s drift checks), not twelve declarations. Each verb needs a test that
  its normal path records the expected sets and its refusal path records `[]`. The no-media
  run and the missing-`ffprobe` run each need their own.
- **Naming.** `validate` is now only a verb, never a set name. Prose that says *"validate runs
  them"* names the verb that runs both halves.
- **Build work stays on #403.** Like #405 before it, the decision closes here and the code sites
  are the open issue's.

## Honest costs

- All nine jurors across the three panels were Claude models given briefs the judge wrote.
  Agreement among them is weaker evidence than it looks. The strongest points are the ones each
  juror derived from the code facts rather than the brief's wording: the refusal-is-not-a-check
  argument (§2), and the no-media closure (§4), which two jurors reached independently.
- The judge's own brief for the second panel described the missing-`ffprobe` run as simply
  *"an internal-failure exit of its own"*, and one juror concluded from that that no partial
  state existed. The third panel was convened because that wording was wrong. §4 rests on the
  corrected fact, read from `verbs/validate.rs`.
- The header wording in §5 is illustrative. What is ratified is the rule (which zeros print,
  and that a partial or empty run says so first), not the words.
