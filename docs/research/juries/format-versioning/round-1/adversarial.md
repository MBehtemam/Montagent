# Verdict — ticket #14, adversarial seat

## Headline

Add nothing to the file format now. Not a top-level `montaget` integer, not
per-object versions, not a migration binary. Every real breaking change this
project has made so far (0012's required fields, 0014's required text box,
0015's required `fit` and retired `gravity`) is a *shape* change, and shape
changes are already fully detectable by a schema-driven `validate`/`render` —
tools that exist in the surface today and that ADR-0006 already commits to
running unconditionally and refusing on error. The one change that a
file-carried field genuinely cannot help with — ADR-0013's floor rule, which
alters what a legal file renders to while altering zero bytes — is *also* a
case a version field cannot help with, for a reason stronger than "it's
inconvenient": the change isn't a fact about the file at all, it's a fact
about which tool binary you run. A field in the document can't encode "my
meaning depends on code outside me." Where the ticket's four questions ask
"what shape should the version field take," my answer to three of them is
"there shouldn't be one yet," and to Q4 specifically the answer is that the
question bundles two unrelated things — file shape and renderer semantics —
under one imagined field, and they need different mechanisms, not one bigger
one.

I went in with this as my assigned default, not my discovered conclusion, and
I want to be honest about where the evidence pushed back on it (see
concessions inline in Q1–Q4, and "What I could not settle"). It did not push
back on the topline recommendation. It did push back on my initial mental
model of *why* — I originally expected to argue "nothing to distinguish, so
no field," and the ADR-0013 exercise (A4) produced a sharper argument than
that: there is something to distinguish, and a version field is structurally
the wrong tool for distinguishing it.

## Part A findings

Full detail in `WORKLOG.md`. Summary:

- **A1**: The real pre-/post-ADR-0015 pair is git history, not a
  reconstruction I had to invent — `3b795255^` is version N. Diff confirms
  fact 3/5 exactly: 8 `gravity` deletions, zero other bytes.
- **A2**: A tool would know the file needs migrating by treating one field
  name, `gravity`, as an unconditional schema error under the current ADR set.
  That signal is unique in this file and (as far as I could check across all
  fifteen ADRs) unique across the format's history — no two ADRs'
  field-presence signals collide. This required a schema artifact that
  **does not exist** (fact 2) — I was reasoning from the ADR prose, which is
  the actual bottleneck, not the absence of a version field.
- **A3**: I constructed the ambiguous file the brief asked for — any
  text/shape/audio-only project (the fixture's own `caption` or `narration`
  track alone) is legal under both revisions, since `gravity`/`fit` never
  applied to those element types either way. But the ambiguity is inert: the
  two revisions validate and render it identically, so nothing is actually at
  stake in not being able to tell them apart. I looked for a *consequential*
  ambiguity — a file that validates or renders differently depending on which
  side of an ADR it's on but that a schema can't tell apart — and did not find
  one among the fifteen ADRs, except ADR-0013, which is a different failure
  mode entirely (see A4/Q4).
- **A4**: this is where my prior actually moved. I expected "zero bytes
  change, so no field would help" to be a weak, slightly cheap argument. It
  turned out to be a strong one for a different reason: the file's status
  under ADR-0013 is a pure function of which validate/render binary is run
  against it, and `render` already refuses on error unconditionally (ADR-0006,
  verified by reading the ADR directly, not taking the brief's summary on
  faith). A version integer changes none of that — it can announce "something
  changed here" but not what, and `render`'s refusal already announces that
  for free, at the one moment it's load-bearing (before a video ships).

## Q1 — Is there a version in the file, and what is versioned?

**Preferred answer: no field, now. If one is ever added, it should be a
single tool-stamped top-level integer (candidate (a)), never agent-authored,
and it should be understood as a changelog pointer, not a compliance claim.**

**Attack, hardest first:** A1–A3 show shape-based schema validation catches
every structural breaking change this project has actually made. But A4 shows
a real class of change — ADR-0013 — that no version field, tool-stamped or
not, can help with, because it changes meaning without changing shape. If
Q1's candidate (a) can't help with the one demonstrated semantic-only
breaking change, and structural changes don't need it either (schema
validation already gets those), what is the integer actually *for*? I could
not find a failure it prevents that `validate`/`render` don't already prevent
on their own. That is a real gap in my own answer and I am not going to
paper over it with "it's cheap so why not" — cheap-but-purposeless is still
ceremony, and ceremony is exactly what I was asked to presume guilty.

