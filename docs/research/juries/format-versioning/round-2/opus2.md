# VERDICT-R2 — jury-14 round 2, opus2

## Headline

I ran A2 and it came out **byte-identical**: acting as `validate` with nothing in my messages
that the accepted ADRs do not license, then acting as the agent with no git and no sight of
the post-0015 file, the repair reproduced `a2e64f7c…` exactly — 8 deletions, no other byte.
But the clean diff is the least interesting part of the result. **The error message alone was
under-determined**, and it was under-determined in the silent direction: the ADRs' own
named-replacement idiom reads *"names `x`/`y`/`origin` and `clip`"* as an instruction to
**substitute**, and the correct repair was to **delete and touch nothing**. I resolved that
from one sentence in `CONTEXT.md`'s Gravity entry, not from the message and not from
ADR-0015. And **M2 is not what it says it is**: forward migration of a removal is value-blind,
so `handle-logo`'s unrecoverable `"center"` never entered the repair. M2 names the cost of
*rollback*, which the project already pays with git. A3 then broke the same way: the
element-type-removal residue is **not** a case with no replacement to name — `CONTEXT.md`
already names one for the exactly-analogous `path`/`line`/`polygon` closure ("commit an SVG
or a PNG instead"). What is true of it is worse for `montaget migrate`, not better: it is the
one migration in the chain that a **program cannot perform and an agent can**. The hard case
is the case the tool cannot do. So: **no version number; a retired-spelling error table that
is permanent and never pruned, living in `validate`; `montaget migrate` never ships.**

---

## Part A findings

### A2 — the diff, and the guesses

`diff repaired.json post-0015.json` → **BYTE IDENTICAL** (md5 `18620b3f…` both sides;
`pre-0015.json` is `93bff91f…`). Eight `"gravity":"…",` deletions, nothing else.

Before the repair, as `validate`, I ran every check the accepted ADRs license against the
real media on disk — strict fitted-extent equality (`(2720*1080)//1536 = 1912` against a
declared 1912, 7/7; 800x800 into clip 68x68 → 68x68), `fit` presence and membership, `clip`
required by `cover`, the text box, aperture coverage, UNCHECKED count 0. All pass. The only
errors are the eight `gravity` keys, and the message is the one sentence ADR-0015 supplies:

```
error  handle-logo: `gravity` is not a field of this format.
       Which part of the source survives is determined by `x`, `y`, `origin` and `clip`.
```

I refused to write "delete `gravity`" because no accepted ADR says it.

**What I had to guess:**

- **G1 — delete or compensate.** Decisive finding. Every other named-replacement error in
  this project (`center-center`→`center`, `none`→`literal`, `anchor`→`origin`,
  `gravity`-on-text→`origin`) means *write the named thing instead*. Here the four named
  fields must be left **untouched**. The message cannot tell you which. I got it from
  `CONTEXT.md`: *"a quantity that does not exist… with no freedom left to spend."* The wrong
  branch fails **silently** — `x`, `y`, `clip` are all author-written and nothing checks them
  against `gravity`'s old value, so a compensating agent gets `0 errors` on a file whose
  picture changed. That is ADR-0006's false-confidence failure reached through a repair.
  It did not bite here only because `gravity` was inert on all 8 elements (photos: rect
  1080x1912 at y=0 through clip [0,0,1080,1300], so `"top"` is already what the geometry
  says; `handle-logo`: rect *equals* clip, nothing cropped). **M1's fixed-point property is
  doing the work again.** Per the anti-drift rule, my clean diff is evidence the repair *can*
  be done — never that it is safe.
- **G2 — the trailing comma.** Byte-exactness required deleting `"gravity":"x",` *including*
  its following comma. Nothing publishes that. Had `gravity` been the object's last key I
  would have had to delete the **preceding** comma, with no published rule and no published
  key order (#73 open) to tell me which. My byte-perfect score rests on an unpublished
  formatting convention I inferred from the line around it.
- **G3 — `handle-logo`'s `fit`.** Exact-aspect, and ADR-0015 itself says `cover` and
  `contain` both describe it with "no canonical preference stated". I left `cover`: right by
  inertia, not by rule. A tidier agent writes a legal file that is not the committed one.
- **G4 — nothing about the rect**, because this file sits at the fixed point and C1's class
  never fired. On a float-authored sibling it fires, and *that* repair is a real guess the
  ADR deliberately refuses to make: *"Write 1546, or `fit:"literal"` if deliberate."* Two
  legal repairs, different pictures, chosen by intent.

**On M2.** Confirmed true and confirmed irrelevant to migration. No accepted ADR records
`handle-logo`'s `"center"` (ADR-0012 and ADR-0013 both publish the `photo-06` element with a
literal `"gravity":"top"`, preserving 7 of 8 values by accident of a worked example; the
eighth is simply gone). **My repair recovered nothing and needed nothing** — deleting a key is
value-blind. M2 is the cost of *un*-migrating, and git is already this project's stated answer
for undo. A migration tool would not have recovered it either.

### A3 — the residue, opened

`ellipse` is a real accepted element type (`CONTEXT.md`: *"`rect` or `ellipse`, each its own
element type… An ellipse inscribes its declared rect"*), used zero times in the fixture. I
checked for an in-format successor: there is none — `radius` is a scalar, so `rect`+`radius`
gives a stadium, never an inscribed ellipse, and `path` is closed for reasons this ticket
cannot reopen.

**The premise "there is no replacement to name" is false.** `CONTEXT.md`'s `Path, line,
polygon` entry already names the replacement for exactly this situation: *"Commit an SVG or a
PNG instead."* The replacement is not a field and not a type — it is an **out-of-format asset
plus an `image` element** — and every clause of the message below is licensed by an accepted
document:

```
error  halo-ring: `ellipse` is not an element type of this format.
       A drawn ellipse is an `image` element whose `source` is a committed SVG or PNG.
       `x`, `y`, `origin`, `width`, `height` carry over unchanged. `fill`, `stroke` and
       `stroke_width` have no spelling on an `image` and must be baked into the source file.
       The new element needs `fit`; use "literal" unless you also declare a `clip`.
```

An agent **can** repair from that: emit an SVG, write it beside the project, rewrite one
line. A program **cannot** — Montaget has no authoring surface, *"contains no model and is
never an agent"*, and nothing publishes the raster's dimensions, DPI or antialiasing.

So the case is the same in kind and different in **executor**. Every migration so far is
doable by a script or an agent; this one is agent-only. **The difficulty gradient runs the
wrong way for `montaget migrate`**: the case invoked to justify the tool is the case the tool
cannot do.

Four costs remain, none of which a tool fixes: byte-determinism dies (two agents, two PNGs,
no regenerate-and-byte-diff); the repair writes outside the project file; it is genuinely
irreversible (baking loses `fill`, `stroke`, and the fact that it was an ellipse — **this**,
not M2, is the unrecoverable case); and a loud refusal is the right alternative outcome,
which `validate` already is the place for.

**The mechanism that supplies what's missing already exists.** `CONTEXT.md` keeps a
graveyard — `Box`, `Align for images`, `Path/line/polygon`, `Gravity` — each with why it went
and what to write instead, and ADR-0011 already ships the format docs as an MCP resource.
That graveyard *is* the migration document; my A2 repair used it and it was precisely the
piece the error message lacked. What it lacks is a guarantee of **permanence**.

**"An ADR may not be accepted without a repair path" is a slogan.** `docs/agents/domain.md`
states exactly two obligations — *ADRs are amended, never rewritten* and *commit the evidence
an ADR rests on*. No migration obligation exists in it, in any ADR, or in `CONTEXT.md`. The
strong form is also unachievable: A3 is a counterexample by construction. The weak, checkable
form that A2 and A3 actually support is the deliverable of this ticket.

### A4 and A1, briefly

**A1.** M1 re-verified, blob-for-blob, exactly as stated. C1's arithmetic confirmed
(`103*(1920/103)` = `1919.9999999999998` → 1919); **C1's 5.8% I could not reproduce** — I get
5.19% for box dims (1080, 1920) and 1.12% for the fixture's real (1080, 1300) box over source
widths 1..10000. Right in kind, unreproducible in magnitude as described.

**C2 is half false.** Its core claim is confirmed by `migrate.py`'s own docstring (cumulative
regenerator from a fixed origin, never N→N+1). But *"a stranger has no `old.json`"* is wrong:
`docs/research/sample-project/pre-migration.montaget.json` is committed on `main` and
`verify.py`'s docstring gives the re-run incantation. `verify.py` is a **differential** checker
over two named files, not a legality predicate over arbitrary ones — which matters for Q7.

**A4.** `reference/beats.json` shares `output` and `duration` with the project file; the
`.montaget.json` suffix is used by both project files on `main` and is **published nowhere** —
zero hits across fifteen ADRs and `CONTEXT.md`. Identification today is an unpublished
filename convention plus shape-sniffing. The failure it causes is real (a wall of schema
errors instead of "this is not a Montaget project file") but is an error-message-quality
failure, not a correctness one, because under file-as-truth Montaget sweeps no directory.

---

## Q5 — Does the file carry a marker at all, and is it a version?

**Preferred answer.** **No version number — that half of P3, on P2's separability argument.**
Identification and revision are two decisions with different referents and opposite staleness
behaviour, and A4 confirms they separate cleanly. The version dies. The marker survives
weakly, and if taken it must **not** be an integer, because any integer in this file will be
read as a revision by every agent that sees it. The cheapest sufficient fix to A4's failure is
not a field at all: **publish the `*.montaget.json` convention in an ADR** — it already
exists, is already used twice, and is currently held up by nothing but habit. If a field is
wanted beyond that, `$schema` pointing at ADR-0011's already-scheduled schema resource is the
honest shape: self-describing, not mistakable for a revision integer, and a stale value names
a real older schema, which is *true* information rather than a false claim.

**Is a confidently wrong version number worse than no version number? Yes** — and A2 shows
why in a way argument cannot. The repair succeeded because the validator reasoned from what
the file *contains*. A number is a second, independent, hand-maintained claim about the same
file, and fact 6 measures agents copying stale fields from examples at 8/8. A file saying
`"montaget": 1` while containing no `gravity` and a required `fit` has a wrong number and
right content; any tool dispatching on the number migrates a correct file. **New failure mode
created, none removed.**

**Strongest attack on my own answer.** My marker case is weaker than I first wrote it. I named
a failure — `render reference/beats.json` emitting *"missing `frame`, missing `fps`"* instead
of *"not a Montaget project file"* — and then had to concede it is cosmetic, because Montaget
holds no database and sweeps no directory, so every path it sees was handed to it by someone
who already believed it was a project file. Under the brief's own rule (*name the failure it
prevents*) a cosmetic failure does not buy a field, and adding `$schema` would be the
second-ever byte change to the committed fixture, bought for error-message polish. Second
attack: `$schema` conventionally means a **fetchable URL**, and ADR-0011 ships the schema as
an MCP resource — a stranger's editor cannot dereference `montaget://…`, so the field promises
a capability it does not have.

**Survives?** The *no-number* half survives intact and is my highest-confidence answer in this
verdict. The *marker* half survives only as: publish the extension in an ADR (free, and a real
documented gap); add `$schema` **only if** #11's schema resource gets a stable dereferenceable
URI, and never an integer. I am deliberately not recommending a marker field today.

**P4 (per-object versions).** I agree with round 1's rejection and add a reason: a per-element
version multiplies the hand-maintained-stale-field problem by 60, on a format whose edit
mechanism is exact-string replace and whose `fmt` normalises on write — the precise pair of
constraints ADR-0013 used to kill `center-center` as an alias.

**Falsifier.** Produce a repair task where the *correct* edit differs depending on which
revision a file was authored against, with the file's content identical in both cases. I
tried to construct one across all fifteen ADRs and could not — see Q8 for why, and for the
one condition that would create one overnight.

---

## Q6 — Lower bound, or equality?

**Preferred answer. P5 (lower bound) — conditionally, since Q5 says the number should not
exist.** M1, which I re-verified myself, forces it: under P6 three consecutive accepted
revisions would each have obliged a rewrite of every correct file in the world as pure
ceremony, and fact 6 measures what agents do with ceremony fields.

**Strongest attack — on P5, which is my own answer.** M1 does not merely settle Q6, it
**dissolves** it. If the declared number would have been stale-but-harmless three revisions
running, it carried zero information three revisions running. A lower bound tells a tool "at
least 1", which it already knew. The only remaining use is dispatch — *run 1→2, 2→3, 3→4* —
and A2 showed dispatch is done from **content**, which is strictly better because it also
handles the half-migrated stranger file that a numbered chain handles **wrongly** (a file
declaring `1` that has already had its `gravity` removed by hand gets the deletion applied
twice, or the chain trusts the number and skips a check the content needed). So P5 is the
right answer to a question that should not be asked.

**Does M1 change my answer?** It confirms P5 over P6 and simultaneously strengthens Q5's "no
number". Both.

**Falsifier.** A revision whose repair is not derivable from content — i.e. two files with
identical bytes needing different repairs. Reissuing a retired spelling with a new meaning
would create exactly that; the project's own discipline currently forbids it.

---

## Q7 — What trips the policy?

**Preferred answer. P8, and A2 corroborates it experimentally.** To run A2 I had to *invent*
the validator: there is no `validate`, no schema artifact, and ADR-0013 explicitly leaves
*"whether `fit` may be omitted"* undefined. A version number indexes a legality predicate, and
Montaget does not yet have one over arbitrary files. All fifteen ADRs collapse into revision 1
and ADR-0016 carries no bump.

**Strongest attack.** P8 conflates *unpublished* with *nonexistent*. `migrate.py` and
`verify.py` are committed, executable, and encode the rules — a predicate in Python rather
than JSON Schema. If that counts, the format has had a machine-checkable predicate since #42
and P8's "no fact of the matter" collapses.

**Survives — I checked the artifact rather than arguing.** `verify.py`'s signature is
`verify.py <old.json> <new.json>`: it is a **differential** checker asserting the mapping
between two specific named files, with the origin hard-committed at
`docs/research/sample-project/pre-migration.montaget.json`. It cannot be handed a stranger's
file and answer *legal / illegal*. `migrate.py` likewise is a regenerator, not a checker. So
no legality predicate over arbitrary files has ever existed, and P8 stands as stated.

I also prefer P8 over P7 and P9 because it names a **checkable event** — the schema resource
lands, or it does not — where "first external user" is unobservable to the project and P9's
split requires knowing which half of a change is which.

**Falsifier.** Publish the schema and then find a legality question it does not answer that a
version number would. If the schema ships with "whether `fit` may be omitted" still undefined,
P8's event has not actually occurred and I would need to move the trigger.

---

## Q8 — Does `montaget migrate` ever exist?

**Preferred answer. P10 — never — but stated more usefully than "never".** The migration
*capability* must exist and it already has a home: **`validate`, carrying a permanently
cumulative retired-spelling error table that names the replacement and is never pruned.**
Two sentences in #14's ADR buy the whole thing:

1. *An ADR that makes a previously legal spelling illegal must add a permanent entry to the
   retired-spelling table, naming what to write instead — or stating explicitly that the
   replacement is out-of-format and what it is.* Entries are never pruned.
2. *A retired spelling is never reissued with a new meaning.*

Sentence 2 is what makes content-dispatch sound, and it is a rule the project already follows
without having written it down — ADR-0015 rejects `fill` as a `fit` value precisely because
*"`fill` is spent"* by ADR-0014.

**Strongest attack — P11's best argument, which is dispatch and ordering.** A stranger four
revisions stale cannot find and sequence four scripts. I tried to break content-dispatch and
**I found the one case where it genuinely fails**: suppose R2 renames `gravity`→`crop_anchor`
and R3 retires `crop_anchor`. An R1 file run against R3's validator meets a validator with no
clause for `gravity` at all, and — per C4, no unknown-key policy — may get **no error at
all**. That is a real failure and a numbered chain does prevent it.

**Survives, because the fix is the table, not a command.** The failure above is exactly
"someone pruned the graveyard". A cumulative table keeps `gravity`'s entry forever, pointing
at the *current* replacement, and repair becomes a fixed-point iteration — run `validate`, fix
what it names, run again — which converges without knowing N and, unlike a numbered chain,
handles the half-migrated file correctly. That is a static table, not a program, and it costs
a sentence.

A2 and A3 decide the rest against P11:

- **A2:** the hardest published migration was a value-blind deletion, byte-perfect from a
  message. A tool would have added nothing, and M2's unrecoverable value was not needed.
- **A3:** the hardest *conceivable* migration requires authoring an asset, which an agent can
  do and a program categorically cannot. The gradient runs the wrong way.
- **C2** (as corrected): no ADR has ever shipped an N→N+1 migration for anyone else's file.
  The "two successful migrations" precedent is a cumulative regenerator; there is no track
  record for P11 to extrapolate.
- **`CONTEXT.md`'s own line** — *"the one edit that is arithmetic rather than authorship"* —
  is the project's stated boundary, and A2's G4 shows ADR-0015's own error message sits
  deliberately on the authorship side: *"Write 1546, or `fit:"literal"` if deliberate."*

P11's second argument — a place to refuse loudly and name affected element ids — is real and
is already `validate`'s job (ADR-0006). It reduces to P10.

**Honest cost of my answer.** The table grows without bound and nothing today commits anyone
to maintaining it; `CONTEXT.md`'s graveyard is a *vocabulary* document, not a normative error
table, and no rule says a retired term's error must survive N revisions. That fragility is
real, it is the whole load, and sentence 1 above is what removes it. If the project will not
write that sentence, my Q8 answer weakens substantially and P11 gets stronger.

**Falsifier.** A concrete migration that (a) a script performs correctly, (b) an agent
performs wrongly from a replacement-naming message plus the graveyard entry, and (c) is not
fixable by improving the message. G1 is the closest thing I found to one — and it *is*
fixable by improving the message, which is why it falsifies "error messages are sufficient as
currently written" without falsifying P10.

---

## Q9 — One field or two mechanisms?

**Preferred answer. Neither: zero fields in the document.** The shape version is the schema's
job; the semantics version is a fact about **which binary you run**, so it belongs on the
binary — `montaget --version`, stamped into the render output or a sidecar so a video can be
traced to the reading that produced it (the stamping is a cheap suggestion, flagged as
speculation). The ADR trail is the human-readable record, as P13 correctly says.

**C1 changes the answer, and it changes it against P12 rather than for it.** P12's whole case
is elimination: *a shape-version is redundant with the schema; the one thing a schema cannot
tell you is that identical legal bytes now mean a different picture; so that is the version's
exclusive job.* C1 destroys the middle premise. A legal pre-0015 file declaring `1919` becomes
an `error` under ADR-0015's strict equality — the schema catches it, loudly, with a message
naming both repairs. So the semantics-only class is **mostly caught by the schema too**, and
P12's elimination argument, run honestly, eliminates the version entirely rather than
eliminating the shape-version. **C1 was offered as a sharpening of P12 and is fatal to it.**

**Strongest attack on my own answer.** A residue survives C1: files sitting at the fixed point
(M1's three revisions, byte-identical and legal on both sides of three accepted changes) and
default changes if #61 resolves to "`fmt` does not materialise". For that residue there is
genuinely nothing in bytes and nothing in a schema — and a renderer-side version is no help
to a *reader* holding only the file. So my answer leaves a class with no machine-readable
record anywhere in the document.

**Survives**, for P3's structural reason, which I think is the single most correct sentence in
the brief's position list: **no field placement lets a document self-describe an
interpretation rule its author never knew existed.** A file that crossed ADR-0013 unchanged
cannot say which rounding rule it was authored under, because when it was written the rule had
not been decided. A field there would be a claim about a fact that did not exist, and a wrong
claim is worse than an absent one for exactly the reason Q5 gives.

**Falsifier.** A revision that changes what legal bytes mean, that no schema check catches,
*and* where knowing the authoring revision would change the correct repair. ADR-0013 is the
best real candidate and it fails the third clause — the correct repair under ADR-0013 is the
same (none) whichever revision the file was authored against.

---

## Q10a — Should #61 block #14?

**No — but #14 must be written #61-agnostic and say so in "Not settled here".**

Four jurors converging on #61 is evidence of something real, and it is this: #61 sets the
*frequency* of the byte-invisible semantics class. If `fmt` materialises the six ADR-0012
defaults, a formatted file pins its own defaults in bytes and a later default change is
spec-text only for it. If not, every default change is C1's class.

But **frequency is not the variable #14's answer depends on**. The class already exists
independently — ADR-0013 produced one before defaults entered the discussion — so #14 must
handle it either way, and my Q9 answer (the class has no document-side home, at any frequency)
is frequency-invariant. Blocking also runs the dependency backwards: whether `fmt`
materialises defaults should be decided on `fmt` grounds — exact-string-replace stability,
ADR-0013's *"`fmt` must never rewrite a declared extent… the integer is content"*, ADR-0012's
rejection of aliases because `fmt` normalises on write — not as a side effect of a versioning
ticket. Deciding it inside #14 would be #14 annexing a question it is badly placed to answer.

**Attack on my own answer.** If #14 ships a sentence that presumes an answer — anything of the
form *"a default change is spec-text only"* — then #61 was load-bearing and I was wrong to
unblock. That is a genuine risk and the mitigation is a drafting constraint, which I am
imposing: **#14 may not contain a sentence whose truth depends on #61's answer.** Under my
Q5 answer this is easy, because with no version field there is no classification table for a
default change to be a row of.

(Speculation, flagged: ADR-0013's *"under declared-authoritative the integer is content"*
reasoning generalises awkwardly to defaults, so I lean toward #61 resolving to "no". I am not
relying on that lean anywhere above.)

---

## Q10b — The missing unknown-key policy (C4)

**A new ticket, opened now, referenced by #14, blocking nothing — and it should be opened with
the finding that the decision has already been taken de facto.**

Not a clause in #14, for the same annexation reason as Q10a: schema strictness has
consequences far outside versioning, and the biggest is **typos**. Under a permissive schema,
`"gravtiy":"top"` is silently ignored; under a strict one it is caught. On a format whose
measured failure mode is *8 of 8 agents reached for a retired field, several copying it out of
the fixture before reading the spec*, that is the load-bearing case, and it has nothing to do
with version numbers.

The sharpened finding the ticket should open with: **ADR-0015 is currently unimplementable as
written.** *"`gravity` is a schema error"* presupposes that the schema rejects that key. So the
project has already decided the question **per-key, eight times**, by naming retired spellings
as errors. What is missing is not a decision but the general form of one, and the ticket's job
is to choose between formalising the per-key denylist (which is exactly my Q8 retired-spelling
table, making the two tickets natural partners) and adopting a general closed-schema rule.

**Attack on my own answer.** The brief says C4 is load-bearing because it decides whether "add
an optional field" is breaking — the one Q4 row round 1 split on — so leaving it open leaves
#14 incomplete. **It does not, under my answer**: with no version number there is no bump, so
there is no row to fill in. That is a real and non-obvious benefit of the no-number position
and I should declare the dependency rather than hide it — if the project overrules Q5 and ships
a number, **C4 becomes blocking**, because a classification table with an undecidable row is
worse than none.

---

## What I could not settle

- **Whether the G1 ambiguity generalises.** I showed the "names the replacement" idiom means
  *substitute* everywhere except retirement, where it means *delete*. One clean fix is to
  spell the two cases differently in the message (`…is not a field of this format; delete it.
  Nothing replaces it: x/y/origin/clip already determine the crop.`). Whether that is
  sufficient in general needs the consumer exercise this project runs for real decisions — a
  worked repair by N agents on a file where `gravity` was **not** inert. I could not run it,
  and my A2 could not test it because the fixture is at the fixed point. **This is the single
  thing I would measure before accepting #14.**
- **Whether any real file outside this repo would fail.** There is exactly one project file
  plus one committed pre-migration origin. Every strong result in this verdict — including my
  own byte-perfect diff — is one data point at a fixed point. The anti-drift rule cuts against
  me here as much as anyone.
- **The `.montaget.json` convention has no author.** Two files use it, zero documents specify
  it. This is a documented gap and, I think, a one-paragraph ADR regardless of #14's outcome.
- **`fmt` and #73 are upstream of any migration claim.** G2 shows byte-exact repair depends on
  an unpublished formatting convention. "The agent repairs and the result is canonical" is not
  yet true; it is true given #73.
- **C1's 5.8%** is not reproducible from its own description. Someone should restate which box
  dimensions it scanned, or replace it with ADR-0013's 4.466%.
- **C2's "a stranger has no `old.json`" is false** — `pre-migration.montaget.json` is on
  `main`. C2's conclusion survives; its premise should be corrected before it is cited again.

---

## Confidence

| | answer | confidence | why |
|---|---|---|---|
| **Q5 (no version number)** | no number; marker separable and not taken today | **High** | A2 is direct evidence: the repair used content, not a number, and a number would have been a second wrong claim. M1 shows it would have been stale three revisions running. |
| **Q5 (marker)** | publish the extension; `$schema` only if the resource is dereferenceable | **Low** | A4's failure is real but cosmetic, and I could not argue past that. I would not fight anyone who wants `$schema` anyway. |
| **Q6** | lower bound, if a number exists | **High** on P5 over P6; **Medium** that the question survives at all | M1 re-verified by me. The dissolution argument is mine and untested by anyone. |
| **Q7** | P8, at schema publication | **Medium-high** | I attacked it with `verify.py` and checked the artifact rather than arguing; it held. Medium because the "predicate exists in prose" reading is not absurd. |
| **Q8** | never a command; permanent retired-spelling table in `validate` | **High** on "never a command"; **Medium** on the table being sufficient | A2 and A3 both land here and A3 in particular inverts the usual intuition. Medium on sufficiency because it rests on a rule nobody has written yet, and because G1 shows the current message quality is not yet good enough. |
| **Q9** | zero fields; semantics belongs to the binary | **Medium-high** | The C1-kills-P12 argument is the strongest piece of reasoning I produced, but the fixed-point residue is a genuine hole I patched with a structural argument rather than evidence. |
| **Q10a** | #61 does not block; #14 must be #61-agnostic | **Medium-high** | Clean argument, and the drafting constraint makes it self-enforcing. |
| **Q10b** | new ticket, non-blocking — **unless** Q5 is overruled | **Medium** | The conditional is the honest part. If a number ships, I flip this to blocking. |

The result I would most want re-tested by someone else is **G1**, and the claim I would most
want overturned if it is wrong is that **A3's element-type removal is agent-executable**. If
someone shows an agent cannot reliably author the replacement asset, Q8 reopens — and it
reopens toward *refusal*, not toward `montaget migrate`, because the tool cannot do it either.
