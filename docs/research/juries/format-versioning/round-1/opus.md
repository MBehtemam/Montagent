# VERDICT — format versioning and migration (ticket #14)

## Headline

The question is well-formed, but **one of its three candidate answers is not a
candidate**. Option (c) — "no version field; the revision is inferred from the file's
shape" — is not a design choice that can be made, because I could not construct a
single file that shape places uniquely. The committed fixture is legal at all four
revisions in the window I tested and carries **zero** distinguishing signal; the
genuine pre-ADR-0015 file narrows only to a **three**-revision window that contains
two breaking changes; and ADR-0013 changed what a legal file means while changing zero
bytes, which defeats shape inference by construction. My answer is therefore (a): **one
top-level integer, `"montagent": N`, which is a claim the author makes about the spec
they wrote against — a lower bound, never an equality — checked by `validate` against
shape and obeyed by no renderer, ever.** It is the same species of field ADR-0015
invented for `fit`: a provenance tag whose only consumer is `validate`. It should not
exist until the JSON Schema resource exists, because a version number names a legality
predicate and Montagent currently has none (fact 2). And there should be **no
`montagent migrate` verb**: migration that is arithmetic can ship as a script beside its
ADR, as #42 and #48 already did, and migration that requires deciding what the author
meant is authorship, which in this project is the agent's job and not the tool's.

---

## Part A findings

Full record in `WORKLOG.md`. The four things worth carrying forward:

**A1 — a deletion migration is not invertible from the specification.** I reversed
ADR-0015 by re-inserting `,"gravity":"top"` after each `"fit":"…"`, taking both the
value and the key position from the pre-0015 `photo-06` element that **ADR-0013
publishes verbatim**. It fired on exactly 8 elements, as ADR-0015's Consequences
predicted. Grading against `git show 3b795255^:…`, I got **7 of 8 byte-exact** and one
wrong: `handle-logo` was `gravity:"center"`, not `"top"`. No published ADR records that
byte. The documentation preserved the field's *presence* and its *inertness*; it never
preserved its *values*. My 7/8 was luck — seven elements happened to share one value
that one ADR happened to print. Under file-as-truth the only witness to a retired value
is git, which is the correct place for it and is also an argument that "downgrade" is
not a capability anyone needs to specify.

**A2 — a tool cannot know.** I wrote a shape-sniffing detector from the ADRs
(`sniff.py`; five licensed signals). On the genuine version-N file the **only** signal
is the key name `gravity`, present 8 times. `fit` carries none, because ADR-0015 states
8 of 8 elements already had it. And `gravity` places the file in a *window*
— created by #42 under ADR-0012, retired by ADR-0015 — spanning three accepted
revisions, two of which (0013's rounding rule, 0014's required text box) are breaking
under any normal policy and are invisible to shape.

**A3 — I could not construct an unambiguous file.** That is the result, and it is
structural rather than incidental. Every change that *removes* a field makes an old
file and a malformed new file byte-identical; every change that *requires* one does the
same. I built both (`amb-B`: a raster element with `fit` deleted; `amb-C`: a text
element with its box deleted) and a sniffer must choose between "silently upgrade" and
"report an error" on identical bytes. The project has already decided that question in
general — **"Schema catches malformed, agent catches wrong."** A shape-sniffing
versioner is a schema that silently repairs instead of reporting. It is against a
standing principle, not merely unreliable. Worse for amb-B: the repair is not
performable. Synthesising `fit` needs the source probed *and* a choice between `cover`,
`contain` and `literal`, and under ADR-0015's strict equality a stale rect may match
neither rule — leaving `literal`, which is an assertion about **what the author meant**.
A migrator writing it would be fabricating intent to make a file pass.

**A4 — the surprise, measured by hash.** The fixture's blob is identical at ADR-0012,
ADR-0013 and ADR-0014:

```
afc12d24 (ADRs 0001–0012)  cd8e797109b833e234c7a4b1ed19339b28182612
ed3db37c (ADR-0013)        cd8e797109b833e234c7a4b1ed19339b28182612
2f8e9405 (ADR-0014)        cd8e797109b833e234c7a4b1ed19339b28182612
3b795255 (ADR-0015)        a2e64f7c162c55d6f23808dad7c28f0a0c67020c
```

