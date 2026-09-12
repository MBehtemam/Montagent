# VERDICT-R2 — jury-14 round 2, opus

## Headline

I ran the migration-by-error-message exercise on the real pre-ADR-0015 file and it
reproduced the committed post-ADR-0015 fixture **byte for byte, with zero value
guessing** — which is the strongest single piece of evidence this ticket has, and it
points away from a migration tool. But the result has a scope limit that I state
rather than hide: it worked because ADR-0015 retired `gravity` *for being inert*, and
an inert field loses nothing when deleted. My answer is therefore a package, not four
independent calls: **one top-level marker `"montaget": 1`, justified on a
destructive-write hazard I found in `fmt` and explicitly *not* on versioning; the
number frozen at 1 and read as a lower bound if it is ever read at all; the revision
counter starting when the JSON Schema resource ships, not at ADR-0016; `montaget
migrate` never, replaced by a one-line constraint on what ADRs may retire; and no
second version field, because the semantics-only class belongs on the renderer, not in
the document.** C1 is the reason the last of those changed: it removes the version
field's last exclusive job rather than reinforcing it. Two corrections to the record
fall out of Part A: I could not reproduce C1's 5.8%, and round-1's fact 3 is false —
`migrate.py` cannot run from a clean checkout of `main`, because its input file is not
on `main`.

---

## Part A findings

### A2 — the core test. Byte-identical repair; nothing guessed; one real fork.

I hand-ran `validate` over `3b795255^`'s fixture under ADRs 0001–0015. The report is
**8 errors, all the same error** — `gravity` on the seven photos and `handle-logo`.
Everything else passes, including the two checks most likely to fire: aperture coverage
(media-free, 8/8 pass) and strict-equality fit deviation (`2720*1080 // 1536 = 1912`,
declared 1912; `800×800 → 68×68`, declared 68 — both exact).

Message emitted, in the pattern every ADR in the chain uses:

```
photo-05-intro: `gravity` is not a legal field on an image element; it was retired
  in ADR-0015. With the declared rect authoritative, nothing is cropped in the sense
  `gravity` means: the rect's position (`x`, `y`, `origin`) and the frame-space `clip`
  already determine which part of the source survives. Remove it.
```

Acting as the agent with only those messages: exact-string delete, 8 removals.

```
$ diff my-repair.json post0015.json && echo BYTE-IDENTICAL
BYTE-IDENTICAL          70c5298b45f948cfd1ebe07fc00b467d5aa836af (both)
```

**What I had to guess: no values at all.** M2 is true — `handle-logo`'s
`gravity:"center"` is recorded in no ADR — and for this repair it was **irrelevant**,
because ADR-0015 retired `gravity` on the ground that it carried no information. A
field with zero information content costs nothing to delete unread. M2 proves git is
the only witness to a retired field's values; A2 proves that for *this* retirement
nobody needed the witness.