**Where it survives, narrowly:** the one job I can defend for it is *time-to-
diagnosis* for a human once strangers exist (post-1.0, see Q2): "you're 4
ADRs behind, see the changelog" is faster to act on than reading eleven
validate errors and reverse-engineering which of fifteen ADRs you're missing.
That is a real, nameable failure — a stranger burning an hour reconstructing
"what changed" from error text alone — but it is a UX/diagnosis convenience,
not a correctness mechanism, and Montaget has never yet needed it (two real
migrations, both done same-day by the person who wrote the breaking ADR).
I'm recording this as the one point in the whole brief where I concede the
ticket found something real: **a version field's only defensible job is as a
compressed pointer into a changelog for a human doing manual triage, and only
once "manual triage by a stranger, long after the fact" is a scenario that
actually occurs** — which it does not yet, on a project with one file, no
users, and no releases.

Also record fact 6's tension honestly: even that narrow job is undermined if
the field is agent-hand-maintained (a stale, copied-forward "montaget: 3" on a
file that's actually still shaped like 2 is worse than nothing, because it
actively misdirects triage rather than leaving it undirected). This is
survivable only if `fmt`/`create_project`/`render` — tools that already
rewrite the file (CONTEXT.md: "`fmt` normalises on write") — are the sole
writer of the field, never the agent. I did not find this stated anywhere in
the ADRs; it would need to be a new rule, which is itself a small piece of
the machinery I'm supposed to be skeptical of. I'm flagging it rather than
hiding it.

**What would falsify this:** a demonstrated case (real, not hypothetical)
where a stranger's stale file causes a costly, hard-to-diagnose failure that
a version integer — not better validate error messages — would have caught
faster. I don't have one; Montaget has never shipped, so this can't be
measured yet, only reasoned about.

**Confidence: medium.** High on "not now, and not for the reason I expected
going in." Lower on the post-1.0 clause being the right permanent home for
it — see Q2's concession about "1.0" being an underspecified trigger.

## Q2 — Does the policy fire now, or only after 1.0?

**Preferred answer: a stated pre-1.0 clause — the format breaks freely, the
only obligation is the one this project already meets (fact 3: a committed,
re-runnable migration alongside the breaking ADR) — with the trigger being
the first tagged release, decided by whoever cuts it.**

**Attack:** "wait for 1.0" answers a ticket asking for a decision *now* with
"decide later," which risks being exactly the kind of deferral the brief
warns against manufacturing. And "1.0" is not itself a technical criterion —
nothing in the fifteen ADRs, in `docs/agents/domain.md`, or in the tool
surface says what makes a release a 1.0 versus a 0.x. The clause just moves
the ambiguity from "when does versioning start" to "when do we ship," which
is equally unanswered today. A juror gaming this answer could claim victory
by naming an event that's just as undefined as the thing it's supposed to
gate.

**Why it survives anyway:** the two are not equally hard to pin down. "When
do we ship" already has a natural, discrete, auditable answer independent of
this ticket — the first time Montaget is installed and run by someone who is
not the person who can also fix the format — because that is the literal
transition described in fact 8 ("no users and exactly one project file") no
longer holding. Tagging a release is a concrete git event with a timestamp;
"is this decision good" is not. I'm not resolving what 1.0 *means* as a
product milestone (out of scope, and genuinely not mine to decide) — only
that "first external user holds a file Montaget's author can't reach to fix
by hand" is a sharp, observable line, and it happens to be exactly the line
Q3 also needs (see below). One pre-1.0 clause, one trigger, reused by both
questions, is less machinery than two separately-justified ones.

I also want to record a fact-check against my own answer: fact 3 shows the
"committed, re-runnable migration script" obligation already applies and
already costs nothing extra — it's how #42 was done. But fact 3 also shows
#48 (ADR-0015, the gravity removal) did **not** ship a standalone
`migrate.py`/`verify.py` the way #42 did — the fixture's 8-byte change is
inline in the PR diff itself, no separate script artifact in
`docs/research/`. So the "committed, re-runnable script" rule I'm proposing
to keep as the pre-1.0 obligation has already been honored unevenly in this
project's own history: once as a real standalone script, once as an
uncommented inline diff. That's evidence the obligation as stated is softer
in practice than the brief's framing implies, and worth someone tightening
regardless of what this ticket decides on the harder questions.