**Three consecutive accepted revisions, one blob.** Not because the fixture is small —
because it is *correct*, and correct files are exactly the files that survive
tightenings unchanged. From which the load-bearing constraint on any design here:

> **A revision bump must not oblige a correct file to be rewritten.** If the declared
> integer had to *equal* the current revision, those three revisions would each have
> forced a rewrite of every correct file in the world as pure ceremony — and ceremony
> fields are precisely what fact 6 says agents copy stale.

This contradicted my prior. I went in expecting to treat shape-sniffing as a
weaker-but-workable fallback and to justify a version integer on ordinary grounds. A2
and A3 removed the fallback; A4 removed "the integer equals the revision."

---

## Q1 — Is there a version in the file, and what is versioned?

### Preferred answer

**(a), with three qualifications that do most of the work.**

One top-level key, first in the file, `"montagent": 1`. And:

1. **It is a lower bound, not an equality.** It states *the revision of the spec this
   file was authored against*. A file saying `1` in a world at revision 4 is **not an
   error**. Forced by A4.
2. **`validate` is its only consumer. No renderer reads it.** There is one renderer and
   it implements the current revision. No chained upgrade functions, no compatibility
   modes, no per-object versions. This is exactly ADR-0015's category for `fit` — *"a
   claim about how the author computed…, and its only consumer is `validate`"* — and
   the format now has a precedent and a vocabulary for it.
3. **It does not exist until the JSON Schema resource exists.** See Q2.

Reject **(b)**, OpenTimelineIO-style per-object versions with chained upgrades, twice
over: it multiplies by ~60 the number of hand-maintained integers in the fixture, each
one a fact-6 copy hazard, in a format whose elements are written **one per line for
exact-string replace** (fact 7) — every element line would carry a token that is stale
the moment it is copied. And its whole purpose is to *drive* a reader, which is the one
thing I am arguing the number must never do. The `renderer-survey.md` remark that this
is worth stealing-but-simplifying is, as the brief flags, an unratified aside; A1–A4
are evidence against even the simplified form.

Reject **(c)** as shown above: it is not a choosable option.

### The strongest attack on my own answer

**The field buys nothing that shape does not already buy, and the brief forbids exactly
this move.** Everything `validate` can report, it reports from shape. The integer adds
one clause to an error message — *"and this rule changed at revision 2"* — and **no one
has measured that the clause helps an agent.** Meanwhile it is a hand-maintained field
in a format whose own consumer exercise measured **8 of 8** agents copying a stale field
out of the fixture, several before reading the spec. The honest null hypothesis is *no
version field*, and I would be proposing machinery on an unmeasured ergonomic claim.

A second attack, sharper for being internal: my own Q4 line means roughly 14 of the 15
accepted ADRs would bump. A number that increments on nearly every change is a
**changelog index, not a compatibility signal**. It tells you nothing about whether
*your* file is affected.

### Why it survives — but not for the reason I first reached for

I concede the ergonomic claim is unmeasured and I am not resting on it. The answer
survives on a different, nameable failure that **does not require versioning at all**:

> **File identification.** Named failure: an agent or a person points `validate` at the
> wrong JSON and gets a cascade of schema errors instead of one sentence. This is not
> hypothetical. The fixture's own directory ships `reference/beats.json` and
> `transcript.json` — real JSON files sitting beside the real project file — and the
> tool surface (ADR-0011) is a CLI plus an MCP server taking file paths. A
> `.montagent.json` file with no marker is indistinguishable from any other JSON object
> with a `tracks` key.

So a magic key is warranted on its own. **Given that the key is there, making its value
an integer rather than `true` costs zero bytes and one digit**, and buys the one thing
shape provably cannot supply: coverage of the ADR-0013 class, where the bytes are legal
and identical on both sides and only the *reading* changed. The version is a free rider
on a marker that is independently justified. That disposes of the brief's "name the
failure" test honestly rather than by inflating the versioning case.

On the brief's direct question — **is a confidently wrong version number worse than no
version number?** Under qualification (2) it is **not**, and that is the design's main
load. Both error directions are safe:

- **Wrong-low** (an agent copies `"montagent": 1` into a file it wrote at revision 4):
  a false but *conservative* claim. Validate finds no revision-1-to-4 discrepancies in
  the shape and can say so: *"declares 1; no discrepancy found; you may raise this."*
- **Wrong-high** (claiming 4 while writing revision-1 shapes): **detected**, because
  the claim contradicts the shape validate is reading anyway.

The direction that is catastrophic — a wrong number that silently selects a different
reading — is unreachable, because nothing reads it. Under (b) it is the normal case.

### What would falsify it

- Construct a file that shape places uniquely across a real accepted revision boundary,
  where the placement is not a negative inference from a removed key. That would revive
  option (c). (I failed; the fixture itself is the counterexample.)
- Measure the error-message claim: run #48's consumer-exercise design on a stale file
  with dated versus undated `validate` output. If dated messages produce no better
  repair rate, the versioning half of this answer collapses to a bare magic marker —
  and I would accept `"montagent": 1` frozen at 1 forever, which is a real and
  defensible outcome.
- If, once the schema resource exists, a released revision can be found that changes
  the rendered output of a byte-identical legal file **and** whose effect is detectable
  from shape, the ADR-0013 class is not unique and the integer loses its one exclusive
  job.

---

## Q2 — Does the policy fire now, or after a stated 1.0?

### Preferred answer

**A pre-1.0 clause, stated explicitly in the ADR — and the trip event is not a version
number, it is the publication of the JSON Schema resource.**

The reason is fact 2, and it is harder than "we haven't shipped". **A version number
names a legality predicate, and Montagent has none.** The format is fifteen ADRs of
prose. ADR-0013 explicitly left *"whether `fit` may be omitted, and what omission
means"* undefined — so at revision 0013 there was **no fact of the matter** about
whether my `amb-B` file was legal. You cannot bump a number that indexes a predicate
that does not exist. The first revision number can only be minted at the moment the
machine-readable schema ships, which ADR-0011 already schedules as an MCP resource.

So: the trip is **the first release that ships the schema resource and a renderer a
stranger can install and run.** Objective, not discretionary; whoever tags decides, but
the criterion is written down so they are not exercising judgement. All fifteen ADRs
collapse into **revision 1**; pre-1.0 churn leaves no numbers behind; **ADR-0016 does
not carry a bump.**

Until then the obligation is the one `docs/agents/domain.md` already imposes and that
has already been discharged twice: **the fixture is migrated in the same change as the
decision, by a committed re-runnable script**, and — the part worth promoting from
practice to rule — **`verify.py` regenerates the fixture and byte-diffs it**, which is
what turned ADR-0013's zero-byte claim from an assertion into a check.

### The strongest attack

**A pre-1.0 clause with no deadline is a blank cheque, and "no users" is false today.**
Fact 6 says 8 of 8 agents copied `gravity` out of the fixture, several *before reading
the spec*. An agent working from a cached or stale spec is a stranger holding a stale
file right now. "There are no users" is a claim about humans; the actual consumer
population is agents, and it is already producing the exact failure versioning is meant
to prevent. Second edge: a project can sit pre-1.0 indefinitely, and a rule whose
obligations begin at an event the author controls is an obligation the author can
decline forever.

### Why it survives

On the first: **#48 is a natural experiment in precisely this failure, and the remedy
that was measured to work was fixture hygiene, not versioning.** ADR-0015 says it
plainly — *"a retirement that leaves the field on 8 shipped elements is a retirement in
name only"* — and deleted the field in the same change. A version integer would not
have helped those eight agents at all: they copied from the fixture *before reading the
spec*, so they would have copied the integer too, and a stale integer in a world with no
validator is inert. The measured fix is already the pre-1.0 rule I am endorsing.

On the second: the trip criterion is an artefact (the schema resource), not a mood, and
it is a thing the project is committed to building anyway (ADR-0011). And pre-1.0 here
is not "no obligation" — it is an obligation discharged twice already, enforced by a
committed byte-diff, which is stricter than most post-1.0 projects manage.

### What would falsify it