**The one real fork.** The error *names* `x`/`y`/`origin` and `clip`. That licenses a
second reading — "re-express the gravity there" — which for `handle-logo` means
rewriting `origin:"top-left", x:478, y:96` as `origin:"center", x:512, y:130`. I
checked it: **same rect, passes every check, legal file, different bytes.** Nothing in
the one-line message excludes it; I excluded it only by reading ADR-0015's structural
argument. The exposure is byte-churn against a format edited by exact-string replace
with unpublished key order (#73) — not corruption. It is the thing I would measure.

### A1 — M1 confirmed; C1 confirmed in kind, not in magnitude

M1's four blob hashes reproduce exactly: three consecutive accepted revisions, one blob.

C1's mechanism is real — `103 * (1920/103)` = `1919.9999999999998` → floors to 1919
where integer arithmetic gives 1920, and under ADR-0015's strict equality that is now an
`error` whose repair rewrites bytes. **I could not reproduce "~5.8%"** under any
denominator I tried: 0.080% (box 1080), 2.160% (box 1300), 10.300% (box 1920), 0.170%
(union of the fixture's two box widths), 0.379% (full float-vs-integer `cover` over
720,000 (source, box) cases), 92.333% (source widths with ≥1 diverging height). Logged
as unreproduced; the claim does not need it.

I add one thing C1 understates: **pre-ADR-0013 no rounding rule was published at all.**
ADR-0013 says ADR-0012 "explicitly asked a successor for" it. So 1919 was not legal
under a *different* rule — there was no fact of the matter. That reframes C1's class
from "a semantics change broke files" to "a predicate was created where none existed,"
which is Q7's whole argument arriving from an unexpected direction.

### C2 — confirmed and sharpened; round-1 fact 3 is false

`migrate.py` has three commits (`afc12d24`, `ed3db37c`, `3b795255`) — edited in place at
each ADR, never an N→N+1 step. No per-change scripts exist. Its input `old.json` is
**nowhere in the worktree under that name**; the origin is
`docs/research/prototypes/sample-project/en-halloween-decorating.montaget.json` on the
**unmerged** branch `prototype/sample-project-file`. Fetching it and running
`python3 migrate.py linear old.json regen.json` reproduces the current fixture exactly —
so the regenerator works, but round-1 fact 3's "the mapping re-runs from a clean
checkout" is **false for `main`**. That is a real, fixable defect independent of #14.

### A3 — the no-successor removal kills P11's best argument rather than supporting it

Constructed: ADR-0017 retires `type:"rect"` with no successor (the fixture has 10). The
agent cannot repair from any message, because what is missing is not information: it is
(1) a 984×169 `#1E344C` PNG that does not exist, (2) a path to invent under ADR-0002's
inline-source rule, and (3) consent to add a binary to the user's git repo. No error
string supplies a decision, a capability, or consent.

The decisive finding: P11 wants `migrate` as "a place to refuse loudly and name the
affected element ids." **That place already exists.** ADR-0006 makes `validate` run
every check on the whole project with no fast mode, and makes `render` run the identical
checks unconditionally and refuse on `error`. The loud, unskippable, id-naming refusal
is already guaranteed. A `migrate` subcommand is a second printer of the same list.

The only mechanism that closes the hole is a **rule**: no element type or load-bearing
field may be retired without a named in-format migration path. Cheaper than a tool, and
it covers A2's scope limit too.

### A4 — identification is separable, and it has exactly one non-ergonomic argument

The file's top-level keys are `frame, fps, background, duration, output, fonts, tracks`.
Nothing in the document identifies it. Two discriminators exist, both outside it: the
`*.montaget.json` filename convention (real, free, defeated by renaming or stdin) and
key-set sniffing (a heuristic; `tracks`+`fps`+`duration` is shared with OTIO and DAW
exports). The "nicer error message" argument is real but unmeasured and I do not lean
on it.

The one argument that is neither ergonomic nor about versioning: **ADR-0011 gives `fmt`
a write bit — "rewrite the file in the canonical convention" — and nothing in ADR-0011
or ADR-0006 conditions it on `validate` passing.** `montaget fmt` aimed at a mis-named
JSON rewrites a file that is not Montaget's, in the user's git repo. Bounded by git, but
a destructive write and a named failure.

**So yes, the marker and the number are separable, and A2 demonstrates it from both
sides**: the project's one real migration completed byte-exactly with no number in the
file, and the identification hazard exists with no versioning argument attached.

---

## Q5 — Does the file carry a marker, and is it a version?

**Preferred answer.** A single top-level `"montaget": 1`, adopted on **P2's shape but
not P2's reasoning**: justified by the `fmt` destructive-write guard above, frozen at 1,
and stated in the ADR as **not yet a version number**. P1 is rejected because it asserts
a contract revision over a contract that does not exist in machine-readable form (fact
2). P3 is rejected on one point only — its structural argument is correct and I adopt it
in Q9 — but "no field at all" leaves a writing tool with no precondition. **P4 is
rejected, and I agree with round 1's unanimity on independent grounds**: per-object
versions require chained upgrade functions, i.e. a tool that rewrites a named field on a
named element, which the standing invariant bans outright ("never a field name, never an
element id"); and OTIO's motivating context — a wire format between many vendors with
independently versioned object libraries — has no analogue in a single-implementation
format.

**Is a confidently wrong version number worse than no version number?** Yes — and that
settles Q6, not Q5. Fact 6 (8 of 8 agents copied a retired field out of the fixture) plus
M1 (three revisions, zero bytes) guarantees a large stale population. But a stale number
is only *wrong* under P6's equality reading. Under P5 it is a true statement about
authorship time. So the objection is an argument against P6, not against the marker.

**Strongest attack on my own answer.** My only non-ergonomic justification is one
sentence long and dies to a one-line fix: make `fmt` — and every future writing tool —
refuse a file that does not `validate`. That closes the destructive-write hole with
**zero format change**, and P3 then wins outright. The fallback justification is option
value ("the marker is the only slot a future version could occupy without a second
breaking change"), and **C4 guts it**: with no unknown-key policy, adding a required
top-level key later is the same move ADR-0014 and ADR-0015 each made at a cost of 0 and
8 bytes respectively. This project demonstrably adds required fields cheaply.

**Does it survive?** Weakly, and I say so rather than inflate it. The marker costs one
line and is reversible; the hazard is real today and unaddressed in ADR-0011. But I
cannot refute P3, and a reader who prefers the `fmt`-precondition fix has the better
argument on cost. I commit to the marker and name its falsifier rather than hedging.

**Falsifier.** ADR-0016 conditioning every writing tool on `validate` passing. Also: a
demonstration that key-set sniffing plus the filename convention has ever
misidentified anything — absence of that demonstration is why I refuse the ergonomic
argument.

---

## Q6 — Lower bound, or equality?

**Preferred answer: P5, the lower bound.** M1, which I re-verified, makes P6
indefensible: under equality, ADR-0013 and ADR-0014 would each have obliged a rewrite of
every correct file in the world to change one integer, on a file that was *byte-identical
across all three revisions*. M1's framing is right and I checked it — the fixture was
unchanged not because it is small but because it was already correct, and correct files
are exactly the ones a tightening leaves alone. A field whose only edit is ceremony is
the field fact 6 says agents copy stale.

**Does M1 change my answer?** It converts it. Fact 6 alone gives an ergonomic argument
for P5. M1 gives a structural one: P6 would have manufactured three mass rewrites that
changed no meaning.

**Strongest attack on my own answer.** A lower bound carries almost no information. If
the number says only "authored at ≥1," a reader still has to validate against the current
schema — which is precisely P3's redundancy charge, now proved by my own answer. P5 wins
Q6 by making the field weak enough to lose Q5.

I accept the attack and follow it: that is *why* my Q5 answer freezes the number and
declines to call it a version. The two answers are one answer.

A second attack, on P11's behalf: a dispatcher needs equality to detect a *forward* file
— one declaring `1` but authored under revision 4 conventions, which fact 6 makes likely
since agents copy the fixture. P11 answers "idempotent by the declared number," which is
circular: if the number is untrustworthy, idempotency must come from the file's *shape*,
and a shape-driven migrator does not need the number at all. That is not a defence of
P6; it is another demonstration that the number does no work.

**Falsifier.** A migration mechanism whose correctness provably requires knowing the
exact authoring revision and cannot be made shape-idempotent. I could not construct one.

---

## Q7 — What trips the policy?

**Preferred answer: P8 — publication of the JSON Schema resource — with P9's mechanics
clause already in force.** Fact 2 is the argument: the format is fifteen ADRs of prose
and no machine-readable artifact. A version number indexes a legality predicate, and
ADR-0013's own *Not settled here* leaves "whether `fit` may be omitted, and what omission
means" undefined — so at revision 13 there was no fact of the matter about whether a
given file was legal. My A1 work supplies a second instance independently: pre-ADR-0013
no rounding rule was published, so 1919 and 1920 were *both* files about which the spec
said nothing. You cannot index a predicate that does not exist. **All fifteen ADRs
collapse into revision 1, and ADR-0016 carries no bump.**

P9's mechanics half is not a rival — it is already true. `docs/agents/domain.md` already
requires the fixture be migrated with committed, re-runnable evidence. My C2 work shows
that obligation is currently **discharged badly**: `migrate.py` cannot run from a clean
checkout of `main`. The near-term action item from this ticket is not versioning; it is
landing the origin file (or an N-1 snapshot) on `main`.

**Strongest attack.** P8 is a definitional move that hands the author a switch they may
decline to throw. "Publish the schema" is controlled by the same person as "bump the
number," so P8 can defer the obligation indefinitely while looking principled. P7 (first
release / first external user) is externally forced and therefore ungameable.

**Does it survive?** Yes, on two grounds. First, ADR-0011 already schedules the schema as
an MCP resource, so it is owed, not optional. Second, P8's trigger is strictly *earlier*
than P7's — you cannot plausibly release without publishing the schema — so P8 dominates
P7 by binding sooner without binding weaker. If the schema were genuinely never going to
ship, P7 is right and P8 is an excuse.

**Falsifier.** A machine-checkable legality predicate existing today that I missed —
I grepped and fact 2 holds. Or: the schema being published while the format is still
knowingly underdetermined, which would make P8 fire on a predicate as absent as before.

---

## Q8 — Does `montaget migrate` ever exist?

**Preferred answer: P10 — never — with one amendment that P10 does not contain.**

A2 and A3 decide this, and both cut the same way. A2: the hardest real removal in the
project's history repaired **byte-exactly** from error messages alone. A3: the hardest
*hypothetical* removal cannot be repaired by any tool, and the "loud refusal naming the
element ids" that P11 offers as its consolation prize is already guaranteed by ADR-0006
at `render`, unconditionally and unskippably.

C2 removes P11's remaining plank. Its dispatch-and-ordering argument presumes a chain of
per-change scripts a stranger cannot sequence. **There is no such chain.** There is one
cumulative regenerator that runs from an origin file the author owns and that is not on
`main`. P11 offers to orchestrate a mechanism that has never existed.

C1 also cuts toward P10, not away. Its class is the one where the error message *does*
carry the repair value — ADR-0015 prints `Write 1546, or fit:"literal" if deliberate`.
That is a fork, and choosing between a re-derived integer and an assertion that the
stale one was deliberate is exactly CONTEXT.md's authorship/arithmetic line
(CONTEXT.md:191). A tool picking one fabricates intent.

**The amendment.** P10 as stated over-generalises from A2, which is the anti-drift
failure in a new costume. A2 worked because `gravity` was inert. So P10 must ship with
the rule A3 forced: **no ADR may retire an element type or a load-bearing field without a
named in-format migration path.** ADR-0015 satisfies it accidentally. Without the rule,
"no migrate tool" is a bet that every future retirement will also be inert.

**Strongest attack on my own answer.** A2 is n=1, and the one case was selected by the
format's own author, who retired the field *because* he had already proved it inert. That
is close to testing a migration policy on the easiest migration that will ever occur. And
A2 surfaced a legal alternative repair (`handle-logo` → `origin:"center"`) that I ruled
out only by reading ADR prose the error message did not contain — meaning the exercise
does not actually show that messages suffice, only that messages *plus the ADR body*
suffice. A stranger reading a schema resource may have both; an agent reading CLI output
may not.

**Does it survive?** Yes, but the attack forces a design consequence I will state as
part of the answer: **the `validate` message must carry the ADR's structural reason, not
just the replacement names** — one clause, not one word. That is a message-quality
requirement, not a tool.

**Falsifier.** A measured consumer exercise (the format the project already uses — eight
agents, real authoring tasks) in which agents given `validate` output and the ADRs
produce files that validate but diverge from each other or from intent. My A2 found the
shadow of one and not an instance. If that exercise produces real divergence, P10 loses
and P11's refusal-point becomes worth its cost.

---

## Q9 — One field or two mechanisms?

**Preferred answer: neither as stated — zero version fields.** A shape marker is
redundant with the schema (P12's elimination argument is correct about that half). A
semantics number in the document is a category error (P3's structural point: whether a
file's meaning has shifted is a fact about *which binary you run*, and a document cannot
self-describe a rule its author never knew existed). So: **the ADR/changelog trail is the
only honest home for semantics-only revisions — P13's substance — and the machine-readable
half belongs on the renderer, not the file.** Concretely: `render` stamps its engine
revision into its report and output metadata. "Which reading produced this video" is a
property of the binary; put it where it is true.

**Does C1 change my answer? Yes, explicitly, and it is the one place in this brief where
a correction flipped me.** Before C1 I would have taken P12's elimination argument, whose
load-bearing premise is "the one thing a schema cannot tell you is that identical legal
bytes now mean a different picture." C1 falsifies that premise for the project's only
instance. Trace ADR-0013 → ADR-0015 through every file, not just the fixture:

- A file at the new fixed point (`1912`) renders identically before and after — because
  agreeing with the new rule is what being at the fixed point *means*.
- A file off it (`1919` from float ULP, `1913` from ceil) is now a **schema error** under
  strict equality — caught mechanically, by the schema, with the repair value printed.

The byte-invisible-yet-meaning-changed set for ADR-0013+0015 is therefore **empty**. The
version field's claimed exclusive job has no instance in the project's history.

**Strongest attack on my own answer.** This is n=1 reasoning over ADR history — the
anti-drift rule pointed at the ADR chain instead of the fixture. One revision happening
to be checkable does not make the class empty. I construct the counterexample myself: if
a future ADR reinterprets `"ease":"linear"` from linear-in-value to linear-in-time, the
bytes are unchanged, the file is legal under both readings, and **no redundancy exists in
the document to check against** — ADR-0015's case was only catchable because `fit` is a
*derivation claim* whose input sits on disk.

**Does it survive?** Yes, because my claim is narrower than the attack assumes. I do not
claim the class is empty; I claim that *for* the byte-invisible residue a document field
cannot help, and that is structural rather than inductive. The `ease` file's status is
determined entirely by which renderer opens it. A number the author wrote before the
reinterpretation existed cannot encode it. Stamping the engine revision on the render
output does.

**Falsifier.** A semantics-only revision where a number written *by the author, in the
file, before the revision* would have changed the outcome for the better. I cannot
construct one, and I think it is impossible by construction — which is why I hold this
one more firmly than Q5.

---

## Q10a — Should #61 block #14?

**Preferred answer: no — but it blocks the schema publication, which under Q7 is when the
policy binds.**

The four jurors' mechanism is correct and I do not dispute it: if `fmt` materialises
ADR-0012's six defaults, a default change is a spec-text edit plus an `fmt` rerun that
rewrites bytes visibly and checkably; if it does not, a default change is C1's class with
a cheaper disguise. #61 really does silently set the severity class of every future
default change.

Three reasons it still should not block:

1. **It is moot under my own answers.** With no version field carrying semantics (Q9) and
   no breaking-change table to populate (Q5/Q6), #14's ADR contains no clause whose text
   depends on #61's outcome.
2. **#14's deliverables are #61-independent.** A marker frozen at 1, a lower-bound
   reading, a P8 trigger, a no-migrate rule — none change under either #61 answer.
3. **Blocking is the drift pattern in a new costume.** An adjacent open question expanding
   to swallow the decision is what the anti-drift rule guards against, applied to tickets
   instead of fixtures.

What #14's ADR *should* do is name #61 as the thing that decides default-change severity,
and record that #61 must land before the schema resource is published — because that is
when the predicate exists and the classification starts to mean something.

**Strongest attack on my own answer.** My "no" is entirely contingent on my Q5/Q9 answers.
Had the jury landed on P1 — a real version number with a Q4-style breaking-change table —
that table must classify "changing a default," and it cannot without knowing whether such
a change is byte-visible. **Under P1, #61 blocks.** So anyone who rejects my Q5 should
reject my Q10a with it. I state that rather than presenting the "no" as unconditional.

A supporting observation, weaker: ADR-0015 made `fit` **required** rather than defaulted,
on the explicit ground that "an omitted field is indistinguishable from a decision not to
check," and ADR-0014 did the same for the text box. If that trend continues, the defaulted
population shrinks and #61's stakes shrink with it. Speculation, labelled.

---

## Q10b — The missing unknown-key policy

**Preferred answer: a clause inside this ticket's ADR — and the clause should be strict.**

C4 is right that nothing states it and right that it is load-bearing. It understates the
case: **#14 cannot avoid the question, because #14 is about to add a top-level key to a
format that has never had one.** Whether a validator may reject unknown keys *is* the
question of whether adding `"montaget"` is itself breaking. It is inside this change, not
adjacent to it. That, not the Q4 row, is the decisive reason it belongs here.

**Which way: strict.** Unknown top-level keys and unknown element keys are a schema error
naming the nearest legal spelling. This is not a new principle — it is the project's
existing one, applied three times in two ADRs: `center-center` is an error naming
`center`, not an alias; `none`/`fill` are errors naming `literal`; `gravity` is an error
naming `x`/`y`/`origin` and `clip`. A permissive unknown-key rule silently contradicts all
three at once — under it `gravity` becomes legal-and-ignored, and ADR-0015's retirement
evaporates. Fact 6 makes permissiveness actively dangerous: the retirement only bites
because the field is *rejected*, and 8 of 8 agents reached for it anyway.

**The consequence I own.** Strict rejection makes "add an optional field" breaking for
**forward** compatibility (new files fail old readers) and not for **backward** (old files
pass new readers). Fact 8 says there are no released readers, and Q7 says nothing binds
until the schema ships — so the cost of stating it strictly today is zero, and it is
cheapest to state before there is anything to break. It also retroactively makes my own
Q5 marker safe to add.

**Strongest attack.** Strictness costs extensibility a format may later want: no
vendor-prefixed or `x-` escape hatch, no room for tooling to annotate a project file
in-band. Real, and I accept the cost: file-as-truth puts the file in git where annotations
belong beside it, and ADR-0002 already rejected an asset table on adjacent grounds. If the
project later wants an escape hatch, an explicitly-reserved prefix is an additive change
it can make then, from a strict baseline — which is the direction that is cheap. The
reverse (tightening from permissive) is not.

**Falsifier.** A named use for in-band third-party annotation on a Montaget project file
that cannot be served by a sidecar in the same git repo.

---

## What I could not settle

- **C1's 5.8%.** Six denominators tried, none reproduces it (0.080% / 0.170% / 0.379% /
  2.160% / 2.232% / 10.300% / 92.333%). The mechanism and its consequence are confirmed
  by the exact 103→1919 case; the statistic should be restated with its denominator or
  withdrawn.
- **Whether `fmt` requires `validate` to pass.** Nothing says. It single-handedly decides
  Q5, since it is the only non-ergonomic argument for the marker. This is a one-sentence
  ADR clause and it should be written whatever #14 concludes.
- **`migrate.py` is not runnable from a clean checkout of `main`** — its input lives on
  the unmerged `prototype/sample-project-file`. Round-1 fact 3 is false as stated. This
  is a defect in the evidence practice `docs/agents/domain.md` imposes, independent of
  #14, and fixing it is worth more than anything #14 decides.
- **The thing I would measure, and the only measurement that could overturn Q8.** Run the
  project's own eight-agent consumer exercise on A2's task: hand agents *only* the
  `validate` output for the pre-0015 file and ask them to repair it. Count how many
  produce the `handle-logo` `origin:"center"` variant instead of a deletion. If the split
  is material, message-driven repair produces byte-churn at scale and P11's refusal-point
  earns its cost. I found the fork; I could not measure who takes it.
- **Where the engine revision goes** (Q9's constructive half). I assert `render` should
  stamp it into its report and output metadata; I have not checked that against ADR-0011's
  output contract, and it may belong in #14's ADR or in a new ticket.

---

## Confidence

| | answer | confidence | note |
|---|---|---|---|
| **Q5** | marker `"montaget": 1`, frozen, not a version | **0.55 — low** | The whole justification is one unaddressed `fmt` hazard, and a one-line precondition kills it. P3 is not refuted and a reasonable jury takes it. My lowest-confidence answer and the one I would most expect to be talked out of. |
| **Q6** | P5, lower bound | **0.85 — high** | M1 re-verified by hand; P6 would have manufactured three meaningless mass rewrites. The main residual is that P5 makes the field so weak it undermines Q5 — which I accept and follow. |
| **Q7** | P8, schema publication; P9's mechanics already binding | **0.70 — moderate** | Fact 2 plus two independently-found instances of an absent predicate. The gameability attack is real and only partly answered by ADR-0011's existing commitment. |
| **Q8** | P10, never — plus the retirement rule | **0.80 — high** | Two direct experiments (A2 byte-exact, A3 no-tool-can-help) and C2 removing P11's only mechanism. Held below 0.9 because A2 is n=1 on the easiest possible case, selected by the format's author. |
| **Q9** | zero version fields; ADRs plus a renderer-stamped engine revision | **0.75 — moderate-high** | The structural half (a document cannot self-describe a rule its author never knew) I hold near 0.9. The empirical half (ADR-0013's byte-invisible set is empty) is n=1 and I constructed my own counterexample. |
| **Q10a** | no, #61 blocks the schema instead | **0.70 — moderate**, and explicitly contingent | Flips to "yes" if Q5 lands on P1. Anyone rejecting my Q5 should reject this with it. |
| **Q10b** | clause here, strict | **0.70 — moderate** | High confidence it belongs in #14 (0.85 — the marker *is* the unknown key). Lower on strictness (0.65), since the extensibility cost is real and I dismissed it on a direction-of-cheapness argument rather than evidence. |
