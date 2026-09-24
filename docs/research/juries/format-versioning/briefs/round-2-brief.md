# Jury brief, round 2 — Montagent format versioning and migration (ticket #14)

Round 1 ran five jurors on four models. This is round 2. You are a fresh juror:
you did not sit in round 1 and you are **not** reviewing round 1's output.

Round 1's splits are reproduced below as **unattributed positions**. You are not
told who held which, how many held each, or which the convener prefers — because
in a previous round on this project, telling jurors the tally is exactly how a
conclusion under test ended up in the jurors' reading list marked accepted, and
that round was voided. Weigh each position on its argument alone.

Read the round-1 brief first, for the project context, the standing principles,
the anti-drift rule and the worked example of breaking it:

`/private/tmp/claude-501/-Users-mohammedehtemam-projects-github-Montagent/6e22850c-f895-4fb4-bef7-dbe42e44211c/scratchpad/jury-14/BRIEF.md`

Everything in that brief still applies **except the corrections below**, which
supersede it. Do not read any other juror's scratchpad directory. You have your
own worktree; the repository is read-only.

---

## Corrections to round 1's brief

Round 1 falsified four things. Three were errors by the person who wrote the
brief. They are stated plainly rather than quietly patched, because a brief that
hides its own corrections teaches you to trust it more than you should.

**C1 — the brief's "fact 5" was wrong, and wrong by the exact failure the brief
lectured jurors about.** It said *"ADR-0013 changed what the bytes mean while
changing zero bytes."* That is true **of the fixture** and false **of the
revision** — generalising from the fixture, which is the drift the anti-drift
rule forbids. A legal pre-ADR-0013 file written by a float implementation may
declare `width: 1919` where exact integer arithmetic gives `1920`, because
`103 * (1920/103)` evaluates to `1919.9999999999998`. Under ADR-0015's strict
equality that is now an **error**, and migrating such a file **rewrites bytes**.
Verified: the divergence occurs on ~5.8% of source widths against the two box
dimensions, the same defect class as ADR-0013's own founding measurement
(4.466% of 31,402,800 combinations). The committed fixture escaped byte-free
only because `1080/1536 = 45/64` is dyadic.

**C2 — the brief's "per-change script" precedent was false.** The brief implied
#42 and #48 each shipped a migration script beside the ADR that caused them.
Neither did. `docs/research/sample-project-migration/migrate.py` regenerates the
current fixture **from the original #9 prototype** (`old.json`), and each new ADR
**edits that one script in place**. So the actual practice is a *cumulative
regenerator from a fixed origin*, which never migrates N→N+1 and works only
because the author owns the origin file. A stranger has no `old.json`. Treat the
"two successful migrations" precedent with that in mind: it may not be evidence
for what round 1 took it to be evidence for.

**C3 — ADR-0015 did ship re-runnable evidence.** One round-1 juror claimed it
shipped only an inline PR diff. Commit `3b795255` **added**
`fit_vocabulary_scan.py` and **modified** `migrate.py` and `verify.py`. The
round-1 brief's fact 3 stands.

**C4 — nothing in this project states an unknown-key policy.** A round-1 juror
argued that adding an *optional* field is breaking because new files break old
readers, on the premise that the schema rejects unknown keys. Grepping all
fifteen ADRs and `CONTEXT.md` for `additionalProperties`, "unknown key/field",
"unrecognised" returns **nothing**. ADR-0015 makes `gravity` specifically a
schema error; no general policy exists. Do not assume either way.

## New verified measurements from round 1

**M1 — three consecutive accepted revisions, one blob.** The committed fixture
is byte-identical at ADR-0012, ADR-0013 and ADR-0014:

```
afc12d24 (ADRs 0001-0012)  cd8e797109b833e234c7a4b1ed19339b28182612
ed3db37c (ADR-0013)        cd8e797109b833e234c7a4b1ed19339b28182612
2f8e9405 (ADR-0014)        cd8e797109b833e234c7a4b1ed19339b28182612
3b795255 (ADR-0015)        a2e64f7c162c55d6f23808dad7c28f0a0c67020c
```

Not because the fixture is small — because it is **correct**, and correct files
are exactly the files that survive a tightening unchanged.