- If the schema resource ships *before* anything a stranger can install — schema without
  renderer — my two conditions come apart and I would take the schema as the trip, since
  it is the one that makes legality decidable.
- If a measured exercise shows agents reading the shipped fixture are meaningfully
  corrected by a declared revision integer even with no validator present, the "wait"
  half is wrong and the field should ship now.
- If ADRs continue accumulating past, say, thirty with no schema in sight, the clause
  has become the blank cheque the attack describes, and the right response is to mint
  revision 1 on the schema alone.

---

## Q3 — Whose files must survive N+1?

### Preferred answer

**(b) now, and after the trip it becomes (a) *as a design target* without becoming a
general migration tool.** Concretely: **`montagent migrate` should not exist.**

The line I propose is one the project already owns. `shift` is in the tool surface
because it is *"the one edit that is arithmetic rather than authorship, and therefore
the one Montagent performs instead of the agent."* Apply the same test to migrations:

- **Arithmetic migrations** — a deletion (`gravity`), a rename, a mechanical
  restructure — are total functions of the document. They may ship as a committed,
  idempotent, dated script beside the ADR that caused them, exactly as #42 and #48
  already did. Idempotence is free for this class: deleting a key twice is deleting it
  once.
- **Authorship migrations** — anything requiring a decision about what the author meant
  — may **not** ship as a tool. `fit` is the worked example from A3: a stale rect can
  match neither `cover` nor `contain` under strict equality, and the only legal value
  left is `literal`, which asserts deliberate intent. A tool that writes it is
  fabricating intent to make a file pass, which is the quiet-failure class ADR-0012 and
  ADR-0006 exist to prevent.

For the authorship class the shipped capability is the one the format already uses on
every decision: **`validate` reports the error and names the replacement.** ADR-0015
does not say "migrate `gravity` away"; it says *"a schema error naming `x`/`y`/`origin`
and `clip`."* Every ADR in the chain is written in that voice. **The format already
migrates by error message**, and the entity that performs the repair is the agent —
which is the product's one reliable assumption.

**What a general migration tool buys over per-change scripts**, stated honestly rather
than dismissed: exactly one thing — **chaining.** A stranger's file several revisions
stale, whose holder does not know which revision it is, needs N scripts run in the right
order. That is a real failure and it is the failure OTIO's machinery is built for. My
claim is that the price is wrong for this project: it requires every migration script to
stay runnable, dependency-frozen and composable forever, and it requires the thing A2
and A3 showed cannot be had — knowing which revision the file is. Under my Q1 answer the
declared integer supplies the starting point, so a chaining migrator becomes *possible*
at 1.0; I am saying it should still not be built until a real stale file exists to
justify it, and that its first version should be the arithmetic subset only.

### The strongest attack

**This is the standard open-source excuse, and it offloads the work onto the person
least equipped to do it.** Someone with a 200-element project and six revisions of drift
will not hand-repair it from error messages. Two supporting jabs: (i) a shipped migrator
*could* get out-of-document input — #42's script needed source dimensions, and a
released Montagent has `probe` and `measure`, so "it needs to look at the disk" is not
the barrier I implied; (ii) my `update_element` analogy is weaker than it sounds. The
banned family takes a *field name from the caller*; a migrator takes none — the field
names are baked in. The letter of the write-tool invariant does not forbid `migrate`,
and I should not pretend otherwise.

### Why it survives, with one concession

The concession first: **the invariant argument is an analogy, not a derivation**, and
the "needs the disk" argument is wrong. The load-bearing argument is arithmetic versus
authorship, and it stands on its own.

On the main attack: the person is not the one doing it. **Montagent is an agent-first
editor with no GUI** — the file is always in the hands of an LLM that authors and edits
by exact-string replace. A migration performed by an agent against precise,
replacement-naming error messages is the *same act* as the authoring the format already
assumes, on a file laid out (one element per line, stable key order, sorted by `start`)
precisely to make that act reliable. A general migrator would be Montagent doing, worse
and speculatively, the one thing it can count on the agent doing well.

And under file-as-truth the sweep a migrator wants to perform does not exist: Montagent
never holds a database, and the old revision of any file is already recoverable — A1's
`gravity:"center"` survived only in git, which is where file-as-truth says it should be.