**What would falsify this:** if the maintainer intends the format to keep
breaking after some already-decided public availability event that isn't
"1.0" in name (e.g. "first GitHub release with a binary," "first MCP registry
listing"), the trigger I named is wrong and needs renaming to match whatever
that event actually is — the substance (discrete, auditable, "can I still fix
every extant file by hand") survives; the label doesn't matter.

**Confidence: medium-high** on the shape of the answer, **low** on whether
"1.0" specifically (versus some other named event) is the right label —
that's a product decision I don't have standing to make.

## Q3 — Whose files must survive N+1?

**Preferred answer: (b), files within reach — a script committed beside the
ADR that caused the break, exactly as #42 and #48 already did. `montaget
migrate` as a standalone, general capability need not exist yet.**

**Attack:** file-as-truth means Montaget can never sweep a database of files
it doesn't know about — but that cuts *for* a general tool, not against one:
if Montaget genuinely cannot see strangers' files, the *only* thing that can
ever fix them is something the stranger runs locally against their own copy,
which is precisely what a general `montaget migrate` is for. Under this
reading, "we can't reach the files" is an argument that a self-service tool
is the *only* possible remedy, not that none is needed.

**Why (b) survives anyway, with a real concession:** I looked for what a
general tool buys over a per-change script and found less than the attack
implies, but not nothing.
- It buys nothing on *correctness*: a per-change script proven against the
  one real fixture (fact 3, #42's regenerate-and-byte-diff) is exactly as
  correct as a general tool would be for that same change; a general tool's
  code path for ADR-0015 is not more correct, just packaged differently.
- It buys nothing on *reachability*: neither approach can reach a file
  Montaget's author has never seen; both require the stranger to run
  something locally.
- **It does buy dispatch** — a stranger with an old file doesn't have to know
  which of fifteen ADR numbers, and which of however-many resulting scripts,
  applies to their file; `montaget migrate` (or `fmt --upgrade`) could try
  each known idempotent transform in order and report what it did. This is
  the concession: **once there is more than a small, memorizable number of
  historical shape-breaks, "run the right numbered script" stops being a
  reasonable ask of a stranger**, and a rule-driven (not version-field-driven
  — A4 already ruled that out) auto-fixer earns its cost. I do not think
  Montaget is there yet — fifteen ADRs produced exactly two fixture-touching
  migrations, both trivially nameable by the human doing it. I would revisit
  this once that count is meaningfully higher, or once there is a second
  real project file in the world that isn't the fixture.

**What would falsify this:** a second real, externally-authored project file
surfacing (even informally, e.g. someone's fork) that needed manual expert
diagnosis to fix would be direct evidence that the dispatch problem is real
today, not hypothetical, and would flip this toward building the general
tool immediately rather than waiting.

**Confidence: medium.** The correctness/reachability claims are solid; the
"count of migrations" threshold for building the general tool is a judgment
call I can't fully defend with data this project doesn't have yet.

## Q4 — What counts as a breaking change?

Classified against "does an old, previously-legal file remain legal" (shape
test) versus "does an old, previously-legal file now mean something
different at render" (semantics test) — because Q1–Q3 already showed these
are different axes needing different mechanisms:

| change | shape-breaking? | semantics-breaking? |
|---|---|---|
| add optional field | no | no |
| add required field (0014, 0015) | **yes** — old files now invalid | no (old files that had it anyway are unaffected; old files that lacked it get a loud, correct error) |
| rename a field | **yes** | no, once renamed files are simply invalid, not silently reinterpreted |
| remove a field (0015's `gravity`) | **yes** — old files now invalid | no — retired fields become errors, not silent reinterpretations |
| change a default (0012, open #61) | **depends entirely on #61** | **depends entirely on #61** |
| change meaning, same bytes (0013) | **no** | **yes** |
| remove an element type | **yes** | no, same reasoning as remove-a-field |

On the default-change row I am declining to resolve #61 as instructed — but I
can state the fork precisely, which the brief's question doesn't quite do:
if `fmt` materializes defaults into the file on write, a default change is
shape-*and*-semantics-safe for every already-written file (old files carry
the literal old value forever) and only affects files written after the
change — i.e., it behaves like "add optional field," not like a break at all.
If `fmt` never materializes defaults, a default change is exactly ADR-0013's
class: zero-byte, meaning-changing, undetectable by schema. **#61 is not a
side issue to this ticket — it silently determines which of two very
different severity classes every future default change falls into,** and I
would not finalize this ticket's classification table without #61 resolved
first, in that specific order.

**Attack on my own classification:** the shape/semantics split makes adding
a required field and removing a field look symmetric (both "shape-breaking,
no semantics risk"), but they are not equally costly to the person holding an
old file: a removed field produces an error on a key the author explicitly
wrote and now must delete; a newly-required field produces an error on a key
the author never thought to write and must now research and add. The second
is strictly more expensive per instance even though both are the same
severity in my table. My table is answering "does validate need to reject
it," not "how expensive is the fix," and the ticket may care about the
latter more than the former.

**Why it survives anyway:** Q4 as literally asked ("which bump the number")
presumes a number exists to bump, which I've already argued against for the
shape-breaking rows (schema validation is the mechanism, not a counter) and
shown is structurally incapable of covering the semantics-breaking row
(0013). So my classification isn't dodging the question — it's showing the
question's own premise (one number, bumped by some breaking changes and not
others) doesn't fit the two failure modes actually present in this project's
history. Answering "which line" as asked would manufacture a single
threshold where the evidence shows two orthogonal ones.

**Direct answer to the "same field or different field" sub-question:**
**different, and not interchangeable.** A file-shape marker (if one existed)
answers "will this validate against a given schema" — already answered
losslessly by running validate, no marker needed. A renderer-semantics marker
would have to answer "was this file's numbers derived under the arithmetic
this tool build uses now" — and the only thing that can truthfully answer
that is the tool build itself re-deriving the numbers and comparing (exactly
what ADR-0013/0015's strict-equality check does), because the file has no
memory of which arithmetic produced its bytes. No field, in the file or
otherwise, substitutes for re-running the derivation. This is the concrete
form of the concession flagged in the headline: Q4's own framing invites
treating these as one decision, and A4 is the proof that it can't be.

**Confidence: high** on the shape/semantics split and on `#61` gating the
default-change row. **Medium** on the "these must be two different
mechanisms, never one field" claim — it's the most novel conclusion in this
verdict and I could not find a fifteenth ADR or a prior jury that had already
tested it; it's my own synthesis from A4, not a re-statement of something the
repo already settled.

## What I could not settle

- **#61 (default materialization)** and **#73 (key order)** are both
  upstream of parts of this ticket in ways the brief's four questions don't
  fully surface — I flagged #61's gating effect on Q4's default-change row
  above, but #73 also matters if anyone later wants `fmt` to stamp a version
  field (Q1's narrow surviving case): a stamped field competes with #73's
  unresolved "no key order is published" for where in the object it goes,
  and whether stamping it counts as the kind of write `fmt` is allowed to
  make unprompted. Neither is decidable from here.
- I could not measure, only reason about, the actual cost of "a stranger
  burns an hour reconstructing what changed from validate error text" (Q1's
  surviving justification) or "a stranger needs expert help to pick the
  right migration script" (Q3's threshold for building a general tool).
  Both are plausible failures I named because the brief requires naming one,
  not measured ones. If this ticket is revisited, running a consumer
  exercise (in this project's own established style — see the fit-vocabulary
  and text/shape juries) with an agent handed a synthetic 3-revisions-stale
  file and no changelog would turn both from reasoned-about into measured.
- The brief asks me to name the trigger for a pre-1.0 clause and I answered
  "first tagged release," decided by "whoever cuts it" — but I have no
  standing to say whether that's the maintainer alone, a jury, or something
  else, and didn't find governance process documented anywhere in the ADRs
  or `docs/agents/`. That's a real gap in the repo, not just in my answer.

## Confidence summary

- Part A findings: **high** — these are direct reproductions from git
  history and direct reading of ADR-0006/0013/0015, not speculation.
- Q1: **medium** — firm on "not now," softer on the post-1.0 shape and on
  whether the concession (changelog-pointer use case) is worth building even
  then.
- Q2: **medium-high** on structure, **low** on whether "1.0" is the right
  name for the trigger versus some other discrete event.
- Q3: **medium** — correctness/reachability claims are solid; the threshold
  for switching to a general tool is a judgment call.
- Q4: **high** on the classification table and the #61 dependency,
  **medium** on the "two different, non-substitutable mechanisms" claim,
  which is this verdict's least-precedented conclusion.
