# Jury brief — Montaget format versioning and migration (ticket #14)

You are being consulted on an open design decision. You are **not** reviewing
someone's answer: no answer exists yet, and the person convening you has
deliberately withheld their own view so it cannot anchor yours. You will be
judged on the quality and falsifiability of your argument, never on agreement.

If you find the question itself is malformed, say so. That is a permitted and
valuable verdict here — two previous tickets on this project were resolved by a
juror showing the question was the wrong one.

---

## What Montaget is

An **agent-first video editor**: files in (images, video, audio), video out.
There is no GUI. An external LLM agent (Claude Code, or any MCP client) authors
and edits a **single declarative JSON project file**, which is the source of
truth and lives in the user's git repo. Montaget itself is an MCP server plus a
small CLI. It contains no model and is never an agent.

Standing principles, all settled and **not** up for reopening in this exercise:

- **File-as-truth.** One declarative project file in git is the source of truth.
  Git supplies undo, diff and branching for free.
- **Inert data, no evaluation.** The file must be fully understandable by
  *reading* it. No expression language, no computed properties whose inputs are
  not in the document.
- **The agent edits the file with ordinary file tools** (read, write,
  exact-string replace). Montaget may expose a write tool, but *a tool that
  writes takes a complete element as a schema-shaped object — never a field
  name, never an element id.* The `update_element` / `delete_element` family is
  banned by that invariant.
- **Schema catches malformed, agent catches wrong.**
- Montaget is a **general-purpose** video editor in the CapCut/Premiere class,
  open source, intended for people other than its author. After Effects-class
  compositing (precomps, arbitrary-property keyframes, expressions, plugin
  effects) is out of scope.

## Where to read

Repo: `/Users/mohammedehtemam/projects/github/Montaget`, branch `origin/main`.

- `CONTEXT.md` — the settled domain vocabulary. Read it.
- `docs/adr/0001` … `docs/adr/0015` — fifteen accepted decisions. Skim all the
  titles; read in full any you rely on.
- `fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json` — the
  **only real project file that exists**, 60 elements, 154 lines, migrated from
  a published 65.2 s YouTube short.
- `docs/research/` — prototypes and prior juries.

**You are read-only on this repository.** You have your own git worktree; do not
push, do not touch the user's checkout, and write every artifact of your own
under the scratchpad path given in your instructions. A previous juror on this
project escaped its worktree and reformatted the committed fixture from 154
lines to 1595 while reporting "no schema errors". Do not be that juror.

**One prior artifact needs a health warning.**
`docs/research/renderer-survey.md` remarks that OpenTimelineIO's per-object
schema versioning is an idea to "steal but simplify". That is an **unratified
opinion in a survey**, not an accepted decision, and it was written before any
of the fifteen ADRs. Weigh it as one person's aside, not as precedent.

---

## Verified facts

Each of these was checked against `origin/main` immediately before this brief
was written. Re-check any you intend to rely on.

1. **The fixture carries no version field of any kind.** Its top-level keys are
   `frame`, `fps`, `background`, `duration`, `output`, `fonts`, `tracks`. There
   is no `"montaget"`, no `$schema`, no per-object version.
2. **No JSON Schema artifact exists anywhere in the repo.** The "format" is
   fifteen ADRs of prose. The thing this ticket proposes to version does not yet
   exist as a machine-readable artifact.
3. **The format has already changed fifteen times**, and the fixture has already
   been migrated twice: wholesale by ticket #42 (with `migrate.py` and
   `verify.py` committed beside it, so the mapping re-runs from a clean
   checkout), and again by ticket #48, which deleted 8 `gravity` keys — the
   first byte change ever made to the committed file.
4. **At least three accepted ADRs would be breaking changes under any normal
   policy.** ADR-0014 made the text box required. ADR-0015 made `fit` required
   and retired `gravity` entirely.
5. **ADR-0013 changed what the bytes mean while changing zero bytes.** It
   legislated the rounding rule for fitted extents (floor, in exact integer
   arithmetic); the committed file is byte-identical before and after, and is
   now proved correct by regenerate-and-byte-diff.
6. **Agents demonstrably copy stale fields out of the fixture.** In #48's
   consumer exercise, 8 of 8 agents reached for `gravity` — a field that ADR-0015
   retired — and several copied it out of the fixture *before reading the spec*.