**M2 — a removal migration is not invertible from the published specification.**
The pre-ADR-0015 file carried `gravity` on 8 elements: `"top"` on the seven
photos and **`"center"` on `handle-logo`**. No accepted ADR records that last
byte. A round-1 juror reconstructing the file from the ADRs alone got 7 of 8
right and that one wrong. The only witness to a retired field's *values* is git.

**M3 — `montagent migrate`'s hardest case has no replacement to name.** Round 1
converged on repairing stale files through `validate` error messages that name
the replacement, as every ADR in the chain already does (ADR-0015 does not say
"migrate `gravity` away"; it says *"a schema error naming `x`/`y`/`origin` and
`clip`"*). **Removing an element type breaks that pattern** — there is no
replacement to name. This was flagged as unresolved by more than one juror.

---

## Part A — do the work before you opine (mandatory)

Write `WORKLOG-R2.md` in your scratchpad before writing your verdict. Part B
answers contradicting your own worklog will be discarded.

**A1 — ground yourself.** Re-verify M1 yourself (`git rev-parse <commit>:<path>`)
and confirm or refute C1's arithmetic. Two minutes; do not take either on trust.

**A2 — run the migration-by-error-message exercise, both roles.** Take the real
pre-ADR-0015 file (`git show '3b795255^:fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json'`)
into your scratchpad. First act as **`validate`**: write out, literally, the
error messages you would emit for that file under the current ADR set. Then act
as **the agent**: using only those messages and the published ADRs — not git,
not the post-0015 file — repair it. Diff your repair against the real
post-ADR-0015 file. Report the byte-level result and, more importantly, **what
you had to guess**. This is the core test of whether a migration tool is needed
at all; do it, do not reason about it.

**A3 — construct the element-type removal case (M3).** Invent a concrete,
plausible retirement — say `ellipse` is retired and its uses must become `rect`
plus something, or an element type is removed with no successor. Write the
`validate` message. Then decide: can an agent repair it from that message? If
not, say precisely what is missing and whether *any* mechanism could supply it.

**A4 — test the identification claim independently of versioning.** List what is
actually in `fixtures/en-halloween-decorating/` and say whether a tool handed an
arbitrary path can currently tell a Montagent project from any other JSON. Then
answer: does a magic marker earn its place on **identification alone**, with no
versioning argument at all? This matters because if it does, the marker and the
version number are separable decisions and must be taken separately.

---

## Part B — the questions

For **every** question: state your preferred answer, then **attack it first and
hardest**, then say whether it survives. A question where you cannot construct a
serious attack on your own answer is one you have not finished thinking about —
say so rather than manufacturing a weak attack. Cite evidence; label speculation
as speculation; say what would falsify you.

### Q5 — Does the project file carry a marker at all, and is it a version?

Positions from round 1, unattributed:

- **P1.** One top-level integer, `"montagent": N`, naming a revision of the whole
  format contract (shape *and* normative reading).
- **P2.** A top-level marker is justified by **file identification alone** — a
  `.montagent.json` with no marker is indistinguishable from any JSON with a
  `tracks` key — and the version number is a *free rider* on a marker that is
  independently warranted. On this view the versioning half rests on an
  **unmeasured** ergonomic claim, and a defensible outcome is a marker frozen at
  `1` forever.
- **P3.** **No field.** Every structural break is already caught by a
  schema-driven `validate`/`render`, which ADR-0006 commits to running
  unconditionally and refusing on error. The one class a field cannot help with
  (C1's class) it cannot help with for a structural reason: the file's status
  there is a fact about *which binary you run*, not about the document. A field
  in the document cannot encode "my meaning depends on code outside me."
- **P4.** Per-object versions, OpenTimelineIO-style. *(Round 1 rejected this
  unanimously; it is listed for completeness. If you disagree with the
  rejection, say so — a unanimous round-1 verdict is not binding on you.)*

Address directly: is a confidently wrong version number worse than no version
number? And does A4's result make the marker and the version separable?

### Q6 — If a number exists: lower bound, or equality?

- **P5.** It states **the revision the file was authored against** — a lower
  bound. A file declaring `1` in a world at revision 4 is **not an error**.
  Forced by M1: if the declared number had to equal the current revision, those
  three revisions would each have obliged a rewrite of every correct file in the
  world as pure ceremony, and ceremony fields are what agents copy stale.
- **P6.** It states **the revision the file conforms to**, and a stale number is
  a defect to be corrected — the number is only useful if it can be trusted to
  be current.

Note that M1 was measured after most round-1 positions were formed, and not
every position addressed it. Say whether it changes your answer.

### Q7 — What event trips the policy from "breaks freely" to binding?

- **P7.** The **first tagged release** / first external user.
- **P8.** The **publication of the JSON Schema resource** (ADR-0011 already
  schedules it as an MCP resource), on the ground that *a version number names a
  legality predicate and Montagent currently has none* — the format is fifteen
  ADRs of prose, and ADR-0013 explicitly left "whether `fit` may be omitted"
  undefined, so at that revision there was no fact of the matter about whether a
  given file was legal. You cannot index a predicate that does not exist. Under
  this, all fifteen ADRs collapse into revision 1 and ADR-0016 carries no bump.
- **P9.** Split it: the **mechanics** bind from the next ADR, the **promise** to
  strangers binds at release.

### Q8 — Does `montagent migrate` ever exist?

- **P10.** **Never.** The distinction is the project's own: `CONTEXT.md`
  justifies `shift` as *"the one edit that is arithmetic rather than authorship,
  and therefore the one Montagent performs instead of the agent."* Deletions and
  renames are arithmetic and may ship as scripts; synthesising a retired `fit`
  value is **authorship**, because under strict equality a stale rect may match
  neither `cover` nor `contain`, leaving only `literal` — an assertion about what
  the author meant. A tool writing it fabricates intent to make a file pass.
  Montagent is agent-first: the repairing entity is always an LLM, and repair from
  a replacement-naming error message is the same act as the authoring the format
  already assumes.
- **P11.** **At release**, as the chain of per-change scripts made idempotent by
  the declared number. What it buys over hand-run scripts is **dispatch and
  ordering** — a stranger several revisions stale cannot be asked to find and
  sequence N scripts — plus a place for M3's case to **refuse loudly** and name
  the affected element ids, which a script nobody finds cannot do.

Your A2 and A3 results bear directly on this. Let them, not your prior, decide
it. Note C2: the precedent both positions lean on may not be what it appears.

### Q9 — Is the version one field or two mechanisms?

- **P12.** **One field, naming the renderer's reading.** By elimination: a
  version describing only the file's *shape* is redundant with the schema, since
  checking shape against a published schema is what a schema is for. The one
  thing a schema cannot tell you is that identical legal bytes now mean a
  different picture. That is the version's exclusive job, so a shape-version has
  no job left and a second field would have nothing to do.
- **P13.** **Two non-substitutable mechanisms** — a shape marker a schema-diff
  verifies mechanically, and the ADR/changelog trail as the only honest home for
  semantics-only revisions, because no field placement lets a document
  self-describe an interpretation rule its author never knew existed. Forcing
  both into one integer either breaks shape-conformance's mechanical
  checkability or leaves semantics revisions unrecorded anywhere machine-readable.

Note C1 sharpens this: the semantics-only class is **not** byte-invisible in
general — only on files that happen to sit at the fixed point. Does that change
the answer? Say so explicitly if it does.

### Q10 — Two housekeeping calls, which need a reasoned yes/no, not a vote

**Q10a.** Four of five round-1 jurors independently said **#61** (does `fmt`
materialise defaulted fields? ADR-0012 defaults six) silently determines the
severity class of every future default change: if `fmt` materialises, a default
change is spec-text only; if not, it is C1's class in a cheaper costume. Should
**#61 block #14**? Argue it; do not just agree.

**Q10b.** Should the missing unknown-key policy (C4) be a **new ticket**, a
clause inside this ticket's ADR, or neither? Note it is load-bearing: it decides
whether "add an optional field" is breaking, the one Q4 row round 1 split on.

---

## Output

Write `VERDICT-R2.md` in your scratchpad and return its full content as your
final message. Structure: **Headline**; **Part A findings** (A2's diff result
and what you had to guess is the single most important thing you will report);
**Q5–Q10** each with preferred answer, strongest self-attack, survival, and
falsifier; **What I could not settle**; **Confidence**, per question.

Your final message IS the deliverable. Do not summarise it for a human.