### What would falsify it

- Exhibit an accepted change that is (i) arithmetic, (ii) not expressible as a
  replacement-naming error, and (iii) not performable by an agent from that error. One
  such change and the per-change-script position is insufficient.
- **Removing an element type** is my own best candidate, and I flag it below: there is
  no replacement to name, so the error message pattern has nothing to say.
- Measure it: give agents a genuinely stale multi-revision file and the current-revision
  spec, and count repairs. If agents cannot cross two revisions from error messages,
  ship the migrator.

---

## Q4 — What counts as a breaking change?

### Preferred answer

One predicate, applied to the format rather than to the corpus:

> **Bump iff there exists a file whose *legality* or whose *rendered output* differs
> between N and N+1.**

| change | legality differs? | output differs? | bump |
| --- | --- | --- | --- |
| add an **optional** field | no | no | **no** |
| add a **required** field (0014 box, 0015 `fit`) | yes | — | **yes** |
| **rename** a field | yes (both directions) | — | **yes** |
| **remove** a field (0015 `gravity`) | yes | — | **yes** |
| **change a default** (ADR-0012's six; #61 open) | no | **yes, silently** | **yes** |
| **change meaning, same spelling** (ADR-0013) | yes, at the edges | **yes** | **yes** |
| remove an **element type** | yes | yes | **yes** |

**Why that line and not one field over.** It is derived, not chosen: *add an optional
field* is the **unique** category in which a file legal at N is legal at N+1 **and**
renders identically at N+1. Every other category fails one limb or the other. There is
no judgement call in the table — each row is a consequence of the predicate.

**Shape or reading?** **The renderer's reading**, and that settles that it is **one
field, the same one as Q1's.** The argument is elimination: a version that described
only the file's *shape* would be redundant with the schema, because checking shape
against a published schema is what the schema is for — that is what *"schema catches
malformed"* means. The one thing a schema provably cannot tell you is that **identical,
legal bytes now mean a different picture**, which is ADR-0013 and the changed-default
case. That is the version's exclusive job, so a shape-version has no job left and a
second field would be a field with nothing to do.

Note what this implies for ADR-0013 specifically, and it is a feature: **the revision
bumps while the committed fixture changes zero bytes** (verified by blob hash across
three ADRs). The bump is a statement about the format, not about any file that exists.
The stranger holding `height: 1913` is the file it is for.

### The strongest attack

**Under this predicate, roughly 14 of 15 accepted ADRs bump, and a number that
increments on nearly every change carries no compatibility information.** It cannot
tell a file's holder whether *their* file is affected — ADR-0013 affected zero of the
committed elements. So the number degenerates to a changelog index, and the honest
description of my design is "an incrementing counter that occasionally decorates an
error message."

A second attack on the exempt row: *add an optional field* is not as safe as the table
says. If `fmt` normalises key order (#73, open) and the new field takes a position, a
correct file's next exact-string replace may miss — the write-read round-trip failure
that CONTEXT.md invokes for `center-center` and `#RRGGBBFF`. And if #61 decides `fmt`
materialises defaulted fields, then "optional with a default" and "required" stop being
distinguishable in a formatted file.

### Why it survives

The first attack is **correct, and it is the design.** I accept the description and
reject that it is a defect, because the two questions were never the same question:

> **Shape answers "is this file broken." The integer answers "since when."**

`validate` runs on every invocation with the actual file in hand and is the only thing
that can answer the first. The integer cannot and must not try — that is exactly the
error OTIO's model makes by letting the number select a reader. A changelog index that
dates a file is a modest thing, and modest is the correct size for a field an agent
hand-maintains (fact 6). It is also why the number must be a lower bound (A4): a
changelog index you are obliged to keep current would be churn on every correct file in
the world.

The second attack lands on #61 and #73, not on the table. I will say what I think the
table implies for them, flagged as a consequence rather than a decision I am entitled
to make: **if #61 decides `fmt` materialises defaults, then changing a default becomes a
change that `fmt` writes into files, which strengthens rather than weakens its place in
the bump column.** And if #73's canonical key order ever changes, that is itself a
format change under the predicate — it alters legality under a `fmt`-normalised reading
— which suggests #73's answer should be published *as part of* revision 1 rather than
left to convention.

### What would falsify it

- A change that is legality-preserving and output-preserving yet still breaks real
  files. That would show the predicate has a hole and the line needs to move outward.
- A measured case where a holder of a stale file benefits from the number *selecting a
  reading* — a compatibility mode that produces the right video where current-revision
  semantics produce the wrong one. That would push toward (b) and toward the version
  driving behaviour, and it is the observation that would most change my mind.
- #61 or #73 resolving in a way that makes "optional field" unobservable in a formatted
  file would collapse the exempt row and mean **every** change bumps — at which point
  the number's information content is zero and Q1's fallback (a frozen magic marker,
  `"montagent": 1` forever) becomes the right answer.

---

## What I could not settle

- **Removing an element type has no replacement to name**, so the "migrate by error
  message" pattern that carries my Q3 answer has nothing to say about it. It is the one
  category where I think an ADR should not be accepted without an explicit repair path,
  and possibly the one category that justifies keeping an old-revision *reader*. The
  brief asked me to classify it; it did not ask how it is repaired, and I do not have an
  answer I believe.
- **Whether the declared integer should also appear in `create_project`'s scaffold.**
  ADR-0011 gives Montagent a tool that writes a legal file from nothing. If the scaffold
  emits the current revision, then most files carry a *correct* integer with no agent
  effort and fact 6's hazard mostly evaporates — the stale-copy path only bites files
  copied from older files. I think this is the strongest practical argument for the
  field and I noticed it too late to weigh properly.
- **Whether `probe`, `measure` and `compare` outputs should carry the revision.** If
  `validate`'s message is the migration mechanism, the revision needs to be in the
  output surface and not only in the file, and nothing in ADR-0011 anticipates that.
- **Interaction with #73 (key order).** A top-level marker key must have a published
  position, and I have assumed "first" without authority.

### Needs measuring rather than deciding

1. **Dated versus undated `validate` messages.** Run #48's consumer-exercise design on a
   deliberately stale file — half the agents get *"declared 1913; rule value 1912"*, half
   get that plus *"your file declares revision 1; this rule changed at revision 2."*
   Count correct repairs and wrong repairs. This is the single measurement my Q1 answer
   rests on and it has not been made.
2. **Multi-revision repair by agent.** Give agents a file two or three revisions stale
   and the current spec; count how far they get from error messages alone. This is the
   measurement that decides Q3.
3. **Stale-integer copy rate.** Ship a fixture carrying `"montagent": 1`, bump the
   revision, and measure whether agents authoring new files copy the stale integer. Fact
   6 predicts they will; my answer predicts it is harmless because wrong-low is
   conservative and wrong-high is detected. Both halves are testable.

---

## Confidence

| | confidence | where it is low |
| --- | --- | --- |
| **Q1** — one integer, lower-bound, validate-only | **moderate** | The *marker* is well justified by a named failure; the *versioning* half rests on an unmeasured ergonomic claim I have labelled as such. If measurement 1 comes back null, I would accept a frozen `"montagent": 1` and call the versioning question closed. The refutation of (b) and (c) I hold at **high** confidence — (c) is measured. |
| **Q2** — pre-1.0 clause; trip is the schema resource | **high** | The fact-2 argument (you cannot index a legality predicate that does not exist) is the one I would defend hardest, and ADR-0013's own undefined `fit` omission is direct evidence for it. Low confidence only on whether schema and renderer ship together. |
| **Q3** — no `montagent migrate`; scripts beside ADRs | **moderate-high pre-1.0, moderate after** | The pre-1.0 half is nearly free: it is current practice, discharged twice. The post-1.0 half is a prediction about an agent population I have not measured, and I conceded two of my own supporting arguments (the `update_element` analogy, the "needs the disk" claim) are weak or wrong. |
| **Q4** — the predicate and its table | **high on the table, moderate on "reading not shape"** | The table follows mechanically from the predicate. The reading-not-shape conclusion is an elimination argument that assumes the schema will be published and complete; if the schema is partial, a shape-version briefly has a job again. It is also exposed to #61 and #73, which are open and which I was told not to assume. |