7. **Every agent edits by exact-string replace** (measured in #8, confirmed in
   #9's five-agent edit exercise). Nobody wanted an `update_element`. The file is
   written one element per line, sorted by `start`, with stable key order,
   precisely so a unique matchable substring exists.
8. **Montaget has never been released.** No renderer exists. There are no users
   and exactly one project file.
9. Two adjacent tickets are open and unresolved, and you may not assume either
   way on them: **#61** — nothing says whether `fmt` materialises defaulted
   fields, and ADR-0012 defaults six. **#73** — no key order is published.
10. The tool surface (ADR-0011) is `validate`, `query`, `frame`, `measure`,
    `compare`, `render`, `shift`, `create_project` over MCP, plus `probe`, `fmt`
    and `timeline` on the CLI. The JSON schema and format docs ship as MCP
    **resources**.

---

## The anti-drift rule, and a worked example of breaking it

`fixtures/en-halloween-decorating/` is a **fixture, not a scope**. It is test
data and a regression guard — evidence that the primitives suffice for real
published work. It has **no authority** over what Montaget must do.

> The fixture is evidence that a capability is **needed**. It is never evidence
> that a capability is **unneeded**.

This guard has been broken twice on this project by jurors who had been given
the rule in writing, so here is a worked example of breaking it, on *this*
ticket specifically:

> ❌ **"The fixture has no version field, and it works fine. Therefore no
> version field is needed."**

That is the exact inversion. The fixture's silence on versioning is a fact about
a file hand-written by one person for a format that has never shipped. It is not
evidence about what a stranger's project needs in three years. A previous round
of this project's juries found that a written guard alone was insufficient and a
written guard *with a worked example* was sufficient — you now have both.

The inverse move is equally forbidden: do not argue *for* machinery on the
grounds that "real formats have it". Name the failure it prevents.

---

## Part A — do the work before you opine (mandatory)

Do not skip to Part B. Several findings on this project came only from agents
who were made to perform the task before being allowed an opinion.

**A1.** Copy the committed fixture into your own scratchpad directory. Working
only on your copy, **perform the ADR-0015 migration in reverse**: produce the
file as it stood *before* ADR-0015 (8 `gravity` keys present, `fit` not yet
required). You now hold a genuine "version N" file and a genuine "version N+1"
file that differ by one real accepted breaking change.

**A2.** Now answer, by doing it rather than by reasoning about it: **given only
the version-N file and the published ADRs, how would a tool know that file needs
migrating?** Try it. Report what you actually had to look at. If you had to
infer the revision from the presence or absence of fields, say exactly which
fields carried the signal and whether that signal is unique.

**A3.** Construct the adversarial case: a file that is **ambiguous** — one that
could legally be either revision, or that a shape-sniffing reader would
misidentify. If you cannot construct one, say so and show why not; that is a
real result.

**A4.** Now take ADR-0013's change (fact 5 above): a revision that alters what a
legal file *renders to* while altering **zero bytes**. Answer concretely: under
your preferred design, what happens to a file that crosses that revision? Show
your work.

Record A1–A4 in a `WORKLOG.md` in your scratchpad directory before you write
Part B. Part B answers that contradict your own worklog will be discarded.

---

## Part B — the four questions

For **every** question: **state your preferred answer, then attack it first and
hardest, before you defend it.** A question where you cannot construct a serious
attack on your own answer is a question you have not finished thinking about —
say so explicitly rather than manufacturing a weak attack.

Cite evidence. Where you are speculating, label it speculation. Where a claim is
falsifiable, say what would falsify it.

**Q1 — Is there a version in the file at all, and what is versioned?**
Candidate shapes: (a) one top-level integer, e.g. `"montaget": 1`, naming a
revision of the whole format; (b) OpenTimelineIO-style per-object versions
(`"Clip.5"`) with chained upgrade functions; (c) no version field at all — the
revision is inferred from the file's shape. Note the tension you must address
either way: a version integer is a field the **agent hand-maintains**, and fact
6 says agents copy stale fields out of examples. Is a confidently wrong version
number worse than no version number?

**Q2 — Does the policy fire now, or only after a stated 1.0?**
Montaget has never shipped (fact 8) and the fifteen changes behind us cost
nothing to make. Does the spec state a pre-1.0 clause — the format breaks freely
and the only obligation is that the fixture is migrated with a committed,
re-runnable script (a rule `docs/agents/domain.md` already imposes) — with the
versioning machinery describing life after release? Or does it bind from the
next ADR onward, making ADR-0016 the first that must carry a version bump and a
migration? If you choose the former, say what event trips the switch, and who
decides.

**Q3 — Whose files must survive N+1?**
Is the design target (a) files Montaget never wrote and cannot see — in
strangers' repos, possibly hand-edited, possibly half-migrated — which makes
migration a shipped, standalone, idempotent capability; or (b) only files within
reach, which makes a migration a script committed beside the ADR that caused it,
exactly as #42 and #48 already did, and may mean `montaget migrate` need not
exist? Under file-as-truth Montaget never holds a database it can sweep. State
what a general migration tool buys over a per-change script, in failures
prevented, or concede that it buys nothing.

**Q4 — What counts as a breaking change?**
Classify these, independent of who pays: adding an **optional** field; adding a
**required** field (ADR-0014's text box, ADR-0015's `fit`); **renaming** one;
**removing** one (ADR-0015's `gravity`); **changing a default** (ADR-0012
defaults six fields, and #61 is open); **changing the meaning of an existing
value with no spelling change** (ADR-0013, fact 5); removing an element type.
Which bump the number, and why that line rather than one a field over? Address
the last case directly: if a revision can change the rendered output while
changing no bytes, is the version a statement about the **file's shape** or
about the **renderer's reading** — and if it is the latter, is it the same field
as the one Q1 asks about, or a different one?

---

## Output

Write `VERDICT.md` in your scratchpad directory, and return its full content as
your final message. Structure:

- **Headline** — one paragraph. If your answer is "the question is malformed",
  lead with that.
- **Part A findings** — what performing the migration actually showed, including
  anything that surprised you or contradicted your prior.
- **Q1 … Q4** — for each: preferred answer; the strongest attack on it; why the
  answer survives (or that it does not); what would falsify it.
- **What I could not settle** — questions you hit that this brief does not ask,
  and anything you believe needs measuring rather than deciding.
- **Confidence** — per question, and say plainly where it is low.

Your final message IS the deliverable. Do not summarise it for a human; return
the document.
